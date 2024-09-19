use std::f32::consts::PI;

use bevy::{prelude::*, utils::HashMap};

use crate::{
    creature::{Creature, Hunt, Ipseity, Player, Soul, Species},
    graphics::{AnimationOffset, Scale, SpriteSheetAtlas},
    OrdDir,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map {
            creatures: HashMap::new(),
            positions: HashMap::new(),
        });
        app.add_systems(Startup, spawn_player);
        app.add_systems(Startup, spawn_seed);
        app.add_systems(Update, displace_creatures);
    }
}

/// The position of every entity, updated automatically.
#[derive(Resource, Clone)]
pub struct Map {
    pub creatures: HashMap<Position, Entity>,
    pub positions: HashMap<Entity, Position>,
}

impl Map {
    /// Is this tile empty?
    pub fn is_empty(&self, x: i32, y: i32) -> bool {
        self.creatures.get(&Position::new(x, y)).is_none()
    }

    pub fn get_entity_at(&self, x: i32, y: i32) -> Option<&Entity> {
        self.creatures.get(&Position::new(x, y))
    }

    /// Find all adjacent accessible tiles to start, and pick the one closest to end.
    pub fn best_manhattan_move(&self, start: Position, end: Position) -> Option<Position> {
        let mut options = [
            Position::new(start.x, start.y + 1),
            Position::new(start.x, start.y - 1),
            Position::new(start.x + 1, start.y),
            Position::new(start.x - 1, start.y),
        ];
        options.sort_by(|&a, &b| manhattan_distance(a, end).cmp(&manhattan_distance(b, end)));

        let best_move = options
            .iter()
            // Only keep either the destination or unblocked tiles.
            .filter(|&p| *p == end || self.is_empty(p.x, p.y))
            .next();

        //FIXME Dereferencing the inner part here seems a little janky. There might be a better way.

        if let Some(best_move) = best_move {
            Some(*best_move)
        } else {
            None
        }
    }

    pub fn update_map(&mut self, entity: Entity, old_pos: Position, new_pos: Position) {
        // If the entity already existed in the Map's records, remove it.
        // TODO Since .insert returns the old one, the old_pos field might be unnecessary.
        if self.positions.get(&entity).is_some() {
            self.creatures.remove(&old_pos);
            self.positions.remove(&entity);
        }
        self.creatures.insert(new_pos, entity);
        self.positions.insert(entity, new_pos);
    }
}

fn manhattan_distance(a: Position, b: Position) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

/// A position on the map.
#[derive(Component, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn update(&mut self, x: i32, y: i32) {
        (self.x, self.y) = (x, y);
    }

    pub fn shift(&mut self, dx: i32, dy: i32) {
        (self.x, self.y) = (self.x + dx, self.y + dy);
    }
}

fn spawn_player(
    mut commands: Commands,
    scale: Res<Scale>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    commands.spawn((
        Creature {
            position: Position { x: 4, y: 4 },
            sprite: SpriteBundle {
                texture: asset_server.load("spritesheet.png"),
                transform: Transform::from_scale(Vec3::new(scale.tile_size, scale.tile_size, 0.)),
                ..default()
            },
            atlas: TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index: 0,
            },
            ipseity: Ipseity::new(&[(Soul::Saintly, 2), (Soul::Ordered, 2), (Soul::Artistic, 2)]),
            animation: AnimationOffset::new(),
            species: Species::Terminal,
        },
        Player,
    ));
}

/// Newly spawned creatures earn their place in the HashMap.
fn displace_creatures(
    mut map: ResMut<Map>,
    displaced_creatures: Query<(Entity, &Position), Added<Position>>,
) {
    for (entity, position) in displaced_creatures.iter() {
        map.creatures.insert(*position, entity);
        map.positions.insert(entity, *position);
    }
}

fn spawn_seed(
    mut commands: Commands,
    scale: Res<Scale>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    let seed = "####V#####.......##.......##.......#<.......>#.......##.......##.......#####^####";
    for (idx, tile_char) in seed.char_indices() {
        let position = Position::new(idx as i32 % 9, idx as i32 / 9);
        let (index, species) = match tile_char {
            '#' => (3, Species::Wall),
            'S' => (4, Species::Scion),
            'V' => (
                17,
                Species::Airlock {
                    orientation: crate::OrdDir::Down,
                },
            ),
            '^' => (
                17,
                Species::Airlock {
                    orientation: crate::OrdDir::Up,
                },
            ),
            '<' => (
                17,
                Species::Airlock {
                    orientation: crate::OrdDir::Left,
                },
            ),
            '>' => (
                17,
                Species::Airlock {
                    orientation: crate::OrdDir::Right,
                },
            ),
            '.' => continue,
            _ => panic!(),
        };
        let mut transform = Transform::from_scale(Vec3::new(scale.tile_size, scale.tile_size, 0.));
        if let Species::Airlock { orientation } = species {
            match orientation {
                OrdDir::Down => transform.rotate_z(0.),
                OrdDir::Right => transform.rotate_z(PI / 2.),
                OrdDir::Up => transform.rotate_z(PI),
                OrdDir::Left => transform.rotate_z(3. * PI / 2.),
            }
        }

        let id = commands
            .spawn((Creature {
                position,
                sprite: SpriteBundle {
                    texture: asset_server.load("spritesheet.png"),
                    transform,
                    ..default()
                },
                atlas: TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index,
                },
                ipseity: Ipseity::new(&[(Soul::Immutable, 1)]),
                animation: AnimationOffset::new(),
                species,
            },))
            .id();
        // TODO If it's a scion, add Hunt
        if index == 4 {
            commands
                .entity(id)
                .insert(Hunt)
                .insert(Ipseity::new(&[(Soul::Feral, 4)]));
        }
    }
}
