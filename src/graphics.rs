use bevy::prelude::*;

use crate::{creature::Player, map::Position};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.insert_resource(Msaa::Off);
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
        app.add_systems(Update, render_new_summons.after(adjust_transforms));
    }
}

#[derive(Resource)]
pub struct SpriteSheetAtlas {
    // Note the pub!
    pub handle: Handle<TextureAtlasLayout>,
}

impl FromWorld for SpriteSheetAtlas {
    fn from_world(world: &mut World) -> Self {
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 80, 1, None, None);
        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();
        Self {
            handle: texture_atlases.add(layout),
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });
}

/// Each frame, adjust every entity's display location to be offset
/// according to the player's location.
fn adjust_transforms(
    player: Query<&Position, With<Player>>,
    mut npcs: Query<(&Position, &mut Transform), Without<Player>>,
) {
    // There should only be one player on any given frame.
    let player_pos = player.get_single();
    if let Ok(player_pos) = player_pos {
        // Get the player's position.
        let (px, py) = (player_pos.x, player_pos.y);
        // For each Position and Transform of each non-player creature...
        for (npc_pos, mut npc_tran) in npcs.iter_mut() {
            // Measure their offset distance from the player's location.
            let (off_x, off_y) = (npc_pos.x - px, npc_pos.y - py);
            // Adjust their visual position to match this offset.
            (npc_tran.translation.x, npc_tran.translation.y) = (
                // Multiplied by the graphical size of a tile, which is 64x64.
                off_x as f32 * 2. * 16.,
                off_y as f32 * 2. * 16.,
            );
        }
    }
}

/// To avoid 1-frame flashes of newly spawned creatures, only make them appear on screen
/// after they have been passed through adjust_transforms.
fn render_new_summons(mut summoned_creatures: Query<&mut Visibility, Added<Position>>) {
    for mut creature_visibility in summoned_creatures.iter_mut() {
        *creature_visibility = Visibility::Visible;
    }
}
