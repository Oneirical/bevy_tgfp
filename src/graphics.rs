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
        app.insert_resource(Msaa::Off);
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
        app.add_systems(Update, render_new_summons);
        app.add_systems(Update, visual_effect_pump);
    }
}

#[derive(Resource)]
pub struct VisualEffectQueue {
    pub queue: VecDeque<VisualEffect>,
    pub cooldown: Timer,
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

/// Each frame, adjust every entity's display location to be offset
/// according to the player's location.
fn adjust_transforms(
    mut player: Query<
        (&Position, Option<&mut SlideAnimation>),
        (With<Player>, Without<AwaitingAnimation>),
    >,
    mut npcs: Query<
        (
            &Position,
            &mut Transform,
            Option<&mut SlideAnimation>,
            Option<&AwaitingAnimation>,
        ),
        Without<Player>,
    >,
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
        for (npc_pos, mut npc_tran, npc_anim, future_animation) in npcs.iter_mut() {
            let (npc_offset_x, npc_offset_y) = if let Some(mut npc_anim) = npc_anim {
                npc_anim.get_animation_offsets(time.delta(), *npc_pos)
            } else {
                (npc_pos.x as f32, npc_pos.y as f32)
            };
            // Measure their offset distance from the player's location.
            let (off_x, off_y) = if let Some(future_animation) = future_animation {
                if let VisualEffect::SlidingCreature { entity: _, origin } =
                    future_animation.future_animation
                {
                    (
                        origin.x as f32 - player_offset_x,
                        origin.y as f32 - player_offset_y,
                    )
                } else {
                    panic!();
                }
            } else {
                (
                    npc_offset_x - player_offset_x,
                    npc_offset_y - player_offset_y,
                )
            };
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

fn visual_effect_pump(
    time: Res<Time>,
    mut commands: Commands,
    mut pump: ResMut<VisualEffectQueue>,
) {
    pump.cooldown.tick(time.delta());
    if !pump.cooldown.finished() {
        return;
    }
    pump.cooldown.reset();
    let effect = pump.queue.pop_front();
    if let Some(effect) = effect {
        match effect {
            VisualEffect::SlidingCreature { entity, origin } => {
                commands.entity(entity).insert(SlideAnimation {
                    elapsed: Timer::from_seconds(0.3, TimerMode::Once),
                    origin,
                });
                pump.cooldown.set_duration(Duration::from_millis(50));
                // FIXME The code this generated is messy, and the queue doesn't
                // get emptied if you spam the buttons.
                commands.entity(entity).remove::<AwaitingAnimation>();
            }
            VisualEffect::HideVisibility { entity } => {
                commands.entity(entity).insert(Visibility::Hidden);
                pump.cooldown.set_duration(Duration::from_millis(0));
            }
        }
    }
}

#[derive(Component)]
pub struct AwaitingAnimation {
    pub future_animation: VisualEffect,
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
