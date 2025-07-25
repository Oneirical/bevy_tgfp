use crate::{
    creature::{Awake, CreatureFlags, Door, FlagEntity, Intangible, Player, Species},
    events::{OpenCloseDoor, RemoveCreature, SummonCreature, SummonProperties, TeleportEntity},
    ui::{AddMessage, Message},
    OrdDir,
};
use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use pathfinding::prelude::astar;
use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng, Rng,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map {
            creatures: HashMap::new(),
        });
        app.insert_resource(FaithsEnd {
            conveyor_active: true,
        });
        app.add_systems(Startup, spawn_cage);
        // app.add_systems(
        //     Update,
        //     slide_conveyor_belt.run_if(on_timer(Duration::from_secs(1))),
        // );
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

    pub fn subtract(&self, pos: &Self) -> Self {
        let x = self.x - pos.x;
        let y = self.y - pos.y;
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

pub fn manhattan_distance(a: &Position, b: &Position) -> i32 {
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
    pub fn get_adjacent_tiles(&self, centre: &Position) -> Vec<Position> {
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
            manhattan_distance(&a, &destination).cmp(&manhattan_distance(&b, &destination))
        });
        tiles
    }

    pub fn random_adjacent_passable_direction(&self, start: Position) -> Option<OrdDir> {
        let adjacent = self.get_adjacent_tiles(&start);
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
        let adjacent = self.get_adjacent_tiles(&start);
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

    pub fn best_astar_move(&self, start: Position, destination: Position) -> Option<OrdDir> {
        // Get all wall positions (blocked tiles)
        let walls: Vec<Position> = self
            .creatures
            .keys()
            .filter(|&&pos| pos != start && pos != destination)
            .cloned()
            .collect();

        let result = astar(
            &start,
            |&pos| {
                let mut successors = Vec::new();

                // Check each direction using OrdDir variants
                let directions = [
                    (OrdDir::Up, Position::new(pos.x, pos.y + 1)),
                    (OrdDir::Down, Position::new(pos.x, pos.y - 1)),
                    (OrdDir::Right, Position::new(pos.x + 1, pos.y)),
                    (OrdDir::Left, Position::new(pos.x - 1, pos.y)),
                ];

                for (dir, new_pos) in directions {
                    // Check if the new position is not a wall
                    if !walls.contains(&new_pos) {
                        successors.push((new_pos, 1)); // Cost of 1 for each move
                    }
                }

                successors
            },
            // Manhattan distance heuristic
            |&pos| (pos.x.abs_diff(destination.x) + pos.y.abs_diff(destination.y)) as u32,
            |&pos| pos == destination,
        );

        // Extract the first move direction from the path
        result.and_then(|(path, _)| {
            if path.len() >= 2 {
                let first_step = path[1];
                OrdDir::direction_towards_adjacent_tile(start, first_step)
            } else {
                None // No path found or already at destination
            }
        })
    }
}

/// Newly spawned creatures earn their place in the HashMap.
pub fn register_creatures(
    mut map: ResMut<Map>,
    // Any entity that has a Position that just got added to it -
    // currently only possible as a result of having just been spawned in.
    // Naturally intangible creatures skip this.
    newly_positioned_creatures: Query<(&Position, Entity, &CreatureFlags), Added<Position>>,
    intangible_query: Query<&FlagEntity, Added<Intangible>>,

    intangible_creature: Query<&Position>,
    tangible_creatures: Query<&Position, With<Species>>,
    flag_query: Query<&FlagEntity>,
    mut tangible_entities: RemovedComponents<Intangible>,
    mut remove: EventWriter<RemoveCreature>,
    mut transform: Query<&mut Transform>,
) {
    for (position, entity, flags) in newly_positioned_creatures.iter() {
        // Intangible creatures are not added to the map.
        if !intangible_query.contains(flags.effects_flags)
            && !intangible_query.contains(flags.species_flags)
        {
            // Insert the new creature in the Map. Position implements Copy,
            // so it can be dereferenced (*), but `.clone()` would have been
            // fine too.
            map.creatures.insert(*position, entity);
        }
    }

    // A creature recovering its tangibility is added to the map.
    for flag_entity in tangible_entities.read() {
        // NOTE: This started occasionally panicking (such as when stepping on an Artistic
        // trap) due to unwrapping flag_query. This hack makes the errors silent, which seems okay
        // since these creatures are despawned and aren't in the map anymore anyways.
        if let Ok(entity) = flag_query.get(flag_entity) {
            let entity = entity.parent_creature;
            if let Ok(tangible_position) = tangible_creatures.get(entity) {
                if map.creatures.get(tangible_position).is_some() {
                    // NOTE: This is kind of like Caves of Qud's death by phasing
                    // ("the pauli principle"). Creatures recovering tangibility
                    // on top of another die. I am mostly adding this so I can
                    // debug the occasional door issue.
                    remove.write(RemoveCreature { entity });
                    dbg!(tangible_position);
                    dbg!("A creature recovered its tangibility while on top of another creature!");
                } else {
                    map.creatures.insert(*tangible_position, entity);
                    // Aligning the sprite at 0. once again so that it appears on top
                    // of intangible creatures.
                    transform.get_mut(entity).unwrap().translation.z = 0.;
                }
            }
        }
    }

    // Newly intangible creatures are removed from the map.
    for flag_entity in intangible_query.iter() {
        let intangible_position = intangible_creature
            .get(flag_entity.parent_creature)
            .unwrap();
        if let Some(preexisting_entity) = map.creatures.get(intangible_position) {
            // Check that the entity being removed is actually the intangible entity.
            // REASON: If a creature spawns in already intangible on top of a
            // tangible creature, without this check, it would remove
            // the tangible creature from the map.
            if *preexisting_entity == flag_entity.parent_creature {
                map.creatures.remove(intangible_position);
            }
        }
        // Lowering the sprite of the intangible creature, ensuring other creatures
        // appear to be on top of it.
        transform
            .get_mut(flag_entity.parent_creature)
            .unwrap()
            .translation
            .z = -5.;
    }
}

#[derive(Resource, Debug)]
pub struct FaithsEnd {
    pub conveyor_active: bool,
}

#[derive(Resource)]
pub struct ConveyorTracker {
    pub number_spawned: usize,
    pub open_doors_next: bool,
}

pub fn new_cage_on_conveyor(
    mut tracker: ResMut<ConveyorTracker>,
    mut summon: EventWriter<SummonCreature>,
) {
    tracker.number_spawned += 1;
    // let mut cage = generate_cage(0, false, true, 9, &[OrdDir::Left, OrdDir::Right]);
    let mut cage = get_vault(0);
    add_creatures(&mut cage.chars, 2 + tracker.number_spawned);

    let cage_corner = Position::new(26, 52 - (cage.size.1 as i32 - 9));
    for (idx, tile_char) in cage.chars.iter().enumerate() {
        let position = Position::new(
            (idx % cage.size.0) as i32 + cage_corner.x,
            cage.size.1 as i32 - (idx / cage.size.0) as i32 + cage_corner.y,
        );
        let species = match tile_char {
            '#' => Species::Wall,
            'S' => Species::Scion,
            'T' => Species::Tinker,
            '@' => Species::Player,
            'W' => Species::WeakWall,
            '2' => Species::Second,
            'A' => Species::Apiarist,
            'F' => Species::Shrike,
            'H' => Species::Hechaton,
            'G' => Species::Grappler,
            'O' => Species::Oracle,
            'E' => Species::EpsilonHead,
            't' => Species::EpsilonTail,
            'x' => Species::CageSlot,
            'C' => Species::AxiomaticSeal,
            '0' => Species::Exploder,
            'R' => Species::Railway,
            '{' => Species::Cart,
            'm' => Species::Ragemaw,
            '^' | '>' | '<' | 'V' => Species::Airlock,
            'w' | 'n' | 'e' | 's' => Species::CageBorder,
            _ => continue,
        };
        let mut properties = vec![SummonProperties::Momentum(match tile_char {
            '^' => OrdDir::Up,
            '>' => OrdDir::Right,
            '<' => OrdDir::Left,
            'n' => OrdDir::Up,
            'e' => OrdDir::Right,
            'w' => OrdDir::Left,
            's' => OrdDir::Down,
            'V' | _ => OrdDir::Down,
        })];
        if [
            Species::Scion,
            Species::Shrike,
            Species::Second,
            Species::Tinker,
            Species::Apiarist,
            Species::Oracle,
            Species::Hechaton,
            Species::Grappler,
            Species::Exploder,
        ]
        .contains(&species)
        {
            properties.push(SummonProperties::Sleeping);
        }
        summon.write(SummonCreature {
            species,
            position,
            properties,
        });
    }
}

pub fn spawn_cage(
    mut summon: EventWriter<SummonCreature>,
    mut text: EventWriter<AddMessage>,
    mut commands: Commands,
) {
    let quarry = "\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
.......#####............#vvvvvvvvvvv#............#####.......\
......##...##...........#vvvvvvvvvvv#...........##...##......\
......#.....#...........#vvvvvvvvvvv#...........#.....#......\
......#..B..#...........#vvvvvvvvvvv#...........#..B..#......\
......#.....#...........#vvvvvvvvvvv#...........#.....#......\
......##...##...........#vvvvvvvvvvv#...........##...##......\
.#####.##+##............#vvvvvvvvvvv#...........###+##.#####.\
##...###...#####........#vvvvvvvvvvv#........#####...###...##\
#.....#........####.....#vvvvvvvvvvv#.....####........#.....#\
#..B..+...........###...#vvvvvvvvvvv#...###...........+..B..#\
#.....#.............##..#vvvvvvvvvvv#..##.............#.....#\
##...###.............#..#vvvvvvvvvvv#..#....@........###...##\
.#####.##+####.......##.#vvvvvvvvvvv#.##.........##+##.#####.\
......##...######.....###vvvvvvvvvvv###.........##...##......\
......#.....#...###....##vvvvvvvvvvv##..........#.....#......\
......#..B..#.....##....#vvvvvvvvvvv#.......#####.....#......\
......#.....#..#####....#vvvvvvvvvvv#....####.........#......\
......##...##.##'''##...##vvvvvvvvv##...##'''.sss.....##.....\
.......#####..#'''''#....#vvvvvvvvv#....#''''exxxw.....##....\
..............#''C''+....>vvvvvvvvv<....+''C'exxxw......#....\
.......#####..#'''''#....#vvvvvvvvv#....#''''exxxw.....##....\
......##...##.##'''##...##vvvvvvvvv##...##'''.nnn.....##.....\
......#.....#..#####....#vvvvvvvvvvv#....####.........#......\
......#..B..#.....##....#vvvvvvvvvvv#.......#####..B..#......\
......#.....#...###....##vvvvvvvvvvv##..........#.....#......\
......##...######.....###vvvvvvvvvvv###.........##...##......\
.#####.##+####.......##.#vvvvvvvvvvv#.##.........##+##.#####.\
##...###.............#..#vvvvvvvvvvv#..#.............###...##\
#.....#.............##..#vvvvvvvvvvv#..##.............#.....#\
#..B..+...........###...#vvvvvvvvvvv#...###...........+..B..#\
#.....#........####.....#vvvvvvvvvvv#.....####........#.....#\
##...###...#####........#vvvvvvvvvvv#........#####...###...##\
.#####.##+##............#vvvvvvvvvvv#............##+##.#####.\
......##...##...........#vvvvvvvvvvv#...........##...##......\
......#.....#...........#vvvvvvvvvvv#...........#.....#......\
......#..B..#...........#vvvvvvvvvvv#...........#..B..#......\
......#.....#...........#vvvvvvvvvvv#...........#.....#......\
......##...##...........#vvvvvvvvvvv#...........##...##......\
.......#####............#vvvvvvvvvvv#............#####.......\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#vvvvvvvvvvv#........................\
........................#$$$$$$$$$$$#........................\
";
    for (idx, tile_char) in quarry.chars().enumerate() {
        let position = Position::new(idx as i32 % 61, 61 - idx as i32 / 61);
        let species = match tile_char {
            '#' => Species::Wall,
            '@' => Species::Player,
            'W' => Species::WeakWall,
            'x' => Species::CageSlot,
            'v' => Species::ConveyorBelt,
            'C' => Species::AxiomaticSeal,
            '$' => Species::Grinder,
            '^' | '>' | '<' | 'V' => Species::Airlock,
            'w' | 'n' | 'e' | 's' => Species::CageBorder,
            _ => continue,
        };
        let properties = vec![SummonProperties::Momentum(match tile_char {
            '^' => OrdDir::Up,
            '>' => OrdDir::Right,
            '<' => OrdDir::Left,
            'n' => OrdDir::Up,
            'e' => OrdDir::Right,
            'w' => OrdDir::Left,
            's' => OrdDir::Down,
            'V' | _ => OrdDir::Down,
        })];
        summon.write(SummonCreature {
            species,
            position,
            properties,
        });
    }
    commands.run_system_cached(new_cage_on_conveyor);
}

fn add_creatures(cage: &mut [char], creatures_amount: usize) {
    let creature_chars = ['A', 'T', 'F', '2', 'H', 'O', 'S', 'G', '0', 'm'];

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
    spawn_walls: bool,
    size: usize,
    connections: &[OrdDir],
) -> Vault {
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
            } else if rng.gen::<f32>() < 0.3 && spawn_walls {
                if floor > 11 {
                    if rng.gen::<f32>() < 0.1 {
                        cage.push('t');
                        passable_tiles += 1;
                    } else {
                        cage.push('.');
                        passable_tiles += 1;
                    }
                } else {
                    cage.push('W');
                }
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
            return Vault {
                chars: cage,
                size: (size, size),
            };
        } else {
            cage.clear();
        }
    }
    panic!("Cage generation timeout achieved.");
}

fn xy_idx(idx: usize, size: usize) -> (usize, usize) {
    (idx % size, idx / size)
}

fn idx_from_xy(x: usize, y: usize, size: usize) -> usize {
    y * size + x
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

pub const VAULTS: &[&str] = &["
#########
#.......#
#.RRRRR.#
#...R...#
###.R.###
###.R.###
###.R.###
###.R.###
#...R...#
#...R...#
<.RRRRR.>
#...R{..#
#...R...#
###.R.###
###.R.###
###.R.###
###.R.###
#...R...#
#.RRRRR.#
#.......#
#########
"];

#[derive(Debug)]
pub struct Vault {
    pub chars: Vec<char>,
    pub size: (usize, usize), // (width, height)
}

pub fn get_vault(index: usize) -> Vault {
    VAULTS
        .get(index)
        .map(|vault_str| {
            // Split into lines and filter out empty ones (in case of leading/trailing newlines)
            let lines: Vec<&str> = vault_str
                .split('\n')
                .filter(|line| !line.is_empty())
                .collect();

            // Calculate dimensions
            let height = lines.len();
            let width = lines.first().map_or(0, |line| line.len());

            // Combine all characters without newlines
            let chars: Vec<char> = lines.join("").chars().collect();

            Vault {
                chars,
                size: (width, height),
            }
        })
        .unwrap()
}
