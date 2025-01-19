use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng, Rng,
};

use crate::{
    creature::{Intangible, Player, Species},
    events::{RemoveCreature, SummonCreature},
    OrdDir,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map {
            creatures: HashMap::new(),
        });
        app.insert_resource(FaithsEnd {
            cage_address_position: HashMap::new(),
            cage_dimensions: HashMap::new(),
            current_cage: 0,
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

    pub fn is_within_range(&self, a: &Position, b: &Position) -> bool {
        let min_x = a.x.min(b.x);
        let max_x = a.x.max(b.x);
        let min_y = a.y.min(b.y);
        let max_y = a.y.max(b.y);

        self.x >= min_x && self.x <= max_x && self.y >= min_y && self.y <= max_y
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
        if let Some(entity) = self.creatures.remove(&old_pos) {
            self.creatures.insert(new_pos, entity);
        }
    }
}

/// Newly spawned creatures earn their place in the HashMap.
pub fn register_creatures(
    mut map: ResMut<Map>,
    // Any entity that has a Position that just got added to it -
    // currently only possible as a result of having just been spawned in.
    // Naturally intangible creatures skip this.
    displaced_creatures: Query<
        (&Position, Entity),
        (Added<Position>, With<Species>, Without<Intangible>),
    >,
    intangible_creatures: Query<(Entity, &Position), (Added<Intangible>, With<Species>)>,
    tangible_creatures: Query<&Position, With<Species>>,
    mut tangible_entities: RemovedComponents<Intangible>,
    mut remove: EventWriter<RemoveCreature>,
) {
    for (position, entity) in displaced_creatures.iter() {
        // Insert the new creature in the Map. Position implements Copy,
        // so it can be dereferenced (*), but `.clone()` would have been
        // fine too.
        map.creatures.insert(*position, entity);
    }

    // Newly intangible creatures are removed from the map.
    for (entity, intangible_position) in intangible_creatures.iter() {
        if let Some(preexisting_entity) = map.creatures.get(intangible_position) {
            // Check that the entity being removed is actually the intangible entity.
            // REASON: If a creature spawns in already intangible on top of a
            // tangible creature, without this check, it would remove
            // the tangible creature from the map.
            if *preexisting_entity != entity {
                continue;
            }
        }
        map.creatures.remove(intangible_position);
    }

    // A creature recovering its tangibility is added to the map.
    for entity in tangible_entities.read() {
        if let Ok(tangible_position) = tangible_creatures.get(entity) {
            if map.creatures.get(tangible_position).is_some() {
                // NOTE: This is kind of like Caves of Qud's death by phasing
                // ("the pauli principle"). Creatures recovering tangibility
                // on top of another die. I am mostly adding this so I can
                // debug the occasional door issue.
                remove.send(RemoveCreature { entity });
                dbg!(tangible_position);
                dbg!("A creature recovered its tangibility while on top of another creature!");
            } else {
                map.creatures.insert(*tangible_position, entity);
            }
        }
    }
}

#[derive(Resource, Debug)]
pub struct FaithsEnd {
    pub cage_address_position: HashMap<Position, usize>,
    pub cage_dimensions: HashMap<usize, (Position, Position)>,
    pub current_cage: usize,
}

pub fn spawn_cage(
    mut summon: EventWriter<SummonCreature>,
    mut faiths_end: ResMut<FaithsEnd>,
    player: Query<&Player>,
) {
    let size = 9;
    for tower_floor in 0..15 {
        let mut cage = generate_cage(
            tower_floor,
            // Spawn the player in the first room
            // (the player must not already exist).
            tower_floor == 0 && player.is_empty(),
            size,
            match tower_floor {
                0 => &[OrdDir::Up],
                14 => &[OrdDir::Down],
                _ => &[OrdDir::Up, OrdDir::Down],
            },
        );
        add_creatures(&mut cage, 2 + tower_floor);

        for (idx, tile_char) in cage.iter().enumerate() {
            let cage_corner = Position::new(0, (tower_floor * size) as i32);
            let position = Position::new(
                cage_corner.x + idx as i32 % size as i32,
                cage_corner.y + size as i32 - 1 - idx as i32 / size as i32,
            );
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
                'O' => Species::Oracle,
                '^' | '>' | '<' | 'V' => Species::Airlock,
                _ => continue,
            };
            let momentum = match tile_char {
                '^' => OrdDir::Up,
                '>' => OrdDir::Right,
                '<' => OrdDir::Left,
                'V' | _ => OrdDir::Down,
            };
            summon.send(SummonCreature {
                species,
                position,
                momentum,
                summoner_tile: Position::new(0, 0),
                summoner: None,
                spellbook: None,
            });
            faiths_end
                .cage_address_position
                .insert(position, tower_floor);
            // If there is no player yet (first run),
            // set the boundaries.
            if player.is_empty() {
                faiths_end.cage_dimensions.insert(
                    tower_floor,
                    (
                        cage_corner,
                        Position::new(
                            cage_corner.x + size as i32 - 1,
                            cage_corner.y + size as i32 - 1,
                        ),
                    ),
                );
            }
        }
    }
}

fn add_creatures(cage: &mut [char], creatures_amount: usize) {
    let creature_chars = ['A', 'T', 'F', '2', 'H', 'O'];

    let floor_positions: Vec<usize> = cage
        .iter()
        .enumerate()
        .filter(|&(_, c)| *c == '.')
        .map(|(i, _)| i)
        .collect();

    let mut rng = thread_rng();
    let creature_spawn_points = floor_positions.choose_multiple(&mut rng, creatures_amount);

    for pos in creature_spawn_points {
        let new_creature = *creature_chars.choose(&mut rng).unwrap();
        cage[*pos] = new_creature;
    }
}

pub fn generate_cage(
    floor: usize,
    spawn_player: bool,
    size: usize,
    connections: &[OrdDir],
) -> Vec<char> {
    let mut cage = Vec::new();

    for _i in 0..100 {
        let mut passable_tiles = 0;
        let mut idx_start = 0;
        let mut rng = thread_rng();
        for i in 0..size.pow(2) {
            // If the player is here, it spawns in the middle.
            if spawn_player && xy_idx(i, size) == ((size - 1) / 2, (size - 1) / 2) {
                cage.push('@');
                passable_tiles += 1;
            // If the player is already spawned, the bottom cage should still
            // not place anything in its centre so the player can be teleported
            // there.
            } else if !spawn_player
                && floor == 0
                && xy_idx(i, size) == ((size - 1) / 2, (size - 1) / 2)
            {
                cage.push('.');
                passable_tiles += 1;
            // Edges get walls 100% of the time, other tiles, 30% of the time.
            } else if is_edge(i, size) {
                cage.push('#');
            } else if rng.gen::<f32>() < 0.3 {
                cage.push('W');
            // Everything else is a floor.
            } else {
                cage.push('.');
                passable_tiles += 1;
                idx_start = i;
            }
        }
        for airlock in connections {
            match airlock {
                OrdDir::Up => {
                    cage[size / 2] = '^';
                }
                OrdDir::Left => {
                    cage[size * (size / 2)] = '<';
                }
                OrdDir::Right => {
                    cage[size * (size / 2 + 1) - 1] = '>';
                }
                OrdDir::Down => {
                    cage[size * size - size / 2 - 1] = 'V';
                }
            }
            passable_tiles += 1;
        }
        // Every passable tile must be connected to all other passable tiles, no "islands".
        if passable_tiles == get_connected_tiles(idx_start, size, &cage) {
            return cage;
        } else {
            cage.clear();
        }
    }
    panic!("Cage generation timeout achieved.");
}

fn xy_idx(idx: usize, size: usize) -> (usize, usize) {
    (idx % size, idx / size)
}

fn is_edge(idx: usize, size: usize) -> bool {
    idx % size == 0 || idx % size == size - 1 || idx / size == 0 || idx / size == size - 1
}

fn get_connected_tiles(idx_start: usize, size: usize, cage: &[char]) -> usize {
    // All previously found floor tiles.
    let mut connected_indices = HashSet::new();
    connected_indices.insert(idx_start);
    // The new neighbours to inspect.
    let mut frontier_indices = vec![idx_start];
    while let Some(frontier) = frontier_indices.pop() {
        // Get each frontier's 4 adjacent neighbours.
        for neighbour in [frontier + 1, frontier - 1, frontier + size, frontier - size] {
            // Add all floors that are not already known.
            if !['W', '#'].contains(&cage[neighbour]) && !connected_indices.contains(&neighbour) {
                // Airlocks are on the edge, and not worth expanding from.
                if !['V', '^', '<', '>'].contains(&cage[neighbour]) {
                    frontier_indices.push(neighbour);
                }
                connected_indices.insert(neighbour);
            }
        }
    }
    connected_indices.len()
}
