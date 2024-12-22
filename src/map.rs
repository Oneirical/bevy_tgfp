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
    for room_x in 0..8 {
        for room_y in 0..8 {
            let cage = generate_room();

            for (idx, tile_char) in cage.iter().enumerate() {
                let position =
                    Position::new(idx as i32 % 10 + room_x * 9, idx as i32 / 10 + room_y * 9);
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
    }
}

use rand::Rng;

const SIZE: usize = 9;

fn generate_room() -> Vec<char> {
    let mut grid = vec![vec!['#'; SIZE]; SIZE];
    let mut rng = rand::thread_rng();

    // Set the X markers
    grid[0][4] = '^';
    grid[4][0] = '<';
    grid[4][8] = '>';
    grid[8][4] = 'V';

    // Create a variable central area
    create_variable_center(&mut grid, &mut rng);

    // Connect X markers to the center
    connect_to_center(&mut grid, 0, 4, 4, 4);
    connect_to_center(&mut grid, 4, 0, 4, 4);
    connect_to_center(&mut grid, 4, 8, 4, 4);
    connect_to_center(&mut grid, 8, 4, 4, 4);

    // Add random paths
    for _ in 0..rng.gen_range(10..20) {
        let x = rng.gen_range(1..8);
        let y = rng.gen_range(1..8);
        if grid[x][y] == '#' {
            grid[x][y] = '.';
            // Expand the path
            for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx > 0 && nx < SIZE as i32 - 1 && ny > 0 && ny < SIZE as i32 - 1 {
                    if rng.gen_bool(0.5) {
                        grid[nx as usize][ny as usize] = '.';
                    }
                }
            }
        }
    }

    // Ensure all floor tiles are connected
    ensure_connectivity(&mut grid);

    // Replace inner walls with 'W'
    replace_inner_walls(&mut grid);

    // Place '@' and 'H' on random floor tiles
    place_special_tiles(&mut grid, &mut rng);

    grid.into_iter()
        .flat_map(|row| row.into_iter().chain(std::iter::once('\n')))
        .collect()
}

fn connect_to_center(
    grid: &mut Vec<Vec<char>>,
    start_x: usize,
    start_y: usize,
    end_x: usize,
    end_y: usize,
) {
    let mut x = start_x;
    let mut y = start_y;
    while x != end_x || y != end_y {
        grid[x][y] = '.';
        if x < end_x {
            x += 1;
        } else if x > end_x {
            x -= 1;
        } else if y < end_y {
            y += 1;
        } else if y > end_y {
            y -= 1;
        }
    }
}

fn ensure_connectivity(grid: &mut Vec<Vec<char>>) {
    let mut visited = vec![vec![false; SIZE]; SIZE];
    let mut stack = vec![];

    // Find the first floor tile
    'outer: for i in 0..SIZE {
        for j in 0..SIZE {
            if grid[i][j] == '.' {
                stack.push((i, j));
                visited[i][j] = true;
                break 'outer;
            }
        }
    }

    // DFS to mark all connected tiles
    while let Some((x, y)) = stack.pop() {
        for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < SIZE as i32 && ny >= 0 && ny < SIZE as i32 {
                let nx = nx as usize;
                let ny = ny as usize;
                if (grid[nx][ny] == '.') && !visited[nx][ny] {
                    visited[nx][ny] = true;
                    stack.push((nx, ny));
                }
            }
        }
    }

    // Connect any unvisited floor tiles
    for i in 0..SIZE {
        for j in 0..SIZE {
            if (grid[i][j] == '.') && !visited[i][j] {
                connect_to_nearest_visited(grid, &visited, i, j);
            }
        }
    }
}

fn connect_to_nearest_visited(
    grid: &mut Vec<Vec<char>>,
    visited: &Vec<Vec<bool>>,
    x: usize,
    y: usize,
) {
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((x, y, Vec::<(usize, usize)>::new()));
    let mut seen = vec![vec![false; SIZE]; SIZE];
    seen[x][y] = true;

    while let Some((cx, cy, path)) = queue.pop_front() {
        if visited[cx][cy] {
            for &(px, py) in &path {
                grid[px][py] = '.';
            }
            return;
        }

        for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx >= 0 && nx < SIZE as i32 && ny >= 0 && ny < SIZE as i32 {
                let nx = nx as usize;
                let ny = ny as usize;
                if !seen[nx][ny] {
                    seen[nx][ny] = true;
                    let mut new_path = path.clone();
                    new_path.push((nx, ny));
                    queue.push_back((nx, ny, new_path));
                }
            }
        }
    }
}

fn create_variable_center(grid: &mut Vec<Vec<char>>, rng: &mut impl Rng) {
    // Start with a clear 5x5 center
    for i in 2..7 {
        for j in 2..7 {
            grid[i][j] = '.';
        }
    }

    // Randomly add some walls in the center
    for _ in 0..rng.gen_range(1..5) {
        let x = rng.gen_range(2..7);
        let y = rng.gen_range(2..7);
        grid[x][y] = '#';
    }

    // Ensure the center remains connected
    let mut center_grid = grid[2..7]
        .iter()
        .map(|row| row[2..7].to_vec())
        .collect::<Vec<_>>();
    ensure_connectivity_subgrid(&mut center_grid);
    for i in 0..5 {
        for j in 0..5 {
            grid[i + 2][j + 2] = center_grid[i][j];
        }
    }
}

fn ensure_connectivity_subgrid(grid: &mut Vec<Vec<char>>) {
    let size = grid.len();
    let mut visited = vec![vec![false; size]; size];
    let mut stack = vec![];

    // Find the first floor tile
    'outer: for i in 0..size {
        for j in 0..size {
            if grid[i][j] == '.' {
                stack.push((i, j));
                visited[i][j] = true;
                break 'outer;
            }
        }
    }

    // DFS to mark all connected tiles
    while let Some((x, y)) = stack.pop() {
        for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < size as i32 && ny >= 0 && ny < size as i32 {
                let nx = nx as usize;
                let ny = ny as usize;
                if grid[nx][ny] == '.' && !visited[nx][ny] {
                    visited[nx][ny] = true;
                    stack.push((nx, ny));
                }
            }
        }
    }

    // Connect any unvisited floor tiles
    for i in 0..size {
        for j in 0..size {
            if grid[i][j] == '.' && !visited[i][j] {
                grid[i][j] = '#';
            }
        }
    }
}

fn replace_inner_walls(grid: &mut Vec<Vec<char>>) {
    for i in 1..SIZE - 1 {
        for j in 1..SIZE - 1 {
            if grid[i][j] == '#' {
                grid[i][j] = 'W';
            }
        }
    }
}

fn place_special_tiles(grid: &mut Vec<Vec<char>>, rng: &mut impl Rng) {
    let mut floor_tiles: Vec<(usize, usize)> = Vec::new();

    grid[0][4] = 'V';
    grid[4][0] = '<';
    grid[4][8] = '>';
    grid[8][4] = '^';

    for i in 0..SIZE {
        for j in 0..SIZE {
            if grid[i][j] == '.' {
                floor_tiles.push((i, j));
            }
        }
    }

    if floor_tiles.len() >= 2 {
        floor_tiles.shuffle(rng);
        let (x, y) = floor_tiles[0];
        grid[x][y] = '@';
        // let (x, y) = floor_tiles[1];
        // grid[x][y] = 'H';
    }
}
