use std::f32::consts::PI;

use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{creature::Player, map::Position, TILE_SIZE};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.add_event::<PlaceMagicVfx>();
        app.add_systems(Startup, setup_camera);
        app.insert_resource(Screenshake { intensity: 0 });
    }
}

#[derive(Resource)]
pub struct Screenshake {
    pub intensity: usize,
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

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0., 0., 0.), Msaa::Off));
}

#[derive(Component)]
pub struct SlideAnimation;

/// Each frame, adjust every entity's display location to match
/// their position on the grid, and make the camera follow the player.
pub fn adjust_transforms(
    mut creatures: Query<(
        Entity,
        &Position,
        &mut Transform,
        Has<SlideAnimation>,
        Has<Player>,
    )>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Position>)>,
    time: Res<Time>,
    mut commands: Commands,
    mut screenshake: ResMut<Screenshake>,
) {
    for (entity, pos, mut trans, is_animated, is_player) in creatures.iter_mut() {
        // If this creature is affected by an animation...
        if is_animated {
            // The sprite approaches its destination.
            let current_translation = trans.translation;
            let target_translation =
                Vec3::new(pos.x as f32 * TILE_SIZE, pos.y as f32 * TILE_SIZE, 0.);
            // The creature is more than 0.5 pixels away from its destination - smooth animation.
            if ((target_translation.x - current_translation.x).abs()
                + (target_translation.y - current_translation.y).abs())
                > 0.05
            {
                trans.translation = trans
                    .translation
                    .lerp(target_translation, 10. * time.delta_secs());
            // Otherwise, the animation is over - clip the creature onto the grid.
            } else {
                commands.entity(entity).remove::<SlideAnimation>();
            }
        } else {
            // For creatures with no animation.
            // Multiplied by the graphical size of a tile, which is TILE_SIZE.
            trans.translation.x = pos.x as f32 * TILE_SIZE;
            trans.translation.y = pos.y as f32 * TILE_SIZE;
        }
        if is_player {
            screenshake.intensity = screenshake.intensity.saturating_sub(1);
            let mut rng = thread_rng();
            let shake_angle = rng.gen::<f32>() * PI * 2.;
            let (shake_x, shake_y) = (
                shake_angle.cos() * screenshake.intensity as f32,
                shake_angle.sin() * screenshake.intensity as f32,
            );
            // The camera follows the player.
            let mut camera_trans = camera.get_single_mut().unwrap();
            (camera_trans.translation.x, camera_trans.translation.y) =
                (trans.translation.x + shake_x, trans.translation.y + shake_y);
        }
    }
}

// graphics.rs
#[derive(Bundle)]
pub struct MagicEffect {
    /// The tile position of this visual effect.
    pub position: Position,
    /// The sprite representing this visual effect.
    pub sprite: Sprite,
    pub visibility: Visibility,
    /// The timers tracking when the effect appears, and how
    /// long it takes to decay.
    pub vfx: MagicVfx,
}

#[derive(Event)]
/// An event to place visual effects on the game board.
pub struct PlaceMagicVfx {
    /// All tile positions on which a visual effect will appear.
    pub targets: Vec<Position>,
    /// Whether the effect appear one by one, or all at the same time.
    pub sequence: EffectSequence,
    /// The effect sprite.
    pub effect: EffectType,
    /// How long these effects take to decay.
    pub decay: f32,
    /// How long these effects take to appear.
    pub appear: f32,
}

#[derive(Clone, Copy)]
pub enum EffectSequence {
    /// All effects appear at the same time.
    Simultaneous,
    /// Effects appear one at a time, in a queue.
    /// `duration` is how long it takes to move from one effect to the next.
    Sequential { duration: f32 },
}

#[derive(Clone, Copy)]
pub enum EffectType {
    HorizontalBeam,
    VerticalBeam,
    RedBlast,
    GreenBlast,
    XCross,
    Airlock,
}

#[derive(Component)]
pub struct MagicVfx {
    /// How long this effect takes to decay.
    pub appear: Timer,
    /// How long this effect takes to appear.
    pub decay: Timer,
}

/// Get the appropriate texture from the spritesheet depending on the effect type.
pub fn get_effect_sprite(effect: &EffectType) -> usize {
    match effect {
        EffectType::HorizontalBeam => 15,
        EffectType::VerticalBeam => 16,
        EffectType::RedBlast => 14,
        EffectType::GreenBlast => 13,
        EffectType::XCross => 1,
        EffectType::Airlock => 17,
    }
}

// graphics.rs
pub fn place_magic_effects(
    mut events: EventReader<PlaceMagicVfx>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    for event in events.read() {
        for (i, target) in event.targets.iter().enumerate() {
            // Place effects on all positions from the event.
            commands.spawn(MagicEffect {
                position: *target,
                sprite: Sprite {
                    image: asset_server.load("spritesheet.png"),
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_layout.handle.clone(),
                        index: get_effect_sprite(&event.effect),
                    }),
                    ..default()
                },
                visibility: Visibility::Hidden,
                vfx: MagicVfx {
                    appear: match event.sequence {
                        // If simultaneous, everything appears at the same time.
                        EffectSequence::Simultaneous => {
                            Timer::from_seconds(event.appear, TimerMode::Once)
                        }
                        // Otherwise, effects gradually get increased appear timers depending on
                        // how far back they are in their queue.
                        EffectSequence::Sequential { duration } => {
                            Timer::from_seconds(i as f32 * duration + event.appear, TimerMode::Once)
                        }
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
        // Effects that have completed their appear timer and are now visible, decay.
        if matches!(*vfx_vis, Visibility::Inherited) {
            vfx_timers.decay.tick(time.delta());
            // Their alpha (transparency) slowly loses opacity as they decay.
            vfx_sprite
                .color
                .set_alpha(vfx_timers.decay.fraction_remaining());
            if vfx_timers.decay.finished() {
                commands.entity(vfx_entity).despawn();
            }
        // Effects that have not appeared yet progress towards appearing for the first time.
        } else {
            vfx_timers.appear.tick(time.delta());
            if vfx_timers.appear.finished() {
                *vfx_vis = Visibility::Inherited;
            }
        }
    }
}
