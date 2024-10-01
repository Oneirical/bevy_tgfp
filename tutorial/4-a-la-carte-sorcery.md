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
        for (i, axiom) in axioms.iter().enumerate() {
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
