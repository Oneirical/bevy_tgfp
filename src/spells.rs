use bevy::prelude::*;

use crate::{
    creature::Species,
    events::{CreatureCollision, SummonCreature, TeleportEntity},
    map::{Map, Position},
    OrdDir,
};

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CastSpell>();
        app.add_event::<SpellEffect>();
        app.add_systems(Update, gather_effects);
        app.add_systems(Update, dispatch_events);
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
    // Target the adjacent tile to the caster's tile, in the direction of the caster's last move.
    Smooch,
    // Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
    // including the first solid tile encountered, which stops the beam.
    MomentumBeam,

    // FUNCTIONS

    // The targeted creatures dash in the direction of the caster's last move.
    Dash,
    // The targeted passable tiles summon a new instance of species.
    SummonCreature { species: Species },
}

impl Axiom {
    fn target(&self, synapse_data: &mut SynapseData, map: &Map) {
        match self {
            // Target the caster's tile.
            Self::Ego => {
                synapse_data.targets.push(synapse_data.caster_position);
            }
            // Target the adjacent tile to the caster's tile, in the direction of the caster's
            // last move.
            Self::Smooch => {
                let mut new_pos = synapse_data.caster_position;
                let offset = synapse_data.caster_momentum.as_offset();
                new_pos.shift(offset.0, offset.1);
                synapse_data.targets.push(new_pos);
            }
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
            _ => (),
        }
    }
    /// Execute Function-type Axioms. Returns true if this produced an actual effect.
    fn execute(&self, synapse_data: &mut SynapseData, map: &Map) -> bool {
        match self {
            Self::Dash => {
                let mut risk_of_collision = None;
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
                        if let Some(collided_entity) = map.get_entity_at(
                            final_dash_destination.x + off_x,
                            final_dash_destination.y + off_y,
                        ) {
                            risk_of_collision = Some(EventDispatch::CreatureCollision {
                                attacker: *collided_entity,
                                defender: dasher,
                                speed: distance_travelled,
                            });
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

                    // If a collision occured, also release a Collision event.
                    if let Some(collision) = risk_of_collision {
                        synapse_data.effects.push(collision);
                    }
                }
                true
            }
            Self::SummonCreature { species } => {
                for position in &synapse_data.targets {
                    synapse_data.effects.push(EventDispatch::SummonCreature {
                        species: *species,
                        position: *position,
                    });
                }
                true
            }
            // Forms (which do not have an in-game effect) return false.
            _ => false,
        }
    }
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

/// An enum with replicas of common game Events, to be translated into the real Events
/// and dispatched to the main game loop.
#[derive(Clone, Copy)]
pub enum EventDispatch {
    TeleportEntity {
        destination: Position,
        entity: Entity,
    },
    SummonCreature {
        species: Species,
        position: Position,
    },
    CreatureCollision {
        attacker: Entity,
        defender: Entity,
        speed: usize,
    },
}

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
    mut collide: EventWriter<CreatureCollision>,
) {
    for effect_list in receiver.read() {
        for effect in &effect_list.events {
            // Each EventDispatch enum is translated into its Event counterpart.
            match effect {
                EventDispatch::TeleportEntity {
                    destination,
                    entity,
                } => {
                    teleport.send(TeleportEntity {
                        destination: *destination,
                        entity: *entity,
                    });
                }
                EventDispatch::SummonCreature { species, position } => {
                    summon.send(SummonCreature {
                        species: *species,
                        position: *position,
                    });
                }
                EventDispatch::CreatureCollision {
                    attacker: attack,
                    defender,
                    speed,
                } => {
                    collide.send(CreatureCollision {
                        attacker: *attack,
                        defender: *defender,
                        speed: *speed,
                    });
                }
            };
        }
    }
}
