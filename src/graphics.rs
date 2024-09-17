use bevy::prelude::*;

use crate::{creature::Player, Position};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.insert_resource(Scale { tile_size: 3. });
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
    }
}

/// The scale of tiles. Non-round floats will cause artifacts.
#[derive(Resource)]
pub struct Scale {
    pub tile_size: f32,
}

#[derive(Resource)]
pub struct SpriteSheetAtlas {
    pub handle: Handle<TextureAtlasLayout>,
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
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });
}
