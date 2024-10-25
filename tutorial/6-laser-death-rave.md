+++
title = "Bevy Traditional Roguelike Quick-Start - 6. Laser Death Rave"
date = 2024-10-24
authors = ["Julien Robert"]
[taxonomies]
tags = ["rust", "bevy", "tutorial"]
+++

Currently, everyone in this little white cage has a heart of gold and inoffensive pool noodles for limbs. Where is the *violence*?

# In a Laser Death Rave, We Need Death

```rust
// creature.rs
pub struct HealthPoint;
```

Ah, now we are getting somewhere.

> **Design Capsule**
\
`HealthPoint` is a placeholder for now. It will go inside a "deck" and a "discard" pile, each `Vec<HealthPoint>`. The reason for this - and why I am not simply incrementing and decrementing a health integer, is because spells and health will be unified later on. Taking damage will cause spells to be "discarded", and using spells will come at a cost to health. This will be the topic of a later chapter. For now, it is not important.

With that said:

```rust
#[derive(Component)]
pub struct HealthBar {
    pub deck: Vec<HealthPoint>,
    pub repressed: Vec<HealthPoint>,
}

impl HealthBar {
    /// Create a new health container with a certain amount of points in its deck.
    pub fn new(max_hp: i32) -> Self {
        let mut deck = Vec::new();
        for _i in 0..max_hp {
            deck.push(HealthPoint);
        }
        Self {
            deck,
            repressed: Vec::new(),
        }
    }

    /// Deal damage, shifting HealthPoints from the deck to the repressed discard.
    /// The bool return is true if a creature was brought to 0 HP.
    pub fn repress(&mut self, damage: i32) -> bool {
        for _i in 0..damage {
            let lost = self.deck.pop();
            if let Some(lost) = lost {
                self.repressed.push(lost);
            } else {
                return true;
            }
            if self.deck.is_empty() {
                return true;
            }
        }
        false
    }
}
```

```rust

#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub momentum: OrdDir,
    pub species: Species,
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
    pub health: HealthBar, // NEW!
}
```

This is the gameplay half of this new feature - the graphical part is below.

```rust
// graphics.rs
#[derive(Bundle)]
pub struct HealthIndicator {
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
}
```

All of this will be attached to all newly spawned creatures from this point onwards... All creatures have 2 HP by default, the player has 6, and walls have 200.

```rust
// events.rs, summon_creature
    // SNIP
            atlas: TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index: get_species_sprite(&event.species),
            },
            momentum: OrdDir::Up,
            health: HealthBar::new(2), // NEW!
    // SNIP
        match &event.species {
            Species::Wall => {
                new_creature.insert(HealthBar::new(200)); // NEW!
            }
            Species::Player => {
                new_creature.insert(Player);
                new_creature.insert(HealthBar::new(6)); // NEW!
            }
    // SNIP
```

The `HealthIndicator` will be added as a **child entity** of the creature. While it may seem like blasphemy to have hierarchies like this in an ECS game development environment, they do exist in a limited form. In Bevy, this is useful to have a sprite follow another as if it were "glued" to it, since the children inherit a Transform component from their parent. This is exactly what we want for a creature-specific HP bar! Children can be attached with `commands.entity(parent).add_child(child);`

```rust
// events.rs, summon_creature
match &event.species {
    // SNIP
    _ => (),
}

// NEW!
// Free the borrow on Commands.
let new_creature_entity = new_creature.id();
let hp_bar = commands
    .spawn(HealthIndicator {
        sprite: SpriteBundle {
            texture: asset_server.load("spritesheet.png"),
            // It already inherits the increased scale from the parent.
            transform: Transform::from_scale(Vec3::new(1., 1., 0.)),
            visibility: Visibility::Hidden,
            ..default()
        },
        atlas: TextureAtlas {
            layout: atlas_layout.handle.clone(),
            index: 168,
        },
    })
    .id();
commands.entity(new_creature_entity).add_child(hp_bar);
// End NEW.
```

Now, for damage to actually exist, it will of course be passed as an ̀`Event`. Note the `&Children` in the `Query`, which allows for easy access of the damaged creature's `HealthIndicator`!

```rust
// events.rs
#[derive(Event)]
pub struct RepressionDamage {
    pub entity: Entity,
    pub damage: i32,
}

pub fn repression_damage(
    mut events: EventReader<RepressionDamage>,
    mut damaged_creature: Query<(&mut HealthBar, &Children)>,
    mut hp_bar: Query<(&mut Visibility, &mut TextureAtlas)>,
) {
    for event in events.read() {
        let (mut hp, children) = damaged_creature.get_mut(event.entity).unwrap();
        // Damage the creature.
        hp.repress(event.damage);
        for child in children.iter() {
            // Get the HP bars attached to the creatures.
            let (mut hp_vis, mut hp_bar) = hp_bar.get_mut(*child).unwrap();
            // Get the maximum HP, and the current HP.
            let max_hp = hp.deck.len() + hp.repressed.len();
            let current_hp = hp.deck.len();
            // If this creature is at 100% or 0% HP, don't show the healthbar.
            if max_hp == current_hp || current_hp == 0 {
                *hp_vis = Visibility::Hidden;
            } else {
                // Otherwise, show a color-coded healthbar.
                *hp_vis = Visibility::Visible;
                match current_hp as f32 / max_hp as f32 {
                    0.85..1.00 => hp_bar.index = 168,
                    0.70..0.85 => hp_bar.index = 169,
                    0.55..0.70 => hp_bar.index = 170,
                    0.40..0.55 => hp_bar.index = 171,
                    0.25..0.40 => hp_bar.index = 172,
                    0.10..0.25 => hp_bar.index = 173,
                    0.00..0.10 => hp_bar.index = 174,
                    _ => panic!("That is not a possible HP %!"),
                }
            }
        }
    }
}
```

All these lines of code and still not a trace of violence anywhere! We now need a way to properly inflict this damage.

First, the boilerplate.

```rust
// spells.rs
pub enum Axiom {
// SNIP
     // FUNCTIONS

    // The targeted creatures dash in the direction of the caster's last move.
    Dash,
    // The targeted passable tiles summon a new instance of species.
    SummonCreature { species: Species },

    // NEW!
    // Deal damage to all creatures on targeted tiles.
    RepressionDamage { damage: i32 },
    // End NEW.
```

```rust
// spells.rs
/// An enum with replicas of common game Events, to be translated into the real Events
/// and dispatched to the main game loop.
pub enum EventDispatch {
    // SNIP
    // NEW!
    RepressionDamage {
        entity: Entity,
        damage: i32,
    },
    // End NEW.
}
```

```rust
/// Translate a list of EventDispatch into their "real" Event counterparts and send them off
/// into the main game loop to modify the game's creatures.
pub fn dispatch_events(
    mut receiver: EventReader<SpellEffect>,
    mut teleport: EventWriter<TeleportEntity>,
    mut summon: EventWriter<SummonCreature>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut repression_damage: EventWriter<RepressionDamage>, // NEW!
) {
                // SNIP
                // NEW!
                EventDispatch::RepressionDamage { entity, damage } => {
                    repression_damage.send(RepressionDamage {
                        entity: *entity,
                        damage: *damage,
                    });
                }
                // End NEW.
            };
        }
    }
}
```

Then, the implementation proper.

```rust
    /// Execute Function-type Axioms. Returns true if this produced an actual effect.
    fn execute(&self, synapse_data: &mut SynapseData, map: &Map) -> bool {
        match self {
            // SNIP

            // NEW!
            Self::RepressionDamage { damage } => {
                for entity in synapse_data.get_all_targeted_entities(map) {
                    synapse_data.effects.push(EventDispatch::RepressionDamage {
                        entity,
                        damage: *damage,
                    });
                }
                true
            }
            // End NEW.

            // Forms (which do not have an in-game effect) return false.
            _ => false,
        }
    }
}
```

Finally, registration.

```rust
// events.rs
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreatureStep>();
        app.add_event::<TeleportEntity>();
        app.add_event::<AlterMomentum>();
        app.add_event::<SummonCreature>();
        app.add_event::<RepressionDamage>(); // NEW!
        app.init_resource::<Events<EndTurn>>();
        app.add_systems(Update, creature_step);
        app.add_systems(Update, teleport_entity);
        app.add_systems(Update, alter_momentum);
        app.add_systems(Update, summon_creature);
        app.add_systems(Update, repression_damage); // NEW!
        app.add_systems(Update, end_turn);
    }
}
```

All ready now. Load it in!

```rust
// input.rs
fn keyboard_input( /* SNIP */ ) { /* SNIP */
        if input.just_pressed(KeyCode::Space) {
            spell.send(CastSpell {
                caster: player,
                spell: Spell {
                    // CHANGED - MomentumBeam, RepressionDamage
                    axioms: vec![
                    Axiom::MomentumBeam, 
                    Axiom::RepressionDamage { damage: 1 }
                    ],
                    // End CHANGED.
                },
            });
            turn_end.send(EndTurn);
        }
```

`cargo run`. You will now be capable of running around, pressing Spacebar to "attack" walls and Hunters, which ultimately does nothing as the concept of "death" has not been implemented yet.

We could simply despawn the entities, but Halloween is coming up at the time I am typing this, and ghosts are cool.

```rust
// creature.rs
// This creature has no collisions with other entities.
#[derive(Component)]
pub struct Intangible;
```

This, unfortunately, invalidates a *critical* assumption - that there could only ever be one creature per tile.
```rust
// map.rs
/// A struct with some information on a creature inside the Map.
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct MapCreature {
    pub entity: Entity,
    pub is_intangible: bool,
}
```

```rust
// map.rs
/// The position of every creature, updated automatically.
#[derive(Resource)]
pub struct Map {
    // CHANGED from Entity to HashSet<MapCreature>
    pub creatures: HashMap<Position, HashSet<MapCreature>>,
    // End CHANGED.    
}
```
