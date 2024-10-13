use std::{collections::VecDeque, time::Duration};

use bevy::prelude::*;

use crate::{creature::Player, map::Position};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.insert_resource(VisualEffectQueue {
            queue: VecDeque::new(),
            cooldown: Timer::from_seconds(0., TimerMode::Repeating),
        });
        app.insert_resource(SlideAnimation {
            elapsed: Timer::from_seconds(0.4, TimerMode::Once),
        });
        app.insert_resource(Msaa::Off);
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transform);
        app.add_systems(Update, render_new_summons);
        // app.add_systems(Update, visual_effect_pump);
    }
}

#[derive(Resource)]
pub struct VisualEffectQueue {
    pub queue: VecDeque<VisualEffect>,
    pub cooldown: Timer,
}

#[derive(Resource)]
pub struct SlideAnimation {
    pub elapsed: Timer,
}

#[derive(Clone, Copy)]
pub enum VisualEffect {
    SlidingCreature { entity: Entity, origin: Position },
    HideVisibility { entity: Entity },
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

fn adjust_transform(
    mut creatures: Query<(&Position, &mut Transform, Has<Player>)>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Position>)>,
    mut animation_timer: ResMut<SlideAnimation>,
    time: Res<Time>,
) {
    let fraction_before_tick = animation_timer.elapsed.fraction();
    animation_timer.elapsed.tick(time.delta());
    let fraction_ticked = animation_timer.elapsed.fraction() - fraction_before_tick;
    for (pos, mut trans, is_player) in creatures.iter_mut() {
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
        target_value // value is already at target
    }
}

/// To avoid 1-frame flashes of newly spawned creatures, only make them appear on screen
/// after they have been passed through adjust_transforms.
fn render_new_summons(mut summoned_creatures: Query<&mut Visibility, Added<Position>>) {
    for mut creature_visibility in summoned_creatures.iter_mut() {
        *creature_visibility = Visibility::Visible;
    }
}

// fn visual_effect_pump(
//     time: Res<Time>,
//     mut commands: Commands,
//     mut pump: ResMut<VisualEffectQueue>,
// ) {
//     pump.cooldown.tick(time.delta());
//     if !pump.cooldown.finished() {
//         return;
//     }
//     pump.cooldown.reset();
//     let effect = pump.queue.pop_front();
//     if let Some(effect) = effect {
//         match effect {
//             VisualEffect::SlidingCreature { entity, origin } => {
//                 commands.entity(entity).insert(SlideAnimation {
//                     elapsed: Timer::from_seconds(0.3, TimerMode::Once),
//                     origin,
//                 });
//                 pump.cooldown.set_duration(Duration::from_millis(50));
//                 // FIXME The code this generated is messy, and the queue doesn't
//                 // get emptied if you spam the buttons.
//                 commands.entity(entity).remove::<AwaitingAnimation>();
//             }
//             VisualEffect::HideVisibility { entity } => {
//                 commands.entity(entity).insert(Visibility::Hidden);
//                 pump.cooldown.set_duration(Duration::from_millis(0));
//             }
//         }
//     }
// }
