use bevy::prelude::*;

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
        app.add_event::<SpellEffect>();
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

impl Axiom {
    fn target(&self, synapse_data: &mut SynapseData, map: &Map) {
        match self {
            // Target the caster's tile.
            Self::Ego => {
                synapse_data.effects.push(EventDispatch::PlaceMagicVfx {
                    targets: vec![synapse_data.caster_position],
                    sequence: EffectSequence::Sequential { duration: 0.4 },
                    effect: EffectType::RedBlast,
                    decay: 0.5,
                });
                synapse_data.targets.push(synapse_data.caster_position);
            }
            // Target all orthogonally adjacent tiles to the caster.
            Self::Plus => {
                let adjacent = [OrdDir::Up, OrdDir::Right, OrdDir::Down, OrdDir::Left];
                let mut output = Vec::new();
                for direction in adjacent {
                    let mut new_pos = synapse_data.caster_position;
                    let offset = direction.as_offset();
                    new_pos.shift(offset.0, offset.1);
                    output.push(new_pos);
                }
                synapse_data.effects.push(EventDispatch::PlaceMagicVfx {
                    targets: output.clone(),
                    sequence: EffectSequence::Sequential { duration: 0.4 },
                    effect: EffectType::RedBlast,
                    decay: 0.5,
                });
                // Add these tiles to `targets`.
                synapse_data.targets.append(&mut output);
            }
            // Shoot a beam from the caster towards its last move, all tiles passed through
            // become targets, including the impact point.
            Self::MomentumBeam => {
                // Start the beam where the caster is standing.
                // The beam travels in the direction of the caster's last move.
                let (off_x, off_y) = synapse_data.caster_momentum.as_offset();
                let mut output = linear_beam(synapse_data.caster_position, 10, off_x, off_y, map);
                // Add some visual beam effects.
                synapse_data.effects.push(EventDispatch::PlaceMagicVfx {
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
            // Fire 4 beams from the caster, towards the diagonal directions. Target all travelled
            // tiles, including the first solid tile encountered, which stops the beam.
            Self::XBeam => {
                let diagonals = [(1, 1), (-1, 1), (1, -1), (-1, -1)];
                for (dx, dy) in diagonals {
                    // Start the beam where the caster is standing.
                    // The beam travels in the direction of each diagonal.
                    let mut output = linear_beam(synapse_data.caster_position, 10, dx, dy, map);
                    // Add some visual beam effects.
                    synapse_data.effects.push(EventDispatch::PlaceMagicVfx {
                        targets: output.clone(),
                        sequence: EffectSequence::Sequential { duration: 0.4 },
                        effect: EffectType::RedBlast,
                        decay: 0.5,
                    });
                    // Add these tiles to `targets`.
                    synapse_data.targets.append(&mut output);
                }
            }
            _ => (),
        }
    }
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
            // The targeted passable tiles summon a new instance of species.
            Self::SummonCreature { species } => {
                for position in &synapse_data.targets {
                    synapse_data.effects.push(EventDispatch::SummonCreature {
                        species: *species,
                        position: *position,
                    });
                }
                true
            }
            Self::RepressionDamage { damage } => {
                for entity in synapse_data.get_all_targeted_entities(map) {
                    synapse_data.effects.push(EventDispatch::RepressionDamage {
                        entity,
                        damage: *damage,
                    });
                }
                true
            }
            // Forms (which do not have an in-game effect) return false.
            _ => false,
        }
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

/// An enum with replicas of common game Events, to be translated into the real Events
/// and dispatched to the main game loop.
pub enum EventDispatch {
    TeleportEntity {
        destination: Position,
        entity: Entity,
    },
    SummonCreature {
        species: Species,
        position: Position,
    },
    PlaceMagicVfx {
        targets: Vec<Position>,
        sequence: EffectSequence,
        effect: EffectType,
        decay: f32,
    },
    RepressionDamage {
        entity: Entity,
        damage: i32,
    },
}

/// Work through the list of Axioms of a spell, translating it into Events to launch onto the game.
pub fn gather_effects(
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
    mut summon: EventWriter<SummonCreature>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut repression_damage: EventWriter<RepressionDamage>,
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
                EventDispatch::SummonCreature { species, position } => {
                    summon.send(SummonCreature {
                        species: *species,
                        position: *position,
                    });
                }
                EventDispatch::PlaceMagicVfx {
                    targets,
                    sequence,
                    effect,
                    decay,
                } => {
                    magic_vfx.send(PlaceMagicVfx {
                        targets: targets.clone(),
                        sequence: *sequence,
                        effect: *effect,
                        decay: *decay,
                    });
                }
                EventDispatch::RepressionDamage { entity, damage } => {
                    repression_damage.send(RepressionDamage {
                        entity: *entity,
                        damage: *damage,
                    });
                }
            };
        }
    }
}
