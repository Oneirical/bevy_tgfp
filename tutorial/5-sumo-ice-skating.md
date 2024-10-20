Thrilling to be jumping, sliding and bashing in fancy acrobatics, but quite lacking in eye candy. Creatures merely blink from one point to another, without any style or intrigue. Animation is a complex topic, but making creatures properly "dash" from one point to another is certainly doable with as little as one new resource, and a rework of `adjust_transforms`.

```rust
// graphics.rs
#[derive(Resource)]
pub struct SlideAnimation {
    pub elapsed: Timer,
}
```

```rust
// graphics.rs
impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        // NEW!
        app.insert_resource(SlideAnimation {
            elapsed: Timer::from_seconds(0.4, TimerMode::Once),
        });
        // End NEW.
        app.insert_resource(Msaa::Off);
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
    }
}
```

This `Resource` will be used to add a 0.4 second delay after each creature motion, during which the entities will slide from their origin point to their destination. Each time a `TeleportEntity` event occurs, this timer will reset, allowing the animation to unfold for each move.

```rust
// events.rs
fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut creature: Query<&mut Position>,
    mut map: ResMut<Map>,
    mut animation_timer: ResMut<SlideAnimation>, // NEW!
) {
    for event in events.read() {
        let mut creature_position = creature
            // Get the Position of the Entity targeted by TeleportEntity.
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        // If motion is possible...
        if map.is_passable(event.destination.x, event.destination.y) {
            // ...update the Map to reflect this...
            map.move_creature(*creature_position, event.destination);
            // NEW!
            // ...begin the sliding animation...
            animation_timer.elapsed.reset();
            // End NEW.
            // ...and move that Entity to TeleportEntity's destination tile.
            creature_position.update(event.destination.x, event.destination.y);
        } else {
            // Nothing here just yet, but this is where collisions between creatures
            // will be handled.
            continue;
        }
    }
}
```

Now... for the main course.

```rust
fn adjust_transforms(
    mut creatures: Query<(&Position, &mut Transform, Has<Player>)>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Position>)>,
    // NEW!
    mut animation_timer: ResMut<SlideAnimation>,
    time: Res<Time>,
    // End NEW.
) {
    // NEW!
    let fraction_before_tick = animation_timer.elapsed.fraction();
    animation_timer.elapsed.tick(time.delta());
    // Calculate what % of the animation has elapsed during this tick.
    let fraction_ticked = animation_timer.elapsed.fraction() - fraction_before_tick;
    // End NEW.
    for (pos, mut trans, is_player) in creatures.iter_mut() {
        // DELETE
        // // Multiplied by the graphical size of a tile, which is 64x64.
        // trans.translation.x = pos.x as f32 * 64.;
        // trans.translation.y = pos.y as f32 * 64.;
        // End DELETE

        // NEW!
        // The distance between where a creature CURRENTLY is,
        // and the destination of a creature's movement.
        // Multiplied by the graphical size of a tile, which is 64x64.
        let (dx, dy) = (
            pos.x as f32 * 64. - trans.translation.x,
            pos.y as f32 * 64. - trans.translation.y,
        );
        // The distance between the original position and the destination position.
        let (ori_dx, ori_dy) = (
            dx / animation_timer.elapsed.fraction_remaining(),
            dy / animation_timer.elapsed.fraction_remaining(),
        );
        // The sprite approaches its destination.
        trans.translation.x = bring_closer_to_target_value(
            trans.translation.x,
            ori_dx * fraction_ticked,
            pos.x as f32 * 64.,
        );
        trans.translation.y = bring_closer_to_target_value(
            trans.translation.y,
            ori_dy * fraction_ticked,
            pos.y as f32 * 64.,
        );
        // End NEW.
        if is_player {
            // The camera follows the player.
            let mut camera_trans = camera.get_single_mut().unwrap();
            (camera_trans.translation.x, camera_trans.translation.y) =
                (trans.translation.x, trans.translation.y);
        }
    }
}
```

Each tick, this system runs... but we cannot know for sure how long a tick is! A computer being turned into a localized micro-Sun due to compiling Bevy in the background while playing our game will see its Frames-Per-Seconds drop, and increase the time elapsed per tick. Therefore, the new first three lines calculate which % of the animation has been processed this tick - stored within `fraction_ticked`.

Let's say that our hero `@` is moving to `X`.

```
...
@.X
...
```

Each tile is 64x64 pixels. Right now, ̀`@` is `(128, 0)` pixels away from its destination, which is the tuple `(dx, dy)`. We need to keep track of this original value! As it approaches its goal, the distance will decrease, but our calculations must be based on the original distance.

Later on, when we reach this point, 0.2 seconds later:

```
...
.@X
...
```

`(dx, dy)` is now `(64, 0)̀`. The fraction elapsed of the timer is 50%. `64 / 0.5 = 128`, meaning the original distance is restored - stored in `(ori_dx, ori_dy)`.

Finally, the `Transform` component is adjusted. If the original distance was 128 and the fraction elapsed this tick is 3%, then the creature will move 3.84 pixels to the right this tick!

In order to avoid little visual "bumps" (in the cases where a creature is, say, at 127.84 pixels, and moves 5 pixels to the right, overshooting its objective), I also added the `bring_closer_to_target_value` function, preventing any increases past the limit no matter if that limit is negative or positive.

```rust
// graphics.rs
fn bring_closer_to_target_value(value: f32, adjustment: f32, target_value: f32) -> f32 {
    let adjustment = adjustment.abs();
    if value > target_value {
        (value - adjustment).max(target_value)
    } else if value < target_value {
        (value + adjustment).min(target_value)
    } else {
        target_value // Value is already at target.
    }
}
```

Finally, `cargo run`, and behold these smooth and graceful motions!

// TODO gif

# The Summoning Circle

A single test subject is insufficient - in research, experiments must be reproducible. We will need *industrial quantities* of these pesky Hunters.

Enter: the Spawner. As this is a fourth, different creature type, it is about time we gave them some distinction beyond merely different sprites.

```rust
// creature.rs
#[derive(Debug, Component, Clone, Copy)]
pub enum Species {
    Player,
    Wall,
    Hunter,
    Spawner,
}

/// Get the appropriate texture from the spritesheet depending on the species type.
pub fn get_species_sprite(species: &Species) -> usize {
    match species {
        Species::Player => 0,
        Species::Wall => 3,
        Species::Hunter => 4,
        Species::Spawner => 75,
    }
}
```

This will be a new component for all `Creaturè`s.

```rust
// creature.rs
#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub momentum: OrdDir,
    pub species: Species, // NEW!
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
}
```

Immediately, `map.rs` will start screaming at you that its `Creature`s need the new component. Leave their prayers unanswered. This part of the code is long overdue for a refactor, as `spawn_player` and `spawn_cage` are repeating each other for little reason. This can be funnelled down a new event, `SummonCreature`, which will handle both initial map generation and new arrivals during the game, possibly from summoning spells or such.

```rust
// events.rs
#[derive(Event)]
pub struct SummonCreature {
    pub species: Species,
    pub position: Position,
}

/// Place a new Creature on the map of Species and at Position.
pub fn summon_creature(
    mut commands: Commands,
    mut events: EventReader<SummonCreature>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    map: Res<Map>,
) {
    for event in events.read() {
        // Avoid summoning if the tile is already occupied.
        if !map.is_passable(event.position.x, event.position.y) {
            continue;
        }
        let mut new_creature = commands.spawn(Creature {
            position: event.position,
            species: event.species,
            sprite: SpriteBundle {
                texture: asset_server.load("spritesheet.png"),
                transform: Transform::from_scale(Vec3::new(4., 4., 0.)),
                ..default()
            },
            atlas: TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index: get_species_sprite(&event.species),
            },
            momentum: OrdDir::Up,
        });
        // Add any species-specific components.
        match &event.species {
            Species::Player => {
                new_creature.insert(Player);
            }
            Species::Hunter => {
                new_creature.insert(Hunt);
            }
            _ => (),
        }
    }
}
```

You may notice that this implementation is extremely similar to `spawn_player` and `spawn_cage`. Speaking of those, let us unify them under this new event. There will now be a single `spawn_cage` function, looking like this:

```rust
// map.rs

// DELETE spawn_player

fn spawn_cage(mut summon: EventWriter<SummonCreature>) {
    let cage = "#########\
                #H......#\
                #.......#\
                #.......#\
                #...S...#\
                #.......#\
                #...@...#\
                #.......#\
                #########";
    for (idx, tile_char) in cage.char_indices() {
        let position = Position::new(idx as i32 % 9, idx as i32 / 9);
        let species = match tile_char {
            '#' => Species::Wall,
            'H' => Species::Hunter,
            '@' => Species::Player,
            'S' => Species::Spawner,
            _ => continue,
        };
        summon.send(SummonCreature { species, position });
    }
}
```

Don't forget to de-register `spawn_player`. As if the compiler wasn't already complaining about it...

```rust
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map {
            creatures: HashMap::new(),
        });
        // app.add_systems(Startup, spawn_player); // DELETE!
        app.add_systems(Startup, spawn_cage);
        app.add_systems(Update, register_creatures);
    }
}
```

Oh, what else did we forget? That's right, registering `SummonCreature`.

```rust
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerStep>();
        app.add_event::<TeleportEntity>();
        app.add_event::<AlterMomentum>();
        app.add_event::<SummonCreature>(); // NEW!
        app.add_systems(Update, player_step);
        app.add_systems(Update, teleport_entity);
        app.add_systems(Update, alter_momentum);
        app.add_systems(Update, summon_creature); // NEW!
    }
}
```

If you `cargo run` now, nothing will appear to have changed, aside from the ominous new spawner in the centre... wait, what is that? An instant panic on startup?

`spawn_cage` runs on `Startup`, sends out some `SummonCreature` events, and then, `summon_creature`, an `Update` system, should handle the rest... right? Wrong!

Bevy's systems are **non-deterministic**. Anything marked as `Update` can run at *any* time. Here, `player_step` occasionally runs before `summon_creature` has made the player exist at all, and the first line tries to fetch a non-existing player.

We'll fix this for now by bumping this line into the event loop itself, preventing it from fetching the player entity when there is no `PlayerStep` event yet. This is a rough fix (if you were sending inputs every millisecond as the game was booting up, you could still manage to try to make a non-existent player step 1 tile). However, it will do for now, until this tutorial touches on **explicit system ordering**.

```rust
// events.rs
fn player_step(
    mut events: EventReader<PlayerStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut momentum: EventWriter<AlterMomentum>,
    player: Query<(Entity, &Position), With<Player>>,
    hunters: Query<(Entity, &Position), With<Hunt>>,
    map: Res<Map>,
) {
    // let (player_entity, player_pos) = player.get_single().expect("0 or 2+ players"); // DELETE!
    for event in events.read() {
        let (player_entity, player_pos) = player.get_single().expect("0 or 2+ players"); // NEW!
```

`cargo run`, one more time. There we go.

// TODO image

# Industrial Production

Now. To make this red thing actually accomplish its purpose, we will need some new spell logic.

```rust
// spells.rs
#[derive(Debug, Clone)]
/// There are Form axioms, which target certain tiles, and Function axioms, which execute an effect
/// onto those tiles.
pub enum Axiom {
    // FORMS

    // Target the caster's tile.
    Ego,

    // NEW!
    // Target all orthogonally adjacent tiles to the caster.
    Plus,
    // End NEW.
    
    // Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
    // including the first solid tile encountered, which stops the beam.
    MomentumBeam,
```

The Form goes first, alongside its implementation:

```rust
// spells.rs
impl Axiom {
    fn target(&self, synapse_data: &mut SynapseData, map: &Map) {
        match self {
            // Target the caster's tile.
            Self::Ego => {
                synapse_data.targets.push(synapse_data.caster_position);
            }
            
            // NEW!
            // Target all orthogonally adjacent tiles to the caster.
            Self::Plus => {
                let adjacent = [OrdDir::Up, OrdDir::Right, OrdDir::Down, OrdDir::Left];
                for direction in adjacent {
                    let mut new_pos = synapse_data.caster_position;
                    let offset = direction.as_offset();
                    new_pos.shift(offset.0, offset.1);
                    synapse_data.targets.push(new_pos);
                }
            }
            // End NEW.
```

Then, the Function:

```rust
    // FUNCTIONS

    // The targeted creatures dash in the direction of the caster's last move.
    Dash,

    // NEW!
    // The targeted passable tiles summon a new instance of species.
    SummonCreature { species: Species },
    // End NEW.
```

It generates a new `EventDispatch`...

```rust
// spells.rs
    fn execute(&self, synapse_data: &mut SynapseData, map: &Map) -> bool {
        match self {

            // SNIP
        
            // The targeted passable tiles summon a new instance of species.
            Self::SummonCreature { species } => {
                for position in &synapse_data.targets {
                    synapse_data.effects.push(EventDispatch::SummonCreature {
                        species: *species,
                        position: *position,
                    });
                }
                true
            }
```

...which is implemented with a little bit of boilerplate.

```rust
// spells.rs
/// An enum with replicas of common game Events, to be translated into the real Events
/// and dispatched to the main game loop.
#[derive(Clone, Copy)]
pub enum EventDispatch {
    TeleportEntity {
        destination: Position,
        entity: Entity,
    },
    // NEW!
    SummonCreature {
        species: Species,
        position: Position,
    },
    // End NEW.
}
```

```rust
// spells.rs
/// Translate a list of EventDispatch into their "real" Event counterparts and send them off
/// into the main game loop to modify the game's creatures.
pub fn dispatch_events(
    mut receiver: EventReader<SpellEffect>,
    mut teleport: EventWriter<TeleportEntity>,
    mut summon: EventWriter<SummonCreature>, // NEW!
) {
    for effect_list in receiver.read() {
        for effect in &effect_list.events {
            // Each EventDispatch enum is translated into its Event counterpart.
            match effect {
                // SNIP
                // NEW!
                EventDispatch::SummonCreature { species, position } => {
                    summon.send(SummonCreature {
                        species: *species,
                        position: *position,
                    });
                }
                // End NEW.
```

Finally, we'll actually give this spell to the ominous Spawner. Right now, the code that lets other entities act after the player is quite unwieldy:

```rust
// Do not add this code, it is already here.
// events.rs
fn player_step( //SNIP
) {
        for (hunter_entity, hunter_pos) in hunters.iter() {
            // Try to find a tile that gets the hunter closer to the player.
            if let Some(move_target) = map.best_manhattan_move(*hunter_pos, *player_pos) {
                // If it is found, cause another TeleportEntity event.
                teleporter.send(TeleportEntity {
                    destination: move_target,
                    entity: hunter_entity,
                });
            }
        }
```

There are numerous problems:

1. This only seeks creatures with the `Hunt` component.
2. This happens only when the player steps, not when they take a different action, such as spellcasting.
3. This does not modify the "momentum" of the hunters in case we ever want to give momentum-reliant spells to NPCs (we will).

This can be solved with a much more robust `EndTurn` event.

```rust
// events.rs
#[derive(Event)]
pub struct EndTurn;

fn end_turn(
    mut events: EventReader<EndTurn>,
    mut step: EventWriter<CreatureStep>,
    mut spell: EventWriter<CastSpell>,
    npcs: Query<(Entity, &Position, &Species), Without<Player>>,
    player: Query<&Position, With<Player>>,
    map: Res<Map>,
    animation_timer: Res<SlideAnimation>,
    mut momentum: EventWriter<AlterMomentum>,
) {
    // Wait for the player's action to complete before starting NPC turns.
    if !animation_timer.elapsed.finished() {
        return;
    }
    for _event in events.read() {
        let player_pos = player.get_single().unwrap();
        for (creature_entity, creature_position, creature_species) in npcs.iter() {
            match creature_species {
                Species::Hunter => {
                    // Try to find a tile that gets the hunter closer to the player.
                    if let Some(move_target) =
                        map.best_manhattan_move(*creature_position, *player_pos)
                    {
                        // If it is found, the hunter approaches the player by stepping.
                        step.send(CreatureStep {
                            direction: OrdDir::as_variant(
                                move_target.x - creature_position.x,
                                move_target.y - creature_position.y,
                            ),
                            entity: creature_entity,
                        });
                    }
                }
                Species::Spawner => {
                    // Cast a spell which tries to summon Hunters on all orthogonally
                    // adjacent tiles.
                    spell.send(CastSpell {
                        caster: creature_entity,
                        spell: Spell {
                            axioms: vec![
                                Axiom::Plus,
                                Axiom::SummonCreature {
                                    species: Species::Hunter,
                                },
                            ],
                        },
                    });
                }
                _ => (),
            }
        }
    }
}
```

Everything in this block should already have an implementation, with the exception of two things. First, a simple helper to transition from `(i32, i32)` to `OrdDir`...

```rust
// main.rs
impl OrdDir {

    // SNIP

    // NEW!
    pub fn as_variant(dx: i32, dy: i32) -> Self {
        match (dx, dy) {
            (0, 1) => OrdDir::Up,
            (0, -1) => OrdDir::Down,
            (1, 0) => OrdDir::Right,
            (-1, 0) => OrdDir::Left,
            _ => panic!("Invalid offset provided."),
        }
    }
    // End NEW.
}
```

And, also, `CreatureStep`. That is right - we will also unify together player and NPC stepping logic, so they become restricted by the same rules.

This will replace the old `player_step`...

```rust
// events.rs

// DELETE player_step and PlayerStep

#[derive(Event)]
pub struct CreatureStep {
    pub entity: Entity,
    pub direction: OrdDir,
}

fn creature_step(
    mut events: EventReader<CreatureStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut momentum: EventWriter<AlterMomentum>,
    mut turn_end: EventWriter<EndTurn>,
    creature: Query<(&Position, Has<Player>)>,
) {
    for event in events.read() {
        let (creature_pos, is_player) = creature.get(event.entity).unwrap();
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportEntity::new(
            event.entity,
            creature_pos.x + off_x,
            creature_pos.y + off_y,
        ));

        momentum.send(AlterMomentum {
            entity: event.entity,
            direction: event.direction,
        });
        // If this creature was the player, this will end the turn.
        if is_player {
            turn_end.send(EndTurn);
        }
    }
}
```

Now, time to register everything.

```rust
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreatureStep>(); // CHANGED to CreatureStep
        app.add_event::<TeleportEntity>();
        app.add_event::<AlterMomentum>();
        app.add_event::<SummonCreature>();
        app.add_event::<EndTurn>(); // NEW!
        app.add_systems(Update, creature_step); // CHANGED to creature_step
        app.add_systems(Update, teleport_entity);
        app.add_systems(Update, alter_momentum);
        app.add_systems(Update, summon_creature);
        app.add_systems(Update, end_turn); // NEW!
    }
}
```

Finally, our `input.rs` file needs to be adapted to these new changes.

```rust

/// Each frame, if a button is pressed, move the player 1 tile.
fn keyboard_input(
    player: Query<Entity, With<Player>>,
    mut events: EventWriter<CreatureStep>, // CHANGED to CreatureStep
    input: Res<ButtonInput<KeyCode>>,
    mut spell: EventWriter<CastSpell>,
    mut turn_end: EventWriter<EndTurn>, // NEW!
    animation_timer: Res<SlideAnimation>, // NEW!
) {
    // NEW!
    // Do not accept input until the animations have finished.
    if !animation_timer.elapsed.finished() {
        return;
    }
    // Wrap everything in an if statement to find the player entity.
    if let Ok(player) = player.get_single() {
    // End NEW.
        if input.just_pressed(KeyCode::Space) {
            spell.send(CastSpell {
                caster: player, // CHANGED to use the player entity.
                spell: Spell {
                    axioms: vec![Axiom::Ego, Axiom::Dash],
                },
            });
            turn_end.send(EndTurn); // NEW!
        }
        if input.just_pressed(KeyCode::KeyW) {
            events.send(CreatureStep { // CHANGED to CreatureStep.
                entity: player, // NEW!
                direction: OrdDir::Up,
            });
        }
        if input.just_pressed(KeyCode::KeyD) {
            events.send(CreatureStep { // CHANGED to CreatureStep.
                entity: player, // NEW!
                direction: OrdDir::Right,
            });
        }
        if input.just_pressed(KeyCode::KeyA) {
            events.send(CreatureStep { // CHANGED to CreatureStep.
                entity: player, // NEW!
                direction: OrdDir::Left,
            });
        }
        if input.just_pressed(KeyCode::KeyS) {
            events.send(CreatureStep { // CHANGED to CreatureStep.
                entity: player, // NEW!
                direction: OrdDir::Down,
            });
        }
    }
}
```

`cargo run`... And very weird things are happening. Everyone seems frozen, until they are not, in a completely unpredictable fashion. What?

The player steps. An `EndTurn` event is sent out. Then, `end_turn` gets locked for a while waiting for the animation to complete. During that time, the `EndTurn` event has been "garbage collected" by Bevy, as it's not a fan of letting unconsumed events lying around the place. If they accumulate, they can take up big chunks of memory! In fact, Bevy is very impatient, letting unattended Events exist for only **two frames**.

However, we know what we're doing, as we have a system to clean up those `EndTurn`s as soon as the animations complete. We can disable this autocollection like this:

```rust
// events.rs
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        // SNIP
        app.init_resource::<Events<EndTurn>>(); // CHANGED
        // SNIP
    }
}
```

With this alternative, `EndTurn` will now patiently wait to be consumed instead of mysteriously disappearing.

`cargo run`, and you should now be getting swarmed very, very fast.

// TODO gif
