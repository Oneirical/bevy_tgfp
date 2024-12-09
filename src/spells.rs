use std::mem::{discriminant, Discriminant};

use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};

use crate::{
    events::TeleportEntity,
    graphics::{EffectSequence, EffectType, PlaceMagicVfx},
    map::{Map, Position},
    OrdDir,
};

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<CastSpell>>();
        app.init_resource::<SpellStack>();
        app.init_resource::<AxiomLibrary>();
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
            discriminant(&Axiom::MomentumBeam),
            world.register_system(axiom_form_momentum_beam),
        );
        axioms.library.insert(
            discriminant(&Axiom::Touch),
            world.register_system(axiom_form_touch),
        );
        axioms.library.insert(
            discriminant(&Axiom::Dash { max_distance: 1 }),
            world.register_system(axiom_function_dash),
        );
        axioms
    }
}

#[derive(Resource)]
/// The current spells being executed.
pub struct SpellStack {
    /// The stack of spells, last in, first out.
    pub spells: Vec<SynapseData>,
    /// A system used to clean up the last spells after each Axiom is processed.
    cleanup_id: SystemId,
}

impl FromWorld for SpellStack {
    fn from_world(world: &mut World) -> Self {
        SpellStack {
            spells: Vec::new(),
            cleanup_id: world.register_system(cleanup_last_axiom),
        }
    }
}

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
    /// Target the caster's tile.
    Ego,
    /// Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
    /// including the first solid tile encountered, which stops the beam.
    MomentumBeam,
    /// Target the tile adjacent to the caster, towards the caster's last move.
    Touch,

    // FUNCTIONS
    /// The targeted creatures dash in the direction of the caster's last move.
    Dash { max_distance: i32 },
}

/// The tracker of everything which determines how a certain spell will act.
pub struct SynapseData {
    /// Where a spell will act.
    targets: Vec<Position>,
    /// How a spell will act.
    pub axioms: Vec<Axiom>,
    /// The nth axiom currently being executed.
    pub step: usize,
    /// Who cast the spell.
    caster: Entity,
}

impl SynapseData {
    /// Create a blank SynapseData.
    fn new(caster: Entity, axioms: Vec<Axiom>) -> Self {
        SynapseData {
            targets: Vec::new(),
            axioms,
            step: 0,
            caster,
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
        let synapse_data = SynapseData::new(cast_spell.caster, axioms);
        // Send it off for processing - right away, for the spell stack is "last in, first out."
        spell_stack.spells.push(synapse_data);
    }
}

/// Get the most recently added spell (re-adding it at the end if it's not complete yet).
/// Get the next axiom, and runs its effects.
pub fn process_axiom(
    mut commands: Commands,
    axioms: Res<AxiomLibrary>,
    spell_stack: Res<SpellStack>,
) {
    // Get the most recently added spell, if it exists.
    if let Some(synapse_data) = spell_stack.spells.last() {
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

/// Target the caster's tile.
fn axiom_form_ego(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
) {
    // Get the currently executed spell.
    let synapse_data = spell_stack.spells.last_mut().unwrap();
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
    synapse_data.targets.push(caster_position);
}

/// The targeted creatures dash in the direction of the caster's last move.
fn axiom_function_dash(
    mut teleport: EventWriter<TeleportEntity>,
    map: Res<Map>,
    spell_stack: Res<SpellStack>,
    momentum: Query<&OrdDir>,
) {
    let synapse_data = spell_stack.spells.last().unwrap();
    let caster_momentum = momentum.get(synapse_data.caster).unwrap();
    if let Axiom::Dash { max_distance } = synapse_data.axioms[synapse_data.step] {
        // For each (Entity, Position) on a targeted tile with a creature on it...
        for (dasher, dasher_pos) in synapse_data.get_all_targeted_entity_pos_pairs(&map) {
            // The dashing creature starts where it currently is standing.
            let mut final_dash_destination = dasher_pos;
            // It will travel in the direction of the caster's last move.
            let (off_x, off_y) = caster_momentum.as_offset();
            // The dash has a maximum travel distance of `max_distance`.
            let mut distance_travelled = 0;
            while distance_travelled < max_distance {
                distance_travelled += 1;
                // Stop dashing if a solid Creature is hit and the dasher is not intangible.
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
    } else {
        // This should NEVER trigger. This system was chosen to run because the
        // next axiom in the SpellStack explicitly requested it by being an Axiom::Dash.
        panic!()
    }
}

/// Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_momentum_beam(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    position_and_momentum: Query<(&Position, &OrdDir)>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let (caster_position, caster_momentum) =
        position_and_momentum.get(synapse_data.caster).unwrap();
    // Start the beam where the caster is standing.
    // The beam travels in the direction of the caster's last move.
    let (off_x, off_y) = caster_momentum.as_offset();
    let mut output = linear_beam(*caster_position, 10, off_x, off_y, &map);
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
    synapse_data.targets.append(&mut output);
}

/// Target the tile adjacent to the caster, towards the caster's last move.
fn axiom_form_touch(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position_and_momentum: Query<(&Position, &OrdDir)>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let (caster_position, caster_momentum) =
        position_and_momentum.get(synapse_data.caster).unwrap();
    let (off_x, off_y) = caster_momentum.as_offset();
    let touch = Position::new(caster_position.x + off_x, caster_position.y + off_y);
    synapse_data.targets.push(touch);
    magic_vfx.send(PlaceMagicVfx {
        targets: vec![touch],
        sequence: EffectSequence::Sequential { duration: 0.04 },
        effect: EffectType::RedBlast,
        decay: 0.5,
        appear: 0.,
    });
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
    // The beam has a maximum distance of max_distance.
    while distance_travelled < max_distance {
        distance_travelled += 1;
        start.shift(off_x, off_y);
        // The new tile is always added, even if it is impassable...
        output.push(start);
        // But if it is impassable, the beam stops.
        if !map.is_passable(start.x, start.y) {
            break;
        }
    }
    output
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

pub fn spell_stack_is_empty(spell_stack: Res<SpellStack>) -> bool {
    spell_stack.spells.is_empty()
}
