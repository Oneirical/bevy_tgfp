use bevy::prelude::*;

use crate::{
    events::TeleportEntity,
    graphics::GameState,
    map::{Map, Position},
    OrdDir,
};

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<CastSpell>>();
        app.add_event::<SpellEffect>();
        app.add_systems(
            Update,
            gather_effects
                .before(dispatch_events)
                .run_if(in_state(GameState::Running)),
        );
        app.add_systems(Update, dispatch_events);
    }
}

#[derive(Event)]
/// Triggered when a creature (the `caster`) casts a `spell`.
pub struct CastSpell {
    pub caster: Entity,
    pub spell: Spell,
}

#[derive(Event)]
/// An event dictating that a list of Events must be sent to the game loop
/// after the completion of a spell.
pub struct SpellEffect {
    events: Vec<EventDispatch>,
}

#[derive(Component, Clone)]
/// A spell is composed of a list of "Axioms", which will select tiles or execute an effect onto
/// those tiles, in the order they are listed.
pub struct Spell {
    pub axioms: Vec<Axiom>,
}

/// An enum with replicas of common game Events, to be translated into the real Events
/// and dispatched to the main game loop.
pub enum EventDispatch {
    TeleportEntity {
        destination: Position,
        entity: Entity,
    },
    SpellChain {
        caster: Entity,
        spell: Spell,
    },
}

#[derive(Debug, Clone)]
/// There are Form axioms, which target certain tiles, and Function axioms, which execute an effect
/// onto those tiles.
pub enum Axiom {
    // FORMS

    // Target the caster's tile.
    Ego,
    // Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
    // including the first solid tile encountered, which stops the beam.
    MomentumBeam,
    // Target all 8 adjacent tiles to the caster.
    Circlet,

    // FUNCTIONS

    // The targeted creatures dash in the direction of the caster's last move.
    Dash,
}

/// Translate a list of EventDispatch into their "real" Event counterparts and send them off
/// into the main game loop to modify the game's creatures.
pub fn dispatch_events(
    mut receiver: EventReader<SpellEffect>,
    mut teleport: EventWriter<TeleportEntity>,
    mut spell_chain: EventWriter<CastSpell>,
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
                EventDispatch::SpellChain { caster, spell } => {
                    spell_chain.send(CastSpell {
                        caster: *caster,
                        spell: spell.clone(),
                    });
                }
            };
        }
    }
}

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

    fn new_from_synapse(synapse: &SynapseData) -> Self {
        SynapseData {
            targets: Vec::new(),
            effects: Vec::new(),
            caster: synapse.caster,
            caster_momentum: synapse.caster_momentum,
            caster_position: synapse.caster_position,
        }
    }

    fn get_all_targeted_entities(&self, map: &Map) -> Vec<Entity> {
        let mut targeted_entities = Vec::new();
        for target in &self.targets {
            if let Some(entity) = map.get_entity_at(target.x, target.y) {
                targeted_entities.push(*entity);
            }
        }
        targeted_entities
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
            // If it's actually a Function and it's not the last element, stop, dispatch events
            // and resume later.
            if axiom.execute(&mut synapse_data, &map) && i != axioms.len() - 1 {
                let spell = Spell {
                    axioms: axioms[i + 1..].to_vec(),
                };
                synapse_data.effects.push(EventDispatch::SpellChain {
                    caster: synapse_data.caster,
                    spell,
                });
                break;
            }
        }

        // Once all Axioms are processed, dispatch everything to the system that will translate
        // all effects into proper events.
        sender.send(SpellEffect {
            events: synapse_data.effects,
        });
    }
}

impl Axiom {
    fn target(&self, synapse_data: &mut SynapseData, map: &Map) {
        match self {
            Self::Ego => {
                synapse_data.targets.push(synapse_data.caster_position);
            }
            Self::Circlet => {
                // TODO could be interesting to filter this by momentum so the front ones are acted on first
                let mut circlet = vec![
                    Position::new(
                        synapse_data.caster_position.x + 1,
                        synapse_data.caster_position.y + 1,
                    ),
                    Position::new(
                        synapse_data.caster_position.x + 1,
                        synapse_data.caster_position.y,
                    ),
                    Position::new(
                        synapse_data.caster_position.x + 1,
                        synapse_data.caster_position.y - 1,
                    ),
                    Position::new(
                        synapse_data.caster_position.x,
                        synapse_data.caster_position.y + 1,
                    ),
                    Position::new(
                        synapse_data.caster_position.x,
                        synapse_data.caster_position.y - 1,
                    ),
                    Position::new(
                        synapse_data.caster_position.x - 1,
                        synapse_data.caster_position.y + 1,
                    ),
                    Position::new(
                        synapse_data.caster_position.x - 1,
                        synapse_data.caster_position.y,
                    ),
                    Position::new(
                        synapse_data.caster_position.x - 1,
                        synapse_data.caster_position.y - 1,
                    ),
                ];
                synapse_data.targets.append(&mut circlet);
            }
            Self::MomentumBeam => {
                let mut start = synapse_data.caster_position;
                let (off_x, off_y) = synapse_data.caster_momentum.as_offset();
                let mut distance_travelled = 0;
                let mut output = Vec::new();
                while distance_travelled < 10 {
                    distance_travelled += 1;
                    start.shift(off_x, off_y);
                    output.push(start);
                    if !map.is_passable(start.x, start.y) {
                        break;
                    }
                }
                synapse_data.targets.append(&mut output);
            }
            _ => (),
        }
    }

    /// Execute Function-type Axioms. Returns true if this produced an actual effect.
    fn execute(&self, synapse_data: &mut SynapseData, map: &Map) -> bool {
        match self {
            Self::Dash => {
                for (dasher, dasher_pos) in synapse_data.get_all_targeted_entity_pos_pairs(map) {
                    // Create a fake synapse just to use a beam.
                    let mut artifical_synapse = SynapseData::new_from_synapse(synapse_data);
                    // Set the fake synapse's caster and caster position to be the targeted creatures.
                    (artifical_synapse.caster, artifical_synapse.caster_position) =
                        (dasher, dasher_pos);
                    // Fire the beam with the caster's momentum.
                    Self::MomentumBeam.target(&mut artifical_synapse, map);
                    // Get the penultimate tile, aka the last passable tile in the beam's path.
                    let destination_tile = artifical_synapse
                        .targets
                        .get(artifical_synapse.targets.len().wrapping_sub(2));
                    // If that penultimate tile existed, teleport to it.
                    if let Some(destination_tile) = destination_tile {
                        synapse_data.effects.push(EventDispatch::TeleportEntity {
                            destination: *destination_tile,
                            entity: dasher,
                        });
                    }
                }
                true
            }
            _ => false,
        }
    }
}
