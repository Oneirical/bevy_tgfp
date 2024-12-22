use bevy::{prelude::*, utils::HashMap};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng,
};

use crate::{
    creature::{Intangible, Species},
    events::SummonCreature,
    OrdDir,
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

/// The position of every creature, updated automatically.
#[derive(Resource)]
pub struct Map {
    pub creatures: HashMap<Position, Entity>,
}

impl Map {
    /// Which creature stands on a certain tile?
    pub fn get_entity_at(&self, x: i32, y: i32) -> Option<&Entity> {
        self.creatures.get(&Position::new(x, y))
    }

    /// Is this tile passable?
    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        self.get_entity_at(x, y).is_none()
    }

    /// Get all tile coordinates of adjacent tiles from a point.
    pub fn get_adjacent_tiles(&self, centre: Position) -> Vec<Position> {
        vec![
            Position::new(centre.x, centre.y + 1),
            Position::new(centre.x, centre.y - 1),
            Position::new(centre.x + 1, centre.y),
            Position::new(centre.x - 1, centre.y),
        ]
    }

    /// Filter tiles from closest to further to another tile.
    pub fn sort_by_manhattan(
        &self,
        mut tiles: Vec<Position>,
        destination: Position,
    ) -> Vec<Position> {
        tiles.sort_by(|&a, &b| {
            manhattan_distance(a, destination).cmp(&manhattan_distance(b, destination))
        });
        tiles
    }

    pub fn random_adjacent_passable_direction(&self, start: Position) -> Option<OrdDir> {
        let adjacent = self.get_adjacent_tiles(start);
        let mut rng = thread_rng();
        let final_choice = adjacent
            .iter()
            // Only keep unblocked tiles.
            .filter(|&p| self.is_passable(p.x, p.y))
            // Remove the borrow.
            // Get the tile that manages to close the most distance to the destination.
            // If it exists, that is. Otherwise, this is just a None.
            .choose(&mut rng);
        if let Some(final_choice) = final_choice {
            OrdDir::direction_towards_adjacent_tile(start, *final_choice)
        } else {
            None
        }
    }

    /// Find all adjacent accessible tiles to start, and pick the one closest to end.
    pub fn best_manhattan_move(&self, start: Position, end: Position) -> Option<OrdDir> {
        let adjacent = self.get_adjacent_tiles(start);
        let adjacent_sorted = self.sort_by_manhattan(adjacent, end);

        let final_choice = adjacent_sorted
            .iter()
            // Only keep either the destination or unblocked tiles.
            .filter(|&p| *p == end || self.is_passable(p.x, p.y))
            // Remove the borrow.
            .copied()
            // Get the tile that manages to close the most distance to the destination.
            // If it exists, that is. Otherwise, this is just a None.
            .next();

        if let Some(final_choice) = final_choice {
            OrdDir::direction_towards_adjacent_tile(start, final_choice)
        } else {
            None
        }
    }

    /// Move a pre-existing entity around the Map.
    pub fn move_creature(&mut self, old_pos: Position, new_pos: Position) {
        // As the entity already existed in the Map's records, remove it.
        let entity = self.creatures.remove(&old_pos).expect(&format!(
            "The map cannot move a nonexistent Entity from {:?} to {:?}.",
            old_pos, new_pos
        ));
        self.creatures.insert(new_pos, entity);
    }
}

/// Newly spawned creatures earn their place in the HashMap.
pub fn register_creatures(
    mut map: ResMut<Map>,
    // Any entity that has a Position that just got added to it -
    // currently only possible as a result of having just been spawned in.
    displaced_creatures: Query<(&Position, Entity), (Added<Position>, With<Species>)>,
    intangible_creatures: Query<&Position, (Added<Intangible>, With<Species>)>,
    tangible_creatures: Query<&Position, With<Species>>,
    mut tangible_entities: RemovedComponents<Intangible>,
) {
    for (position, entity) in displaced_creatures.iter() {
        // Insert the new creature in the Map. Position implements Copy,
        // so it can be dereferenced (*), but `.clone()` would have been
        // fine too.
        map.creatures.insert(*position, entity);
    }

    // Newly intangible creatures are removed from the map.
    for intangible_position in intangible_creatures.iter() {
        map.creatures.remove(intangible_position);
    }

    // A creature recovering its tangibility is added to the map.
    for entity in tangible_entities.read() {
        let tangible_position = tangible_creatures.get(entity).unwrap();
        if map.creatures.get(tangible_position).is_some() {
            panic!("A creature recovered its tangibility while on top of another creature!");
        }
        map.creatures.insert(*tangible_position, entity);
    }
}

fn spawn_cage(mut summon: EventWriter<SummonCreature>) {
    let mut spawned_player = false;
    let cage = "
####.....###......#######......###.....####
##########......###.....###......##########
.#...###......###.........###......###...#.
.#...#......###.............###......#...#.
.#...#....###.................###....#...#.
###########.....................###########
##...#...............................#...##
#....#...............................#....#
...###...............................###...
..##...................................##..
.##.....................................##.
##.......................................##
#.........................................#
#.........................................#
#.........................................#
#.........................................#
#.........................................#
#.........................................#
#.........................................#
#.........................................#
#...........................@.............#
#....................B....................#
#.........................................#
#.........................................#
#.........................................#
#.........................................#
#.........................................#
#.........................................#
#.........................................#
#.........................................#
#.........................................#
##.......................................##
.##.....................................##.
..##...................................##..
...###...............................###...
#....#...............................#....#
##...#...............................#...##
###########.....................###########
.#...#....###.................###....#...#.
.#...#......###.............###......#...#.
.#...###......###.........###......###...#.
##########......###.....###......##########
####.....###......#######......###.....####        
    ";

    for (idx, tile_char) in cage.chars().enumerate() {
        let position = Position::new(idx as i32 % 44, idx as i32 / 44);
        let species = match tile_char {
            '#' => Species::Wall,
            'H' => Species::Hunter,
            'S' => Species::Spawner,
            'T' => Species::Tinker,
            '@' => Species::Player,
            'W' => Species::WeakWall,
            '2' => Species::Second,
            'A' => Species::Apiarist,
            'F' => Species::Shrike,
            'B' => Species::Architect,
            '^' | '>' | '<' | 'V' => Species::Airlock,
            _ => continue,
        };
        let momentum = match tile_char {
            '^' => OrdDir::Up,
            '>' => OrdDir::Right,
            '<' => OrdDir::Left,
            'V' | _ => OrdDir::Down,
        };
        if spawned_player && matches!(species, Species::Player) {
            continue;
        } else if !spawned_player && matches!(species, Species::Player) {
            spawned_player = true;
        }
        summon.send(SummonCreature {
            species,
            position,
            momentum,
            summon_tile: Position::new(0, 0),
        });
    }
}
