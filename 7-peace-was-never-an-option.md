+++
title = "Bevy Traditional Roguelike Quick-Start - 7. Peace Was Never An Option"
date = 2024-12-11
authors = ["Julien Robert"]
[taxonomies]
tags = ["rust", "bevy", "tutorial"]
+++

It is finally possible to engineer something which somewhat resembles a challenge - or a "game", as the optimists would put it. This lonely little cage has been dreary long enough.

```rust
// map.rs
fn spawn_cage(mut summon: EventWriter<SummonCreature>) {
// NEW!
    let cage = "\
##################\
#H.H..H.##...HH..#\
#.#####.##..###..#\
#...#...##.......#\
#..H#......#####.#\
#...#...##...H...#\
#.#####.##..###..#\
#..H...H##.......#\
####.########.####\
####.########.####\
#.......##H......#\
#.#####.##.......#\
#.#H....##..#.#..#\
#.#.##.......@...#\
#.#H....##..#.#..#\
#.#####.##.......#\
#.......##......H#\
##################\
    ";
// End NEW.
```

A little maze, full of dastardly foes. There is immediately a problem: spells allow both the player and the Hunters to deconstruct our beautiful architecture.

```rust
// creature.rs
#[derive(Component)]
pub struct Spellproof;

#[derive(Component)]
pub struct Attackproof;
```

Note the physical/magical duality! Right now, only `Spellproof` will be implemented, but `Attackproof` will be next very soon in this chapter.

```rust
// Add any species-specific components.
match &event.species {
    Species::Player => {
        new_creature.insert(Player);
    }
    // NEW!
    Species::Wall => {
        new_creature.insert((Attackproof, Spellproof));
    }
    // End NEW.
```

Now, to ensure that `Axiom::Dash` has no effect on `Spellproof` creatures:

```rust
// spells.rs
/// The targeted creatures dash in the direction of the caster's last move.
fn axiom_function_dash(
    mut teleport: EventWriter<TeleportEntity>,
    map: Res<Map>,
    spell_stack: Res<SpellStack>,
    momentum: Query<&OrdDir>,
    is_spellproof: Query<Has<Spellproof>>, // NEW!
) {
    let synapse_data = spell_stack.spells.last().unwrap();
    let caster_momentum = momentum.get(synapse_data.caster).unwrap();
    if let Axiom::Dash { max_distance } = synapse_data.axioms[synapse_data.step] {
        // For each (Entity, Position) on a targeted tile with a creature on it...
        for (dasher, dasher_pos) in synapse_data.get_all_targeted_entity_pos_pairs(&map) {
        // NEW!
            // Spellproof entities cannot be affected.
            if is_spellproof.get(dasher).unwrap() {
                continue;
            }
        // End NEW.
```

If you `cargo run` now, the cage will now be fully inescapable.

// TODO gif

And, in such a doomed prison, the only thing left to do is fight for entertainment.

```rust
#[derive(Component)]
pub struct Health {
    pub hp: usize,
    pub max_hp: usize,
}

// The graphical representation of Health: a health bar.
#[derive(Bundle)]
pub struct HealthIndicator {
    pub sprite: Sprite,
    pub visibility: Visibility,
    pub transform: Transform,
}

#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub momentum: OrdDir,
    pub sprite: Sprite,
    pub species: Species,
    pub health: Health, // NEW!
}
```

Let's add `Health` to all creatures, with individual robustness values for each different `Species`...

```rust
// events.rs

/// Place a new Creature on the map of Species and at Position.
pub fn summon_creature(
    // SNIP
) {
let mut new_creature = commands.spawn((
Creature {
    // SNIP
    momentum: event.momentum,
    // NEW!
    health: {
        let max_hp = match &event.species {
            Species::Player => 7,
            Species::Wall => 10,
            Species::Hunter => 2,
            Species::Spawner => 3,
            Species::Airlock => 10,
        };
        // Start at full health.
        let hp = max_hp;
        Health { max_hp, hp }
    },
    // End NEW.
},
```

...and give them all a `HealthIndicator` to visually track this.

```rust
// events.rs
/// Place a new Creature on the map of Species and at Position.
pub fn summon_creature(
    // SNIP
) {
    // Add any species-specific components.
    match &event.species {
        // SNIP
    }

    // NEW!
    // Free the borrow on Commands.
    let new_creature_entity = new_creature.id();
    let hp_bar = commands
        .spawn(HealthIndicator {
            sprite: Sprite {
                image: asset_server.load("spritesheet.png"),
                custom_size: Some(Vec2::new(64., 64.)),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index: 178,
                }),
                ..default()
            },
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(0., 0., 1.),
        })
        .id();
    commands.entity(new_creature_entity).add_child(hp_bar);
    // End NEW.
}
```

The `HealthIndicator` will be added as a **child entity** of the creature. While it may seem like blasphemy to have hierarchies like this in an ECS game development environment, they do exist in a limited form. In Bevy, this is useful to have a sprite follow another as if it were "glued" to it, since the children inherit a Transform component from their parent. This is exactly what we want for a creature-specific HP bar! Children can be attached with `commands.entity(parent).add_child(child);`

Now, for damage to actually exist, it will of course be passed as an ̀`Event`. Note the `&Children` in the `Query`, which allows for easy access of the damaged creature's `HealthIndicator`!


```rust
// events.rs
#[derive(Event)]
pub struct HarmCreature {
    entity: Entity,
    culprit: Entity,
    damage: usize,
}

pub fn harm_creature(
    mut events: EventReader<HarmCreature>,
    mut remove: EventWriter<RemoveCreature>,
    mut creature: Query<(&mut Health, &Children)>,
    mut hp_bar: Query<(&mut Visibility, &mut Sprite)>,
) {
    for event in events.read() {
        let (mut health, children) = creature.get_mut(event.entity).unwrap();
        // Deduct damage from hp.
        health.hp = health.hp.saturating_sub(event.damage);
        // Update the healthbar.
        for child in children.iter() {
            let (mut hp_vis, mut hp_bar) = hp_bar.get_mut(*child).unwrap();
            // Don't show the healthbar at full hp.
            if health.max_hp == health.hp {
                *hp_vis = Visibility::Hidden;
            } else {
                *hp_vis = Visibility::Inherited;
                let hp_percent = health.hp as f32 / health.max_hp as f32;
                hp_bar.texture_atlas.as_mut().unwrap().index = match hp_percent {
                    0.86..1.00 => 178,
                    0.72..0.86 => 179,
                    0.58..0.72 => 180,
                    0.44..0.58 => 181,
                    0.30..0.44 => 182,
                    0.16..0.30 => 183,
                    0.00..0.16 => 184,
                    _ => panic!("That is not a possible HP %!"),
                }
            }
        }
        // 0 hp creatures are removed.
        if health.hp == 0 {
            remove.send(RemoveCreature {
                entity: event.entity,
            });
        }
    }
}
```

`saturating_sub` prevents integer overflow by stopping subtraction at 0. The healthbar is hidden when it is full, and then gradually deteriorates by shifting its sprite into increasingly dire variations as the HP percentage lowers.

Note the yet unimplemented event at the end for creatures to remove from the game board - which we will attend to immediately.

```rust
#[derive(Event)]
pub struct RemoveCreature {
    entity: Entity,
}

pub fn remove_creature(
    mut events: EventReader<RemoveCreature>,
    mut commands: Commands,
    mut map: ResMut<Map>,
    creature: Query<(&Position, Has<Player>)>,
    mut spell_stack: ResMut<SpellStack>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
) {
    for event in events.read() {
        let (position, is_player) = creature.get(event.entity).unwrap();
        // Visually flash an X where the creature was removed.
        magic_vfx.send(PlaceMagicVfx {
            targets: vec![*position],
            sequence: EffectSequence::Simultaneous,
            effect: EffectType::XCross,
            decay: 0.5,
            appear: 0.,
        });
        // For now, avoid removing the player - the game panics without a player.
        if !is_player {
            // Remove the creature from Map
            map.creatures.remove(position);
            // Remove the creature AND its children (health bar)
            commands.entity(event.entity).despawn_recursive();
            // Remove all spells cast by this creature
            // (this entity doesn't exist anymore, casting its spells would crash the game)
            spell_stack
                .spells
                .retain(|spell| spell.caster != event.entity);
        }
    }
}
```

We do have a little bit of upkeep to make sure the (very involved!) task of removing an Entity goes according to plan. Merely calling `despawn` instead of `despawn_recursive` would keep floating health bars that belong to no one! Not to mention the instant panic that would result from a spell still in the `SpellStack` with the removed creature as a caster, trying to target itself, something which does not exist.

We still have no way to inflict harm from within the game. But, perhaps you remember this little placeholder?

> `// Nothing here just yet, but this is where collisions between creatures will be handled.̀` (events.rs, `teleport_entity`)

It is time to put it to use.

```rust
// events.rs
pub fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut creature: Query<&mut Position>,
    mut map: ResMut<Map>,
    mut commands: Commands,
    mut collision: EventWriter<CreatureCollision>, // NEW!
) {
    // SNIP
    if map.is_passable(event.destination.x, event.destination.y) {
        // SNIP
    } else {
        // NEW!
        // A creature collides with another entity.
        let collided_with = map
            .get_entity_at(event.destination.x, event.destination.y)
            .unwrap();
        collision.send(CreatureCollision {
            culprit: event.entity,
            collided_with: *collided_with,
        });
        // End NEW.
    }
```

A new, unimplemented event... yes, because not all collisions will necessarily be harmful. Some could be interacting with a mechanism, talking to an NPC, or opening a door. The later of which will be shown later in this chapter!

```rust
#[derive(Event)]
pub struct CreatureCollision {
    culprit: Entity,
    collided_with: Entity,
}

pub fn creature_collision(
    mut events: EventReader<CreatureCollision>,
    mut harm: EventWriter<HarmCreature>,
    mut open: EventWriter<OpenDoor>,
    flags: Query<(Has<Door>, Has<Attackproof>)>,
    mut turn_manager: ResMut<TurnManager>,
    mut creature: Query<(&OrdDir, &mut Transform)>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.culprit == event.collided_with {
            // No colliding with yourself.
            continue;
        }
        let (is_door, cannot_be_attacked) = flags.get(event.collided_with).unwrap();
        if is_door {
            // Open doors.
            open.send(OpenDoor {
                entity: event.collided_with,
            });
        } else if !cannot_be_attacked {
            // Melee attack.
            harm.send(HarmCreature {
                entity: event.collided_with,
                culprit: event.culprit,
                damage: 1,
            });
            // Melee attack animation.
            let (attacker_orientation, mut attacker_transform) =
                creature.get_mut(event.culprit).unwrap();
            attacker_transform.translation.x +=
                attacker_orientation.as_offset().0 as f32 * 64. / 4.;
            attacker_transform.translation.y +=
                attacker_orientation.as_offset().1 as f32 * 64. / 4.;
            commands.entity(event.culprit).insert(SlideAnimation);
        } else if matches!(turn_manager.action_this_turn, PlayerAction::Step) {
            // The player spent their turn walking into a wall, disallow the turn from ending.
            turn_manager.action_this_turn = PlayerAction::Invalid;
        }
    }
}
```
