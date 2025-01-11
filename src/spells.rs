use std::{
    cmp::Ordering,
    mem::{discriminant, Discriminant},
};

use bevy::{
    ecs::system::SystemId,
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{
    creature::{
        EffectDuration, Player, Soul, Species, Spellbook, Spellproof, StatusEffect,
        StatusEffectsList, Summoned, Wall,
    },
    events::{
        AddStatusEffect, DamageOrHealCreature, RemoveCreature, SummonCreature, TeleportEntity,
        TransformCreature,
    },
    graphics::{EffectSequence, EffectType, PlaceMagicVfx},
    map::{Map, Position},
    OrdDir,
};

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<CastSpell>>();
        app.insert_resource(SpellStack { spells: Vec::new() });
        app.init_resource::<AxiomLibrary>();
        app.add_event::<TriggerContingency>();
    }
}

#[derive(Resource)]
/// All available Axioms and their corresponding systems.
pub struct AxiomLibrary {
    pub library: HashMap<Discriminant<Axiom>, SystemId<In<usize>>>,
    pub teleport: SystemId<In<(TeleportEntity, usize)>>,
}

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
        axioms.library.insert(
            discriminant(&Axiom::Player),
            world.register_system(axiom_form_player),
        );
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
        axioms.library.insert(
            discriminant(&Axiom::PlusBeam),
            world.register_system(axiom_form_plus_beam),
        );
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
                stacks: EffectDuration::Infinite,
            }),
            world.register_system(axiom_function_status_effect),
        );
        axioms.library.insert(
            discriminant(&Axiom::UpgradeStatusEffect {
                effect: StatusEffect::Invincible,
                potency: 0,
                stacks: EffectDuration::Infinite,
            }),
            world.register_system(axiom_function_upgrade_status_effect),
        );
        axioms.library.insert(
            discriminant(&Axiom::IncrementCounter {
                amount: 0,
                count: 0,
            }),
            world.register_system(axiom_function_increment_counter),
        );
        axioms.library.insert(
            discriminant(&Axiom::Transform {
                species: Species::Player,
            }),
            world.register_system(axiom_function_transform),
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
        axioms.library.insert(
            discriminant(&Axiom::Terminate),
            world.register_system(axiom_mutator_terminate),
        );
        axioms.library.insert(
            discriminant(&Axiom::TerminateIfCounter {
                condition: CounterCondition::LessThan,
                threshold: 0,
            }),
            world.register_system(axiom_mutator_terminate_if_counter),
        );
        axioms.library.insert(
            discriminant(&Axiom::FilterBySpecies {
                species: Species::Player,
            }),
            world.register_system(axiom_mutator_filter_by_species),
        );
        axioms
    }
}

#[derive(Resource)]
/// The current spells being executed.
pub struct SpellStack {
    /// The stack of spells, last in, first out.
    pub spells: Vec<SynapseData>,
}

#[derive(Event, Debug)]
/// Triggered when a creature performs an action corresponding to a certain Contingency.
pub struct TriggerContingency {
    pub caster: Entity,
    pub contingency: Axiom,
}

pub fn trigger_contingency(
    mut events: EventReader<TriggerContingency>,
    spellbook: Query<&Spellbook>,
    mut cast_spell: EventWriter<CastSpell>,
) {
    for event in events.read() {
        let spellbook = spellbook.get(event.caster).unwrap();
        for (soul, spell) in spellbook.spells.iter() {
            if let Some(contingency_index) = spell
                .axioms
                .iter()
                .position(|axiom| axiom == &event.contingency)
            {
                cast_spell.send(CastSpell {
                    caster: event.caster,
                    spell: spell.clone(),
                    starting_step: contingency_index,
                    soul_caste: *soul,
                });
            }
        }
    }
}

#[derive(Event)]
/// Triggered when a creature (the `caster`) casts a `spell`.
pub struct CastSpell {
    pub caster: Entity,
    pub spell: Spell,
    pub starting_step: usize,
    pub soul_caste: Soul,
}

#[derive(Component, Clone, Debug)]
/// A spell is composed of a list of "Axioms", which will select tiles or execute an effect onto
/// those tiles, in the order they are listed.
pub struct Spell {
    pub axioms: Vec<Axiom>,
}

#[derive(Debug, Clone, PartialEq)]
/// There are Form axioms, which target certain tiles, and Function axioms, which execute an effect
/// onto those tiles.
pub enum Axiom {
    // CONTINGENCIES
    // Triggers when the caster teleports.
    WhenMoved,
    // Triggers when a creature teleports onto the same tile as the caster.
    WhenSteppedOn,
    // Triggers when this creature is removed.
    WhenRemoved,

    // FORMS
    /// Target the caster's tile.
    Ego,
    /// Target the player's tile.
    Player,
    /// Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
    /// including the first solid tile encountered, which stops the beam.
    MomentumBeam,
    /// Fire 4 beams from the caster, towards the diagonal directions. Target all travelled tiles,
    /// including the first solid tile encountered, which stops the beam.
    XBeam,
    /// Fire 4 beams from the caster, towards the cardinal directions. Target all travelled tiles,
    /// including the first solid tile encountered, which stops the beam.
    PlusBeam,
    /// Target all orthogonally adjacent tiles to the caster.
    Plus,
    /// Target the tile adjacent to the caster, towards the caster's last move.
    Touch,
    /// Target a ring of `radius` around the caster.
    Halo {
        radius: i32,
    },

    // FUNCTIONS
    /// The targeted creatures dash in the direction of the caster's last move.
    Dash {
        max_distance: i32,
    },
    /// The targeted passable tiles summon a new instance of species.
    SummonCreature {
        species: Species,
    },
    /// The targeted tiles summon a step-triggered trap with following axioms as the payload.
    /// This terminates the spell.
    PlaceStepTrap,
    /// Any targeted creature with the Wall component is removed.
    /// Each removed wall heals the caster +1.
    DevourWall,
    /// All creatures summoned by targeted creatures are removed.
    Abjuration,
    /// All targeted creatures heal or are harmed by this amount.
    HealOrHarm {
        amount: isize,
    },
    /// Give a status effect to all targeted creatures.
    StatusEffect {
        effect: StatusEffect,
        potency: usize,
        stacks: EffectDuration,
    },
    /// Upgrade an already present status effect with new potency and stacks.
    UpgradeStatusEffect {
        effect: StatusEffect,
        potency: usize,
        stacks: EffectDuration,
    },
    /// Add a certain amount to the counter, for use with "TerminateIfCounter"
    IncrementCounter {
        amount: i32,
        count: i32,
    },
    /// Transform a creature into another species.
    Transform {
        species: Species,
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
    /// If the synapse's counter is [condition] than the value, terminate.
    // NOTE: Instead of a SynapseFlag, it may be more interesting to simply fetch
    // the previous axiom and see if it is a counter.
    TerminateIfCounter {
        condition: CounterCondition,
        threshold: i32,
    },
    /// Remove all targets not targeting a creature of this species.
    FilterBySpecies {
        species: Species,
    },
    // End this spell.
    Terminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CounterCondition {
    LessThan,
    NotModuloOf { modulo: i32 },
}

/// The tracker of everything which determines how a certain spell will act.
#[derive(Debug)]
pub struct SynapseData {
    /// Where a spell will act.
    targets: HashSet<Position>,
    /// How a spell will act.
    pub axioms: Vec<Axiom>,
    /// The nth axiom currently being executed.
    pub step: usize,
    /// Who cast the spell.
    pub caster: Entity,
    /// Flags that alter the behaviour of an active synapse.
    synapse_flags: HashSet<SynapseFlag>,
    soul_caste: Soul,
}

impl SynapseData {
    /// Create a blank SynapseData.
    fn new(caster: Entity, axioms: Vec<Axiom>, step: usize, soul_caste: Soul) -> Self {
        SynapseData {
            targets: HashSet::new(),
            axioms,
            step,
            caster,
            synapse_flags: HashSet::new(),
            soul_caste,
        }
    }

    /// Get the Entity of each creature standing on a tile inside `targets` and its position.
    fn get_all_targeted_entity_pos_pairs(&self, map: &Map) -> Vec<(Entity, Position)> {
        let mut targeted_pairs = Vec::new();
        for target in &self.targets {
            if let Some(creature) = map.get_entity_at(target.x, target.y) {
                targeted_pairs.push((*creature, *target));
            }
        }
        targeted_pairs
    }

    /// Get the Entity of each creature standing on a tile inside `targets`.
    fn get_all_targeted_entities(&self, map: &Map) -> Vec<Entity> {
        self.get_all_targeted_entity_pos_pairs(map)
            .into_iter()
            .map(|(entity, _)| entity)
            .collect()
    }
}

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
    /// A Counter, to go in tandem with TerminateIfCounter
    Counter { count: i32 },
}

pub fn cast_new_spell(
    mut cast_spells: EventReader<CastSpell>,
    mut spell_stack: ResMut<SpellStack>,
) {
    for cast_spell in cast_spells.read() {
        // First, get the list of Axioms.
        let axioms = cast_spell.spell.axioms.clone();
        // Create a new synapse to start "rolling down the hill" accumulating targets and
        // dispatching events.
        let synapse_data = SynapseData::new(
            cast_spell.caster,
            axioms,
            cast_spell.starting_step,
            cast_spell.soul_caste,
        );
        // Send it off for processing - right away, for the spell stack is "last in, first out."
        spell_stack.spells.push(synapse_data);
    }
}

/// Target the caster's tile.
fn axiom_form_ego(
    In(spell_idx): In<usize>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
) {
    // Get the currently executed spell.
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    // Get the caster's position.
    let caster_position = *position.get(synapse_data.caster).unwrap();
    // Place the visual effect.
    magic_vfx.send(PlaceMagicVfx {
        targets: vec![caster_position],
        sequence: EffectSequence::Sequential { duration: 0.04 },
        effect: EffectType::RedBlast,
        decay: 0.5,
        appear: 0.,
    });
    // Add that caster's position to the targets.
    synapse_data.targets.insert(caster_position);
}

/// Target the player's tile.
fn axiom_form_player(
    In(spell_idx): In<usize>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position, With<Player>>,
) {
    // Get the currently executed spell.
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
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

/// Target all orthogonally adjacent tiles to the caster.
fn axiom_form_plus(
    In(spell_idx): In<usize>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
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
        effect: EffectType::GreenBlast,
        decay: 0.5,
        appear: 0.,
    });
    synapse_data.targets.extend(&output);
}

/// The targeted creatures dash in the direction of the caster's last move.
fn axiom_function_dash(
    In(spell_idx): In<usize>,
    library: Res<AxiomLibrary>,
    mut commands: Commands,
    map: Res<Map>,
    spell_stack: Res<SpellStack>,
    momentum: Query<&OrdDir>,
    is_spellproof: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    let caster_momentum = momentum.get(synapse_data.caster).unwrap();
    if let Axiom::Dash { max_distance } = synapse_data.axioms[synapse_data.step] {
        // For each (Entity, Position) on a targeted tile with a creature on it...
        for (dasher, dasher_pos) in synapse_data.get_all_targeted_entity_pos_pairs(&map) {
            // Spellproof entities cannot be affected.
            if is_spellproof.get(dasher).unwrap() {
                continue;
            }
            // The dashing creature starts where it currently is standing.
            let mut final_dash_destination = dasher_pos;
            // It will travel in the direction of the caster's last move.
            let (off_x, off_y) = caster_momentum.as_offset();
            // The dash has a maximum travel distance of `max_distance`.
            let mut distance_travelled = 0;
            while distance_travelled < max_distance {
                distance_travelled += 1;
                // Stop dashing if a solid Creature is hit (not implemented: "and the dasher is not intangible").
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
            commands.run_system_with_input(
                library.teleport,
                (
                    TeleportEntity {
                        destination: final_dash_destination,
                        entity: dasher,
                    },
                    spell_idx,
                ),
            );
        }
    } else {
        // This should NEVER trigger. This system was chosen to run because the
        // next axiom in the SpellStack explicitly requested it by being an Axiom::Dash.
        panic!()
    }
}

/// Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_momentum_beam(
    In(spell_idx): In<usize>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    position_and_momentum: Query<(&Position, &OrdDir)>,
    spellproof_query: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    let (caster_position, caster_momentum) =
        position_and_momentum.get(synapse_data.caster).unwrap();
    // Start the beam where the caster is standing.
    // The beam travels in the direction of the caster's last move.
    let (off_x, off_y) = caster_momentum.as_offset();
    let output = linear_beam(
        *caster_position,
        10,
        off_x,
        off_y,
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
        effect: match caster_momentum {
            OrdDir::Up | OrdDir::Down => EffectType::VerticalBeam,
            OrdDir::Right | OrdDir::Left => EffectType::HorizontalBeam,
        },
        decay: 0.5,
        appear: 0.,
    });
    // Add these tiles to `targets`.
    synapse_data.targets.extend(&output);
}

/// Fire 4 beams from the caster, towards the diagonal directions. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_xbeam(
    In(spell_idx): In<usize>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
    spellproof_query: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
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

/// Fire 4 beams from the caster, towards the cardinal directions. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_plus_beam(
    In(spell_idx): In<usize>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
    spellproof_query: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
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

/// Target the tile adjacent to the caster, towards the caster's last move.
fn axiom_form_touch(
    In(spell_idx): In<usize>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position_and_momentum: Query<(&Position, &OrdDir)>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    let (caster_position, caster_momentum) =
        position_and_momentum.get(synapse_data.caster).unwrap();
    let (off_x, off_y) = caster_momentum.as_offset();
    let touch = Position::new(caster_position.x + off_x, caster_position.y + off_y);
    synapse_data.targets.insert(touch);
    magic_vfx.send(PlaceMagicVfx {
        targets: vec![touch],
        sequence: EffectSequence::Sequential { duration: 0.04 },
        effect: EffectType::RedBlast,
        decay: 0.5,
        appear: 0.,
    });
}

/// Target a ring of `radius` around the caster.
fn axiom_form_halo(
    In(spell_idx): In<usize>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
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
        synapse_data.targets.extend(&circle);
    } else {
        panic!()
    }
}

/// The targeted passable tiles summon a new instance of species.
fn axiom_function_summon_creature(
    In(spell_idx): In<usize>,
    mut summon: EventWriter<SummonCreature>,
    spell_stack: Res<SpellStack>,
    position: Query<&Position>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    let caster_position = position.get(synapse_data.caster).unwrap();
    if let Axiom::SummonCreature { species } = synapse_data.axioms[synapse_data.step] {
        for position in &synapse_data.targets {
            summon.send(SummonCreature {
                species,
                position: *position,
                momentum: OrdDir::Down,
                summoner_tile: *caster_position,
                summoner: Some(synapse_data.caster),
                spellbook: None,
            });
        }
    } else {
        panic!()
    }
}

/// The targeted tiles summon a step-triggered trap with following axioms as the payload.
/// This terminates the spell.
fn axiom_function_place_step_trap(
    In(spell_idx): In<usize>,
    mut summon: EventWriter<SummonCreature>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    let caster_position = position.get(synapse_data.caster).unwrap();
    for position in &synapse_data.targets {
        summon.send(SummonCreature {
            species: Species::Trap,
            position: *position,
            momentum: OrdDir::Down,
            summoner_tile: *caster_position,
            summoner: Some(synapse_data.caster),
            spellbook: Some(Spellbook::new([
                None,
                None,
                Some(Spell {
                    axioms: {
                        let mut step_trigger = vec![Axiom::WhenSteppedOn];
                        step_trigger.extend(synapse_data.axioms[synapse_data.step + 1..].to_vec());
                        step_trigger
                    },
                }),
                None,
                None,
                None,
            ])),
        });
    }
    synapse_data.synapse_flags.insert(SynapseFlag::Terminate);
}

/// If the synapse's counter is [condition] than the value, terminate.
fn axiom_mutator_terminate_if_counter(
    In(spell_idx): In<usize>,
    mut spell_stack: ResMut<SpellStack>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();

    if let Axiom::TerminateIfCounter {
        condition,
        threshold,
    } = synapse_data.axioms[synapse_data.step]
    {
        if let Some(SynapseFlag::Counter { count }) = synapse_data
            .synapse_flags
            .iter()
            .find(|s| matches!(&s, SynapseFlag::Counter { .. }))
        {
            if match condition {
                CounterCondition::LessThan => count < &threshold,
                CounterCondition::NotModuloOf { modulo } => count % modulo != threshold,
            } {
                synapse_data.synapse_flags.insert(SynapseFlag::Terminate);
            }
        }
    } else {
        panic!()
    }
}

/// End this spell.
fn axiom_mutator_terminate(In(spell_idx): In<usize>, mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    synapse_data.synapse_flags.insert(SynapseFlag::Terminate);
}

/// Any targeted creature with the Wall component is removed.
/// Each removed wall heals the caster +1.
fn axiom_function_devour_wall(
    In(spell_idx): In<usize>,
    mut remove: EventWriter<RemoveCreature>,
    mut heal: EventWriter<DamageOrHealCreature>,
    spell_stack: Res<SpellStack>,
    map: Res<Map>,
    wall_check: Query<(Has<Wall>, Has<Spellproof>)>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    let mut total_heal: isize = 0;
    for entity in synapse_data.get_all_targeted_entities(&map) {
        let (is_wall, is_spellproof) = wall_check.get(entity).unwrap();
        if is_wall && !is_spellproof {
            remove.send(RemoveCreature { entity });
            total_heal = total_heal.saturating_add(1);
        }
    }
    heal.send(DamageOrHealCreature {
        entity: synapse_data.caster,
        culprit: synapse_data.caster,
        hp_mod: total_heal,
    });
}

/// All targeted creatures heal or are harmed by this amount.
fn axiom_function_heal_or_harm(
    In(spell_idx): In<usize>,
    mut heal: EventWriter<DamageOrHealCreature>,
    spell_stack: Res<SpellStack>,
    map: Res<Map>,
    is_spellproof: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
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

/// Give a status effect to all targeted creatures.
fn axiom_function_status_effect(
    In(spell_idx): In<usize>,
    mut status_effect: EventWriter<AddStatusEffect>,
    spell_stack: Res<SpellStack>,
    map: Res<Map>,
    is_spellproof: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    if let Axiom::StatusEffect {
        effect,
        potency,
        stacks,
    } = synapse_data.axioms[synapse_data.step]
    {
        for entity in synapse_data.get_all_targeted_entities(&map) {
            if !is_spellproof.get(entity).unwrap() {
                status_effect.send(AddStatusEffect {
                    entity,
                    effect,
                    potency,
                    stacks,
                    culprit: synapse_data.caster,
                });
            }
        }
    } else {
        panic!();
    }
}

/// Upgrade an already present status effect with new potency and stacks.
fn axiom_function_upgrade_status_effect(
    In(spell_idx): In<usize>,
    mut status_effect: EventWriter<AddStatusEffect>,
    creature_status_effect: Query<&mut StatusEffectsList>,
    spell_stack: Res<SpellStack>,
    map: Res<Map>,
    is_spellproof: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    if let Axiom::UpgradeStatusEffect {
        effect,
        potency,
        stacks,
    } = synapse_data.axioms[synapse_data.step]
    {
        for entity in synapse_data.get_all_targeted_entities(&map) {
            if !is_spellproof.get(entity).unwrap() {
                let status_list = creature_status_effect.get(entity).unwrap();
                if let Some(upgrade_effect) = status_list.effects.get(&effect) {
                    status_effect.send(AddStatusEffect {
                        entity,
                        effect,
                        potency: upgrade_effect.potency + potency,
                        stacks: upgrade_effect.stacks.add(stacks),
                        culprit: synapse_data.caster,
                    });
                }
            }
        }
    } else {
        panic!();
    }
}

fn axiom_function_increment_counter(
    In(spell_idx): In<usize>,
    mut spellbook: Query<&mut Spellbook>,
    mut spell_stack: ResMut<SpellStack>,
    is_spellproof: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    if let Axiom::IncrementCounter { amount, count } = synapse_data.axioms[synapse_data.step] {
        if !is_spellproof.get(synapse_data.caster).unwrap() {
            let mut book = spellbook.get_mut(synapse_data.caster).unwrap();
            // Access itself, deep inside the creature's spellbook
            let counter_axiom = book
                .spells
                .get_mut(&synapse_data.soul_caste)
                .unwrap()
                .axioms
                .get_mut(synapse_data.step)
                .unwrap();
            // It modifies itself, how cool is that
            let current_count = if let Axiom::IncrementCounter {
                amount: _amount_in_book,
                count: count_in_book,
            } = counter_axiom
            {
                *count_in_book = count.saturating_add(amount);
                count_in_book
            } else {
                panic!()
            };
            // Also add the flag for the if conditions.
            synapse_data.synapse_flags.insert(SynapseFlag::Counter {
                count: *current_count,
            });
        }
    } else {
        panic!();
    }
}

/// All creatures summoned by targeted creatures are removed.
fn axiom_function_abjuration(
    In(spell_idx): In<usize>,
    mut remove: EventWriter<RemoveCreature>,
    spell_stack: Res<SpellStack>,
    map: Res<Map>,
    summons: Query<(Entity, &Summoned)>,
    is_spellproof: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    for entity in synapse_data.get_all_targeted_entities(&map) {
        // Spellproof entities cannot be affected.
        if is_spellproof.get(entity).unwrap() {
            continue;
        }
        for (summoned_entity, summon) in summons.iter() {
            if summon.summoner == entity {
                remove.send(RemoveCreature {
                    entity: summoned_entity,
                });
            }
        }
    }
}

fn axiom_function_transform(
    In(spell_idx): In<usize>,
    spell_stack: Res<SpellStack>,
    map: Res<Map>,
    creature_query: Query<(Has<Spellproof>, &Species)>,
    mut transform: EventWriter<TransformCreature>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    if let Axiom::Transform { species } = synapse_data.axioms[synapse_data.step] {
        for entity in synapse_data.get_all_targeted_entities(&map) {
            let (is_spellproof, old_species) = creature_query.get(entity).unwrap();
            if !is_spellproof {
                transform.send(TransformCreature {
                    entity,
                    old_species: *old_species,
                    new_species: species,
                });
            }
        }
    }
}

/// Any Teleport event will target all tiles between its start and destination tiles.
fn axiom_mutator_trace(In(spell_idx): In<usize>, mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    synapse_data.synapse_flags.insert(SynapseFlag::Trace);
}

/// All Beam-type Forms will pierce through non-Spellproof creatures.
fn axiom_mutator_piercing_beams(In(spell_idx): In<usize>, mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    synapse_data
        .synapse_flags
        .insert(SynapseFlag::PiercingBeams);
}

/// All targeted tiles expand to also target their orthogonally adjacent tiles.
fn axiom_mutator_spread(
    In(spell_idx): In<usize>,
    mut spell_stack: ResMut<SpellStack>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
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

/// Remove the Caster's tile from targets.
fn axiom_mutator_untarget_caster(
    In(spell_idx): In<usize>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    let caster_position = position.get(synapse_data.caster).unwrap();
    synapse_data.targets.remove(caster_position);
}

/// Delete all targets.
fn axiom_mutator_purge_targets(In(spell_idx): In<usize>, mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    synapse_data.targets.clear();
}

/// Remove all targets not targeting a creature of this species.
fn axiom_mutator_filter_by_species(
    In(spell_idx): In<usize>,
    mut spell_stack: ResMut<SpellStack>,
    species_query: Query<&Species>,
    map: Res<Map>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    if let Axiom::FilterBySpecies { species } = synapse_data.axioms[synapse_data.step] {
        let mut retained_creatures = HashSet::new();
        for (entity, position) in synapse_data.get_all_targeted_entity_pos_pairs(&map) {
            if species == *species_query.get(entity).unwrap() {
                retained_creatures.insert(position);
            }
        }
        synapse_data.targets = retained_creatures;
    }
}

fn teleport_transmission(
    In((teleport_event, spell_idx)): In<(TeleportEntity, usize)>,
    position: Query<&Position>,
    mut teleport_writer: EventWriter<TeleportEntity>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
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

fn linear_beam(
    mut start: Position,
    max_distance: usize,
    off_x: i32,
    off_y: i32,
    map: &Map,
    is_piercing: bool,
    spellproof_query: &Query<Has<Spellproof>>,
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
        if is_piercing {
            if let Some(possible_block) = map.get_entity_at(start.x, start.y) {
                if spellproof_query.get(*possible_block).unwrap() {
                    break;
                }
            }
        } else if !map.is_passable(start.x, start.y) {
            break;
        }
    }
    output
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

/// Get the spells active this turn.
/// Get the next axiom, and runs its effects.
pub fn process_axiom(
    mut commands: Commands,
    axioms: Res<AxiomLibrary>,
    spell_stack: Res<SpellStack>,
) {
    // Get the spells active this turn.
    for (i, synapse_data) in spell_stack.spells.iter().enumerate() {
        // Get this spell's first axiom.
        let axiom = synapse_data.axioms.get(synapse_data.step).unwrap();
        // Launch the axiom, which will send out some Events (if it's a Function,
        // which affect the game world) or add some target tiles (if it's a Form, which
        // decides where the Functions will take place.)
        // Axioms not in the library are discarded: they are Contingencies.
        if let Some(one_shot_system) = axioms.library.get(&discriminant(axiom)) {
            commands.run_system_with_input(*one_shot_system, i);
        }
    }
}

/// Remove all terminated spells.
pub fn cleanup_synapses(mut spell_stack: ResMut<SpellStack>) {
    let mut renewed_spells = Vec::new();
    let len = spell_stack.spells.len();
    for mut synapse_data in spell_stack.spells.drain(0..len) {
        // Get the currently executed spell, removing it temporarily.
        // Step forwards in the axiom queue.
        synapse_data.step += 1;
        // If the spell is finished, do not push it back.
        // The Terminate flag also prevents further execution.
        if synapse_data.axioms.get(synapse_data.step).is_some()
            && !synapse_data.synapse_flags.contains(&SynapseFlag::Terminate)
        {
            renewed_spells.push(synapse_data);
        }
    }
    spell_stack.spells.append(&mut renewed_spells);
}

pub fn spell_stack_is_empty(spell_stack: Res<SpellStack>) -> bool {
    spell_stack.spells.is_empty()
}

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
