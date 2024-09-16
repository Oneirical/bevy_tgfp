use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_resource::<SpriteSheetAtlas>()
        .insert_resource(Scale { tile_size: 3. })
        .insert_resource(Map {
            creatures: HashMap::new(),
        })
        .insert_resource(InputDelay {
            timer: Timer::new(Duration::from_millis(50), TimerMode::Once),
        })
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_player)
        .add_systems(Startup, spawn_seed)
        .add_systems(Update, adjust_transforms)
        .add_systems(Update, place_creatures)
        .add_systems(Update, keyboard_input)
        .run();
}

/// Marker for the player
#[derive(Component)]
struct Player;

/// The scale of tiles. Non-round floats will cause artifacts.
#[derive(Resource)]
struct Scale {
    tile_size: f32,
}

/// How long to wait until input is accepted again.
#[derive(Resource)]
pub struct InputDelay {
    pub timer: Timer,
}

/// The position of every entity, updated automatically.
#[derive(Resource)]
struct Map {
    creatures: HashMap<Position, Entity>,
}

/// A position on the map.
#[derive(Component, PartialEq, Eq, Hash, Copy, Clone, Debug)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Component)]
struct Camera;

#[derive(Bundle)]
struct Creature {
    position: Position,
    sprite: SpriteBundle,
    atlas: TextureAtlas,
}

/// Adjust every entity's display location to be offset according to the player.
fn adjust_transforms(
    player: Query<&Position, With<Player>>,
    scale: Res<Scale>,
    mut npcs: Query<(&Position, &mut Transform), Without<Player>>,
) {
    let player_pos = player.get_single().expect("0 or 2+ players");
    let (px, py) = (player_pos.x, player_pos.y);
    for (npc_pos, mut npc_tran) in npcs.iter_mut() {
        let (off_x, off_y) = (npc_pos.x - px, npc_pos.y - py);
        (npc_tran.translation.x, npc_tran.translation.y) = (
            off_x as f32 * scale.tile_size * 16.,
            off_y as f32 * scale.tile_size * 16.,
        );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Camera,
    ));
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
                ..Default::default()
            },
            atlas: TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index: 0,
            },
        },
        Player,
    ));
}

fn place_creatures(
    mut map: ResMut<Map>,
    displaced_creatures: Query<(Entity, &Position), Changed<Position>>,
) {
    for (entity, position) in displaced_creatures.iter() {
        map.creatures.insert(*position, entity);
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
                ..Default::default()
            },
            atlas: TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index,
            },
        });
    }
}

fn keyboard_input(
    input: Res<ButtonInput<KeyCode>>,
    mut player: Query<&mut Position, With<Player>>,
) {
    let mut player = player.get_single_mut().expect("0 or 2+ players");
    if input.pressed(KeyCode::KeyW) {
        player.y += 1;
    }
    if input.pressed(KeyCode::KeyD) {
        player.x += 1;
    }
    if input.pressed(KeyCode::KeyA) {
        player.x -= 1;
    }
    if input.pressed(KeyCode::KeyS) {
        player.y -= 1;
    }
}

#[derive(Resource)]
struct SpriteSheetAtlas {
    handle: Handle<TextureAtlasLayout>,
}

impl FromWorld for SpriteSheetAtlas {
    fn from_world(world: &mut World) -> Self {
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 160, 2, None, None);
        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();
        Self {
            handle: texture_atlases.add(layout),
        }
    }
}
