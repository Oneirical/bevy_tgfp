use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_resource::<SpriteSheetAtlas>()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_player)
        .add_systems(Startup, spawn_cage)
        .add_systems(Update, adjust_transforms)
        .add_systems(Update, keyboard_input)
        .run();
}

/// Marker for the player
#[derive(Component)]
pub struct Player;

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

#[derive(Bundle)]
struct Creature {
    position: Position,
    sprite: Sprite,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    commands.spawn((
        Creature {
            position: Position { x: 4, y: 4 },
            sprite: Sprite {
                image: asset_server.load("spritesheet.png"),
                custom_size: Some(Vec2::new(64., 64.)),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index: 0,
                }),
                ..default()
            },
        },
        Player,
    ));
}

/// Each frame, adjust every entity's display location to match
/// their position on the grid, and make the camera follow the player.
fn adjust_transforms(
    mut creatures: Query<(&Position, &mut Transform, Has<Player>)>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Position>)>,
) {
    for (pos, mut trans, is_player) in creatures.iter_mut() {
        // Multiplied by the graphical size of a tile, which is 64x64.
        trans.translation.x = pos.x as f32 * 64.;
        trans.translation.y = pos.y as f32 * 64.;
        if is_player {
            // The camera follows the player.
            let mut camera_trans = camera.get_single_mut().unwrap();
            (camera_trans.translation.x, camera_trans.translation.y) =
                (trans.translation.x, trans.translation.y);
        }
    }
}

fn spawn_cage(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    let cage = "#########\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #########";
    for (idx, tile_char) in cage.char_indices() {
        let position = Position::new(idx as i32 % 9, idx as i32 / 9);
        let index = match tile_char {
            '#' => 3,
            _ => continue,
        };
        commands.spawn(Creature {
            position,
            sprite: Sprite {
                image: asset_server.load("spritesheet.png"),
                custom_size: Some(Vec2::new(64., 64.)),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index,
                }),
                ..default()
            },
        });
    }
}

/// Each frame, if a button is pressed, move the player 1 tile.
fn keyboard_input(
    input: Res<ButtonInput<KeyCode>>,
    mut player: Query<&mut Position, With<Player>>,
) {
    let mut player = player.get_single_mut().expect("0 or 2+ players");
    // WASD keys are used here. If your keyboard uses a different layout
    // (such as AZERTY), change the KeyCodes.
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
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 1, None, None);
        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();
        Self {
            handle: texture_atlases.add(layout),
        }
    }
}
