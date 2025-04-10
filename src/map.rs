use std::time::Duration;

use bevy::{
    prelude::*,
    time::common_conditions::on_timer,
    utils::{HashMap, HashSet},
};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng, Rng,
};

use crate::{
    creature::{Awake, ConveyorBelt, CreatureFlags, Door, FlagEntity, Intangible, Player, Species},
    events::{OpenCloseDoor, RemoveCreature, SummonCreature, SummonProperties, TeleportEntity},
    ui::{AddMessage, Message},
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
            manhattan_distance(&a, &destination).cmp(&manhattan_distance(&b, &destination))
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
                    remove.send(RemoveCreature { entity });
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
            if *preexisting_entity != flag_entity.parent_creature {
                continue;
            }
        }
        // Lowering the sprite of the intangible creature, ensuring other creatures
        // appear to be on top of it.
        transform
            .get_mut(flag_entity.parent_creature)
            .unwrap()
            .translation
            .z = -5.;
        map.creatures.remove(intangible_position);
    }
}

#[derive(Resource, Debug)]
pub struct FaithsEnd {
    pub cage_address_position: HashMap<Position, usize>,
    pub cage_dimensions: HashMap<usize, (Position, Position)>,
    pub current_cage: usize,
}

pub fn is_soul_cage_room(room: usize) -> bool {
    false
}

// TODO: If a creature is transformed into something Immobile,
// like Abazon, it will get stuck and make other entities
// bump into it.
pub fn slide_conveyor_belt(
    query: Query<(Entity, &Position), With<ConveyorBelt>>,
    doors: Query<(Entity, &Position, &CreatureFlags, Has<ConveyorBelt>)>,
    closed_door_query: Query<&Door, Without<Intangible>>,
    mut teleport: EventWriter<TeleportEntity>,
    mut commands: Commands,
    mut remove: EventWriter<RemoveCreature>,
    mut open: EventWriter<OpenCloseDoor>,
    mut tracker: ResMut<ConveyorTracker>,
    awake_creatures: Query<&Position, With<Awake>>,
) {
    if tracker.open_doors_next {
        let (boundary_a, boundary_b) = (Position::new(27, 24), Position::new(33, 30));
        for pos in awake_creatures.iter() {
            if pos.is_within_range(&boundary_a, &boundary_b) {
                return;
            }
        }
        for (door, pos, flags, is_on_conveyor) in doors.iter() {
            if (closed_door_query.contains(flags.species_flags)
                || closed_door_query.contains(flags.effects_flags))
                && ((is_on_conveyor && pos.y == 27)
                    || pos == &Position::new(25, 27)
                    || pos == &Position::new(35, 27))
            {
                open.send(OpenCloseDoor {
                    entity: door,
                    open: true,
                });
            }
        }
        return;
    }
    let mut creatures = query.iter().collect::<Vec<(Entity, &Position)>>();
    let mut send_next = false;
    let mut close_door = false;
    creatures.sort_by(|&a, &b| a.1.y.cmp(&b.1.y));
    for (entity, pos) in creatures {
        if pos.y == 32 {
            send_next = true;
        } else if pos.y == 23 {
            close_door = true;
        }
        if pos.y < -10 {
            remove.send(RemoveCreature { entity });
        } else {
            // teleport.send(TeleportEntity {
            //     destination: Position::new(pos.x, pos.y - 9),
            //     entity,
            // });
        }
    }
    if send_next {
        tracker.open_doors_next = true;
    }
    if close_door {
        commands.run_system_cached(new_cage_on_conveyor);
        for (door, pos, flags, _is_on_conveyor) in doors.iter() {
            if (!closed_door_query.contains(flags.species_flags)
                && !closed_door_query.contains(flags.effects_flags))
                && (pos == &Position::new(25, 27) || pos == &Position::new(35, 27))
            {
                open.send(OpenCloseDoor {
                    entity: door,
                    open: false,
                });
            }
        }
    }
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
    let mut cage = generate_cage(0, false, true, 9, &[OrdDir::Left, OrdDir::Right]);
    add_creatures(&mut cage, 2 + tracker.number_spawned, false);

    let cage_corner = Position::new(26, -5 + 9 * 7);
    for (idx, tile_char) in cage.iter().enumerate() {
        let position = Position::new(
            (idx % 9) as i32 + cage_corner.x,
            9 as i32 - (idx / 9) as i32 + cage_corner.y,
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
            'E' => Species::EpsilonHead,
            't' => Species::EpsilonTail,
            'x' => Species::CageSlot,
            'C' => Species::AxiomaticSeal,
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
        properties.push(SummonProperties::ConveyorBelt);
        if [
            Species::Hunter,
            Species::Shrike,
            Species::Second,
            Species::Tinker,
            Species::Apiarist,
            Species::Oracle,
        ]
        .contains(&species)
        {
            properties.push(SummonProperties::Sleeping);
        }
        summon.send(SummonCreature {
            species,
            position,
            properties,
        });
    }
}

pub fn spawn_cage(
    mut summon: EventWriter<SummonCreature>,
    mut faiths_end: ResMut<FaithsEnd>,
    player: Query<&Player>,
    mut text: EventWriter<AddMessage>,
) {
    text.send(AddMessage {
        message: Message::Tutorial,
    });
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
#..B..+..G........###...#vvvvvvvvvvv#...###........G..+..B..#\
#.....#.............##..#vvvvvvvvvvv#..##.............#.....#\
##...###.............#..#vvvvvvvvvvv#..#....@........###...##\
.#####.##+####.......##.#vvvvvvvvvvv#.##.........##+##.#####.\
......##...######.....###vvvvvvvvvvv###.........##...##......\
......#.....#...###....##vvvvvvvvvvv##..........#.....#......\
......#..B..#.....##....#vvvvvvvvvvv#.......#####.....#......\
......#.....#..#####....#vvvvvvvvvvv#....####.........#......\
......##...##.##'''##...##vvvvvvvvv##...##'''.sss.....##.....\
.......#####..#'''''#....#vvvvvvvvv.....#''''exxxw.....##....\
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
#..B..+..G........###...#vvvvvvvvvvv#...###........G..+..B..#\
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
    let tower_height = 2;
    let mut tower_height_tiles = 0;
    let first_room_size = 9;
    for tower_floor in 0..tower_height {
        let size = 9;
        let mut cage = generate_cage(
            tower_floor,
            // Spawn the player in the first room
            // (the player must not already exist).
            tower_floor == 0 && player.is_empty(),
            !is_soul_cage_room(tower_floor),
            size,
            &[OrdDir::Left, OrdDir::Right],
        );
        if !is_soul_cage_room(tower_floor) {
            add_creatures(&mut cage, 2 + tower_floor, tower_floor > 11);
        }

        let cage_corner = Position::new(26, tower_height_tiles as i32 - 5 + 9 * 4);
        let iterator = if tower_floor == 0 {
            quarry.chars().collect::<Vec<char>>()
        } else {
            cage
        };
        for (idx, tile_char) in iterator.iter().enumerate() {
            let position = if tower_floor == 0 {
                Position::new(idx as i32 % 61, 61 - idx as i32 / 61)
            } else {
                Position::new(
                    (idx % size) as i32 + cage_corner.x,
                    size as i32 - (idx / size) as i32 + cage_corner.y,
                )
            };
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
                'E' => Species::EpsilonHead,
                't' => Species::EpsilonTail,
                'x' => Species::CageSlot,
                'v' => Species::ConveyorBelt,
                'C' => Species::AxiomaticSeal,
                '$' => Species::Grinder,
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
            if tower_floor > 0 {
                properties.push(SummonProperties::ConveyorBelt);
            }
            if [
                Species::Hunter,
                Species::Shrike,
                Species::Second,
                Species::Tinker,
                Species::Apiarist,
                Species::Oracle,
            ]
            .contains(&species)
            {
                properties.push(SummonProperties::Sleeping);
            }
            summon.send(SummonCreature {
                species,
                position,
                properties,
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
        if tower_floor != 0 {
            tower_height_tiles += size;
        }
    }
}

fn add_creatures(cage: &mut [char], creatures_amount: usize, spawn_snake: bool) {
    let creature_chars = if spawn_snake {
        ['E', 'F', 'H', 'E', 't', 't']
    } else {
        ['A', 'T', 'F', '2', 'H', 'O']
    };

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
        if is_soul_cage_room(floor) {
            cage[idx_from_xy(4, 1, size)] = 'C';
            for i in 3..6 {
                cage[idx_from_xy(i, 2, size)] = 's';
                cage[idx_from_xy(i, 6, size)] = 'n';
                cage[idx_from_xy(6, i, size)] = 'w';
                cage[idx_from_xy(2, i, size)] = 'e';
                for j in 3..6 {
                    cage[idx_from_xy(i, j, size)] = 'x';
                }
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
