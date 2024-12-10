+++
title = "Bevy Traditional Roguelike Quick-Start - 6. Let Chaos Reign"
date = 2024-12-10
authors = ["Julien Robert"]
[taxonomies]
tags = ["rust", "bevy", "tutorial"]
+++

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
pub fn player_step(
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

If you `cargo run` again, everything will work as planned.

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

{{ image(src="https://raw.githubusercontent.com/Oneirical/oneirical.github.io/main/6-let-chaos-reign/laser.gif", alt="The Hunter, now with a knockback laser of its own which shoots at walls, then the player.",
         position="center", style="border-radius: 8px;") }}

# Magical Barricades

To conclude this chapter, we'll tie in `SummonCreature` with spells that call upon this event on demand!

Before anything else, we'll need to know *who* is summoning *what*, which can be solved by adding a pretty animation for which we already have all necessary components.

```rust
// events.rs

#[derive(Event)]
pub struct SummonCreature {
    pub position: Position,
    pub species: Species,
    pub summon_tile: Position, // NEW!
}
```

When a creature is summoned, they will now visibly move from their summoner to their assigned tile, giving a feel like they are being "thrown out" by the caster. We'll just need to add `Transform` and `SlideAnimation`:

```rust
// events.rs
pub fn summon_creature(/* SNIP */) {
    // SNIP
    let mut new_creature = commands.spawn(( // CHANGED - added "("
        Creature {
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
        },
        // NEW!
        Transform::from_xyz(
            event.summon_tile.x as f32 * 64.,
            event.summon_tile.y as f32 * 64.,
            0.,
        ),
        SlideAnimation,
        // End NEW.
    )); // CHANGED - added ")"

```

This is a great example of Bevy's signature ECS modularity - once the building blocks of your game are well established, tacking on a few labels is all you need to radically change the behaviour of some Entities. Creatures will start with their sprite visually placed by `Transform`, moving towards their real tile position with `SlideAnimation`.

Fix the fields in `summon_cage`.

```rust
// map.rs
fn summon_cage(/* SNIP */) {
    // SNIP
    summon.send(SummonCreature {
        species,
        position,
        summon_tile: Position::new(4, 4), // NEW!
    });
```

We may now add the spell itself.

```rust
// spells.rs
#[derive(Debug, Clone)]
/// There are Form axioms, which target certain tiles, and Function axioms, which execute an effect
/// onto those tiles.
pub enum Axiom {

    // SNIP

    // FUNCTIONS
    /// The targeted creatures dash in the direction of the caster's last move.
    Dash { max_distance: i32 },

    // NEW!
    /// The targeted passable tiles summon a new instance of species.
    SummonCreature { species: Species },
    // End NEW.
}

```

```rust
// spells.rs
impl FromWorld for AxiomLibrary {
    fn from_world(world: &mut World) -> Self {
        let mut axioms = AxiomLibrary {
            library: HashMap::new(),
        };
        // SNIP
        // NEW!
        axioms.library.insert(
            discriminant(&Axiom::SummonCreature {
                species: Species::Player,
            }),
            world.register_system(axiom_function_summon_creature),
        );
        // End NEW.
        axioms
    }
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
    let synapse_data = spell_stack.spells.last().unwrap();
    let caster_position = position.get(synapse_data.caster).unwrap();
    if let Axiom::SummonCreature { species } = synapse_data.axioms[synapse_data.step] {
        for position in &synapse_data.targets {
            summon.send(SummonCreature {
                species,
                position: *position,
                summon_tile: *caster_position,
            });
        }
    } else {
        panic!()
    }
}
```

If you now modify the Hunter's spellcasting like so:

```rust
// events.rs
pub fn end_turn(/* SNIP */) {

    // SNIP

    spell.send(CastSpell {
        caster: hunter_entity,
        spell: Spell {
            axioms: vec![
            // CHANGED
                Axiom::MomentumBeam,
                Axiom::SummonCreature {
                    species: Species::Wall,
                },
            // End CHANGED.
            ],
        },
    });
```

You'll find (after `cargo run`) a green friend who seems a little too enthusiastic about modern architecture.

{{ image(src="https://raw.githubusercontent.com/Oneirical/oneirical.github.io/main/6-let-chaos-reign/archi.gif", alt="The Hunter, using its laser to fill the cage with additional walls",
         position="center", style="border-radius: 8px;") }}

To up the stakes, we'll now add a new Form `Axiom` and a new `Species` who will use it.

```rust
// spells.rs
pub enum Axiom {
    // FORMS
    
    // SNIP

    // NEW!
    /// Target a ring of `radius` around the caster.
    Halo { radius: i32 },
    // End NEW.
```

```rust
// spells.rs
impl FromWorld for AxiomLibrary {
    fn from_world(world: &mut World) -> Self {
        let mut axioms = AxiomLibrary {
            library: HashMap::new(),
        };
        // SNIP
        // NEW!
        axioms.library.insert(
            discriminant(&Axiom::Halo { radius: 1 }),
            world.register_system(axiom_form_halo),
        );
        // End NEW.
```

```rust
// spells.rs
/// Target a ring of `radius` around the caster.
fn axiom_form_halo(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let caster_position = position.get(synapse_data.caster).unwrap();
    if let Axiom::Halo { radius } = synapse_data.axioms[synapse_data.step] {
        let mut circle = circle_around(caster_position, radius);
        // Sort by clockwise rotation.
        circle.sort_by(|a, b| {
            let angle_a = angle_from_center(caster_position, a);
            let angle_b = angle_from_center(caster_position, b);
            angle_a.partial_cmp(&angle_b).unwrap()
        });
        // Add some visual halo effects.
        magic_vfx.send(PlaceMagicVfx {
            targets: circle.clone(),
            sequence: EffectSequence::Sequential { duration: 0.04 },
            effect: EffectType::GreenBlast,
            decay: 0.5,
            appear: 0.,
        });
        // Add these tiles to `targets`.
        synapse_data.targets.append(&mut circle);
    } else {
        panic!()
    }
}

/// Generate the points across the outline of a circle.
fn circle_around(center: &Position, radius: i32) -> Vec<Position> {
    let mut circle = Vec::new();
    for r in 0..=(radius as f32 * (0.5f32).sqrt()).floor() as i32 {
        let d = (((radius * radius - r * r) as f32).sqrt()).floor() as i32;
        let adds = [
            Position::new(center.x - d, center.y + r),
            Position::new(center.x + d, center.y + r),
            Position::new(center.x - d, center.y - r),
            Position::new(center.x + d, center.y - r),
            Position::new(center.x + r, center.y - d),
            Position::new(center.x + r, center.y + d),
            Position::new(center.x - r, center.y - d),
            Position::new(center.x - r, center.y + d),
        ];
        for new_add in adds {
            if !circle.contains(&new_add) {
                circle.push(new_add);
            }
        }
    }
    circle
}

/// Find the angle of a point on a circle relative to its center.
fn angle_from_center(center: &Position, point: &Position) -> f64 {
    let delta_x = point.x - center.x;
    let delta_y = point.y - center.y;
    (delta_y as f64).atan2(delta_x as f64)
}
```

Create a circle, then rotate around it in a clockwise maneer so the animation looks pretty. If you are curious about my circle-making function, I highly recommend [Red Blob Game's entry on the topic](https://www.redblobgames.com/grids/circle-drawing/).

Now, for the new species:

```rust
// creature.rs
#[derive(Debug, Component, Clone, Copy)]
pub enum Species {
    Player,
    Wall,
    Hunter,
    Spawner, // NEW!
}

/// Get the appropriate texture from the spritesheet depending on the species type.
pub fn get_species_sprite(species: &Species) -> usize {
    match species {
        Species::Player => 0,
        Species::Wall => 3,
        Species::Hunter => 4,
        Species::Spawner => 5, // NEW!
    }
}
```

```rust
// events.rs
/// Place a new Creature on the map of Species and at Position.
pub fn summon_creature(/* SNIP */) {

        // SNIP

        // Add any species-specific components.
        match &event.species {
            Species::Player => {
                new_creature.insert(Player);
            }
            Species::Hunter | Species::Spawner => { // CHANGED: Added Spawner.
                new_creature.insert(Hunt);
            }
            _ => (),
        }
```

And for its spellcasting:

```rust
// events.rs
pub fn end_turn(
    // SNIP
    hunters: Query<(Entity, &Position, &Species), (With<Hunt>, Without<Player>)>, // CHANGED: Added Species.
    map: Res<Map>,
) {
    for _event in events.read() {
        turn_count.turns += 1;
        let player_pos = player.get_single().unwrap();
        for (hunter_entity, hunter_pos, hunter_species) in hunters.iter() { // CHANGED: Added hunter_species.
            // Occasionally cast a spell.
            if turn_count.turns % 5 == 0 {
                // NEW!
                match hunter_species {
                    Species::Hunter => {
                        spell.send(CastSpell {
                            caster: hunter_entity,
                            spell: Spell {
                                axioms: vec![Axiom::MomentumBeam, Axiom::Dash { max_distance: 5 }],
                            },
                        });
                    }
                    Species::Spawner => {
                        spell.send(CastSpell {
                            caster: hunter_entity,
                            spell: Spell {
                                axioms: vec![
                                    Axiom::Halo { radius: 3 },
                                    Axiom::SummonCreature {
                                        species: Species::Hunter,
                                    },
                                ],
                            },
                        });
                    }
                    _ => (),
                }
                // End NEW.
            }
            // Try to find a tile that gets the hunter closer to the player.
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

That's right - halo summoning of Hunters every 5 turns, who all have knockback beams. Whatever it is you are imagining right now, it is nowhere as glorious as the pandemonium about to be unleashed.

```rust
// map.rs
fn spawn_cage(mut summon: EventWriter<SummonCreature>) {
    // CHANGED
    let cage = ".........\
                .........\
                ....S....\
                .........\
                .........\
                .........\
                ....@....\
                .........\
                .........";
    // End CHANGED.
    for (idx, tile_char) in cage.char_indices() {
        let position = Position::new(idx as i32 % 9, idx as i32 / 9);
        let species = match tile_char {
            '#' => Species::Wall,
            'H' => Species::Hunter,
            'S' => Species::Spawner, // NEW!
            '@' => Species::Player,
            _ => continue,
        };
```

`cargo run`, and LET CHAOS REIGN.

{{ image(src="https://raw.githubusercontent.com/Oneirical/oneirical.github.io/main/6-let-chaos-reign/chaos.gif", alt="The Spawner creating an armada of Hunters, which then proceed to laser everything and cause chaotic knockback fun!",
         position="center", style="border-radius: 8px;") }}
