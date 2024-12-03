use bevy::{prelude::*, utils::HashMap};

use crate::{
    creature::{Creature, Hunt, Player, Species},
    events::SummonCreature,
    graphics::{MagicVfx, SpriteSheetAtlas},
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

    /// Find all adjacent accessible tiles to start, and pick the one closest to end.
    pub fn best_manhattan_move(&self, start: Position, end: Position) -> Option<OrdDir> {
        let mut options = [
            (OrdDir::Up, Position::new(start.x, start.y + 1)),
            (OrdDir::Down, Position::new(start.x, start.y - 1)),
            (OrdDir::Right, Position::new(start.x + 1, start.y)),
            (OrdDir::Left, Position::new(start.x - 1, start.y)),
        ];

        // Sort all candidate tiles by their distance to the `end` destination.
        options.sort_by(|&a, &b| manhattan_distance(a.1, end).cmp(&manhattan_distance(b.1, end)));

        let final_choice = options
            .iter()
            // Only keep either the destination or unblocked tiles.
            .filter(|&p| p.1 == end || self.is_passable(p.1.x, p.1.y))
            // Remove the borrow.
            .copied()
            // Get the tile that manages to close the most distance to the destination.
            // If it exists, that is. Otherwise, this is just a None.
            .next();

        if let Some(final_direction) = final_choice {
            Some(final_direction.0)
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
    displaced_creatures: Query<(&Position, Entity), (Added<Position>, Without<MagicVfx>)>,
) {
    for (position, entity) in displaced_creatures.iter() {
        // Insert the new creature in the Map. Position implements Copy,
        // so it can be dereferenced (*), but `.clone()` would have been
        // fine too.
        map.creatures.insert(*position, entity);
    }
}

fn spawn_cage(mut summon: EventWriter<SummonCreature>) {
    let cage = "#########\
                #HHHHHHH#\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #...@...#\
                #.......#\
                #########";
    for (idx, tile_char) in cage.char_indices() {
        let position = Position::new(idx as i32 % 9, idx as i32 / 9);
        let species = match tile_char {
            '#' => Species::Wall,
            'H' => Species::Hunter,
            '@' => Species::Player,
            _ => continue,
        };
        summon.send(SummonCreature { species, position });
    }
}
