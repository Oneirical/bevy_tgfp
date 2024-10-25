use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{
    creature::{Intangible, Species},
    events::SummonCreature,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map {
            creatures: HashMap::new(),
        });
        app.add_systems(Startup, spawn_cage);
    }
}

/// A position on the map.
#[derive(Component, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    /// Create a new Position instance.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Edit an existing Position with new coordinates.
    pub fn update(&mut self, x: i32, y: i32) {
        (self.x, self.y) = (x, y);
    }

    /// Shift the position by a delta.
    pub fn shift(&mut self, dx: i32, dy: i32) {
        (self.x, self.y) = (self.x + dx, self.y + dy);
    }
}

fn manhattan_distance(a: Position, b: Position) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

/// A struct with some information on a creature inside the Map.
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct MapCreature {
    pub entity: Entity,
    pub is_intangible: bool,
}

pub fn are_orthogonally_adjacent(source: Position, destination: Position) -> bool {
    (destination.x - source.x).abs() + (destination.y - source.y).abs() == 1
}

/// The position of every creature, updated automatically.
#[derive(Resource)]
pub struct Map {
    pub creatures: HashMap<Position, HashSet<MapCreature>>,
}

impl Map {
    /// Which creatures stand on a certain tile?
    pub fn get_creatures_at(&self, x: i32, y: i32) -> Option<&HashSet<MapCreature>> {
        if let Some(entry) = self.creatures.get(&Position::new(x, y)) {
            Some(&entry)
        } else {
            None
        }
    }

    /// Which tangible creature stands on a certain tile?
    pub fn get_tangible_entity_at(&self, x: i32, y: i32) -> Option<&Entity> {
        if let Some(entry) = self.get_creatures_at(x, y) {
            if let Some(creature) = entry.iter().find(|tangibility| !tangibility.is_intangible) {
                Some(&creature.entity)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Is this tile passable?
    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        if let Some(entry) = self.creatures.get(&Position::new(x, y)) {
            // Check if there is no creature that is tangible in this tile.
            entry
                .iter()
                .find(|creature| !creature.is_intangible)
                .is_none()
        } else {
            true
        }
    }

    /// Find all adjacent accessible tiles to start, and pick the one closest to end.
    pub fn best_manhattan_move(&self, start: Position, end: Position) -> Option<Position> {
        let mut options = [
            Position::new(start.x, start.y + 1),
            Position::new(start.x, start.y - 1),
            Position::new(start.x + 1, start.y),
            Position::new(start.x - 1, start.y),
        ];

        // Sort all candidate tiles by their distance to the `end` destination.
        options.sort_by(|&a, &b| manhattan_distance(a, end).cmp(&manhattan_distance(b, end)));

        options
            .iter()
            // Only keep either the destination or unblocked tiles.
            .filter(|&p| *p == end || self.is_passable(p.x, p.y))
            // Remove the borrow.
            .copied()
            // Get the tile that manages to close the most distance to the destination.
            // If it exists, that is. Otherwise, this is just a None.
            .next()
    }

    /// Move a pre-existing entity around the Map.
    pub fn move_creature(&mut self, entity: Entity, old_pos: Position, new_pos: Position) {
        // Get all the creatures on the old position.
        let creatures = self.creatures.get_mut(&old_pos).unwrap();
        // Find the specific creature which needs to be moved.
        let target_creature = &creatures
            .iter()
            .find(|creature| creature.entity == entity)
            .unwrap()
            .clone();
        // Remove that creature from the old position.
        creatures.remove(target_creature);
        // If there are no more creatures on this tile, remove the entry.
        if creatures.is_empty() {
            self.creatures.remove(&old_pos);
        }
        // Move the creature onto the new position, creating it if needed.
        self.creatures
            .entry(new_pos)
            .or_default()
            .insert(*target_creature);
    }
}

/// Newly spawned creatures earn their place in the HashMap.
pub fn register_creatures(
    mut map: ResMut<Map>,
    // Any entity that has a Position that just got added to it -
    // currently only possible as a result of having just been spawned in.
    displaced_creatures: Query<
        (&Position, Entity, Has<Intangible>),
        (Added<Position>, With<Species>),
    >,
    intangible_creatures: Query<(Entity, &Position), Added<Intangible>>,
    tangible_creatures: Query<(Entity, &Position)>,
    mut tangible_entities: RemovedComponents<Intangible>,
) {
    for (position, entity, is_intangible) in displaced_creatures.iter() {
        // Insert the new creature in the Map.
        // If this key doesn't exist yet, it is created, otherwise it pushes on
        // the HashSet.
        map.creatures
            .entry(*position)
            .or_default()
            .insert(MapCreature {
                entity,
                is_intangible,
            });
    }
    // Update all newly intangible creatures.
    for (intangible_entity, intangible_position) in intangible_creatures.iter() {
        let creatures = map.creatures.get_mut(intangible_position).unwrap();
        let mut intangible_creature = creatures
            .iter()
            .find(|creature| creature.entity == intangible_entity)
            .unwrap()
            .clone();
        creatures.remove(&intangible_creature);
        intangible_creature.is_intangible = true;
        creatures.insert(intangible_creature);
    }
    // Update all newly tangible creatures.
    for entity in tangible_entities.read() {
        let (tangible_entity, tangible_position) = tangible_creatures.get(entity).unwrap();
        let creatures = map.creatures.get_mut(tangible_position).unwrap();
        let mut intangible_creature = creatures
            .iter()
            .find(|creature| creature.entity == tangible_entity)
            .unwrap()
            .clone();
        creatures.remove(&intangible_creature);
        intangible_creature.is_intangible = false;
        creatures.insert(intangible_creature);
    }
}

fn spawn_cage(mut summon: EventWriter<SummonCreature>) {
    let cage = "##################\
                #H.H.H.H.H.H.H.H.#\
                #...............H#\
                #H...............#\
                #...............H#\
                #H...............#\
                #...............H#\
                #H...............#\
                #........@......H#\
                #H...............#\
                #...............H#\
                #H...............#\
                #...............H#\
                #H...............#\
                #...............H#\
                #H...............#\
                #.H.H.H.H.H.H.H.H#\
                ##################";
    for (idx, tile_char) in cage.char_indices() {
        let position = Position::new(idx as i32 % 18, idx as i32 / 18);
        let species = match tile_char {
            '#' => Species::Wall,
            'H' => Species::Hunter,
            '@' => Species::Player,
            'S' => Species::Spawner,
            _ => continue,
        };
        summon.send(SummonCreature { species, position });
    }
}
