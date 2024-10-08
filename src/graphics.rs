use std::time::Duration;

use bevy::prelude::*;

use crate::{creature::Player, map::Position};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.insert_resource(Msaa::Off);
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
        app.add_systems(Update, render_new_summons);
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
    mut player: Query<(&Position, Option<&mut SlideAnimation>), With<Player>>,
    mut npcs: Query<(&Position, &mut Transform, Option<&mut SlideAnimation>), Without<Player>>,
    time: Res<Time>,
) {
    // There should only be one player on any given frame.
    if let Ok((player_pos, player_animation)) = player.get_single_mut() {
        // Get the player's position, adjusted for its current animation.
        let (player_offset_x, player_offset_y) =
            if let Some(mut player_animation) = player_animation {
                player_animation.get_animation_offsets(time.delta(), *player_pos)
            } else {
                (player_pos.x as f32, player_pos.y as f32)
            };
        // For each Position, Transform, SlideAnimation of each non-player creature...
        for (npc_pos, mut npc_tran, npc_anim) in npcs.iter_mut() {
            let (npc_offset_x, npc_offset_y) = if let Some(mut npc_anim) = npc_anim {
                npc_anim.get_animation_offsets(time.delta(), *npc_pos)
            } else {
                (npc_pos.x as f32, npc_pos.y as f32)
            };
            // Measure their offset distance from the player's location.
            let (off_x, off_y) = (
                npc_offset_x - player_offset_x,
                npc_offset_y - player_offset_y,
            );
            // Adjust their visual position to match this offset.
            (npc_tran.translation.x, npc_tran.translation.y) = (
                // Multiplied by the graphical size of a tile, which is 64x64.
                off_x as f32 * 4. * 16.,
                off_y as f32 * 4. * 16.,
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

#[derive(Component)]
pub struct SlideAnimation {
    pub elapsed: Timer,
    pub origin: Position,
}

impl SlideAnimation {
    fn get_animation_offsets(&mut self, delta_time: Duration, destination: Position) -> (f32, f32) {
        let source = self.origin;
        self.elapsed.tick(delta_time);
        let (dx, dy) = (destination.x - source.x, destination.y - source.y);
        (
            destination.x as f32 - dx as f32 * self.elapsed.fraction_remaining(),
            destination.y as f32 - dy as f32 * self.elapsed.fraction_remaining(),
        )
    }
}
