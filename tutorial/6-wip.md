# The Summoning Circle

The more prolific programmers among readers may have been frothing at the mouth for quite some time now. Why? Well, `spawn_cage` and `spawn_player` have been sitting there since chapter 2, violating the "Don't Repeat Yourself" principle. Let us cure them of their wrath.

```rust
// map.rs

// DELETE spawn_cage and spawn_player.

// NEW!
fn spawn_cage(mut summon: EventWriter<SummonCreature>) {
    let cage = "#########\
                #H......#\
                #.......#\
                #.......#\
                #.......#\
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
            _ => continue,
        };
        summon.send(SummonCreature { species, position });
    }
}
// End NEW.
```

This new system does a couple of things:

- Funnel the spawning of the player and the cage in the same SummonCreature event, instead of having two systems doing the same thing for both.
- Introduce a new concept, `Species`.

Previously, we had only sprite indices (with the `texture_atlas`) to differentiate one creature from another. This new marker `Component` will help us know whether something is a Wall, Player, or Hunter.

```rust
// creature.rs
#[derive(Debug, Component, Clone, Copy)]
pub enum Species {
    Player,
    Wall,
    Hunter,
}

/// Get the appropriate texture from the spritesheet depending on the species type.
pub fn get_species_sprite(species: &Species) -> usize {
    match species {
        Species::Player => 0,
        Species::Wall => 3,
        Species::Hunter => 4,
    }
}
```

We will add this as a mandatory field to all new `Creature` instances.

```rust
// creature.rs
#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub momentum: OrdDir,
    pub sprite: Sprite,
    pub species: Species, // NEW!
}
```

Now, for the `SummonCreature` event proper.

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
        // Avoid summoning if the tile is already occupied.
        if !map.is_passable(event.position.x, event.position.y) {
            continue;
        }
        let mut new_creature = commands.spawn(Creature {
            position: event.position,
            species: event.species,
            sprite: Sprite {
                image: asset_server.load("spritesheet.png"),
                custom_size: Some(Vec2::new(64., 64.)),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index: get_species_sprite(&event.species),
                }),
                ..default()
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

Register the system and event.

```rust
// sets.rs
app.add_systems(
    Update,
    // CHANGED - added summon_creature
    ((summon_creature, register_creatures, teleport_entity).chain())
        .in_set(ResolutionPhase),
);
```

```rust
// events.rs
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SummonCreature>(); // NEW!
        app.add_event::<PlayerStep>();
        app.add_event::<TeleportEntity>();
    }
}
```

If you `cargo run` now, you will- wait, what is that? An instant panic on startup?

Here, `player_step` occasionally runs before `summon_creature` has made the player exist at all, and its first line tries to fetch a non-existing player.

We'll fix this by bumping this line into the event loop itself, preventing it from fetching the player entity when there is no `PlayerStep` event yet. This was intentionally written in this way to showcase an important fact: **event systems marked with Update run every tick regardless of whether their event has been triggered or not. Only the `for` loop with `events.read()` is restricted to run only when an event arrives.**

```rust
// events.rs
fn player_step(
    mut events: EventReader<PlayerStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut player: Query<(Entity, &Position, &mut OrdDir), With<Player>>,
    hunters: Query<(Entity, &Position), With<Hunt>>,
    map: Res<Map>,
) {
    // let (player_entity, player_pos, mut player_momentum) = // DELETE!
    //     player.get_single_mut().expect("0 or 2+ players"); // DELETE!
    for event in events.read() {
    // NEW!
        let (player_entity, player_pos, mut player_momentum) =
            player.get_single_mut().expect("0 or 2+ players");
    // End NEW.
```

`cargo run` again, and everything works - with seemingly no change to the game itself, but with much more flexible code!

# Leveling the Playing Field

Here's another function that you may have found limiting:

```rust
// DO NOT ADD THIS! It is already in the code.
// events.rs
pub fn player_step(
    mut events: EventReader<PlayerStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut player: Query<(Entity, &Position, &mut OrdDir), With<Player>>,
    hunters: Query<(Entity, &Position), With<Hunt>>,
    map: Res<Map>,
) {
    for event in events.read() {
        let (player_entity, player_pos, mut player_momentum) =
            player.get_single_mut().expect("0 or 2+ players");
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportEntity::new(
            player_entity,
            player_pos.x + off_x,
            player_pos.y + off_y,
        ));

        // Update the direction towards which this creature is facing.
        *player_momentum = event.direction;

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
    }
}
```

The player gets to update their momentum, while the hunters do not. Talk about unequal treatment! This system deserves to be democratized.

```rust
// events.rs

// DELETE PlayerStep and player_step.

#[derive(Event)]
pub struct CreatureStep {
    pub entity: Entity,
    pub direction: OrdDir,
}

pub fn creature_step(
    mut events: EventReader<CreatureStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut turn_end: EventWriter<EndTurn>,
    mut creature: Query<(&Position, Has<Player>, &mut OrdDir)>,
) {
    for event in events.read() {
        let (creature_pos, is_player, mut creature_momentum) =
            creature.get_mut(event.entity).unwrap();
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportEntity::new(
            event.entity,
            creature_pos.x + off_x,
            creature_pos.y + off_y,
        ));
        // Update the direction towards which this creature is facing.
        *creature_momentum = event.direction;
        // If this creature was the player, this will end the turn.
        if is_player {
            turn_end.send(EndTurn);
        }
    }
}
```

Due to this rename, you'll have to replace all instances of `PlayerStep` across the code, and also add the new field to `input.rs`.

```rust
// input.rs
if input.just_pressed(KeyCode::KeyW) {
    events.send(CreatureStep { // CHANGED to CreatureStep
        direction: OrdDir::Up,
        entity: player.get_single().unwrap(), // NEW!
    });
}
if input.just_pressed(KeyCode::KeyD) {
    events.send(CreatureStep { // CHANGED to CreatureStep
        direction: OrdDir::Right,
        entity: player.get_single().unwrap(), // NEW!
    });
}
if input.just_pressed(KeyCode::KeyA) {
    events.send(CreatureStep { // CHANGED to CreatureStep
        direction: OrdDir::Left,
        entity: player.get_single().unwrap(), // NEW!
    });
}
if input.just_pressed(KeyCode::KeyS) {
    events.send(CreatureStep { // CHANGED to CreatureStep
        direction: OrdDir::Down,
        entity: player.get_single().unwrap(), // NEW!
    });
}
```

Note the newly introduced `EndTurn`, which will ensure that each non-player character gets to perform an action after the player's action. It will also be a new system:

```rust
// events.rs
#[derive(Event)]
pub struct EndTurn;

pub fn end_turn(
    mut events: EventReader<EndTurn>,
    mut step: EventWriter<CreatureStep>,
    player: Query<&Position, With<Player>>,
    hunters: Query<(Entity, &Position), (With<Hunt>, Without<Player>)>,
    map: Res<Map>,
) {
    for _event in events.read() {
        let player_pos = player.get_single().unwrap();
        for (hunter_entity, hunter_pos) in hunters.iter() {
            // Try to find a tile that gets the hunter closer to the player.
            if let Some(move_direction) = map.best_manhattan_move(*hunter_pos, *player_pos) {
                // If it is found, cause a CreatureStep event.
                step.send(CreatureStep {
                    direction: move_direction,
                    entity: hunter_entity,
                });
            }
        }
    }
}
```

Finally, register everything.

```rust
// events.rs
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SummonCreature>();
        app.add_event::<CreatureStep>(); // CHANGED
        app.add_event::<EndTurn>(); // NEW!
        app.add_event::<TeleportEntity>();
    }
}
```

```rust
// sets.rs
app.add_systems(
    Update,
    ((
        keyboard_input,
        creature_step, // CHANGED
        cast_new_spell,
        process_axiom,
    )
        .chain())
    .in_set(ActionPhase),
);
app.add_systems(
    Update,
    ((
        summon_creature,
        register_creatures,
        teleport_entity,
        end_turn, // NEW!
    )
        .chain())
    .in_set(ResolutionPhase),
);
```

If you `cargo run` now, you'll notice something peculiar - the Hunter is completely paralyzed and does nothing. Why? All the events are in place, this makes no sense...

The key is in Bevy's background event manager. When an event is sent, Bevy will **only hold onto it for 2 frames**, then delete it if it has not been handled yet. This is to prevent clogging of the event queue by a rogue system adding tons of events that never get read, leading to major performance issues!

However, in our case, here is what happens:

- The player moves, `EndTurn` is sent, then `CreatureStep` for the Hunter.
- The player's movement animation executes over multiple frames.
- Bevy drops `CreatureStep` during the animation, as it has run out of patience.
- The animation ends.
- `creature_step` is triggered, and cries because its precious `CreatureStep` has been taken away. It does nothing.

Tell Bevy to stop being so mean by disabling its event auto-cleanup:

```rust
// events.rs
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SummonCreature>();
        app.add_event::<EndTurn>();
        app.add_event::<TeleportEntity>();
        app.init_resource::<Events<CreatureStep>>(); // CHANGED
    }
}
```

If you `cargo run` again, everything will work as planned. You'll notice a slight difference in the way turns are displayed on the screen - hunters visibly move after the player instead of undertaking a simultaneous movement.

# Lasers For Everyone

This new `end_turn` system has opened up a whole new possibility space: spells for non-player characters. 

First, we'll track the number of elapsed turns:

```rust
// events.rs
#[derive(Resource)]
pub struct TurnCount {
    turns: usize,
}
```

```rust
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SummonCreature>();
        app.add_event::<EndTurn>();
        app.add_event::<TeleportEntity>();
        app.init_resource::<Events<CreatureStep>>();
        app.insert_resource(TurnCount { turns: 0 }); // NEW!
    }
}
```

Next up, we'll make all Hunters fire a knockback laser every 5 turns.

```rust
// events.rs
pub fn end_turn(
    mut events: EventReader<EndTurn>,
    mut step: EventWriter<CreatureStep>,
    mut spell: EventWriter<CastSpell>, // NEW!
    mut turn_count: ResMut<TurnCount>, // NEW!
    player: Query<&Position, With<Player>>,
    hunters: Query<(Entity, &Position), (With<Hunt>, Without<Player>)>,
    map: Res<Map>,
) {
    for _event in events.read() {
        turn_count.turns += 1; // NEW!
        let player_pos = player.get_single().unwrap();
        for (hunter_entity, hunter_pos) in hunters.iter() {
        // NEW!
            // Occasionally cast a spell.
            if turn_count.turns % 5 == 0 {
                spell.send(CastSpell {
                    caster: hunter_entity,
                    spell: Spell {
                        axioms: vec![Axiom::MomentumBeam, Axiom::Dash { max_distance: 5 }],
                    },
                });
            }
            // Try to find a tile that gets the hunter closer to the player.
        // End NEW.
        // CHANGED: now an else if
            else if let Some(move_direction) = map.best_manhattan_move(*hunter_pos, *player_pos) {
                // If it is found, cause a CreatureStep event.

                step.send(CreatureStep {
                    direction: move_direction,
                    entity: hunter_entity,
                });
            }
        }
    }
}
```

This will have the exact same problem as `CreatureStep` - Bevy cleans up unused events after 2 frames. Remove `CastSpell` from the cleanup routine:

```rust
// spells.rs
impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<CastSpell>>(); // CHANGED
        app.init_resource::<SpellStack>();
        app.init_resource::<AxiomLibrary>();
    }
}
```

If you `cargo run` now, the Hunter will occasionally shoot lasers at you and the surrounding walls!
