use bevy::prelude::*;

use crate::{creature::Player, map::Position, OrdDir};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.insert_resource(Msaa::Off);
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
        app.add_event::<PlaceMagicVfx>();
    }
}

#[derive(Component)]
pub struct SlideAnimation {
    pub elapsed: Timer,
}

#[derive(Component)]
pub struct AttackAnimation {
    pub elapsed: Timer,
    pub direction: OrdDir,
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

#[derive(Bundle)]
pub struct MagicEffect {
    pub position: Position,
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
    pub vfx: MagicVfx,
}

#[derive(Event)]
pub struct PlaceMagicVfx {
    pub targets: Vec<Position>,
    pub sequence: EffectSequence,
    pub effect: EffectType,
    pub decay: f32,
}

#[derive(Clone, Copy)]
pub enum EffectSequence {
    Simultaneous,
    Sequential { duration: f32 },
}

#[derive(Clone, Copy)]
pub enum EffectType {
    HorizontalBeam,
    VerticalBeam,
    RedBlast,
    GreenBlast,
}

#[derive(Component)]
pub struct MagicVfx {
    appear: Timer,
    decay: Timer,
}

/// Get the appropriate texture from the spritesheet depending on the effect type.
pub fn get_effect_sprite(effect: &EffectType) -> usize {
    match effect {
        EffectType::HorizontalBeam => 15,
        EffectType::VerticalBeam => 16,
        EffectType::RedBlast => 14,
        EffectType::GreenBlast => 13,
    }
}

pub fn place_magic_effects(
    mut events: EventReader<PlaceMagicVfx>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    for event in events.read() {
        for (i, target) in event.targets.iter().enumerate() {
            commands.spawn(MagicEffect {
                position: *target,
                sprite: SpriteBundle {
                    texture: asset_server.load("spritesheet.png"),
                    transform: Transform::from_scale(Vec3::new(4., 4., 0.)),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                atlas: TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index: get_effect_sprite(&event.effect),
                },
                vfx: MagicVfx {
                    appear: match event.sequence {
                        EffectSequence::Simultaneous => Timer::from_seconds(0., TimerMode::Once),
                        EffectSequence::Sequential { duration } => Timer::from_seconds(
                            (i as f32 / event.targets.len() as f32) * duration,
                            TimerMode::Once,
                        ),
                    },
                    decay: Timer::from_seconds(event.decay, TimerMode::Once),
                },
            });
        }
    }
}

pub fn decay_magic_effects(
    mut commands: Commands,
    mut magic_vfx: Query<(Entity, &mut Visibility, &mut MagicVfx, &mut Sprite)>,
    time: Res<Time>,
) {
    for (vfx_entity, mut vfx_vis, mut vfx_timers, mut vfx_sprite) in magic_vfx.iter_mut() {
        if matches!(*vfx_vis, Visibility::Visible) {
            vfx_timers.decay.tick(time.delta());
            vfx_sprite
                .color
                .set_alpha(vfx_timers.decay.fraction_remaining());
            if vfx_timers.decay.finished() {
                commands.entity(vfx_entity).despawn();
            }
        } else {
            vfx_timers.appear.tick(time.delta());
            if vfx_timers.appear.finished() {
                *vfx_vis = Visibility::Visible;
            }
        }
    }
}

pub fn all_animations_complete(
    magic_vfx: Query<&MagicVfx>,
    sliding: Query<&SlideAnimation>,
    attacking: Query<&AttackAnimation>,
) -> bool {
    magic_vfx.iter().len() == 0 && sliding.iter().len() == 0 && attacking.iter().len() == 0
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });
}

/// Each frame, adjust every entity's display location to match
/// their position on the grid, and make the camera follow the player.
fn adjust_transforms(
    mut creatures: Query<(
        Entity,
        &Position,
        &mut Transform,
        Option<&mut SlideAnimation>,
        Option<&mut AttackAnimation>,
        Has<Player>,
    )>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Position>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, pos, mut trans, anim, attack, is_player) in creatures.iter_mut() {
        // If this creature is affected by an animation...
        if let Some(mut attack) = attack {
            let (strike_translation_x, strike_translation_y) = (
                (pos.x as f32 + attack.direction.as_offset().0 as f32 / 4.) * 64.,
                (pos.y as f32 + attack.direction.as_offset().1 as f32 / 4.) * 64.,
            );
            if attack.elapsed.fraction_remaining() == 1. {
                trans.translation.x = strike_translation_x;
                trans.translation.y = strike_translation_y;
            }
            let fraction_before_tick = attack.elapsed.fraction();
            attack.elapsed.tick(time.delta());
            // Calculate what % of the animation has elapsed during this tick.
            let fraction_advanced_this_frame = attack.elapsed.fraction() - fraction_before_tick;
            // The distance between where a creature CURRENTLY is,
            // and the destination of a creature's movement.
            // Multiplied by the graphical size of a tile, which is 64x64.
            let (ori_dx, ori_dy) = (
                strike_translation_x - pos.x as f32 * 64.,
                strike_translation_y - pos.y as f32 * 64.,
            );
            // The sprite approaches its destination.
            trans.translation.x = bring_closer_to_target_value(
                trans.translation.x,
                ori_dx * fraction_advanced_this_frame,
                pos.x as f32 * 64.,
            );
            trans.translation.y = bring_closer_to_target_value(
                trans.translation.y,
                ori_dy * fraction_advanced_this_frame,
                pos.y as f32 * 64.,
            );
            if attack.elapsed.finished() {
                commands.entity(entity).remove::<AttackAnimation>();
            }
        } else if let Some(mut animation_timer) = anim {
            let fraction_before_tick = animation_timer.elapsed.fraction();
            animation_timer.elapsed.tick(time.delta());
            // Calculate what % of the animation has elapsed during this tick.
            let fraction_advanced_this_frame =
                animation_timer.elapsed.fraction() - fraction_before_tick;
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
                ori_dx * fraction_advanced_this_frame,
                pos.x as f32 * 64.,
            );
            trans.translation.y = bring_closer_to_target_value(
                trans.translation.y,
                ori_dy * fraction_advanced_this_frame,
                pos.y as f32 * 64.,
            );
            if animation_timer.elapsed.finished() {
                commands.entity(entity).remove::<SlideAnimation>();
            }
        } else {
            // For creatures with no animation.
            // Multiplied by the graphical size of a tile, which is 64x64.
            trans.translation.x = pos.x as f32 * 64.;
            trans.translation.y = pos.y as f32 * 64.;
        }
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
