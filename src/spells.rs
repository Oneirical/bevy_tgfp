use std::{
    collections::VecDeque,
    mem::{discriminant, Discriminant},
};

use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};

use crate::{
    creature::Species,
    events::{RepressionDamage, SummonCreature, TeleportEntity},
    graphics::{EffectSequence, EffectType, PlaceMagicVfx},
    map::{Map, Position},
    OrdDir,
};

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CastSpell>();
        app.init_resource::<SpellStack>();
        app.init_resource::<AxiomLibrary>();
    }
}

#[derive(Event)]
/// Triggered when a creature (the `caster`) casts a `spell`.
pub struct CastSpell {
    pub caster: Entity,
    pub spell: Spell,
}

#[derive(Resource)]
/// The current spells being executed.
pub struct SpellStack {
    spells: Vec<SynapseData>,
    cleanup_id: SystemId,
}

impl FromWorld for SpellStack {
    fn from_world(world: &mut World) -> Self {
        let stack = SpellStack {
            spells: Vec::new(),
            cleanup_id: world.register_system(cleanup_last_axiom),
        };
        stack
    }
}

#[derive(Resource)]
/// All available Axioms and their corresponding systems.
pub struct AxiomLibrary {
    pub library: HashMap<Discriminant<Axiom>, SystemId>,
}

impl FromWorld for AxiomLibrary {
    fn from_world(world: &mut World) -> Self {
        let mut axioms = AxiomLibrary {
            library: HashMap::new(),
        };
        axioms.library.insert(
            discriminant(&Axiom::Ego),
            world.register_system(axiom_form_ego),
        );
        axioms.library.insert(
            discriminant(&Axiom::Plus),
            world.register_system(axiom_form_plus),
        );
        axioms.library.insert(
            discriminant(&Axiom::MomentumBeam),
            world.register_system(axiom_form_momentum_beam),
        );
        axioms.library.insert(
            discriminant(&Axiom::XBeam),
            world.register_system(axiom_form_xbeam),
        );
        axioms.library.insert(
            discriminant(&Axiom::Dash),
            world.register_system(axiom_function_dash),
        );
        axioms.library.insert(
            discriminant(&Axiom::SummonCreature {
                species: Species::Player,
            }),
            world.register_system(axiom_function_summon_creature),
        );
        axioms.library.insert(
            discriminant(&Axiom::RepressionDamage { damage: 1 }),
            world.register_system(axiom_function_repression_damage),
        );
        axioms
    }
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
    // Target all orthogonally adjacent tiles to the caster.
    Plus,
    // Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
    // including the first solid tile encountered, which stops the beam.
    MomentumBeam,
    // Fire 4 beams from the caster, towards the diagonal directions. Target all travelled tiles,
    // including the first solid tile encountered, which stops the beam.
    XBeam,

    // FUNCTIONS

    // The targeted creatures dash in the direction of the caster's last move.
    Dash,
    // The targeted passable tiles summon a new instance of species.
    SummonCreature { species: Species },
    // Deal damage to all creatures on targeted tiles.
    RepressionDamage { damage: i32 },
}

/// Target the caster's tile.
fn axiom_form_ego(mut magic_vfx: EventWriter<PlaceMagicVfx>, mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.get_mut(0).unwrap();
    magic_vfx.send(PlaceMagicVfx {
        targets: vec![synapse_data.caster_position],
        sequence: EffectSequence::Sequential { duration: 0.4 },
        effect: EffectType::RedBlast,
        decay: 0.5,
    });
    synapse_data.targets.push(synapse_data.caster_position);
}

/// Target all orthogonally adjacent tiles to the caster.
fn axiom_form_plus(mut magic_vfx: EventWriter<PlaceMagicVfx>, mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.get_mut(0).unwrap();
    let adjacent = [OrdDir::Up, OrdDir::Right, OrdDir::Down, OrdDir::Left];
    let mut output = Vec::new();
    for direction in adjacent {
        let mut new_pos = synapse_data.caster_position;
        let offset = direction.as_offset();
        new_pos.shift(offset.0, offset.1);
        output.push(new_pos);
    }
    magic_vfx.send(PlaceMagicVfx {
        targets: output.clone(),
        sequence: EffectSequence::Sequential { duration: 0.4 },
        effect: EffectType::RedBlast,
        decay: 0.5,
    });
    // Add these tiles to `targets`.
    synapse_data.targets.append(&mut output);
}

/// Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_momentum_beam(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
) {
    let synapse_data = spell_stack.spells.get_mut(0).unwrap();
    // Start the beam where the caster is standing.
    // The beam travels in the direction of the caster's last move.
    let (off_x, off_y) = synapse_data.caster_momentum.as_offset();
    let mut output = linear_beam(synapse_data.caster_position, 10, off_x, off_y, &map);
    // Add some visual beam effects.
    magic_vfx.send(PlaceMagicVfx {
        targets: output.clone(),
        sequence: EffectSequence::Sequential { duration: 0.4 },
        effect: match synapse_data.caster_momentum {
            OrdDir::Up | OrdDir::Down => EffectType::VerticalBeam,
            OrdDir::Right | OrdDir::Left => EffectType::HorizontalBeam,
        },
        decay: 0.5,
    });
    // Add these tiles to `targets`.
    synapse_data.targets.append(&mut output);
}

/// Fire 4 beams from the caster, towards the diagonal directions. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_xbeam(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
) {
    let synapse_data = spell_stack.spells.get_mut(0).unwrap();
    let diagonals = [(1, 1), (-1, 1), (1, -1), (-1, -1)];
    for (dx, dy) in diagonals {
        // Start the beam where the caster is standing.
        // The beam travels in the direction of each diagonal.
        let mut output = linear_beam(synapse_data.caster_position, 10, dx, dy, &map);
        // Add some visual beam effects.
        magic_vfx.send(PlaceMagicVfx {
            targets: output.clone(),
            sequence: EffectSequence::Sequential { duration: 0.4 },
            effect: EffectType::RedBlast,
            decay: 0.5,
        });
        // Add these tiles to `targets`.
        synapse_data.targets.append(&mut output);
    }
}

/// The targeted passable tiles summon a new instance of species.
fn axiom_function_summon_creature(
    mut summon: EventWriter<SummonCreature>,
    spell_stack: Res<SpellStack>,
) {
    let synapse_data = spell_stack.spells.get(0).unwrap();
    if let Axiom::SummonCreature { species } = synapse_data.axioms[synapse_data.step] {
        for position in &synapse_data.targets {
            summon.send(SummonCreature {
                species,
                position: *position,
            });
        }
    } else {
        panic!()
    }
}

/// Deal damage to all creatures on targeted tiles.
fn axiom_function_repression_damage(
    mut repression_damage: EventWriter<RepressionDamage>,
    map: Res<Map>,
    spell_stack: Res<SpellStack>,
) {
    let synapse_data = spell_stack.spells.get(0).unwrap();
    if let Axiom::RepressionDamage { damage } = synapse_data.axioms[synapse_data.step] {
        for entity in synapse_data.get_all_targeted_entities(&map) {
            repression_damage.send(RepressionDamage { entity, damage });
        }
    } else {
        panic!()
    }
}

/// The targeted creatures dash in the direction of the caster's last move.
fn axiom_function_dash(
    mut teleport: EventWriter<TeleportEntity>,
    map: Res<Map>,
    spell_stack: Res<SpellStack>,
) {
    let synapse_data = spell_stack.spells.get(0).unwrap();
    // For each (Entity, Position) on a targeted tile...
    for (dasher, dasher_pos) in synapse_data.get_all_targeted_entity_pos_pairs(&map) {
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
        teleport.send(TeleportEntity {
            destination: final_dash_destination,
            entity: dasher,
        });
    }
}

fn linear_beam(
    mut start: Position,
    max_distance: usize,
    off_x: i32,
    off_y: i32,
    map: &Map,
) -> Vec<Position> {
    let mut distance_travelled = 0;
    let mut output = Vec::new();
    // The beam has a maximum distance of 10.
    while distance_travelled < max_distance {
        distance_travelled += 1;
        start.shift(off_x, off_y);
        // The new tile is always added, even if it is impassable...
        output.push(start);
        // But if it is impassable, it is the last added tile.
        if !map.is_passable(start.x, start.y) {
            break;
        }
    }
    output
}

/// The tracker of everything which determines how a certain spell will act.
struct SynapseData {
    /// Where a spell will act.
    targets: Vec<Position>,
    /// How a spell will act.
    axioms: VecDeque<Axiom>,
    /// The nth axiom currently being executed.
    step: usize,
    /// Who cast the spell.
    caster: Entity,
    /// In which direction did the caster move the last time they did so?
    // NOTE: This could be done with a Query instead, but it's accessed
    // so commonly that this field exists for convenience.
    caster_momentum: OrdDir,
    /// Where is the caster on the map?
    // NOTE: This could be done with a Query instead, but it's accessed
    // so commonly that this field exists for convenience.
    caster_position: Position,
}

impl SynapseData {
    /// Create a blank SynapseData.
    fn new(
        caster: Entity,
        caster_momentum: OrdDir,
        caster_position: Position,
        axioms: VecDeque<Axiom>,
    ) -> Self {
        SynapseData {
            targets: Vec::new(),
            axioms,
            step: 0,
            caster,
            caster_momentum,
            caster_position,
        }
    }

    fn get_all_targeted_entities(&self, map: &Map) -> Vec<Entity> {
        self.get_all_targeted_entity_pos_pairs(map)
            .into_iter()
            .map(|(entity, _)| entity)
            .collect()
    }

    fn get_all_targeted_entity_pos_pairs(&self, map: &Map) -> Vec<(Entity, Position)> {
        let mut targeted_pairs = Vec::new();
        for target in &self.targets {
            if let Some(creatures) = map.get_creatures_at(target.x, target.y) {
                for creature in creatures {
                    targeted_pairs.push((creature.entity, *target));
                }
            }
        }
        targeted_pairs
    }
}

pub fn queue_up_spell(
    mut cast_spells: EventReader<CastSpell>,
    mut spell_stack: ResMut<SpellStack>,
    caster: Query<(&Position, &OrdDir)>,
) {
    for cast_spell in cast_spells.read() {
        // First, get the list of Axioms.
        let axioms = VecDeque::from(cast_spell.spell.axioms.clone());
        // And the caster's position and last move direction.
        let (caster_position, caster_momentum) = caster.get(cast_spell.caster).unwrap();

        // Create a new synapse to start "rolling down the hill" accumulating targets and
        // dispatching events.
        let synapse_data = SynapseData::new(
            cast_spell.caster,
            *caster_momentum,
            *caster_position,
            axioms,
        );
        // Send it off for processing - right away, for the spell stack is "last in, first out."
        spell_stack.spells.push(synapse_data);
    }
}

pub fn spell_stack_is_not_empty(spell_stack: Res<SpellStack>) -> bool {
    !spell_stack.spells.is_empty()
}

/// Pops the most recently added spell (re-adding it at the end if it's not complete yet).
/// Pops the next axiom, and runs its effects.
/// This will not run if `spell_stack_is_empty`.
pub fn process_axiom(
    mut commands: Commands,
    axioms: Res<AxiomLibrary>,
    spell_stack: Res<SpellStack>,
) {
    // Get the most recently added spell, if it exists.
    if let Some(synapse_data) = spell_stack.spells.get(0) {
        // Get its first axiom.
        let axiom = synapse_data.axioms.get(synapse_data.step).unwrap();
        // Launch the axiom, which will send out some Events (if it's a Function,
        // which affect the game world) or add some target tiles (if it's a Form, which
        // decides where the Functions will take place.)
        commands.run_system(*axioms.library.get(&discriminant(axiom)).unwrap());
        // Clean up afterwards, continuing the spell execution.
        commands.run_system(spell_stack.cleanup_id);
    }
}

fn cleanup_last_axiom(mut spell_stack: ResMut<SpellStack>) {
    // Get the currently executed spell, removing it temporarily.
    let mut synapse_data = spell_stack.spells.pop().unwrap();
    // Step forwards in the axiom queue.
    synapse_data.step += 1;
    // If the spell is finished, do not push it back.
    if synapse_data.axioms.get(synapse_data.step).is_some() {
        spell_stack.spells.push(synapse_data);
    }
}
