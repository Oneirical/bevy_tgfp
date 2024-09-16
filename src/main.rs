use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_resource::<SpriteSheetAtlas>()
        .insert_resource(Scale { tile_size: 3. })
        .insert_resource(Map {
            creatures: HashMap::new(),
            positions: HashMap::new(),
        })
        .insert_resource(InputDelay {
            timer: Timer::new(Duration::from_millis(120), TimerMode::Once),
        })
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_player)
        .add_systems(Startup, spawn_seed)
        .add_systems(Update, adjust_transforms)
        .add_systems(Update, displace_creatures)
        .add_systems(Update, keyboard_input)
        .add_systems(Update, player_step)
        .add_systems(Update, teleport_entity)
        .add_event::<PlayerStep>()
        .add_event::<TeleportEntity>()
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
#[derive(Resource, Clone)]
struct Map {
    creatures: HashMap<Position, Entity>,
    positions: HashMap<Entity, Position>,
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

#[derive(Event)]
struct PlayerStep {
    direction: OrdDir,
}

#[derive(Event)]
struct TeleportEntity {
    destination: Position,
    entity: Entity,
}

impl TeleportEntity {
    fn new(entity: Entity, x: i32, y: i32) -> Self {
        Self {
            destination: Position::new(x, y),
            entity,
        }
    }
}

#[derive(Bundle)]
struct Creature {
    position: Position,
    sprite: SpriteBundle,
    atlas: TextureAtlas,
}

#[derive(Copy, Clone, Debug)]
pub enum OrdDir {
    Up,
    Right,
    Down,
    Left,
}

impl OrdDir {
    pub fn as_offset(self) -> (i32, i32) {
        let (x, y) = match self {
            OrdDir::Up => (0, 1),
            OrdDir::Right => (1, 0),
            OrdDir::Down => (0, -1),
            OrdDir::Left => (-1, 0),
        };
        (x, y)
    }
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
                ..Default::default()
            },
            atlas: TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index,
            },
        });
    }
}

fn player_step(
    mut events: EventReader<PlayerStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    player: Query<(Entity, &Position), With<Player>>,
    mut delay: ResMut<InputDelay>,
) {
    let (player_entity, player_pos) = player.get_single().expect("0 or 2+ players");
    for event in events.read() {
        delay.timer.reset();
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportEntity::new(
            player_entity,
            player_pos.x + off_x,
            player_pos.y + off_y,
        ));
    }
}

fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut creature: Query<&mut Position>,
    map: Res<Map>,
) {
    for event in events.read() {
        let mut creature = creature
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        if map
            .creatures
            .get(&Position::new(event.destination.x, event.destination.y))
            .is_some()
        {
            // TODO: Raise a collision event here.
            continue;
        }
        (creature.x, creature.y) = (event.destination.x, event.destination.y);
    }
}

fn keyboard_input(
    time: Res<Time>,

    mut delay: ResMut<InputDelay>,
    mut events: EventWriter<PlayerStep>,
    input: Res<ButtonInput<KeyCode>>,
) {
    delay.timer.tick(time.delta());
    if !delay.timer.finished() {
        return;
    }

    if input.pressed(KeyCode::KeyW) {
        events.send(PlayerStep {
            direction: OrdDir::Up,
        });
    }
    if input.pressed(KeyCode::KeyD) {
        events.send(PlayerStep {
            direction: OrdDir::Right,
        });
    }
    if input.pressed(KeyCode::KeyA) {
        events.send(PlayerStep {
            direction: OrdDir::Left,
        });
    }
    if input.pressed(KeyCode::KeyS) {
        events.send(PlayerStep {
            direction: OrdDir::Down,
        });
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
