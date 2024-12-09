+++
title = "Bevy Traditional Roguelike Quick-Start - 5. Laser Sumo Rave"
date = 2024-11-19
authors = ["Julien Robert"]
[taxonomies]
tags = ["rust", "bevy", "tutorial"]
+++

Magic is nothing without the *sparkles*! The *artifice*! We need laser beams and confetti.

# Visual Effects

Let us start with the preliminary `Bundle`, `Event` and other supporting structs and enums. Whenever this event will be triggered, effects of a certain colour and sprite will be placed on all selected tiles, and will gradually decay with time.

```rust
// graphics.rs
#[derive(Bundle)]
pub struct MagicEffect {
    /// The tile position of this visual effect.
    pub position: Position,
    /// The sprite representing this visual effect.
    pub sprite: Sprite,
    pub visibility: Visibility,
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
    /// `duration` is how long it takes to move from one effect to the next.
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

There are only two systems to make, now - one to place the effects, and one to ensure they decay appropriately.
Each effect initially has its `Visibility` set to `Hidden` - if the effect is the tip of a laser beam, we want it to be displayed *after* the start of the laser beam, as to create a "trailing" effect.

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
                sprite: Sprite {
                    image: asset_server.load("spritesheet.png"),
                    custom_size: Some(Vec2::new(64., 64.)),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_layout.handle.clone(),
                        index: get_effect_sprite(&event.effect),
                    }),
                    ..default()
                },
                visibility: Visibility::Hidden,
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

Then, effects appear one by one (if sequential) or all at once (if simultaneous), and decay into 100% transparency after a delay preset by their `decay` timers.

```rust
// graphics.rs
pub fn decay_magic_effects(
    mut commands: Commands,
    mut magic_vfx: Query<(Entity, &mut Visibility, &mut MagicVfx, &mut Sprite)>,
    time: Res<Time>,
) {
    for (vfx_entity, mut vfx_vis, mut vfx_timers, mut vfx_sprite) in magic_vfx.iter_mut() {
        // Effects that have completed their appear timer and are now visible, decay.
        if matches!(*vfx_vis, Visibility::Inherited) {
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
                *vfx_vis = Visibility::Inherited;
            }
        }
    }
}
```

All we need now is to properly place these effects as a result of casting spells.

```rust
// spells.rs
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
// spells.rs
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

If you `cargo run` now, you'll run into an amusing bug - the laser beams create invisible walls. This is because `register_creatures` cannot make the difference between a creature and a magic effect - they both have the `Position` component! It thinks lasers are creatures, and adds them to the `Map`. Let us filter them out.

```rust
// map.rs
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

Don't forget to register.

```rust
// graphics.rs
impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
        // NEW!
        app.add_systems(Update, place_magic_effects);
        app.add_systems(Update, decay_magic_effects);
        app.add_event::<PlaceMagicVfx>();
        // End NEW.
    }
}

```

All done! `cargo run`, and enjoy the fireworks.

{{ image(src="https://raw.githubusercontent.com/Oneirical/oneirical.github.io/main/5-laser-sumo-rave/beams.gif", alt="The player lasering the wall, and cyan blue beam effects leaving a trail as they do so.",
         position="center", style="border-radius: 8px;") }}

# Motion Animations

All fun and good, but the instant-teleports don't fit in with all this aesthetic superiority we have just implemented. Let us fix that.

Whenever a creature teleports, it will instead interpolate its `Transform` `translation` from its origin to its destination. To do so, we'll need to rewrite `adjust_transforms`.

```rust
// graphics.rs
#[derive(Component)]
pub struct SlideAnimation;

/// Each frame, adjust every entity's display location to match
/// their position on the grid, and make the camera follow the player.
pub fn adjust_transforms(
    mut creatures: Query<(
        Entity, // NEW!
        &Position,
        &mut Transform,
        Has<SlideAnimation>, // NEW!
        Has<Player>,
    )>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Position>)>,
    time: Res<Time>, // NEW!
    mut commands: Commands, // NEW!
) {
    // NEW!
    for (entity, pos, mut trans, is_animated, is_player) in creatures.iter_mut() {
        // If this creature is affected by an animation...
        if is_animated {
            // The sprite approaches its destination.
            let current_translation = trans.translation;
            let target_translation = Vec3::new(pos.x as f32 * 64., pos.y as f32 * 64., 0.);
            // The creature is more than 0.5 pixels away from its destination - smooth animation.
            if ((target_translation.x - current_translation.x).abs()
                + (target_translation.y - current_translation.y).abs())
                > 0.5
            {
                trans.translation = trans
                    .translation
                    .lerp(target_translation, 10. * time.delta_secs());
            // Otherwise, the animation is over - clip the creature onto the grid.
            } else {
                commands.entity(entity).remove::<SlideAnimation>();
            }
        } else {
    // End NEW.

            // For creatures with no animation.
            // Multiplied by the graphical size of a tile, which is 64x64.
            trans.translation.x = pos.x as f32 * 64.;
            trans.translation.y = pos.y as f32 * 64.;
        }
        if is_player {
            // The camera follows the player.
            let mut camera_trans = camera.get_single_mut().unwrap();
            (camera_trans.translation.x, camera_trans.translation.y) =
                (trans.translation.x, trans.translation.y);
        }
    }
}
```

Creatures have their `translation` interpolate (`lerp`) towards their target translation, until the animation completes and the component responsible for orchestrating this (`SlideAnimation`) is removed.

Great, now, any creature with the new `SlideAnimation` component will gracefully make its way to its destination. Let us add that component each time a creature teleports.

```rust
// events.rs
fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut creature: Query<&mut Position>,
    mut map: ResMut<Map>,
    mut commands: Commands, // NEW!
) {
    for event in events.read() {
        // SNIP
        // If motion is possible...
        if map.is_passable(event.destination.x, event.destination.y) {
            // SNIP
            creature_position.update(event.destination.x, event.destination.y);
            // Also, animate this creature, making its teleport action visible on the screen.
            commands.entity(event.entity).insert(SlideAnimation); // NEW!
        } else {
            // Nothing here just yet, but this is where collisions between creatures
            // will be handled.
            continue;
        }
    }
}
```

`cargo run`, and your two caged creatures *may* have developed a little bit of sliding grace, as shown below.

{{ image(src="https://raw.githubusercontent.com/Oneirical/oneirical.github.io/main/5-laser-sumo-rave/slide.gif", alt="The player lasering the wall, and the wall sliding smoothly instead of sharply teleporting.",
         position="center", style="border-radius: 8px;") }}

Wait... why "may"? Indeed, should you restart the game multiple times, you might notice that it *sometimes* works, and *sometimes* doesn't.

Oh heavens, the worse type of bug - the non-deterministic error! How will we ever solve this? Is it time to give up?

No. We simply have not **scheduled our systems**.

# System Scheduling

Marking systems as `Update` only means they trigger every tick. In which *order* they trigger, however, that's fully up to chance! This causes a dizzying amount of non-deterministic bugs.

To prevent this, we must tell Bevy in which order all our events and animations are handled. As a starting point, let us diagnose just how bad it has gotten.

```rust
// main.rs
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        // SNIP
        // NEW!
        .edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        // End NEW.
        .run();
}
```

You should obtain this ominous message:

```
7 pairs of systems with conflicting data access have indeterminate execution order. Consider adding `before`, `after`, or `ambiguous_with` relationships between these:
 -- adjust_transforms and teleport_entity
    conflict on: ["redesign_tgfp::map::Position"]
 -- teleport_entity and player_step
    conflict on: ["bevy_ecs::event::Events<redesign_tgfp::events::TeleportEntity>", "redesign_tgfp::map::Map", "redesign_tgfp::map::Position"]
 -- teleport_entity and register_creatures
    conflict on: ["redesign_tgfp::map::Map", "redesign_tgfp::map::Position"]
 -- keyboard_input and player_step
    conflict on: ["bevy_ecs::event::Events<redesign_tgfp::events::PlayerStep>"]
 -- keyboard_input and cast_new_spell
    conflict on: ["bevy_ecs::event::Events<redesign_tgfp::spells::CastSpell>"]
 -- player_step and register_creatures
    conflict on: ["redesign_tgfp::map::Map"]
 -- process_axiom and cast_new_spell
    conflict on: ["redesign_tgfp::spells::SpellStack"]
```

All of these are systems where at least one of the two modifies a certain component, leading the other system to behave completely differently depending on whether it runs before or after.

Let's bring on board a grand overseer to resolve all of these inconsistencies. Create the file `sets.rs`, in which a new plugin will take shape.

```rust
// sets.rs
use bevy::prelude::*;

pub struct SetsPlugin;

impl Plugin for SetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                keyboard_input.run_if(spell_stack_is_empty),
                creature_step,
                cast_new_spell,
                process_axiom,
            )
                .in_set(ActionPhase),
        );
        app.add_systems(
            Update,
            ((register_creatures, teleport_entity).chain()).in_set(ResolutionPhase),
        );
        app.add_systems(
            Update,
            ((place_magic_effects, adjust_transforms, decay_magic_effects).chain())
                .in_set(AnimationPhase),
        );
        app.configure_sets(
            Update,
            (ActionPhase, AnimationPhase, ResolutionPhase).chain(),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct ActionPhase;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct ResolutionPhase;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct AnimationPhase;
```

```rust
// spells.rs
pub fn spell_stack_is_empty(spell_stack: Res<SpellStack>) -> bool {
    spell_stack.spells.is_empty()
}
```

`chain` effectively means the systems run one after the other. First, I place everything related to making choices and casting spells into `ActionPhase`, then everything related to gameplay actions in `ResolutionPhase`, and finally put all the visuals and animations in `AnimationPhase`. The ordering was not chosen at random:

- `decay_magic_effects` goes after `adjust_transforms`, so the magical effects can snap to their tile position before starting to decay.
- `register_creatures` goes before `teleport_entity`, so that moving into the tile of a newly-summoned creature won't succeed due to the creature not having been registered into the `Map` yet.
- `keyboard_input` is disallowed until all spells have finished executing, so the player cannot twitch-reflex their way out of an incoming beam.

Add this new `Plugin` to `main.rs`.

```rust
// main.rs
mod sets; // NEW!

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            SetsPlugin, // NEW!
            SpellPlugin,
            EventPlugin,
            GraphicsPlugin,
            MapPlugin,
            InputPlugin,
        ))
```

Also, remove all instance of the `Update` systems in other files to avoid duplicate systems.

```rust
// graphics.rs
impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.add_event::<PlaceMagicVfx>();
        app.add_systems(Startup, setup_camera);
        // REMOVED Update systems.
    }
}
```

```rust
// map.rs
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map {
            creatures: HashMap::new(),
        });
        app.add_systems(Startup, spawn_player);
        app.add_systems(Startup, spawn_cage);
        // REMOVED Update systems.
    }
}
```

```rust
// events.rs
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerStep>();
        app.add_event::<TeleportEntity>();
        // REMOVED Update systems.
    }
}
```

```rust
// spells.rs
impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CastSpell>();
        app.init_resource::<SpellStack>();
        app.init_resource::<AxiomLibrary>();
        // REMOVED Update systems.
    }
}
```

```rust
// input.rs
// REMOVE all of the following.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, keyboard_input);
    }
}
```

```rust
// main.rs
use events::EventPlugin;
use graphics::GraphicsPlugin;
// REMOVED InputPlugin.
use map::MapPlugin;
use sets::SetsPlugin;
use spells::SpellPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            SetsPlugin,
            SpellPlugin,
            EventPlugin,
            GraphicsPlugin,
            MapPlugin,
            // REMOVED InputPlugin.
        ))
```

With all that done, `cargo run`! Note that this time around, the beam hits the wall, and *then* the wall is knocked back, all thanks to our system ordering.

{{ image(src="https://raw.githubusercontent.com/Oneirical/oneirical.github.io/main/5-laser-sumo-rave/ordering.gif", alt="The player lasering the wall, and the wall only being knocked back after it has been hit by the beam.",
         position="center", style="border-radius: 8px;") }}
