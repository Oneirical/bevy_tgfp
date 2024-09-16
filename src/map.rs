use bevy::{prelude::*, utils::HashMap};

use crate::{
    graphics::{Scale, SpriteSheetAtlas},
    Creature, Player,
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
        },
        Player,
    ));
}

/// Remove the old entry from the map, replace with the new position
/// of a recently moved entity.
fn displace_creatures(
    mut map: ResMut<Map>,
    displaced_creatures: Query<(Entity, &Position), Changed<Position>>,
) {
    for (entity, position) in displaced_creatures.iter() {
        let old_map = map.clone();
        let old_pos = old_map.positions.get(&entity);
        if let Some(old_pos) = old_pos {
            map.creatures.remove(old_pos);
            map.positions.remove(&entity);
        }
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
    let seed = "##########.......##.......##.......##.......##.......##.......##.......##########";
    for (idx, tile_char) in seed.char_indices() {
        let position = Position::new(idx as i32 % 9, idx as i32 / 9);
        let index = match tile_char {
            '#' => 3,
            '.' => continue,
            _ => panic!(),
        };
        commands.spawn(Creature {
            position,
            sprite: SpriteBundle {
                texture: asset_server.load("spritesheet.png"),
                transform: Transform::from_scale(Vec3::new(scale.tile_size, scale.tile_size, 0.)),
                ..default()
            },
            atlas: TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index,
            },
        });
    }
}
