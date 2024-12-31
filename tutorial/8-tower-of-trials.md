+++
title = "Bevy Traditional Roguelike Quick-Start - 8. Tower Of Trials"
date = 2024-12-30
authors = ["Julien Robert"]
[taxonomies]
tags = ["rust", "bevy", "tutorial"]
+++

The first thing we'll do in this chapter: **extending the player spell selection with 6 new additions**.

- A **Heal** spell will heal the caster and any adjacent creature by 2 points. This makes it useful for recovery out of combat, but dangerous in an emergency with a melee foe, as it will also heal your opponent.
- A **Shield** spell will protect the caster from all damage for 1 turn, and will not end the turn when cast, allowing it to be used with some foresight.
- A **Trap** spell will place a trap on the ground, which will blow up in a cross-shaped damaging laser when stepped on, which deals 2 damage. Powerful, but requires some setup.
- A **Beam** spell will shoot lasers in all 4 diagonal directions, dealing 2 damage. Diagonal enemies are annoying - they will attack you first if you engage them in melee, so this spell solves this issue.
- A **Dash** spell will allow the caster to yank themselves forwards, dealing 1 damage to all foes close to their travel path, and to knock back the collided creature in the end. Mobility and damage all in one!
- A **Stab** spell will enchant the caster with 5 bonus damage on their next melee attack, which will dispel on their next attack. Good for slaying enemies with a lot of health, one of which will be developed in this chapter.

The Axioms of these spells will be as follows, many of which are not implemented yet:

```rust
// Do not write this anywhere, it is a demonstration.
// "Saintly" - Heal
axioms: vec![Axiom::Ego, Axiom::Plus, Axiom::HealOrHarm { amount: 2 }],
// "Ordered" - Shield
axioms: vec![
    Axiom::Ego,
    Axiom::StatusEffect {
        effect: StatusEffect::Invincible,
        potency: 1,
        stacks: 2,
    },
],
// "Artistic" - Trap
axioms: vec![
    Axiom::Ego,
    Axiom::PlaceStepTrap,
    Axiom::PiercingBeams,
    Axiom::PlusBeam,
    Axiom::Ego,
    Axiom::HealOrHarm { amount: -2 },
],
// "Unhinged" - Beam
axioms: vec![Axiom::XBeam, Axiom::HealOrHarm { amount: -2 }],
// "Feral" - Dash
axioms: vec![
    Axiom::Ego,
    Axiom::Trace,
    Axiom::Dash { max_distance: 5 },
    Axiom::Spread,
    Axiom::UntargetCaster,
    Axiom::HealOrHarm { amount: -1 },
    Axiom::PurgeTargets,
    Axiom::Touch,
    Axiom::StatusEffect {
        effect: StatusEffect::Dizzy,
        potency: 1,
        stacks: 2,
    },
    Axiom::Dash { max_distance: 1 },
],
// "Vile" - Stab
axioms: vec![
    Axiom::Ego,
    Axiom::StatusEffect {
        effect: StatusEffect::Stab,
        potency: 5,
        stacks: 20,
    },
],
```

Let's begin. Be ready, as I'll be going through each of these Axioms in an order which I judge easier for understanding, implementing each one as a one-shot system.

```rust
// spells.rs
#[derive(Debug, Clone)]
/// There are Form axioms, which target certain tiles, and Function axioms, which execute an effect
/// onto those tiles.
pub enum Axiom {
    // FORMS
    /// Target the caster's tile.
    Ego,

    // NEW!
    /// Target the player's tile.
    Player,
    // End NEW.
    
    /// Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
    /// including the first solid tile encountered, which stops the beam.
    MomentumBeam,

    // NEW!
    /// Fire 4 beams from the caster, towards the diagonal directions. Target all travelled tiles,
    /// including the first solid tile encountered, which stops the beam.
    XBeam,
    /// Fire 4 beams from the caster, towards the cardinal directions. Target all travelled tiles,
    /// including the first solid tile encountered, which stops the beam.
    PlusBeam,
    /// Target all orthogonally adjacent tiles to the caster.
    Plus,
    // End NEW.

    /// Target the tile adjacent to the caster, towards the caster's last move.
    Touch,
    /// Target a ring of `radius` around the caster.
    Halo { radius: i32 },

    // FUNCTIONS
    /// The targeted creatures dash in the direction of the caster's last move.
    Dash { max_distance: i32 },
    /// The targeted passable tiles summon a new instance of species.
    SummonCreature { species: Species },

    // NEW!
    /// The targeted tiles summon a step-triggered trap with following axioms as the payload.
    /// This terminates the spell.
    PlaceStepTrap,
    /// Any targeted creature with the Wall component is removed.
    /// Each removed wall heals the caster +1.
    DevourWall,
    /// All creatures summoned by targeted creatures are removed.
    Abjuration,
    /// All targeted creatures heal or are harmed by this amount.
    HealOrHarm { amount: isize },
    /// Give a status effect to all targeted creatures.
    StatusEffect {
        effect: StatusEffect,
        potency: usize,
        stacks: usize,
    },

    // MUTATORS
    /// Any Teleport event will target all tiles between its start and destination tiles.
    Trace,
    /// All targeted tiles expand to also target their orthogonally adjacent tiles.
    Spread,
    /// Remove the Caster's tile from targets.
    UntargetCaster,
    /// All Beam-type Forms will pierce through non-Spellproof creatures.
    PiercingBeams,
    /// Remove all targets.
    PurgeTargets,
    // End NEW.
}
```

Three `Axiom`s are unused by the new spells. `DevourWall` will be used by a new creature species, while `Abjuration` and `Player` will remained unused for now, and are simply a convenience addition for later while we are busy doing massive edits to the spell system.

All these `Axiom`s will need to be registered by `AxiomLibrary`.

```rust
// spells.rs
impl FromWorld for AxiomLibrary {
    fn from_world(world: &mut World) -> Self {
        let mut axioms = AxiomLibrary {
            teleport: world.register_system(teleport_transmission),
            library: HashMap::new(),
        };
        axioms.library.insert(
            discriminant(&Axiom::Ego),
            world.register_system(axiom_form_ego),
        );

        // NEW!
        axioms.library.insert(
            discriminant(&Axiom::Player),
            world.register_system(axiom_form_player),
        );
        // End NEW.
        
        axioms.library.insert(
            discriminant(&Axiom::MomentumBeam),
            world.register_system(axiom_form_momentum_beam),
        );
        axioms.library.insert(
            discriminant(&Axiom::Plus),
            world.register_system(axiom_form_plus),
        );
        axioms.library.insert(
            discriminant(&Axiom::Halo { radius: 1 }),
            world.register_system(axiom_form_halo),
        );
        axioms.library.insert(
            discriminant(&Axiom::XBeam),
            world.register_system(axiom_form_xbeam),
        );

        // NEW!
        axioms.library.insert(
            discriminant(&Axiom::PlusBeam),
            world.register_system(axiom_form_plus_beam),
        );
        // End NEW.
        
        axioms.library.insert(
            discriminant(&Axiom::Touch),
            world.register_system(axiom_form_touch),
        );
        axioms.library.insert(
            discriminant(&Axiom::Dash { max_distance: 1 }),
            world.register_system(axiom_function_dash),
        );
        axioms.library.insert(
            discriminant(&Axiom::SummonCreature {
                species: Species::Player,
            }),
            world.register_system(axiom_function_summon_creature),
        );

        // NEW!
        axioms.library.insert(
            discriminant(&Axiom::PlaceStepTrap),
            world.register_system(axiom_function_place_step_trap),
        );
        axioms.library.insert(
            discriminant(&Axiom::DevourWall),
            world.register_system(axiom_function_devour_wall),
        );
        axioms.library.insert(
            discriminant(&Axiom::Abjuration),
            world.register_system(axiom_function_abjuration),
        );
        axioms.library.insert(
            discriminant(&Axiom::HealOrHarm { amount: 1 }),
            world.register_system(axiom_function_heal_or_harm),
        );
        axioms.library.insert(
            discriminant(&Axiom::StatusEffect {
                effect: StatusEffect::Invincible,
                potency: 0,
                stacks: 0,
            }),
            world.register_system(axiom_function_status_effect),
        );
        axioms.library.insert(
            discriminant(&Axiom::Trace),
            world.register_system(axiom_mutator_trace),
        );
        axioms.library.insert(
            discriminant(&Axiom::Spread),
            world.register_system(axiom_mutator_spread),
        );
        axioms.library.insert(
            discriminant(&Axiom::UntargetCaster),
            world.register_system(axiom_mutator_untarget_caster),
        );
        axioms.library.insert(
            discriminant(&Axiom::PiercingBeams),
            world.register_system(axiom_mutator_piercing_beams),
        );
        axioms.library.insert(
            discriminant(&Axiom::PurgeTargets),
            world.register_system(axiom_mutator_purge_targets),
        );
        // End NEW.

        axioms
    }
}
```

Let's get to the implementation, one by one.

```rust
// spells.rs
/// Target the player's tile.
fn axiom_form_player(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position, With<Player>>,
) {
    // Get the currently executed spell.
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    // Get the caster's position.
    let player_position = *position.get_single().unwrap();
    // Place the visual effect.
    magic_vfx.send(PlaceMagicVfx {
        targets: vec![player_position],
        sequence: EffectSequence::Sequential { duration: 0.04 },
        effect: EffectType::RedBlast,
        decay: 0.5,
        appear: 0.,
    });
    // Add that caster's position to the targets.
    synapse_data.targets.insert(player_position);
}
```

The `Player` Form is extremely similar to `Ego`, fetching the Player's position instead of the caster's.

```rust
// spells.rs
/// Fire 4 beams from the caster, towards the diagonal directions. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_xbeam(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
    spellproof_query: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let caster_position = *position.get(synapse_data.caster).unwrap();
    let diagonals = [(1, 1), (-1, 1), (1, -1), (-1, -1)];
    for (dx, dy) in diagonals {
        // Start the beam where the caster is standing.
        // The beam travels in the direction of each diagonal.
        let output = linear_beam(
            caster_position,
            10,
            dx,
            dy,
            &map,
            synapse_data
                .synapse_flags
                .contains(&SynapseFlag::PiercingBeams),
            &spellproof_query,
        );
        // Add some visual beam effects.
        magic_vfx.send(PlaceMagicVfx {
            targets: output.clone(),
            sequence: EffectSequence::Sequential { duration: 0.04 },
            effect: EffectType::RedBlast,
            decay: 0.5,
            appear: 0.,
        });
        // Add these tiles to `targets`.
        synapse_data.targets.extend(&output);
    }
}
```

```rust
// spells.rs
/// Fire 4 beams from the caster, towards the cardinal directions. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_plus_beam(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
    spellproof_query: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let caster_position = *position.get(synapse_data.caster).unwrap();
    let cardinals = [OrdDir::Up, OrdDir::Down, OrdDir::Left, OrdDir::Right];
    for cardinal in cardinals {
        let (dx, dy) = cardinal.as_offset();
        // Start the beam where the caster is standing.
        // The beam travels in the direction of each diagonal.
        let output = linear_beam(
            caster_position,
            10,
            dx,
            dy,
            &map,
            synapse_data
                .synapse_flags
                .contains(&SynapseFlag::PiercingBeams),
            &spellproof_query,
        );
        // Add some visual beam effects.
        magic_vfx.send(PlaceMagicVfx {
            targets: output.clone(),
            sequence: EffectSequence::Sequential { duration: 0.04 },
            effect: match cardinal {
                OrdDir::Up | OrdDir::Down => EffectType::VerticalBeam,
                OrdDir::Right | OrdDir::Left => EffectType::HorizontalBeam,
            },
            decay: 0.5,
            appear: 0.,
        });
        // Add these tiles to `targets`.
        synapse_data.targets.extend(&output);
    }
}
```

`XBeam` is simply a `MomentumBeam̀` which fires 4 times, in each diagonal direction instead of the momentum-direction.

The `PlusBeam` Form is a clone of `XBeam`, with its diagonal directions changed to cardinal directions. However, you'll notice two extra arguments in the `linear_beam` function... Let's start by showing how the `linear_beam` function has changed.

Beams can now be **Piercing**. Piercing beams pass through creatures, and are only stopped by `Spellproof` tiles. Note how it is possible to pass the `&Query<Has<Spellproof>>` in its borrowed form, in a simple Rust function which is NOT a Bevy system, and still use its methods!

```rust
// spells.rs
fn linear_beam(
    mut start: Position,
    max_distance: usize,
    off_x: i32,
    off_y: i32,
    map: &Map,
    // NEW!
    is_piercing: bool,
    spellproof_query: &Query<Has<Spellproof>>,
    // End NEW.
) -> Vec<Position> {
    let mut distance_travelled = 0;
    let mut output = Vec::new();
    // The beam has a maximum distance of max_distance.
    while distance_travelled < max_distance {
        distance_travelled += 1;
        start.shift(off_x, off_y);
        // The new tile is always added, even if it is impassable...
        output.push(start);
        // But if it is impassable, the beam stops.

        // NEW!
        if is_piercing {
            if let Some(possible_block) = map.get_entity_at(start.x, start.y) {
                if spellproof_query.get(*possible_block).unwrap() {
                    break;
                }
            }
        }
        // End NEW. 
        else if !map.is_passable(start.x, start.y) { // CHANGED - else if
            break;
        }
    }
    output
}
```

Back to the `PlusBeam`, there is another unimplemented part:

```rust
// Do not add this, it is already written.
let output = linear_beam(
    caster_position,
    10,
    dx,
    dy,
    &map,
    synapse_data
        .synapse_flags // What even is a "synapse_flags"?
        .contains(&SynapseFlag::PiercingBeams),
    &spellproof_query,
);
```

`SynapseFlag`s will be a type fitting inside a new field in `SynapseData`. They will track spell-specific behaviour, such as, "from now on, all beams will be piercing" or "end this spell early without going through all the `Axiom`s".

```rust
// spells.rs
#[derive(Eq, Debug, PartialEq, Hash)]
/// Flags that alter the behaviour of an active synapse.
pub enum SynapseFlag {
    /// Delete this synapse and abandon all future Axioms.
    Terminate,
    /// Do not advance the step counter. Only runs once, is deleted instead of incrementing
    /// the step counter.
    NoStep,
    /// Any Teleport event will target all tiles between its start and destination tiles.
    Trace,
    /// All Beam-type Forms will pierce non-Wall creatures.
    PiercingBeams,
}
```

```rust
// spells.rs
/// The tracker of everything which determines how a certain spell will act.
pub struct SynapseData {
    /// Where a spell will act.
    targets: HashSet<Position>,
    /// How a spell will act.
    pub axioms: Vec<Axiom>,
    /// The nth axiom currently being executed.
    pub step: usize,
    /// Who cast the spell.
    pub caster: Entity,

    // NEW!
    /// Flags that alter the behaviour of an active synapse.
    synapse_flags: HashSet<SynapseFlag>,
    // End NEW.
}

impl SynapseData {
    /// Create a blank SynapseData.
    fn new(caster: Entity, axioms: Vec<Axiom>) -> Self {
        SynapseData {
            targets: HashSet::new(),
            axioms,
            step: 0,
            caster,

            // NEW!
            synapse_flags: HashSet::new(),
            // End NEW.
        }
    }
```

We will need to edit the `MomentumBeam` Form so it reacts to this new Flag.

```rust
// spells.rs
/// Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_momentum_beam(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    position_and_momentum: Query<(&Position, &OrdDir)>,
    spellproof_query: Query<Has<Spellproof>>, // NEW!
) {
    // SNIP
    let output = linear_beam(
        *caster_position,
        10,
        off_x,
        off_y,
        &map,
        // NEW!
        synapse_data
            .synapse_flags
            .contains(&SynapseFlag::PiercingBeams),
        &spellproof_query,
        // End NEW.
    );
    // SNIP
}
```

And now, let's properly add the ̀`Axiom̀` which will weaponize our laser beams further with extra penetration! It will be a "Mutator", something separate from "Forms" and "Functions", which will change something in the way a spell operates.

```rust
// spells.rs
/// All Beam-type Forms will pierce through non-Spellproof creatures.
fn axiom_mutator_piercing_beams(mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    synapse_data
        .synapse_flags
        .insert(SynapseFlag::PiercingBeams);
}
```

While we're at it, we can implement another one of these `SynapseFlag` systems: `Trace`.

```rust
// spells.rs
/// Any Teleport event will target all tiles between its start and destination tiles.
fn axiom_mutator_trace(mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    synapse_data.synapse_flags.insert(SynapseFlag::Trace);
}
```

This will cause movement effects - such as `Dash` to leave behind a "trail" as creatures move around. For example, with `SummonCreature`, you could be leaving a summoned creature in each tile in the wake of your passage.

However, there might eventually be more transport/teleport `Axiom`s than the simple `Dash`. If we start adding "if `Trace` is active, target all these tiles" to every single `Axiom` with some teleporting-related behaviour, things are going to get spammy fast. Instead, let's narrow down the concept of "teleporting due to a spell" in a single one-shot system.

```rust
// spells.rs
fn teleport_transmission(
    In(teleport_event): In<TeleportEntity>,
    position: Query<&Position>,
    mut teleport_writer: EventWriter<TeleportEntity>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    if synapse_data.synapse_flags.contains(&SynapseFlag::Trace) {
        let start = position.get(teleport_event.entity).unwrap();
        let mut output = walk_grid(*start, teleport_event.destination);
        if output.len() > 2 {
            // Remove the start and ending.
            output.pop();
            output.remove(0);
            // Add some visual beam effects.
            magic_vfx.send(PlaceMagicVfx {
                targets: output.clone(),
                sequence: EffectSequence::Sequential { duration: 0.04 },
                effect: EffectType::RedBlast,
                decay: 0.5,
                appear: 0.,
            });
            // Add these tiles to `targets`.
            synapse_data.targets.extend(&output);
        }
    }
    teleport_writer.send(teleport_event);
}
```

This system has something we have not seen before - an `In` field, meaning we'll be able to run `commands.run_system_with_input` and pass arguments to it - specifically, the `TeleportEntity` event we wish to relay into the main event loop.

Before proceeding, let's implement the `walk_grid` function, which lets us draw lines between two `Position`s for the purpose of the `Trace` flag.

This logic is proudly stolen from [Red Blob Games](https://www.redblobgames.com/grids/line-drawing/), translated from JavaScript to Rust. I truly recommend this resource for all things related to procedural generation algorithms! I recommend reading the article if you are curious about the inner workings of this line-drawing function.

```rust
// spells.rs
fn walk_grid(p0: Position, p1: Position) -> Vec<Position> {
    let dx = p1.x - p0.x;
    let dy = p1.y - p0.y;
    let nx = dx.abs();
    let ny = dy.abs();
    let sign_x = dx.signum();
    let sign_y = dy.signum();

    let mut p = Position { x: p0.x, y: p0.y };
    let mut points = vec![p];
    let mut ix = 0;
    let mut iy = 0;

    while ix < nx || iy < ny {
        match ((0.5 + ix as f32) / nx as f32).partial_cmp(&((0.5 + iy as f32) / ny as f32)) {
            Some(Ordering::Less) => {
                p.x += sign_x;
                ix += 1;
            }
            _ => {
                p.y += sign_y;
                iy += 1;
            }
        }
        points.push(p);
    }

    points
}
```

Back to our one-shot systems, we need to ensure `Dash` will call upon the newly added `teleport_transmission`.

```rust
// spells.rs
/// The targeted creatures dash in the direction of the caster's last move.
fn axiom_function_dash(
    library: Res<AxiomLibrary>, // CHANGED - this has replaced the EventWriter.
    mut commands: Commands, // NEW!
    map: Res<Map>,
    spell_stack: Res<SpellStack>,
    momentum: Query<&OrdDir>,
    is_spellproof: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.last().unwrap();
    let caster_momentum = momentum.get(synapse_data.caster).unwrap();
    if let Axiom::Dash { max_distance } = synapse_data.axioms[synapse_data.step] {
        // For each (Entity, Position) on a targeted tile with a creature on it...
        for (dasher, dasher_pos) in synapse_data.get_all_targeted_entity_pos_pairs(&map) {
            // SNIP

            // CHANGED - This is now using our one-shot system instead of
            // sending the TeleportEntity.
            // Once finished, release the Teleport event.
            commands.run_system_with_input(
                library.teleport,
                TeleportEntity {
                    destination: final_dash_destination,
                    entity: dasher,
                },
            );
            // End CHANGED
  // SNIP
}
```

You'll notice we are storing `teleport_transmission`'s ID inside of the `AxiomLibrary` - something which we need to implement as well.

```rust
// spells.rs
#[derive(Resource)]
/// All available Axioms and their corresponding systems.
pub struct AxiomLibrary {
    pub library: HashMap<Discriminant<Axiom>, SystemId>,
    pub teleport: SystemId<In<TeleportEntity>>, // NEW!
}

impl FromWorld for AxiomLibrary {
    fn from_world(world: &mut World) -> Self {
        let mut axioms = AxiomLibrary {
            teleport: world.register_system(teleport_transmission), // NEW!
            library: HashMap::new(),
        };
        // SNIP
```

We're done with `Trace`! Let's move on to a simpler one, `Plus`. Like `PlusBeam`, but only through a single tile. Like a `Touch`, but in all 4 adjacent tiles.

```rust
// spells.rs
/// Target all orthogonally adjacent tiles to the caster.
fn axiom_form_plus(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let caster_position = *position.get(synapse_data.caster).unwrap();
    let adjacent = [OrdDir::Up, OrdDir::Right, OrdDir::Down, OrdDir::Left];
    let mut output = Vec::new();
    for direction in adjacent {
        let mut new_pos = caster_position;
        let offset = direction.as_offset();
        new_pos.shift(offset.0, offset.1);
        output.push(new_pos);
    }
    magic_vfx.send(PlaceMagicVfx {
        targets: output.clone(),
        sequence: EffectSequence::Sequential { duration: 0.04 },
        effect: EffectType::RedBlast,
        decay: 0.5,
        appear: 0.,
    });
    synapse_data.targets.extend(&output);
}
```

With just one more system, we'll complete our first of the six spells, "Heal". `HealOrHarm` will do exactly what it says on the tin.

```rust
// spells.rs
/// All targeted creatures heal or are harmed by this amount.
fn axiom_function_heal_or_harm(
    mut heal: EventWriter<DamageOrHealCreature>,
    spell_stack: Res<SpellStack>,
    map: Res<Map>,
    is_spellproof: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.last().unwrap();
    if let Axiom::HealOrHarm { amount } = synapse_data.axioms[synapse_data.step] {
        for entity in synapse_data.get_all_targeted_entities(&map) {
            let is_spellproof = is_spellproof.get(entity).unwrap();
            if !is_spellproof {
                heal.send(DamageOrHealCreature {
                    entity,
                    culprit: synapse_data.caster,
                    hp_mod: amount,
                });
            }
        }
    } else {
        panic!();
    }
}
```

This completes implementation of the following!

```rust
// Merely a demonstration, do not add.
// "Saintly" - Heal
axioms: vec![Axiom::Ego, Axiom::Plus, Axiom::HealOrHarm { amount: 2 }],
```

Onto the next one, `Spread`. For each target, it will fetch all its adjacent tiles, and target those as well. It is somewhat similar to the `Plus` we recently implemented, but does so for every single target instead of only the caster's tile.

```rust
// spells.rs
/// All targeted tiles expand to also target their orthogonally adjacent tiles.
fn axiom_mutator_spread(
    mut spell_stack: ResMut<SpellStack>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let mut output = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for target in &synapse_data.targets {
        let adjacent = [OrdDir::Up, OrdDir::Right, OrdDir::Down, OrdDir::Left];
        for (i, direction) in adjacent.iter().enumerate() {
            let mut new_pos = *target;
            let offset = direction.as_offset();
            new_pos.shift(offset.0, offset.1);
            output[i].push(new_pos);
        }
    }
    // All upwards, then all rightwards, etc, for a consistent animation effect.
    for ord_dir_vec in output {
        magic_vfx.send(PlaceMagicVfx {
            targets: ord_dir_vec.clone(),
            sequence: EffectSequence::Sequential { duration: 0.04 },
            effect: EffectType::RedBlast,
            decay: 0.5,
            appear: 0.,
        });
        synapse_data.targets.extend(&ord_dir_vec);
    }
}
```

We're getting increasingly closer to finishing implementation of the most `Axiom`-intensive spell among the 6!

```rust
// Merely a demonstration, do not add.
// "Feral" - Dash
axioms: vec![
    Axiom::Ego,
    Axiom::Trace,
    Axiom::Dash { max_distance: 5 },
    Axiom::Spread,
    Axiom::UntargetCaster,
    Axiom::HealOrHarm { amount: -1 },
    Axiom::PurgeTargets,
    Axiom::Touch,
    Axiom::StatusEffect {
        effect: StatusEffect::Dizzy,
        potency: 1,
        stacks: 2,
    },
    Axiom::Dash { max_distance: 1 },
],
```

We'll just need 2 more simple "Mutators" - `PurgeTargets` and `UntargetCaster`, in addition to the `StatusEffect` mechanic, which will be a whole other ordeal! Let's start with the simpler, former two.

```rust
// spells.rs
/// Remove the Caster's tile from targets.
fn axiom_mutator_untarget_caster(mut spell_stack: ResMut<SpellStack>, position: Query<&Position>) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let caster_position = position.get(synapse_data.caster).unwrap();
    synapse_data.targets.remove(caster_position);
}

/// Delete all targets.
fn axiom_mutator_purge_targets(mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    synapse_data.targets.clear();
}
```

These are simply some manipulation of the spell's targets. Without `UntargetCaster`, the "trail" left by the dashing player by `Trace` would expand with `Spread` to include the player's tile, and harm them as well. Without `PurgeTargets`, the knockback and stun effect would then be applied to every single creature hit by the lateral damaging segment of the dash.

And now, moving on to the real main course: `StatusEffect`.

```rust
// creature.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatusEffect {
    // Cannot take damage.
    Invincible,
    // Bonus melee damage, dispels on melee attack.
    Stab,
    // Stun, no action.
    Dizzy,
}

pub struct PotencyAndStacks {
    pub potency: usize,
    pub stacks: usize,
}

#[derive(Component)]
pub struct StatusEffectsList {
    pub effects: HashMap<StatusEffect, PotencyAndStacks>,
}

#[derive(Component)]
pub struct Stab {
    pub bonus_damage: isize,
}

#[derive(Component)]
pub struct Invincible;

#[derive(Component)]
pub struct Dizzy;
```

Status effects will have a potency (example: Stab VI deals 6 bonus damage) and an amount of stacks (example: after 5 turns, the effect is dispelled).

`StatusEffectsList` will be a new Component, part of the `Creature` bundle.

```rust
// creature.rs
#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub momentum: OrdDir,
    pub sprite: Sprite,
    pub species: Species,
    pub health: Health,
    pub effects: StatusEffectsList, // NEW!
}
```

```rust
// events.rs
/// Place a new Creature on the map of Species and at Position.
pub fn summon_creature(
    // SNIP
) {
        // SNIP
        let mut new_creature = commands.spawn((
            Creature {
                // SNIP
                health: Health { max_hp, hp },
                // NEW!
                effects: StatusEffectsList {
                    effects: HashMap::new(),
                },
                // End NEW.
        // SNIP

```

This new mechanic comes with its own event-based system, where respective `Component`s are added to affected creatures whenever they receive a new status effect.

```rust
// events.rs
#[derive(Event)]
pub struct AddStatusEffect {
    pub entity: Entity,
    pub effect: StatusEffect,
    pub potency: usize,
    pub stacks: usize,
}

pub fn add_status_effects(
    mut events: EventReader<AddStatusEffect>,
    mut effects: Query<&mut StatusEffectsList>,
    mut commands: Commands,
) {
    for event in events.read() {
        let mut effects_list = effects.get_mut(event.entity).unwrap();
        if let Some(effect) = effects_list.effects.get(&event.effect) {
            // Re-applying a status effect which is already possessed does not work
            // if the new effect has a lesser potency.
            if event.potency < effect.potency {
                continue;
            }
        }
        // Mark the creature as possessing that status effect.
        effects_list.effects.insert(
            event.effect,
            PotencyAndStacks {
                potency: event.potency,
                stacks: event.stacks,
            },
        );
        // Insert the components corresponding to the new status effect.
        match event.effect {
            StatusEffect::Invincible => {
                commands.entity(event.entity).insert(Invincible);
            }
            StatusEffect::Stab => {
                commands.entity(event.entity).insert(Stab {
                    bonus_damage: event.potency as isize,
                });
            }
            StatusEffect::Dizzy => {
                commands.entity(event.entity).insert(Dizzy);
            }
        }
    }
}
```

Each of these 3 status effects has an impact somewhere in related systems. Let's start with `Invinciblè`, which will block attempted health point deductions.

```rust
// events.rs
pub fn harm_creature(
    mut events: EventReader<DamageOrHealCreature>,
    mut remove: EventWriter<RemoveCreature>,
    mut creature: Query<(&mut Health, &Children)>,
    mut hp_bar: Query<(&mut Visibility, &mut Sprite)>,
    defender_flags: Query<Has<Invincible>>, // NEW!
) {
    for event in events.read() {
        let (mut health, children) = creature.get_mut(event.entity).unwrap();
        let is_invincible = defender_flags.get(event.entity).unwrap();
        // Apply damage or healing.
        match event.hp_mod.signum() {
            -1 => {
            // NEW!
                if is_invincible {
                    continue;
                }
            // End NEW.
```

Next up, `Stab` will add bonus damage equal to its `potency` on a melee attack, but will set its own duration to 0 afterwards.

```rust
// events.rs
pub fn creature_collision(
    mut events: EventReader<CreatureCollision>,
    mut harm: EventWriter<DamageOrHealCreature>,
    mut open: EventWriter<OpenDoor>,
    attacker_flags: Query<&Stab>, // NEW!
    defender_flags: Query<(Has<Door>, Has<Meleeproof>)>,
    mut turn_manager: ResMut<TurnManager>,
    mut creature: Query<(&OrdDir, &mut Transform, Has<Player>)>,
    mut commands: Commands,
    mut effects: Query<&mut StatusEffectsList>,
) {
    for event in events.read() {
        if event.culprit == event.collided_with {
            // No colliding with yourself.
            continue;
        }
        let (is_door, cannot_be_melee_attacked) = defender_flags.get(event.collided_with).unwrap();
        let (attacker_orientation, mut attacker_transform, is_player) =
            creature.get_mut(event.culprit).unwrap();
        if is_door {
            // Open doors.
            open.send(OpenDoor {
                entity: event.collided_with,
            });
        } else if !cannot_be_melee_attacked {
        // NEW!
            let damage = if let Ok(stab) = attacker_flags.get(event.culprit) {
                // Attacking something with Stab active resets the Stab bonus.
                let mut status_effects = effects.get_mut(event.culprit).unwrap();
                status_effects
                    .effects
                    .get_mut(&StatusEffect::Stab)
                    .unwrap()
                    .stacks = 0;
                -1 - stab.bonus_damage
            } else {
                -1
            };
        // End NEW.
            // Melee attack.
            harm.send(DamageOrHealCreature {
                entity: event.collided_with,
                culprit: event.culprit,
                hp_mod: damage, // CHANGED, now uses the variable "damage"
            });
```

Finally, `Dizzy` will exclude affected creatures from the loop in `end_turn`, preventing them from taking actions.

```rust
```

```rust
```

```rust
```

```rust
```

```rust
```

```rust
```

```rust
```

```rust
```

```rust
```

```rust
```

```rust
```

```rust
```

