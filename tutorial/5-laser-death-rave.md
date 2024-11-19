

```rust
// graphics.rs
#[derive(Bundle)]
pub struct MagicEffect {
    /// The tile position of this visual effect.
    pub position: Position,
    /// The sprite representing this visual effect.
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
    /// The timers tracking when the effect appears, and how
    /// long it takes to decay.
    pub vfx: MagicVfx,
}

#[derive(Event)]
/// An event to place visual effects on the game board.
pub struct PlaceMagicVfx {
    /// All tile positions on which a visual effect will appear.
    pub targets: Vec<Position>,
    /// Whether the effect appear one by one, or all at the same time.
    pub sequence: EffectSequence,
    /// The effect sprite.
    pub effect: EffectType,
    /// How long these effects take to decay.
    pub decay: f32,
    /// How long these effects take to appear.
    pub appear: f32,
}

#[derive(Clone, Copy)]
pub enum EffectSequence {
    /// All effects appear at the same time.
    Simultaneous,
    /// Effects appear one at a time, in a queue.
    /// `duration` is how long it takes to unroll the entire queue.
    Sequential { duration: f32 },
}

#[derive(Clone, Copy)]
pub enum EffectType {
    HorizontalBeam,
    VerticalBeam,
    RedBlast,
    GreenBlast,
    XCross,
}

#[derive(Component)]
pub struct MagicVfx {
    /// How long this effect takes to decay.
    appear: Timer,
    /// How long this effect takes to appear.
    decay: Timer,
}

/// Get the appropriate texture from the spritesheet depending on the effect type.
pub fn get_effect_sprite(effect: &EffectType) -> usize {
    match effect {
        EffectType::HorizontalBeam => 15,
        EffectType::VerticalBeam => 16,
        EffectType::RedBlast => 14,
        EffectType::GreenBlast => 13,
        EffectType::XCross => 1,
    }
}
```

```rust
// graphics.rs
pub fn place_magic_effects(
    mut events: EventReader<PlaceMagicVfx>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    for event in events.read() {
        for (i, target) in event.targets.iter().enumerate() {
            // Place effects on all positions from the event.
            commands.spawn(MagicEffect {
                position: *target,
                sprite: SpriteBundle {
                    texture: asset_server.load("spritesheet.png"),
                    transform: Transform::from_scale(Vec3::new(4., 4., 0.)),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                atlas: TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index: get_effect_sprite(&event.effect),
                },
                vfx: MagicVfx {
                    appear: match event.sequence {
                        // If simultaneous, everything appears at the same time.
                        EffectSequence::Simultaneous => {
                            Timer::from_seconds(event.appear, TimerMode::Once)
                        }
                        // Otherwise, effects gradually get increased appear timers depending on
                        // how far back they are in their queue.
                        EffectSequence::Sequential { duration } => Timer::from_seconds(
                            i as f32 * duration + event.appear,
                            TimerMode::Once,
                        ),
                    },
                    decay: Timer::from_seconds(event.decay, TimerMode::Once),
                },
            });
        }
    }
}
```

```rust
pub fn decay_magic_effects(
    mut commands: Commands,
    mut magic_vfx: Query<(Entity, &mut Visibility, &mut MagicVfx, &mut Sprite)>,
    time: Res<Time>,
) {
    for (vfx_entity, mut vfx_vis, mut vfx_timers, mut vfx_sprite) in magic_vfx.iter_mut() {
        // Effects that have completed their appear timer and are now visible, decay.
        if matches!(*vfx_vis, Visibility::Visible) {
            vfx_timers.decay.tick(time.delta());
            // Their alpha (transparency) slowly loses opacity as they decay.
            vfx_sprite
                .color
                .set_alpha(vfx_timers.decay.fraction_remaining());
            if vfx_timers.decay.finished() {
                commands.entity(vfx_entity).despawn();
            }
        // Effects that have not appeared yet progress towards appearing for the first time.
        } else {
            vfx_timers.appear.tick(time.delta());
            if vfx_timers.appear.finished() {
                *vfx_vis = Visibility::Visible;
            }
        }
    }
}
```

```rust
/// Target the caster's tile.
fn axiom_form_ego(
    mut magic_vfx: EventWriter<PlaceMagicVfx>, // NEW!
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
) {
    // Get the currently executed spell.
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    // Get the caster's position.
    let caster_position = *position.get(synapse_data.caster).unwrap();

    // NEW!
    // Place the visual effect.
    magic_vfx.send(PlaceMagicVfx {
        targets: vec![caster_position],
        sequence: EffectSequence::Sequential { duration: 0.04 },
        effect: EffectType::RedBlast,
        decay: 0.5,
        appear: 0.,
    });
    // End NEW.

    // Add that caster's position to the targets.
    synapse_data.targets.push(caster_position);
}
```

```rust
/// Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_momentum_beam(
    mut magic_vfx: EventWriter<PlaceMagicVfx>, // NEW!
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    position_and_momentum: Query<(&Position, &OrdDir)>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let (caster_position, caster_momentum) =
        position_and_momentum.get(synapse_data.caster).unwrap();
    // Start the beam where the caster is standing.
    // The beam travels in the direction of the caster's last move.
    let (off_x, off_y) = caster_momentum.as_offset();
    let mut output = linear_beam(*caster_position, 10, off_x, off_y, &map);

    // NEW!
    // Add some visual beam effects.
    magic_vfx.send(PlaceMagicVfx {
        targets: output.clone(),
        sequence: EffectSequence::Sequential { duration: 0.04 },
        effect: match caster_momentum {
            OrdDir::Up | OrdDir::Down => EffectType::VerticalBeam,
            OrdDir::Right | OrdDir::Left => EffectType::HorizontalBeam,
        },
        decay: 0.5,
        appear: 0.,
    });
    // End NEW.
    
    // Add these tiles to `targets`.
    synapse_data.targets.append(&mut output);
}
```

```rust
/// Newly spawned creatures earn their place in the HashMap.
fn register_creatures(
    mut map: ResMut<Map>,
    // Any entity that has a Position that just got added to it -
    // currently only possible as a result of having just been spawned in.

    // CHANGED - Added Without<MagicVfx>
    displaced_creatures: Query<(&Position, Entity), (Added<Position>, Without<MagicVfx>)>,
) {
    for (position, entity) in displaced_creatures.iter() {
        // Insert the new creature in the Map. Position implements Copy,
        // so it can be dereferenced (*), but `.clone()` would have been
        // fine too.
        map.creatures.insert(*position, entity);
    }
}
```
