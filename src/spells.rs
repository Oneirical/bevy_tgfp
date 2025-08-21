use std::{
    cmp::Ordering,
    mem::{discriminant, Discriminant},
};

use bevy::{
    ecs::system::SystemId,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use uuid::Uuid;

use crate::{
    creature::{
        Awake, CreatureFlags, DoesNotLockInput, EffectDuration, FlagEntity, Intangible, Player,
        RealityBreak, RealityShield, Sleeping, Soul, Species, Spellbook, StatusEffect,
        StatusEffectsList, Summoned, Targeting, Wall,
    },
    events::{
        AddStatusEffect, ChooseStepAction, DamageOrHealCreature, OpenCloseDoor, RemoveCreature,
        SummonCreature, SummonProperties, TeleportEntity, TransformCreature,
    },
    graphics::{EffectSequence, EffectType, PlaceMagicVfx},
    map::{new_cage_on_conveyor, FaithsEnd, Map, Position},
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
            discriminant(&Axiom::TeleportDash { distance: 1 }),
            world.register_system(axiom_function_teleport_dash),
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
            discriminant(&Axiom::ToggleUntarget),
            world.register_system(axiom_mutator_toggle_untarget),
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
            discriminant(&Axiom::DisableVfx),
            world.register_system(axiom_mutator_disable_vfx),
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
        axioms.library.insert(
            discriminant(&Axiom::LoopBack { steps: 1 }),
            world.register_system(axiom_mutator_loop_back),
        );
        axioms.library.insert(
            discriminant(&Axiom::ForceCast),
            world.register_system(axiom_function_force_cast),
        );
        axioms.library.insert(
            discriminant(&Axiom::TargetIntangibleToo),
            world.register_system(axiom_mutator_target_intangible_too),
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

#[derive(Resource)]
/// All contingencies triggered this turn, to prevent infinite loops.
pub struct AntiContingencyLoop {
    pub contingencies_this_turn: HashSet<(Entity, Axiom)>,
}

#[derive(Event, Debug)]
/// Triggered when a creature performs an action corresponding to a certain Contingency.
pub struct TriggerContingency {
    pub caster: Entity,
    pub contingency: Axiom,
}

pub fn tick_time_contingency(
    mut contingency: EventWriter<TriggerContingency>,
    creatures: Query<(Entity, &Position, &Species, &Spellbook, &CreatureFlags)>,
    mut spell: EventWriter<CastSpell>,
    mut faith: ResMut<FaithsEnd>,
    mut open: EventWriter<OpenCloseDoor>,
    player_position: Query<&Position, With<Player>>,
    creatures_in_room: Query<&Position, Or<(With<Awake>, With<Sleeping>)>>,
    intangible_query: Query<&Intangible>,
    mut commands: Commands,
    map: Res<Map>,
) {
    let mut creatures =
        creatures
            .iter()
            .collect::<Vec<(Entity, &Position, &Species, &Spellbook, &CreatureFlags)>>();
    creatures.sort_by(|&a, &b| {
        // First, sort by whether the species is Airlock (Airlock comes first)
        // This is essential to make sure the conveyor belt is enabled before we start
        // iterating through the conveyor tiles.
        match (a.2 == &Species::Airlock, b.2 == &Species::Airlock) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.1.y.cmp(&b.1.y), // If both are or aren't Airlock, sort by Y coordinate
        }
    });
    for (creature, pos, species, spellbook, flags) in creatures.iter() {
        contingency.write(TriggerContingency {
            caster: *creature,
            contingency: Axiom::WhenTimePasses,
        });
        let (boundary_a, boundary_b) = (Position::new(25, 24), Position::new(35, 30));
        match species {
            // Species::ConveyorBelt | Species::Grinder => {
            //     if faith.conveyor_active || pos.y < 15 {
            //         spell.write(CastSpell {
            //             caster: *creature,
            //             spell: spellbook.spells.get(&Soul::Ordered).unwrap().clone(),
            //             starting_step: 0,
            //             soul_caste: Soul::Ordered,
            //             prediction: false,
            //         });
            //     }
            // }
            Species::Airlock => {
                if !faith.conveyor_active
                    && (pos == &&Position::new(25, 27)
                        || pos == &&Position::new(26, 27)
                        || pos == &&Position::new(34, 27)
                        || pos == &&Position::new(35, 27))
                {
                    if !player_position
                        .single()
                        .unwrap()
                        .is_within_range(&boundary_a, &boundary_b)
                    {
                        if {
                            let mut creature_left_in_the_room = false;
                            for pos in creatures_in_room.iter() {
                                if pos.is_within_range(&boundary_a, &boundary_b) {
                                    creature_left_in_the_room = true;
                                }
                            }
                            !creature_left_in_the_room
                        } {
                            if intangible_query.contains(flags.species_flags)
                                || intangible_query.contains(flags.effects_flags)
                            {
                                open.write(OpenCloseDoor {
                                    entity: *creature,
                                    open: false,
                                });
                            } else {
                                faith.conveyor_active = true;
                                commands.run_system_cached(new_cage_on_conveyor);
                            }
                        } else {
                            open.write(OpenCloseDoor {
                                entity: *creature,
                                open: true,
                            });
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

pub fn trigger_contingency(
    mut events: EventReader<TriggerContingency>,
    spellbook: Query<&Spellbook>,
    mut loop_protection: ResMut<AntiContingencyLoop>,
    mut cast_spell: EventWriter<CastSpell>,
) {
    for event in events.read() {
        if let Ok(spellbook) = spellbook.get(event.caster) {
            for (soul, spell) in spellbook.spells.iter() {
                if event.contingency != Axiom::WhenTimePasses
                    && loop_protection
                        .contingencies_this_turn
                        .contains(&(event.caster, event.contingency.clone()))
                {
                    // Do not allow infinite contingency loops
                    // (such as, "when dealing damage, deal damage")
                    break;
                }
                if let Some(contingency_index) = spell
                    .axioms
                    .iter()
                    .position(|axiom| axiom == &event.contingency)
                {
                    loop_protection
                        .contingencies_this_turn
                        .insert((event.caster, event.contingency.clone()));
                    cast_spell.write(CastSpell {
                        caster: event.caster,
                        spell: spell.clone(),
                        starting_step: contingency_index,
                        soul_caste: *soul,
                        prediction: false,
                    });
                }
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
    pub prediction: bool,
}

#[derive(Component, Clone, Debug)]
/// A spell is composed of a list of "Axioms", which will select tiles or execute an effect onto
/// those tiles, in the order they are listed.
pub struct Spell {
    pub axioms: Vec<Axiom>,
    pub caste: Soul,
    pub icon: usize,
    pub id: Uuid,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(u16)]
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
    // Triggers when this creature deals damage.
    WhenDealingDamage,
    // Triggers when this creature takes damage.
    WhenTakingDamage,
    // Triggers every 0.2 seconds.
    WhenTimePasses,

    // FORMS
    FormSeparator,
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
    FunctionSeparator,
    /// The targeted creatures dash in the direction of the caster's last move.
    Dash {
        max_distance: i32,
    },
    /// Offset targeted creatures' position in the direction of the caster's last move.
    TeleportDash {
        distance: i32,
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
    /// Force all creatures on targeted tiles to cast the remainder of the spell.
    /// This terminates execution of the spell.
    ForceCast,

    // MUTATORS
    MutatorSeparator,
    /// Any Teleport event will target all tiles between its start and destination tiles.
    Trace,
    /// All targeted tiles expand to also target their orthogonally adjacent tiles.
    Spread,
    /// Toggle Untarget flag on or off. While on, Forms remove targets instead
    /// of adding them..
    ToggleUntarget,
    /// All Beam-type Forms will pierce through non-Spellproof creatures.
    // NOTE: Maybe this could be used to make filters ban a species instead
    // of select only that species.
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
    /// Block visual magic effects from being drawn to the screen.
    DisableVfx,
    /// Only once, loop backwards `steps` in the axiom queue.
    LoopBack {
        steps: usize,
    },
    /// Also include intangible creatures in the targets.
    TargetIntangibleToo,
}

impl Axiom {
    // NOTE: After some discussion with my friend
    // I was informed that this implementation is irredeemable garbage.
    // It adds useless "FormSeparator" variants and takes memory for no reason.
    // The correct way to do it would involve:
    // enum Axiom { Form(Form), Function(Function) }
    // enum Form { Ego, Touch }
    // enum Function { Dash }
    // but refactoring this seems nightmarish so I'll let it stay like this for now.
    fn discriminant(&self) -> u16 {
        // SAFETY: Because `Self` is marked `repr(u16)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u16` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u16>() }
    }
    pub fn return_axiom_type(&self) -> AxiomType {
        if self.discriminant() < Axiom::FormSeparator.discriminant() {
            AxiomType::Contingency
        } else if self.discriminant() < Axiom::FunctionSeparator.discriminant() {
            AxiomType::Form
        } else if self.discriminant() < Axiom::MutatorSeparator.discriminant() {
            AxiomType::Function
        } else {
            AxiomType::Mutator
        }
    }
}

pub enum AxiomType {
    Contingency,
    Form,
    Function,
    Mutator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// The caste type of the spell.
    soul_caste: Soul,
    /// A cache of intangible creatures, to assist in targeting.
    intangible_cache: HashMap<Position, Entity>,
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
            intangible_cache: HashMap::new(),
        }
    }

    /// Get the Entity of each creature standing on a tile inside `targets` and its position.
    fn get_all_targeted_entity_pos_pairs(&self, map: &Map) -> Vec<(Entity, Position)> {
        let mut targeted_pairs = Vec::new();
        for target in &self.targets {
            if let Some(creature) = map.get_entity_at(target.x, target.y) {
                targeted_pairs.push((*creature, *target));
            }
            if self
                .synapse_flags
                .contains(&SynapseFlag::TargetIntangibleToo)
            {
                if let Some(creature) = self.intangible_cache.get(target) {
                    targeted_pairs.push((*creature, *target));
                }
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

    fn target_tiles(&mut self, target: Vec<Position>) {
        if self.synapse_flags.contains(&SynapseFlag::Untarget) {
            self.targets.retain(|&t| !target.contains(&t));
        } else {
            self.targets.extend(target);
        }
    }

    fn target_tile(&mut self, target: Position) {
        if self.synapse_flags.contains(&SynapseFlag::Untarget) {
            self.targets.remove(&target);
        } else {
            self.targets.insert(target);
        }
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
    /// Block visual magic effects from being drawn to the screen.
    DisableVfx,
    /// A fake spell with its Functions blocked, used by AI to contemplate whether or not it is
    /// wise to cast the real version of a spell.
    Prediction,
    /// Forms will remove targets instead of adding them.
    Untarget,
    /// Also include intangible creatures in the targets.
    TargetIntangibleToo,
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
        let mut synapse_data = SynapseData::new(
            cast_spell.caster,
            axioms,
            cast_spell.starting_step,
            cast_spell.soul_caste,
        );
        // Prediction-type spells should be invisible and not alter the gamestate.
        if cast_spell.prediction {
            synapse_data.synapse_flags.insert(SynapseFlag::Prediction);
            synapse_data.synapse_flags.insert(SynapseFlag::DisableVfx);
        }
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
    if !synapse_data
        .synapse_flags
        .contains(&SynapseFlag::DisableVfx)
    {
        magic_vfx.write(PlaceMagicVfx {
            targets: vec![caster_position],
            sequence: EffectSequence::Sequential { duration: 0.04 },
            effect: EffectType::RedBlast,
            decay: 0.5,
            appear: 0.,
        });
    }
    // Add that caster's position to the targets.
    synapse_data.target_tile(caster_position);
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
    // TODO replace this by ? when the bevy bug is fixed
    let player_position = *position.single().unwrap();
    // Place the visual effect.
    if !synapse_data
        .synapse_flags
        .contains(&SynapseFlag::DisableVfx)
    {
        magic_vfx.write(PlaceMagicVfx {
            targets: vec![player_position],
            sequence: EffectSequence::Sequential { duration: 0.04 },
            effect: EffectType::RedBlast,
            decay: 0.5,
            appear: 0.,
        });
    }
    // Add that caster's position to the targets.
    synapse_data.target_tile(player_position);
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
    if !synapse_data
        .synapse_flags
        .contains(&SynapseFlag::DisableVfx)
    {
        magic_vfx.write(PlaceMagicVfx {
            targets: output.clone(),
            sequence: EffectSequence::Sequential { duration: 0.04 },
            effect: EffectType::GreenBlast,
            decay: 0.5,
            appear: 0.,
        });
    }
    synapse_data.target_tiles(output);
}

/// The targeted creatures dash in the direction of the caster's last move.
fn axiom_function_dash(
    In(spell_idx): In<usize>,
    library: Res<AxiomLibrary>,
    mut commands: Commands,
    map: Res<Map>,
    spell_stack: Res<SpellStack>,
    momentum: Query<&OrdDir>,
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    flags: Query<&CreatureFlags>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    let caster_momentum = momentum.get(synapse_data.caster).unwrap();
    if let Axiom::Dash { mut max_distance } = synapse_data.axioms[synapse_data.step] {
        // For each (Entity, Position) on a targeted tile with a creature on it...
        for (dasher, dasher_pos) in synapse_data.get_all_targeted_entity_pos_pairs(&map) {
            // Spellproof entities cannot be affected.
            if is_spellproof(
                synapse_data.caster,
                dasher,
                &flags,
                &break_query,
                &shield_query,
            ) {
                continue;
            }
            // The dashing creature starts where it currently is standing.
            let mut final_dash_destination = dasher_pos;
            // It will travel in the direction of the caster's last move.
            let (mut off_x, mut off_y) = caster_momentum.as_offset();
            // If the max distance is negative, the direction of travel
            // is inverted.
            if max_distance.signum() == -1 {
                off_x = -off_x;
                off_y = -off_y;
                max_distance = -max_distance;
            }
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
            commands.run_system_with(
                library.teleport,
                (
                    TeleportEntity {
                        destination: final_dash_destination,
                        entity: dasher,
                        culprit: synapse_data.caster,
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

/// Offset targeted creatures' position by dx, dy.
fn axiom_function_teleport_dash(
    In(spell_idx): In<usize>,
    library: Res<AxiomLibrary>,
    mut commands: Commands,
    map: Res<Map>,
    spell_stack: Res<SpellStack>,
    momentum: Query<&OrdDir>,
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    flags: Query<&CreatureFlags>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    let caster_momentum = momentum.get(synapse_data.caster).unwrap();
    if let Axiom::TeleportDash { distance } = synapse_data.axioms[synapse_data.step] {
        // For each (Entity, Position) on a targeted tile with a creature on it...
        for (entity, entity_pos) in synapse_data.get_all_targeted_entity_pos_pairs(&map) {
            // Spellproof entities cannot be affected.
            if is_spellproof(
                synapse_data.caster,
                entity,
                &flags,
                &break_query,
                &shield_query,
            ) {
                continue;
            }
            let (off_x, off_y) = caster_momentum.as_offset();
            commands.run_system_with(
                library.teleport,
                (
                    TeleportEntity {
                        destination: Position::new(
                            entity_pos.x + off_x * distance,
                            entity_pos.y + off_y * distance,
                        ),
                        entity,
                        culprit: synapse_data.caster,
                    },
                    spell_idx,
                ),
            );
        }
    } else {
        // This should NEVER trigger. This system was chosen to run because the
        // next axiom in the SpellStack explicitly requested it by being an Axiom::Teleport.
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
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    flags: Query<&CreatureFlags>,
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
        synapse_data.caster,
        (&flags, &break_query, &shield_query),
    );
    // Add some visual beam effects.
    if !synapse_data
        .synapse_flags
        .contains(&SynapseFlag::DisableVfx)
    {
        magic_vfx.write(PlaceMagicVfx {
            targets: output.clone(),
            sequence: EffectSequence::Sequential { duration: 0.04 },
            effect: match caster_momentum {
                OrdDir::Up | OrdDir::Down => EffectType::VerticalBeam,
                OrdDir::Right | OrdDir::Left => EffectType::HorizontalBeam,
            },
            decay: 0.5,
            appear: 0.,
        });
    }
    // Add these tiles to `targets`.
    synapse_data.target_tiles(output);
}

/// Fire 4 beams from the caster, towards the diagonal directions. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_xbeam(
    In(spell_idx): In<usize>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    flags: Query<&CreatureFlags>,
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
            synapse_data.caster,
            (&flags, &break_query, &shield_query),
        );
        // Add some visual beam effects.
        if !synapse_data
            .synapse_flags
            .contains(&SynapseFlag::DisableVfx)
        {
            magic_vfx.write(PlaceMagicVfx {
                targets: output.clone(),
                sequence: EffectSequence::Sequential { duration: 0.04 },
                effect: EffectType::RedBlast,
                decay: 0.5,
                appear: 0.,
            });
        }
        // Add these tiles to `targets`.
        synapse_data.target_tiles(output);
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
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    flags: Query<&CreatureFlags>,
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
            synapse_data.caster,
            (&flags, &break_query, &shield_query),
        );
        // Add some visual beam effects.
        if !synapse_data
            .synapse_flags
            .contains(&SynapseFlag::DisableVfx)
        {
            magic_vfx.write(PlaceMagicVfx {
                targets: output.clone(),
                sequence: EffectSequence::Sequential { duration: 0.04 },
                effect: match cardinal {
                    OrdDir::Up | OrdDir::Down => EffectType::VerticalBeam,
                    OrdDir::Right | OrdDir::Left => EffectType::HorizontalBeam,
                },
                decay: 0.5,
                appear: 0.,
            });
        }
        // Add these tiles to `targets`.
        synapse_data.target_tiles(output);
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
    synapse_data.target_tile(touch);
    if !synapse_data
        .synapse_flags
        .contains(&SynapseFlag::DisableVfx)
    {
        magic_vfx.write(PlaceMagicVfx {
            targets: vec![touch],
            sequence: EffectSequence::Sequential { duration: 0.04 },
            effect: EffectType::RedBlast,
            decay: 0.5,
            appear: 0.,
        });
    }
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
        if !synapse_data
            .synapse_flags
            .contains(&SynapseFlag::DisableVfx)
        {
            magic_vfx.write(PlaceMagicVfx {
                targets: circle.clone(),
                sequence: EffectSequence::Sequential { duration: 0.04 },
                effect: EffectType::GreenBlast,
                decay: 0.5,
                appear: 0.,
            });
        }
        // Add these tiles to `targets`.
        synapse_data.target_tiles(circle);
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
            summon.write(SummonCreature {
                species,
                position: *position,
                properties: vec![SummonProperties::Summoned {
                    summoner_tile: *caster_position,
                    summoner: synapse_data.caster,
                }],
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
        summon.write(SummonCreature {
            species: Species::Trap,
            position: *position,
            properties: vec![
                SummonProperties::Summoned {
                    summoner_tile: *caster_position,
                    summoner: synapse_data.caster,
                },
                SummonProperties::Spellbook(Spellbook::new([
                    None,
                    None,
                    Some({
                        let mut step_trigger = vec![Axiom::WhenSteppedOn];
                        step_trigger.extend(synapse_data.axioms[synapse_data.step + 1..].to_vec());
                        step_trigger
                    }),
                    None,
                    None,
                    None,
                ])),
            ],
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

/// Also include intangible creatures in the targets.
fn axiom_mutator_target_intangible_too(
    In(spell_idx): In<usize>,
    mut spell_stack: ResMut<SpellStack>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    synapse_data
        .synapse_flags
        .insert(SynapseFlag::TargetIntangibleToo);
}

/// Disable all visual magic effects.
fn axiom_mutator_disable_vfx(In(spell_idx): In<usize>, mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    synapse_data.synapse_flags.insert(SynapseFlag::DisableVfx);
}

/// Any targeted creature with the Wall component is removed.
/// Each removed wall heals the caster +1.
fn axiom_function_devour_wall(
    In(spell_idx): In<usize>,
    mut remove: EventWriter<RemoveCreature>,
    mut heal: EventWriter<DamageOrHealCreature>,
    spell_stack: Res<SpellStack>,
    map: Res<Map>,
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    wall_query: Query<&Wall>,
    flags: Query<&CreatureFlags>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    let mut total_heal: isize = 0;
    for entity in synapse_data.get_all_targeted_entities(&map) {
        let is_wall = {
            let flags = flags.get(entity).unwrap();
            wall_query.contains(flags.effects_flags) || wall_query.contains(flags.species_flags)
        };
        let is_spellproof = is_spellproof(
            synapse_data.caster,
            entity,
            &flags,
            &break_query,
            &shield_query,
        );

        if is_wall && !is_spellproof {
            remove.write(RemoveCreature { entity });
            total_heal = total_heal.saturating_add(1);
        }
    }
    heal.write(DamageOrHealCreature {
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
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    flags: Query<&CreatureFlags>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    if let Axiom::HealOrHarm { amount } = synapse_data.axioms[synapse_data.step] {
        for entity in synapse_data.get_all_targeted_entities(&map) {
            if is_spellproof(
                synapse_data.caster,
                entity,
                &flags,
                &break_query,
                &shield_query,
            ) {
                continue;
            }
            heal.write(DamageOrHealCreature {
                entity,
                culprit: synapse_data.caster,
                hp_mod: amount,
            });
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
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    flags: Query<&CreatureFlags>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    if let Axiom::StatusEffect {
        effect,
        potency,
        stacks,
    } = synapse_data.axioms[synapse_data.step]
    {
        for entity in synapse_data.get_all_targeted_entities(&map) {
            if is_spellproof(
                synapse_data.caster,
                entity,
                &flags,
                &break_query,
                &shield_query,
            ) {
                continue;
            }
            status_effect.write(AddStatusEffect {
                entity,
                effect,
                potency,
                stacks,
                culprit: synapse_data.caster,
            });
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
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    flags: Query<&CreatureFlags>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    if let Axiom::UpgradeStatusEffect {
        effect,
        potency,
        stacks,
    } = synapse_data.axioms[synapse_data.step]
    {
        for entity in synapse_data.get_all_targeted_entities(&map) {
            if is_spellproof(
                synapse_data.caster,
                entity,
                &flags,
                &break_query,
                &shield_query,
            ) {
                continue;
            }
            let status_list = creature_status_effect.get(entity).unwrap();
            if let Some(upgrade_effect) = status_list.effects.get(&effect) {
                status_effect.write(AddStatusEffect {
                    entity,
                    effect,
                    potency: upgrade_effect.potency + potency,
                    stacks: upgrade_effect.stacks.add(stacks),
                    culprit: synapse_data.caster,
                });
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
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    flags: Query<&CreatureFlags>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    if let Axiom::IncrementCounter { amount, count } = synapse_data.axioms[synapse_data.step] {
        if !is_spellproof(
            synapse_data.caster,
            synapse_data.caster,
            &flags,
            &break_query,
            &shield_query,
        ) {
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
    summons: Query<(&Summoned, &FlagEntity)>,
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    flags: Query<&CreatureFlags>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    for entity in synapse_data.get_all_targeted_entities(&map) {
        // Spellproof entities cannot be affected.
        if is_spellproof(
            synapse_data.caster,
            entity,
            &flags,
            &break_query,
            &shield_query,
        ) {
            continue;
        }
        for (summoned_component, flag_entity) in summons.iter() {
            if summoned_component.summoner == entity {
                remove.write(RemoveCreature {
                    entity: flag_entity.parent_creature,
                });
            }
        }
    }
}

fn axiom_function_transform(
    In(spell_idx): In<usize>,
    spell_stack: Res<SpellStack>,
    map: Res<Map>,
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    flags: Query<&CreatureFlags>,
    mut transform: EventWriter<TransformCreature>,
) {
    let synapse_data = spell_stack.spells.get(spell_idx).unwrap();
    if let Axiom::Transform { species } = synapse_data.axioms[synapse_data.step] {
        for entity in synapse_data.get_all_targeted_entities(&map) {
            if is_spellproof(
                synapse_data.caster,
                entity,
                &flags,
                &break_query,
                &shield_query,
            ) {
                continue;
            }
            transform.write(TransformCreature {
                entity,
                new_species: species,
            });
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
        if !synapse_data
            .synapse_flags
            .contains(&SynapseFlag::DisableVfx)
        {
            magic_vfx.write(PlaceMagicVfx {
                targets: ord_dir_vec.clone(),
                sequence: EffectSequence::Sequential { duration: 0.04 },
                effect: EffectType::RedBlast,
                decay: 0.5,
                appear: 0.,
            });
        }
        synapse_data.target_tiles(ord_dir_vec);
    }
}

/// Make it so future Forms will remove targets instead of adding them..
fn axiom_mutator_toggle_untarget(In(spell_idx): In<usize>, mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    synapse_data.synapse_flags.insert(SynapseFlag::Untarget);
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

/// Only once, loop backwards `steps` in the axiom queue.
fn axiom_mutator_loop_back(In(spell_idx): In<usize>, mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    if let Axiom::LoopBack { steps } = synapse_data.axioms[synapse_data.step] {
        // Remove the LoopBack.
        synapse_data.axioms.remove(synapse_data.step);
        // Rewind back n steps. Prevent the cleanup from adding one step by default.
        synapse_data.step = synapse_data.step.saturating_sub(steps);
        synapse_data.synapse_flags.insert(SynapseFlag::NoStep);
    } else {
        panic!()
    }
}

/// Force all creatures on targeted tiles to cast the remainder of the spell.
/// This terminates execution of the spell.
fn axiom_function_force_cast(
    In(spell_idx): In<usize>,
    mut cast_spell: EventWriter<CastSpell>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    break_query: Query<&RealityBreak>,
    shield_query: Query<&RealityShield>,
    flags: Query<&CreatureFlags>,
) {
    let synapse_data = spell_stack.spells.get_mut(spell_idx).unwrap();
    for entity in synapse_data.get_all_targeted_entities(&map) {
        if is_spellproof(
            synapse_data.caster,
            entity,
            &flags,
            &break_query,
            &shield_query,
        ) {
            continue;
        }
        cast_spell.write(CastSpell {
            caster: entity,
            spell: Spell {
                axioms: synapse_data.axioms[synapse_data.step + 1..].to_vec(),
                caste: Soul::Saintly,
                icon: 170,
                id: Uuid::new_v4(),
                description: String::new(),
            },
            soul_caste: synapse_data.soul_caste,
            starting_step: 0,
            prediction: false,
        });
    }
    synapse_data.synapse_flags.insert(SynapseFlag::Terminate);
}

/// This is required for effects such as Trace.
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
            if !synapse_data
                .synapse_flags
                .contains(&SynapseFlag::DisableVfx)
            {
                magic_vfx.write(PlaceMagicVfx {
                    targets: output.clone(),
                    sequence: EffectSequence::Sequential { duration: 0.04 },
                    effect: EffectType::RedBlast,
                    decay: 0.5,
                    appear: 0.,
                });
            }
            // Add these tiles to `targets`.
            synapse_data.target_tiles(output);
        }
    }
    teleport_writer.write(teleport_event);
}

fn linear_beam(
    mut start: Position,
    max_distance: usize,
    off_x: i32,
    off_y: i32,
    map: &Map,
    is_piercing: bool,
    caster: Entity,
    queries: (
        &Query<&CreatureFlags>,
        &Query<&RealityBreak>,
        &Query<&RealityShield>,
    ),
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
                if is_spellproof(caster, *possible_block, queries.0, queries.1, queries.2) {
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

pub fn collect_intangible(
    In(synapse_idx): In<usize>,
    mut spell_stack: ResMut<SpellStack>,
    intangible: Query<&FlagEntity, With<Intangible>>,
    position: Query<&Position>,
) {
    let synapse = spell_stack.spells.get_mut(synapse_idx).unwrap();
    synapse.intangible_cache.clear();
    for flag_entity in intangible.iter() {
        synapse.intangible_cache.insert(
            *position.get(flag_entity.parent_creature).unwrap(),
            flag_entity.parent_creature,
        );
    }
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
        if matches!(axiom.return_axiom_type(), AxiomType::Function) {
            if synapse_data
                .synapse_flags
                .contains(&SynapseFlag::Prediction)
            {
                // Predictor-type spells should ignore any Axiom that would affect the gamestate.
                continue;
            } else if synapse_data
                .synapse_flags
                .contains(&SynapseFlag::TargetIntangibleToo)
            {
                commands.run_system_cached_with(collect_intangible, i);
            }
        }
        // Launch the axiom, which will send out some Events (if it's a Function,
        // which affect the game world) or add some target tiles (if it's a Form, which
        // decides where the Functions will take place.)
        // Axioms not in the library are discarded: they are Contingencies.
        if let Some(one_shot_system) = axioms.library.get(&discriminant(axiom)) {
            commands.run_system_with(*one_shot_system, i);
        }
    }
}

/// Remove all terminated spells.
pub fn cleanup_synapses(mut spell_stack: ResMut<SpellStack>, mut commands: Commands) {
    let mut renewed_spells = Vec::new();
    let len = spell_stack.spells.len();
    // We count the number of prediction (fake spells) used by each creature.
    let mut predictions = spell_stack
        .spells
        .iter()
        .filter(|s| s.synapse_flags.contains(&SynapseFlag::Prediction))
        .fold(HashMap::new(), |mut acc, s| {
            *acc.entry(s.caster).or_insert(0) += 1;
            acc
        });
    for mut synapse_data in spell_stack.spells.drain(0..len) {
        // Get the currently executed spell, removing it temporarily.
        // Step forwards in the axiom queue, if it is allowed.
        if synapse_data.synapse_flags.contains(&SynapseFlag::NoStep) {
            synapse_data.synapse_flags.remove(&SynapseFlag::NoStep);
        } else {
            synapse_data.step += 1;
        }
        // If the spell is finished, do not push it back.
        // The Terminate flag also prevents further execution.
        if synapse_data.axioms.get(synapse_data.step).is_some()
            && !synapse_data.synapse_flags.contains(&SynapseFlag::Terminate)
        {
            renewed_spells.push(synapse_data);
        } else if synapse_data
            .synapse_flags
            .contains(&SynapseFlag::Prediction)
        {
            // Only if this is the last possible prediction does this return 0,
            // which enables the creature to choose non-spellcasting actions instead.
            let number_of_predictions = {
                let count = predictions.get_mut(&synapse_data.caster).unwrap();
                *count -= 1;
                *count
            };
            // The prediction has completed, judge if it should be turned into an actual spell.
            commands.run_system_cached_with(
                ai_prediction_into_action,
                (synapse_data, number_of_predictions),
            );
        }
    }
    spell_stack.spells.append(&mut renewed_spells);
}

/// Consider transforming Predictions into actual spells
pub fn ai_prediction_into_action(
    In((synapse, number_of_predictions)): In<(SynapseData, usize)>,
    flags: Query<&CreatureFlags>,
    spellbook: Query<&Spellbook>,
    target_query: Query<&Targeting>,
    species_query: Query<&Species>,
    map: Res<Map>,
    mut spell: EventWriter<CastSpell>,
    mut commands: Commands,
    // TODO: Change this to -> Result after bevy bug is fixed
) {
    let targets = synapse.get_all_targeted_entities(&map);
    let species_target_list: Vec<Species> = {
        let flags = flags.get(synapse.caster).unwrap();
        target_query
            .get(flags.species_flags)
            .map(|t| t.0.clone())
            .unwrap_or_default()
            .into_iter()
            .chain(
                target_query
                    .get(flags.effects_flags)
                    .map(|t| t.0.clone())
                    .unwrap_or_default(),
            )
            .collect()
    };
    if species_target_list.is_empty() && number_of_predictions == 0 {
        // Early return if we are not looking to target anything.
        commands.trigger_targets(ChooseStepAction, synapse.caster);
        return;
    }
    for target in targets {
        let species = species_query.get(target).unwrap();
        // If a species we WANT to target with our spell was caught in the prediction,
        // fire the actual spell at it.
        if species_target_list.contains(species) {
            spell.write(CastSpell {
                caster: synapse.caster,
                spell: spellbook
                    .get(synapse.caster)
                    .unwrap()
                    .spells
                    .get(&synapse.soul_caste)
                    .unwrap()
                    .clone(),
                starting_step: 0,
                soul_caste: synapse.soul_caste,
                prediction: false,
            });
            return;
        }
    }
    // There needs to be no other spells waiting for their prediction to
    // resolve to assert "well, guess I'll take a step this turn"
    if number_of_predictions == 0 {
        // If no spell was fired, take a step instead.
        commands.trigger_targets(ChooseStepAction, synapse.caster);
    }
}

pub fn spell_stack_is_empty(
    spell_stack: Res<SpellStack>,
    flags: Query<&CreatureFlags>,
    no_disrupt: Query<&DoesNotLockInput>,
) -> bool {
    spell_stack
        .spells
        .iter()
        .filter(|&s| {
            let flags = flags.get(s.caster).unwrap();
            !(no_disrupt.contains(flags.species_flags) || no_disrupt.contains(flags.effects_flags))
        })
        .count()
        == 0
}

pub fn walk_grid(p0: Position, p1: Position) -> Vec<Position> {
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

fn is_spellproof(
    caster: Entity,
    target: Entity,
    creature_flags: &Query<&CreatureFlags>,
    caster_query: &Query<&RealityBreak>,
    target_query: &Query<&RealityShield>,
) -> bool {
    let flags_caster = creature_flags.get(caster).unwrap();
    let flags_target = creature_flags.get(target).unwrap();
    let pierce = caster_query
        .get(flags_caster.species_flags)
        .or(caster_query.get(flags_caster.effects_flags))
        .unwrap_or(&RealityBreak(0));
    let shield = target_query
        .get(flags_target.species_flags)
        .or(target_query.get(flags_target.effects_flags))
        .unwrap_or(&RealityShield(0));

    pierce.0 < shield.0
}
