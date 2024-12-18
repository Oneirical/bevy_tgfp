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

`saturating_sub` prevents integer overflow by stopping subtraction at 0. The healthbar is hidden when it is full, and then gradually deteriorates by shifting its sprite into increasingly dire variations as the HP percentage lowers. `Visibility::Inherited` means the health bar will also be hidden should the parent (the creature itself) be hidden.

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

A new, unimplemented event... yes, because not all collisions will necessarily be harmful. Some could be interacting with a mechanism, talking to an NPC, or... **opening a door**.

```rust
#[derive(Event)]
pub struct CreatureCollision {
    culprit: Entity,
    collided_with: Entity,
}

pub fn creature_collision(
    mut events: EventReader<CreatureCollision>,
    mut harm: EventWriter<HarmCreature>,
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

There are quite a few things of interest in this new system:

- `Attackproof` is finally checked. This prevents the player from melee-attacking walls to break them, and escape the cage.
- There is a melee attack animation. It shifts the attacking entity 1/4th of a tile closer to their attack direction, and the added `SlideAnimation` returns them to their original placement, making it look like a "jab" onto the attacked creature.
- There is a yet unimplemented resource, `TurnManager`, which will be addressed next.
- There is a yet unimplemented event, `OpenDoor`, which will be showcased later down the chapter, accompanied by a `Door` component.

# Wallhack Anticheat

Currently, it is possible to wait for enemies to get into melee range by scratching at the walls over and over again. Even though they are indestructible (because of `Attackproof`), this still skips turns even though nothing is actually happening!

This is because `end_turn` triggers no matter what, even if the player performed an invalid action. This must be checked.

```rust
// events.rs
#[derive(Resource)]
pub struct TurnManager {
    pub turn_count: usize,
    // NEW!
    /// Whether the player took a step, cast a spell, or did something useless (like step into a wall) this turn.
    pub action_this_turn: PlayerAction,
    // End NEW.
}

// NEW!
pub enum PlayerAction {
    Step,
    Spell,
    Invalid,
}
// End NEW.
```

```rust
// events.rs
pub fn end_turn(
    // SNIP
) {
    for _event in events.read() {
        // NEW!
        // The player shouldn't be allowed to "wait" turns by stepping into walls.
        if matches!(turn_manager.action_this_turn, PlayerAction::Invalid) {
            return;
        }
        // End NEW.
```

For this to do anything, we'll ensure each possible action is registered the moment the player presses a key:

```rust
// input.rs

/// Each frame, if a button is pressed, move the player 1 tile.
pub fn keyboard_input(
    // SNIP
    mut turn_manager: ResMut<TurnManager>, // NEW!
    mut turn_end: EventWriter<EndTurn>, // NEW!
) {
    if input.just_pressed(KeyCode::Space) {
        // SNIP
        turn_manager.action_this_turn = PlayerAction::Spell; // NEW!
        turn_end.send(EndTurn); // NEW!
    }
    if input.just_pressed(KeyCode::KeyW) {
        // SNIP
        turn_manager.action_this_turn = PlayerAction::Step; // NEW!
        turn_end.send(EndTurn); // NEW!
    }
    if input.just_pressed(KeyCode::KeyD) {
        // SNIP
        turn_manager.action_this_turn = PlayerAction::Step; // NEW!
        turn_end.send(EndTurn); // NEW!
    }
    if input.just_pressed(KeyCode::KeyA) {
        // SNIP
        turn_manager.action_this_turn = PlayerAction::Step; // NEW!
        turn_end.send(EndTurn); // NEW!
    }
    if input.just_pressed(KeyCode::KeyS) {
        // SNIP
        turn_manager.action_this_turn = PlayerAction::Step; // NEW!
        turn_end.send(EndTurn); // NEW!
    }
}
```

Note how this is offshoring `EndTurn` to `keyboard_input` - this is because we want spells to cost a turn as well. We'll remove the original `EndTurn` send in `creature_step`... and we'll offshore the momentum shift to a whole new event, `AlterMomentum`.

The reason for this is simple - now that `Invalid` moves are a thing, we don't want the player to be able to change the momentum of their laser beams by uselessly pushing against walls. One valid step or melee attack = one momentum shift!

```rust
// events.rs

pub fn creature_step(
    mut events: EventReader<CreatureStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut momentum: EventWriter<AlterMomentum>, // CHANGED EndTurn for AlterMomentum.
    mut creature: Query<&Position>, // CHANGED removed all components except Position.
) {
    for event in events.read() {
        // CHANGED only the Position is accessed.
        let creature_pos = creature.get_mut(event.entity).unwrap();
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportEntity::new(
            event.entity,
            creature_pos.x + off_x,
            creature_pos.y + off_y,
        ));
        // CHANGED momentum update is now an event, and there is no more EndTurn.
        // Update the direction towards which this creature is facing.
        momentum.send(AlterMomentum {
            entity: event.entity,
            direction: event.direction,
        });
        // End CHANGED
    }
}
```

`AlterMomentum` ensures your move is not invalid to properly change the creature's momentum. To signify this graphically, it will also rotate the sprite around to indicate in which direction it is currently "facing", as well as ensuring the health bar always stays on the bottom of the sprite despite this rotation.

```rust
// events.rs

#[derive(Event)]
pub struct AlterMomentum {
    pub entity: Entity,
    pub direction: OrdDir,
}

pub fn alter_momentum(
    mut events: EventReader<AlterMomentum>,
    mut creature: Query<(&mut OrdDir, &mut Transform, &Children)>,
    mut hp_bar: Query<&mut Transform, Without<OrdDir>>,
    turn_manager: Res<TurnManager>,
) {
    for event in events.read() {
        // Don't allow changing your momentum by stepping into walls.
        if matches!(turn_manager.action_this_turn, PlayerAction::Invalid) {
            return;
        }
        let (mut creature_momentum, mut creature_transform, children) =
            creature.get_mut(event.entity).unwrap();
        *creature_momentum = event.direction;
        match event.direction {
            OrdDir::Down => creature_transform.rotation = Quat::from_rotation_z(0.),
            OrdDir::Right => creature_transform.rotation = Quat::from_rotation_z(PI / 2.),
            OrdDir::Up => creature_transform.rotation = Quat::from_rotation_z(PI),
            OrdDir::Left => creature_transform.rotation = Quat::from_rotation_z(3. * PI / 2.),
        }
        // Keep the HP bar on the bottom.
        for child in children.iter() {
            let mut hp_transform = hp_bar.get_mut(*child).unwrap();
            match event.direction {
                OrdDir::Down => hp_transform.rotation = Quat::from_rotation_z(0.),
                OrdDir::Right => hp_transform.rotation = Quat::from_rotation_z(3. * PI / 2.),
                OrdDir::Up => hp_transform.rotation = Quat::from_rotation_z(PI),
                OrdDir::Left => hp_transform.rotation = Quat::from_rotation_z(PI / 2.),
            }
        }
    }
}
```

# Sliding Dystopian Airlocks

Now, for `OpenDoor` and `Door`.

```rust
// creature.rs

#[derive(Component)]
pub struct Door;
```

```rust
// events.rs

#[derive(Event)]
pub struct OpenDoor {
    entity: Entity,
}
```

```rust
// creature.rs

#[derive(Debug, Component, Clone, Copy)]
pub enum Species {
    Player,
    Wall,
    Hunter,
    Spawner,
    Airlock, // NEW!
}

/// Get the appropriate texture from the spritesheet depending on the species type.
pub fn get_species_sprite(species: &Species) -> usize {
    match species {
        Species::Player => 0,
        Species::Wall => 3,
        Species::Hunter => 4,
        Species::Spawner => 5,
        Species::Airlock => 17, // NEW!
    }
}
```

```rust
// map.rs
fn spawn_cage(mut summon: EventWriter<SummonCreature>) {
// CHANGED - added <>V^
    let cage = "\
##################\
#H.H..H.##...HH..#\
#.#####.##..###..#\
#...#...##.......#\
#..H#...><.#####.#\
#...#...##...H...#\
#.#####.##..###..#\
#..H...H##.......#\
####^########^####\
####V########V####\
#.......##H......#\
#.#####.##.......#\
#.#H....##..#.#..#\
#.#.##..><...@...#\
#.#H....##..#.#..#\
#.#####.##.......#\
#.......##......H#\
##################\
    ";
// End CHANGED
    for (idx, tile_char) in cage.char_indices() {
        let position = Position::new(idx as i32 % 18, idx as i32 / 18);
        let species = match tile_char {
            '#' => Species::Wall,
            'H' => Species::Hunter,
            'S' => Species::Spawner,
            '@' => Species::Player,
            '^' | '>' | '<' | 'V' => Species::Airlock, // NEW!
            _ => continue,
        };
        // NEW!
        let momentum = match tile_char {
            '^' => OrdDir::Up,
            '>' => OrdDir::Right,
            '<' => OrdDir::Left,
            'V' | _ => OrdDir::Down,
        };
        // End NEW.
        summon.send(SummonCreature {
            species,
            position,
            momentum, // NEW!
            summon_tile: Position::new(0, 0),
        });
    }
}
```

Airlocks face a direction, represented by a graphical arrow on their tile - this will allow us to know in which direction to slide their panes, so it looks like they are retreating inside the walls. To this end, we must add an additional field to `SummonCreature`.

```rust
// events.rs

#[derive(Event)]
pub struct SummonCreature {
    pub position: Position,
    pub species: Species,
    pub momentum: OrdDir, // NEW!
    pub summon_tile: Position,
}
```

```rust
// spells.rs

/// The targeted passable tiles summon a new instance of species.
fn axiom_function_summon_creature(
    mut summon: EventWriter<SummonCreature>,
    spell_stack: Res<SpellStack>,
    position: Query<&Position>,
) {
    // SNIP
            summon.send(SummonCreature {
                species,
                position: *position,
                momentum: OrdDir::Down, // NEW!
                summon_tile: *caster_position,
            });
    // SNIP
}
```

We'll need to ensure all newly spawned creatures start with their proper momentum, both graphically and in game logic.

```rust
// events.rs

/// Place a new Creature on the map of Species and at Position.
pub fn summon_creature(
    mut commands: Commands,
    mut events: EventReader<SummonCreature>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    map: Res<Map>,
) {
    for event in events.read() {
            // SNIP
            Creature {
                // SNIP
                momentum: event.momentum, // CHANGED - no longer defaults to Down
                health: // SNIP
            },
            // NEW!
            Transform {
                translation: Vec3 {
                    x: event.summon_tile.x as f32 * 64.,
                    y: event.summon_tile.y as f32 * 64.,
                    z: 0.,
                },
                rotation: Quat::from_rotation_z(match event.momentum {
                    OrdDir::Down => 0.,
                    OrdDir::Right => PI / 2.,
                    OrdDir::Up => PI,
                    OrdDir::Left => 3. * PI / 2.,
                }),
                scale: Vec3::new(1., 1., 1.),
            },
            // End NEW.
            SlideAnimation,
        ));
```

Before proceeding, we'll register absolutely everything we've added before we forget!

```rust
// sets.rs
        app.add_systems(
            Update,
            ((
                summon_creature,
                register_creatures,
                teleport_entity,
                creature_collision, // NEW!
                alter_momentum, // NEW!
                harm_creature, // NEW!
                remove_creature, // NEW!
                end_turn.run_if(spell_stack_is_empty),
            )
                .chain())
            .in_set(ResolutionPhase),
        );
```

```rust
// events.rs

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SummonCreature>();
        app.init_resource::<Events<EndTurn>>();
        app.add_event::<TeleportEntity>();
        app.add_event::<CreatureCollision>(); // NEW!
        app.add_event::<AlterMomentum>(); // NEW!
        app.add_event::<HarmCreature>(); // NEW!
        app.add_event::<OpenDoor>(); // NEW!
        app.add_event::<RemoveCreature>(); // NEW!
        app.init_resource::<Events<CreatureStep>>();
        // NEW!
        app.insert_resource(TurnManager {
            turn_count: 0,
            action_this_turn: PlayerAction::Invalid,
        });
        // End NEW.
    }
}
```

Try `cargo run`.

You'll find everything in working order: the sprite rotations when walking around, the inescapable cage, and melee attacking the pesky denizens intruding on your personal space... but everyone is nicely isolated in their own cubicles, unable to transit between the four quadrants.

We'll fix that. When a door is opened, it will become `Intangible`, which means it will be removed from the `Map`. It will still exist, but will no longer be included in any collisions or spell targeting.

```rust
// creature.rs

#[derive(Component)]
pub struct Intangible;
```

```rust
// map.rs

/// Newly spawned creatures earn their place in the HashMap.
pub fn register_creatures(
    mut map: ResMut<Map>,
    // Any entity that has a Position that just got added to it -
    // currently only possible as a result of having just been spawned in.
    displaced_creatures: Query<(&Position, Entity), (Added<Position>, With<Species>)>,
    intangible_creatures: Query<&Position, (Added<Intangible>, With<Species>)>, // NEW!
    tangible_creatures: Query<&Position, With<Species>>, // NEW!
    mut tangible_entities: RemovedComponents<Intangible>, // NEW!
) {
    for (position, entity) in displaced_creatures.iter() {
        // Insert the new creature in the Map. Position implements Copy,
        // so it can be dereferenced (*), but `.clone()` would have been
        // fine too.
        map.creatures.insert(*position, entity);
    }

    // NEW!
    // Newly intangible creatures are removed from the map.
    for intangible_position in intangible_creatures.iter() {
        map.creatures.remove(intangible_position);
    }

    // A creature recovering its tangibility is added to the map.
    for entity in tangible_entities.read() {
        let tangible_position = tangible_creatures.get(entity).unwrap();
        if map.creatures.get(tangible_position).is_some() {
            panic!("A creature recovered its tangibility while on top of another creature!");
        }
        map.creatures.insert(*tangible_position, entity);
    }
    // End NEW.
}
```

```rust
// events.rs

pub fn open_door(
    mut events: EventReader<OpenDoor>,
    mut commands: Commands,
    mut door: Query<(&mut Visibility, &Position, &OrdDir)>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    for event in events.read() {
        // Gather component values of the door.
        let (mut visibility, position, orientation) = door.get_mut(event.entity).unwrap();
        // The door becomes intangible, and can be walked through.
        commands.entity(event.entity).insert(Intangible);
        // The door is no longer visible, as it is open.
        *visibility = Visibility::Hidden;
        // Find the direction in which the door was facing to play its animation correctly.
        let (offset_1, offset_2) = match orientation {
            OrdDir::Up | OrdDir::Down => (OrdDir::Left.as_offset(), OrdDir::Right.as_offset()),
            OrdDir::Right | OrdDir::Left => (OrdDir::Down.as_offset(), OrdDir::Up.as_offset()),
        };
        // Loop twice: for each pane of the door.
        for offset in [offset_1, offset_2] {
            commands.spawn((
                // The sliding panes are represented as a MagicEffect with a very slow decay.
                MagicEffect {
                    // The panes slide into the adjacent walls to the door, hence the offset.
                    position: Position::new(position.x + offset.0, position.y + offset.1),
                    sprite: Sprite {
                        image: asset_server.load("spritesheet.png"),
                        custom_size: Some(Vec2::new(64., 64.)),
                        texture_atlas: Some(TextureAtlas {
                            layout: atlas_layout.handle.clone(),
                            index: get_effect_sprite(&EffectType::Airlock),
                        }),
                        ..default()
                    },
                    visibility: Visibility::Inherited,
                    vfx: MagicVfx {
                        appear: Timer::from_seconds(0., TimerMode::Once),
                        // Very slow decay - the alpha shouldn't be reduced too much
                        // while the panes are still visible.
                        decay: Timer::from_seconds(3., TimerMode::Once),
                    },
                },
                // Ensure the panes are sliding.
                SlideAnimation,
                Transform {
                    translation: Vec3 {
                        x: position.x as f32 * 64.,
                        y: position.y as f32 * 64.,
                        // The pane needs to hide under actual tiles, such as walls.
                        z: -1.,
                    },
                    // Adjust the pane's rotation with its door.
                    rotation: Quat::from_rotation_z(match orientation {
                        OrdDir::Down => 0.,
                        OrdDir::Right => PI / 2.,
                        OrdDir::Up => PI,
                        OrdDir::Left => 3. * PI / 2.,
                    }),
                    scale: Vec3::new(1., 1., 1.),
                },
            ));
        }
    }
}
```
