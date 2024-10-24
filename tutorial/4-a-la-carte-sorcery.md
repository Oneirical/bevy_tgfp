With our only unique skill of note being moving around, it's hard to feel emotionally invested in these poor critters running in circles forever in an unbreakable cage. An inevitable component of fantasy gaming is required: magic.

Now, with the way the system is currently set up, "pressing this button to dash forwards 4 spaces" would be extremely easy. We can do better - a system which would normally be painful to implement, but which takes advantage of Rust's pattern matching and enums, as well as Bevy's system ordering... Enter - **Spell Crafting**.

> **Design Capsule**
\
Spells will be composed of a series of **Forms** and **Functions**. Forms choose tiles on the screen, and Functions execute an effect on those tiles. In the case of a lasso, for example, the Form is a projectile and the Function is getting constricted.

Create a new file, `spells.rs`.

```rust
// spells.rs
use bevy::prelude::*;

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {}
}
```

Don't forget to link it into `main.rs`.

```rust
// main.rs

// SNIP
mod spells;

// SNIP
use spells::SpellPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            SpellPlugin, // NEW!
            EventPlugin,
            GraphicsPlugin,
            MapPlugin,
            InputPlugin,
        ))
        .run();
}
```

**It is important to add `SpellPlugin` as the first plugin** for the sake of this tutorial. This will cause a minor bug later on, which will be explained and fixed afterwards.

# D.I.Y. Wizard

Now, we may start to populate this new plugin with some starting structs and enums. I named the individual components that form a `Spell` "`Axiom`" because:

1. Calling them "Components" would get confusing fast with Bevy components.
2. They are things that happen, an enforceable truth.
3. The word "Axiom" is just dripping with flair and style.

```rust
// spells.rs
#[derive(Event)]
/// Triggered when a creature (the `caster`) casts a `spell`.
pub struct CastSpell {
    pub caster: Entity,
    pub spell: Spell,
}

#[derive(Component, Clone)]
/// A spell is composed of a list of "Axioms", which will select tiles or execute an effect onto
/// those tiles, in the order they are listed.
pub struct Spell {
    pub axioms: Vec<Axiom>,
}

#[derive(Debug, Clone)]
/// There are Form axioms, which target certain tiles, and Function axioms, which execute an effect
/// onto those tiles.
pub enum Axiom {
    // FORMS

    // Target the caster's tile.
    Ego,

    // FUNCTIONS

    // The targeted creatures dash in the direction of the caster's last move.
    Dash,
}
```

We will begin with the very simple spell **"Ego, Dash"**. When cast, the caster dashes in the direction of their last move. Note that I didn't use "Self" for the self-target, because it's already taken by Rust as a keyword, and "Ego" sounds very cool.

The implementation will rely on a struct with yet another cute name: Synapses. Named after the transmission of signals between neurons, they are like a snowball rolling down a hill and accumulating debris.

When a new `SynapseData` is created, it is blank except for the fact that it knows its `caster`, how the caster moved last turn (`caster_momentum`) and where that caster is (`caster_position`). It still has no tiles to target, and no effects to execute onto the game. As it "rolls" down the list of `Axiom`s, it will accumulate `targets` and `effects` to execute on those targets. The `effects` are simply replicas of Events, like "teleport this entity here" or "summon this entity here" - named `EventDispatch`.

```rust
// spells.rs
/// The tracker of everything which determines how a certain spell will act.
struct SynapseData {
    /// Where a spell will act.
    targets: Vec<Position>,
    /// How a spell will act.
    effects: Vec<EventDispatch>,
    /// Who cast the spell.
    caster: Entity,
    /// In which direction did the caster move the last time they did so?
    caster_momentum: OrdDir,
    /// Where is the caster on the map?
    caster_position: Position,
}

impl SynapseData {
    /// Create a blank SynapseData.
    fn new(caster: Entity, caster_momentum: OrdDir, caster_position: Position) -> Self {
        SynapseData {
            targets: Vec::new(),
            effects: Vec::new(),
            caster,
            caster_momentum,
            caster_position,
        }
    }
}

/// An enum with replicas of common game Events, to be translated into the real Events
/// and dispatched to the main game loop.
pub enum EventDispatch {
    TeleportEntity {
        destination: Position,
        entity: Entity,
    },
}
```

Now, a prototype Bevy system can be written to handle all of this.

```rust
// spells.rs
/// Work through the list of Axioms of a spell, translating it into Events to launch onto the game.
fn gather_effects(
    mut cast_spells: EventReader<CastSpell>,
    mut sender: EventWriter<SpellEffect>,
    caster: Query<(&Position, &OrdDir)>,
    map: Res<Map>,
) {
    for cast_spell in cast_spells.read() {
        // First, get the list of Axioms.
        let axioms = &cast_spell.spell.axioms;
        // And the caster's position and last move direction.
        let (caster_position, caster_momentum) = caster.get(cast_spell.caster).unwrap();

        // Create a new synapse to start "rolling down the hill" accumulating targets and effects.
        let mut synapse_data =
            SynapseData::new(cast_spell.caster, *caster_momentum, *caster_position);

        // Loop through each axiom.
        for axiom in axioms.iter() {
            // For Forms, add targets.
            axiom.target(&mut synapse_data, &map);
            // For Functions, add effects that operate on those targets.
            axiom.execute(&mut synapse_data, &map);
        }

        // Once all Axioms are processed, dispatch everything to the system that will translate
        // all effects into proper events.
        sender.send(SpellEffect {
            events: synapse_data.effects,
        });
    }
}
```

You may have noticed the following:

1. The yet uninitialized `SpellEffect` event.
2. The `OrdDir` struct is not a component and does not appear on any entities, meaning `Query<(&Position, &OrdDir)>` will return nothing.
3. `axiom.target` and `axiom.execute` functions not yet existing.

Let us work through these problems one by one.

## 1. Transforming Spell Effects into Events

Our spell effects currently do nothing - they are only replicas of real Events, such as `TeleportEntity`. They need to be translated into the real deal.

```rust
// spells.rs

#[derive(Event)]
/// An event dictating that a list of Events must be sent to the game loop
/// after the completion of a spell.
pub struct SpellEffect {
    events: Vec<EventDispatch>,
}

/// Translate a list of EventDispatch into their "real" Event counterparts and send them off
/// into the main game loop to modify the game's creatures.
pub fn dispatch_events(
    mut receiver: EventReader<SpellEffect>,
    mut teleport: EventWriter<TeleportEntity>,
) {
    for effect_list in receiver.read() {
        for effect in &effect_list.events {
            // Each EventDispatch enum is translated into its Event counterpart.
            match effect {
                EventDispatch::TeleportEntity {
                    destination,
                    entity,
                } => {
                    teleport.send(TeleportEntity::new(*entity, destination.x, destination.y));
                }
            };
        }
    }
}
```

With this new system, every completed spell with dispatch all corresponding Events once it is concluded!

## 2. Tracking Creatures' Last Move (Momentum)

`OrdDir` already exists, but it is currently nothing but a simple enum. It could be elevated into a much greater `Component`...

```rust
// main.rs
#[derive(Component, PartialEq, Eq, Copy, Clone, Debug)] // CHANGED: Added Component.
pub enum OrdDir {
    Up,
    Right,
    Down,
    Left,
}
```

It will also need to be a crucial part of each `Creature`.

```rust
// creature.rs
#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub momentum: OrdDir, // NEW!
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
}
```

This will instantly rain down errors into the crate - all Creatures must now receive this new Component.

```rust
// map.rs
fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
            // SNIP
            atlas: TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index: 0,
            },
            momentum: OrdDir::Up, // NEW!
        },
        Player,
    ));
}
```

```rust
fn spawn_cage(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
            // SNIP
            atlas: TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index,
            },
            momentum: OrdDir::Up, // NEW!
        });
        if tile_char == 'H' {
            creature.insert(Hunt);
        }
    }
}
```

All good, but all Creatures are now eternally "facing" upwards regardless of their actions. Let us track this with a new `Event`.

```rust
// events.rs
#[derive(Event)]
pub struct AlterMomentum {
    pub entity: Entity,
    pub direction: OrdDir,
}

fn alter_momentum(mut events: EventReader<AlterMomentum>, mut creature: Query<&mut OrdDir>) {
    for momentum_alteration in events.read() {
        *creature.get_mut(momentum_alteration.entity).unwrap() = momentum_alteration.direction;
    }
}
```

This event receives its trigger, right now, from when the player steps. It won't track any other creatures... for now.

```rust
// events.rs
fn player_step(
    mut events: EventReader<PlayerStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut momentum: EventWriter<AlterMomentum>, // NEW!
    player: Query<(Entity, &Position), With<Player>>,
    hunters: Query<(Entity, &Position), With<Hunt>>,
    map: Res<Map>,
) {
        // SNIP
        teleporter.send(TeleportEntity::new(
            player_entity,
            player_pos.x + off_x,
            player_pos.y + off_y,
        ));
        // NEW!
        momentum.send(AlterMomentum {
            entity: player_entity,
            direction: event.direction,
        });
        // End NEW.

        for (hunter_entity, hunter_pos) in hunters.iter() {
        // SNIP
        }
    }
}
```

## 3. Actually Making The Spell Do Something

Now, for the true main course...

```rust
// spells.rs
impl Axiom {
    fn target(&self, synapse_data: &mut SynapseData, map: &Map) {
        match self {
            // Target the caster's tile.
            Self::Ego => {
                synapse_data.targets.push(synapse_data.caster_position);
            }
            _ => (),
        }
    }
}
```

The ̀`target` function should handle all potential "Forms" in a spell targeting certain tiles. Anything that isn't a "Form" gets flushed down the final `_ => ()`.

`Ego` is quite simple. Push the `caster_position` into the `targets`, done.

As for `Dash`... it is a little more involved. Its implementation will reside inside another `impl Axiom` function, `execute`.

```rust
// events.rs
impl Axiom {
    // SNIP
    /// Execute Function-type Axioms. Returns true if this produced an actual effect.
    fn execute(&self, synapse_data: &mut SynapseData, map: &Map) -> bool {
        match self {
            Self::Dash => {
                // For each (Entity, Position) on a targeted tile...
                for (dasher, dasher_pos) in synapse_data.get_all_targeted_entity_pos_pairs(map) {
                    // The dashing creature starts where it currently is standing.
                    let mut final_dash_destination = dasher_pos;
                    // It will travel in the direction of the caster's last move.
                    let (off_x, off_y) = synapse_data.caster_momentum.as_offset();
                    // The dash has a maximum travel distance of 10.
                    let mut distance_travelled = 0;
                    while distance_travelled < 10 {
                        distance_travelled += 1;
                        // Stop dashing if a solid Creature is hit.
                        if !map.is_passable(
                            final_dash_destination.x + off_x,
                            final_dash_destination.y + off_y,
                        ) {
                            break;
                        }
                        // Otherwise, keep offsetting the dashing creature's position.
                        final_dash_destination.shift(off_x, off_y);
                    }

                    // Once finished, release the Teleport event.
                    synapse_data.effects.push(EventDispatch::TeleportEntity {
                        destination: final_dash_destination,
                        entity: dasher,
                    });
                }
                true
            }
            // Forms (which do not have an in-game effect) return false.
            _ => false,
        }
    }
 }
```

There is only one unimplemented function in this block, `get_all_targeted_entity_pos_pairs`, which inspects the `Map` to pull out all the corresponding key-value pairs.

```rust
// spells.rs
impl SynapseData {

    // SNIP

    fn get_all_targeted_entity_pos_pairs(&self, map: &Map) -> Vec<(Entity, Position)> {
        let mut targeted_pairs = Vec::new();
        for target in &self.targets {
            if let Some(entity) = map.get_entity_at(target.x, target.y) {
                targeted_pairs.push((*entity, *target));
            }
        }
        targeted_pairs
    }
}
```

# The Test Run

After all this, the first spell **Ego, Dash** is ready to enter our grimoire - and while that was a lot, future spell effects will be a lot easier to implement from now on. Simply add more entries in the `match` statements of `target` and `execute`!

One last thing: actually casting it.

```rust
// input.rs
/// Each frame, if a button is pressed, move the player 1 tile.
fn keyboard_input(
    player: Query<Entity, With<Player>>, // NEW!
    mut spell: EventWriter<CastSpell>, // NEW!
    mut events: EventWriter<PlayerStep>,
    input: Res<ButtonInput<KeyCode>>,
) {
    // NEW!
    if input.just_pressed(KeyCode::Space) {
        spell.send(CastSpell {
            caster: player.get_single().unwrap(),
            spell: Spell {
                axioms: vec![Axiom::Ego, Axiom::Dash],
            },
        });
    }
    // End NEW.

    // SNIP
}
```

Finally, `cargo ruǹ` will allow us to escape the sticky grasp of the Hunter by pressing the spacebar! What's that? It crashed? Of course, we now need to register absolutely everything that was just added into Bevy's ECS.

This is extremely easy to forget and is mostly indicated by "struct is never constructed"-type warnings. If you are ever testing your changes and things seem to be going wrong, check first that you registered your systems, events and resources!

With that said:

```rust
// events.rs
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerStep>();
        app.add_event::<TeleportEntity>();
        app.add_event::<AlterMomentum>(); // NEW!
        app.add_systems(Update, player_step);
        app.add_systems(Update, teleport_entity);
        app.add_systems(Update, alter_momentum); // NEW!
    }
}
```

```rust
// spells.rs
impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CastSpell>();
        app.add_event::<SpellEffect>();
        app.add_systems(Update, gather_effects);
        app.add_systems(Update, dispatch_events);
    }
}
```

And there's just one last thing I'd like to change for now: knocking down the light-speed movement down a notch.

```rust
// input.rs
    if input.just_pressed(KeyCode::KeyW) { // CHANGED to just_pressed
        events.send(PlayerStep {
            direction: OrdDir::Up,
        });
    }
    if input.just_pressed(KeyCode::KeyD) { // CHANGED to just_pressed
        events.send(PlayerStep {
            direction: OrdDir::Right,
        });
    }
    if input.just_pressed(KeyCode::KeyA) { // CHANGED to just_pressed
        events.send(PlayerStep {
            direction: OrdDir::Left,
        });
    }
    if input.just_pressed(KeyCode::KeyS) { // CHANGED to just_pressed
        events.send(PlayerStep {
            direction: OrdDir::Down,
        });
    }
```

Try again. `cargo run`. Pressing the space bar will now allow you to escape your sticky little friend!

// TODO gif

# Intermediate Wizardry 201 

The player dashing around is fun and good... but what about a projectile that knocks back whatever critter it hits? This sounds slightly far-fetched, but it actually takes almost no code that we have not already seen. Enter... **MomentumBeam, Dash**.

```rust
// spells.rs
pub enum Axiom {
    // FORMS

    // Target the caster's tile.
    Ego,

    // NEW!
    // Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
    // including the first solid tile encountered, which stops the beam.
    MomentumBeam,
    // End NEW.

    // SNIP
}
```

It, of course, receives its own implementation.

```rust
// spells.rs
    fn target(&self, synapse_data: &mut SynapseData, map: &Map) {
        match self {
        // SNIP
        // NEW!
            // Shoot a beam from the caster towards its last move, all tiles passed through
            // become targets, including the impact point.
            Self::MomentumBeam => {
                // Start the beam where the caster is standing.
                let mut start = synapse_data.caster_position;
                // The beam travels in the direction of the caster's last move.
                let (off_x, off_y) = synapse_data.caster_momentum.as_offset();
                let mut distance_travelled = 0;
                let mut output = Vec::new();
                // The beam has a maximum distance of 10.
                while distance_travelled < 10 {
                    distance_travelled += 1;
                    start.shift(off_x, off_y);
                    // The new tile is always added, even if it is impassable...
                    output.push(start);
                    // But if it is impassable, it is the last added tile.
                    if !map.is_passable(start.x, start.y) {
                        break;
                    }
                }
                // Add these tiles to `targets`.
                synapse_data.targets.append(&mut output);
            }
        // End NEW.
        }
    }
```

You may notice that this is extremely similar to the `Dash` logic... Its differences are the inclusion of the final impact tile (which is solid), and how it collects all travelled tiles in an output vector, added to `targets`.

```rust
// Do not add this block, it is merely a demonstration.
// The dashing creature starts where it currently is standing.
let mut final_dash_destination = dasher_pos;
// It will travel in the direction of the caster's last move.
let (off_x, off_y) = synapse_data.caster_momentum.as_offset();
// The dash has a maximum travel distance of 10.
let mut distance_travelled = 0;
while distance_travelled < 10 {
    distance_travelled += 1;
    // Stop dashing if a solid Creature is hit.
    if !map.is_passable(
        final_dash_destination.x + off_x,
        final_dash_destination.y + off_y,
    ) {
        break;
    }
    // Otherwise, keep offsetting the dashing creature's position.
    final_dash_destination.shift(off_x, off_y);
}
```

In software development, "don't repeat yourself" is a common wisdom, but in games development, sometimes, it must be done within reason. Think, what if we add later a magic forcefield that blocks beams but not movement? In that case, if we had done something like this to adhere to "don't repeat yourself":

```rust
// Do not add this block, it is merely a demonstration.
Self::Dash => {
    for (dasher, dasher_pos) in synapse_data.get_all_targeted_entity_pos_pairs(map) {
        // Create a fake synapse just to use a beam.
        let mut artificial_synapse = SynapseData::new_from_synapse(synapse_data);
        // Set the fake synapse's caster and caster position to be the targeted creatures.
        (
            artificial_synapse.caster,
            artificial_synapse.caster_position,
        ) = (dasher, dasher_pos);
        // Fire the beam with the caster's momentum.
        Self::MomentumBeam.target(&mut artificial_synapse, map);
        // Get the penultimate tile, aka the last passable tile in the beam's path.
        let destination_tile = artificial_synapse
            .targets
            .get(artificial_synapse.targets.len().wrapping_sub(2));
        // If that penultimate tile existed, teleport to it.
        if let Some(destination_tile) = destination_tile {
            synapse_data.effects.push(EventDispatch::TeleportEntity {
                destination: *destination_tile,
                entity: dasher,
            });
        }
    }
    true
}
```

Here, a fake beam is invented, which needs a fake "synapse" to go alongside it. This "beam" is fired for the sole purpose of finding the penultimate tile in its path (the ultimate tile is the solid impact point). This is the tile where the affected dash to.

Should the "anti beam forcefield" be invented later, this would need an added exception, potentially implemented as an extra parameter passed to the `target` function... lots of complexity for not much reward.

And just like that, with only 11 added lines of code (which were very similar to our `Dash` implementation), the projectile is ready:

```rust
// input.rs
    if input.just_pressed(KeyCode::Space) {
        spell.send(CastSpell {
            caster: player.get_single().unwrap(),
            spell: Spell {
                axioms: vec![Axiom::MomentumBeam, Axiom::Dash],
            },
        });
    }
```

`cargo run`. Not only can you teach your sticky companion some manners, you can even break the walls of the cage, and escape into the abyss beyond.
