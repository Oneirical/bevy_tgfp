use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_resource::<SpriteSheetAtlas>()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_player)
        .run();
}

#[derive(Bundle)]
struct Creature {
    sprite: SpriteBundle,
    atlas: TextureAtlas,
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
    commands.spawn(Creature {
        sprite: SpriteBundle {
            texture: asset_server.load("spritesheet.png"),
            transform: Transform::from_scale(Vec3::new(4., 4., 0.)),
            ..default()
        },
        atlas: TextureAtlas {
            layout: atlas_layout.handle.clone(),
            index: 0,
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
