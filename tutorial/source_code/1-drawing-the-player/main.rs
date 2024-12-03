use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_resource::<SpriteSheetAtlas>()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_player)
        .run();
}

/// Common components relating to spawning a new Creature.
#[derive(Bundle)]
struct Creature {
    sprite: Sprite,
}

/// The camera, allowing Entities to be seen through the App window.
fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d::default(), Transform::from_xyz(0., 0., 0.)));
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    commands.spawn(Creature {
        sprite: Sprite {
            image: asset_server.load("spritesheet.png"),
            // Custom size, for 64x64 pixel tiles.
            custom_size: Some(Vec2::new(64., 64.)),
            // Our atlas.
            texture_atlas: Some(TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index: 0,
            }),
            ..default()
        },
    });
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
