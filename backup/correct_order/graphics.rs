use bevy::prelude::*;

use crate::{creature::Player, map::Position};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.insert_resource(SlideAnimation {
            elapsed: Timer::from_seconds(0.4, TimerMode::Once),
        });
        app.insert_resource(Msaa::Off);
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
        app.add_systems(Update, render_new_summons);
    }
}

#[derive(Resource)]
pub struct SlideAnimation {
    pub elapsed: Timer,
}

#[derive(Resource)]
pub struct SpriteSheetAtlas {
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

fn adjust_transforms(
    mut creatures: Query<(&Position, &mut Transform, Has<Player>)>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Position>)>,
    mut animation_timer: ResMut<SlideAnimation>,
    time: Res<Time>,
) {
    let fraction_before_tick = animation_timer.elapsed.fraction();
    animation_timer.elapsed.tick(time.delta());
    // Calculate what % of the animation has elapsed during this tick.
    let fraction_ticked = animation_timer.elapsed.fraction() - fraction_before_tick;
    for (pos, mut trans, is_player) in creatures.iter_mut() {
        // The distance between where a creature CURRENTLY is,
        // and the destination of a creature's movement.
        // Multiplied by the graphical size of a tile, which is 64x64.
        let (dx, dy) = (
            pos.x as f32 * 64. - trans.translation.x,
            pos.y as f32 * 64. - trans.translation.y,
        );
        // The distance between the original position and the destination position.
        let (ori_dx, ori_dy) = (
            dx / animation_timer.elapsed.fraction_remaining(),
            dy / animation_timer.elapsed.fraction_remaining(),
        );
        // The sprite approaches its destination.
        trans.translation.x = bring_closer_to_target_value(
            trans.translation.x,
            ori_dx * fraction_ticked,
            pos.x as f32 * 64.,
        );
        trans.translation.y = bring_closer_to_target_value(
            trans.translation.y,
            ori_dy * fraction_ticked,
            pos.y as f32 * 64.,
        );
        if is_player {
            // The camera follows the player.
            let mut camera_trans = camera.get_single_mut().unwrap();
            (camera_trans.translation.x, camera_trans.translation.y) =
                (trans.translation.x, trans.translation.y);
        }
    }
}

fn bring_closer_to_target_value(value: f32, adjustment: f32, target_value: f32) -> f32 {
    let adjustment = adjustment.abs();
    if value > target_value {
        (value - adjustment).max(target_value)
    } else if value < target_value {
        (value + adjustment).min(target_value)
    } else {
        target_value // Value is already at target.
    }
}

/// To avoid 1-frame flashes of newly spawned creatures, only make them appear on screen
/// after they have been passed through adjust_transforms.
fn render_new_summons(mut summoned_creatures: Query<&mut Visibility, Added<Position>>) {
    for mut creature_visibility in summoned_creatures.iter_mut() {
        *creature_visibility = Visibility::Visible;
    }
}
