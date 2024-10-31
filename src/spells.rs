use std::mem::{discriminant, Discriminant};

use bevy::{
    ecs::system::SystemId,
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{
    creature::{Intangible, Species},
    events::{RepressionDamage, SummonCreature, TeleportEntity},
    graphics::{AnimationDelay, EffectSequence, EffectType, PlaceMagicVfx},
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
        SpellStack {
            spells: Vec::new(),
            cleanup_id: world.register_system(cleanup_last_axiom),
        }
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
            discriminant(&Axiom::CrossBeam),
            world.register_system(axiom_form_cross_beam),
        );
        axioms.library.insert(
            discriminant(&Axiom::Halo { radius: 1 }),
            world.register_system(axiom_form_halo),
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
            discriminant(&Axiom::RepressionDamage { damage: 1 }),
            world.register_system(axiom_function_repression_damage),
        );
        axioms.library.insert(
            discriminant(&Axiom::LoopBack { steps: 1 }),
            world.register_system(axiom_mutator_loop_back),
        );
        axioms.library.insert(
            discriminant(&Axiom::ForceCast),
            world.register_system(axiom_mutator_force_cast),
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
    /// Target the caster's tile.
    Ego,
    /// Target all orthogonally adjacent tiles to the caster.
    Plus,
    /// Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
    /// including the first solid tile encountered, which stops the beam.
    MomentumBeam,
    /// Fire 4 beams from the caster, towards the orthogonal directions. Target all travelled tiles,
    /// including the first solid tile encountered, which stops the beam.
    CrossBeam,
    /// Fire 4 beams from the caster, towards the diagonal directions. Target all travelled tiles,
    /// including the first solid tile encountered, which stops the beam.
    XBeam,
    /// Target a ring of `radius` around the caster.
    Halo { radius: i32 },

    // FUNCTIONS
    /// The targeted creatures dash in the direction of the caster's last move.
    Dash { max_distance: i32 },
    /// The targeted passable tiles summon a new instance of species.
    SummonCreature { species: Species },
    /// Deal damage to all creatures on targeted tiles.
    RepressionDamage { damage: i32 },

    // MUTATORS
    /// Only once, loop backwards `steps` in the axiom queue.
    LoopBack { steps: usize },
    /// Force all creatures on targeted tiles to cast the remainder of the spell.
    /// This terminates execution of the spell.
    ForceCast,
}

/// Target the caster's tile.
fn axiom_form_ego(
    mut animation_delay: ResMut<AnimationDelay>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    magic_vfx.send(PlaceMagicVfx {
        targets: vec![synapse_data.caster_position],
        sequence: EffectSequence::Sequential { duration: 0.4 },
        effect: EffectType::RedBlast,
        decay: 0.5,
        appear: animation_delay.delay,
    });
    animation_delay.delay += 0.1;
    synapse_data.targets.push(synapse_data.caster_position);
}

/// Target all orthogonally adjacent tiles to the caster.
fn axiom_form_plus(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    mut animation_delay: ResMut<AnimationDelay>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
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
        appear: animation_delay.delay,
    });
    animation_delay.delay += 0.1;
    // Add these tiles to `targets`.
    synapse_data.targets.append(&mut output);
}

/// Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_momentum_beam(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    mut animation_delay: ResMut<AnimationDelay>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
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
        appear: animation_delay.delay,
    });
    animation_delay.delay += 0.1;
    // Add these tiles to `targets`.
    synapse_data.targets.append(&mut output);
}

/// Fire 4 beams from the caster, towards the orthogonal directions. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_cross_beam(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    mut animation_delay: ResMut<AnimationDelay>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let orthogonals = [OrdDir::Up, OrdDir::Left, OrdDir::Down, OrdDir::Right];
    for orthogonal in orthogonals {
        let (dx, dy) = orthogonal.as_offset();
        // Start the beam where the caster is standing.
        // The beam travels in the direction of each orthogonal.
        let mut output = linear_beam(synapse_data.caster_position, 10, dx, dy, &map);
        // Add some visual beam effects.
        magic_vfx.send(PlaceMagicVfx {
            targets: output.clone(),
            sequence: EffectSequence::Sequential { duration: 0.4 },
            effect: match orthogonal {
                OrdDir::Up | OrdDir::Down => EffectType::VerticalBeam,
                OrdDir::Right | OrdDir::Left => EffectType::HorizontalBeam,
            },
            decay: 0.5,
            appear: animation_delay.delay,
        });
        animation_delay.delay += 0.1;
        // Add these tiles to `targets`.
        synapse_data.targets.append(&mut output);
    }
}

/// Fire 4 beams from the caster, towards the diagonal directions. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_xbeam(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    mut animation_delay: ResMut<AnimationDelay>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
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
            appear: animation_delay.delay,
        });
        animation_delay.delay += 0.1;
        // Add these tiles to `targets`.
        synapse_data.targets.append(&mut output);
    }
}

/// Target a ring of `radius` around the caster.
fn axiom_form_halo(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    mut animation_delay: ResMut<AnimationDelay>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    if let Axiom::Halo { radius } = synapse_data.axioms[synapse_data.step] {
        let mut circle = circle_around(&synapse_data.caster_position, radius);
        // Sort by clockwise rotation.
        circle.sort_by(|a, b| {
            let angle_a = angle_from_center(&synapse_data.caster_position, a);
            let angle_b = angle_from_center(&synapse_data.caster_position, b);
            angle_a.partial_cmp(&angle_b).unwrap()
        });
        // Add some visual halo effects.
        magic_vfx.send(PlaceMagicVfx {
            targets: circle.clone(),
            sequence: EffectSequence::Sequential { duration: 0.4 },
            effect: EffectType::GreenBlast,
            decay: 0.5,
            appear: animation_delay.delay,
        });
        animation_delay.delay += 0.1;
        // Add these tiles to `targets`.
        synapse_data.targets.append(&mut circle);
    } else {
        panic!()
    }
}

/// The targeted passable tiles summon a new instance of species.
fn axiom_function_summon_creature(
    mut summon: EventWriter<SummonCreature>,
    spell_stack: Res<SpellStack>,
) {
    let synapse_data = spell_stack.spells.last().unwrap();
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
    let synapse_data = spell_stack.spells.last().unwrap();
    if let Axiom::RepressionDamage { damage } = synapse_data.axioms[synapse_data.step] {
        for entity in synapse_data.get_all_targeted_entities(&map) {
            repression_damage.send(RepressionDamage { entity, damage });
        }
    } else {
        panic!()
    }
}

/// Force all creatures on targeted tiles to cast the remainder of the spell.
/// This terminates execution of the spell.
fn axiom_mutator_force_cast(
    mut cast_spell: EventWriter<CastSpell>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    for entity in synapse_data.get_all_targeted_entities(&map) {
        cast_spell.send(CastSpell {
            caster: entity,
            spell: Spell {
                axioms: synapse_data.axioms[synapse_data.step + 1..].to_vec(),
            },
        });
    }
    synapse_data.synapse_flags.insert(SynapseFlag::Terminate);
}

/// The targeted creatures dash in the direction of the caster's last move.
fn axiom_function_dash(
    mut teleport: EventWriter<TeleportEntity>,
    is_intangible: Query<Has<Intangible>>,
    map: Res<Map>,
    spell_stack: Res<SpellStack>,
) {
    let synapse_data = spell_stack.spells.last().unwrap();
    if let Axiom::Dash { max_distance } = synapse_data.axioms[synapse_data.step] {
        // For each (Entity, Position) on a targeted tile...
        for (dasher, dasher_pos) in synapse_data.get_all_targeted_entity_pos_pairs(&map) {
            // The dashing creature starts where it currently is standing.
            let mut final_dash_destination = dasher_pos;
            // It will travel in the direction of the caster's last move.
            let (off_x, off_y) = synapse_data.caster_momentum.as_offset();
            // The dash has a maximum travel distance of `max_distance`.
            let mut distance_travelled = 0;
            // Check if the dashing creature is allowed to move through other creatures.
            let is_intangible = is_intangible.get(dasher).unwrap();
            while distance_travelled < max_distance {
                distance_travelled += 1;
                // Stop dashing if a solid Creature is hit and the dasher is not intangible.
                if !is_intangible
                    && !map.is_passable(
                        final_dash_destination.x + off_x,
                        final_dash_destination.y + off_y,
                    )
                {
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
        panic!()
    }
}

/// Only once, loop backwards `steps` in the axiom queue.
fn axiom_mutator_loop_back(mut spell_stack: ResMut<SpellStack>) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    if let Axiom::LoopBack { steps } = synapse_data.axioms[synapse_data.step] {
        // Remove the LoopBack.
        synapse_data.axioms.remove(synapse_data.step);
        // Rewind back n steps, + 1 because the cleanup will add one step by default.
        synapse_data.step = synapse_data.step.saturating_sub(steps + 1);
    } else {
        panic!()
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

#[derive(Debug)]
/// The tracker of everything which determines how a certain spell will act.
struct SynapseData {
    /// Where a spell will act.
    targets: Vec<Position>,
    /// How a spell will act.
    axioms: Vec<Axiom>,
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
    /// Flags that alter the behaviour of an active synapse.
    synapse_flags: HashSet<SynapseFlag>,
}

impl SynapseData {
    /// Create a blank SynapseData.
    fn new(
        caster: Entity,
        caster_momentum: OrdDir,
        caster_position: Position,
        axioms: Vec<Axiom>,
    ) -> Self {
        SynapseData {
            targets: Vec::new(),
            axioms,
            step: 0,
            caster,
            caster_momentum,
            caster_position,
            synapse_flags: HashSet::new(),
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

#[derive(Eq, Debug, PartialEq, Hash)]
/// Flags that alter the behaviour of an active synapse.
pub enum SynapseFlag {
    /// Delete this synapse and abandon all future Axioms.
    Terminate,
}

pub fn queue_up_spell(
    mut cast_spells: EventReader<CastSpell>,
    mut spell_stack: ResMut<SpellStack>,
    caster: Query<(&Position, &OrdDir)>,
) {
    for cast_spell in cast_spells.read() {
        // First, get the list of Axioms.
        let axioms = cast_spell.spell.axioms.clone();
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

pub fn all_spells_complete(
    incoming_spells: EventReader<CastSpell>,
    spell_stack: Res<SpellStack>,
) -> bool {
    spell_stack.spells.is_empty() && incoming_spells.is_empty()
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

fn cleanup_last_axiom(mut spell_stack: ResMut<SpellStack>) {
    // Get the currently executed spell, removing it temporarily.
    let mut synapse_data = spell_stack.spells.pop().unwrap();
    // Step forwards in the axiom queue.
    synapse_data.step += 1;
    // If the spell is finished, do not push it back.
    // The Terminate flag also prevents further execution.
    if synapse_data.axioms.get(synapse_data.step).is_some()
        && !synapse_data.synapse_flags.contains(&SynapseFlag::Terminate)
    {
        spell_stack.spells.push(synapse_data);
    }
}
