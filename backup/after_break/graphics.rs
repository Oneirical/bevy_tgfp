use bevy::{ecs::system::SystemId, prelude::*};

use crate::{creature::Player, map::Position, OrdDir};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.init_resource::<WaypointSystem>();
        app.insert_resource(Msaa::Off);
        app.insert_resource(AnimationDelay { delay: 0. });
        app.add_systems(Startup, setup_camera);
        app.init_resource::<Events<PlaceMagicVfx>>();
    }
}

#[derive(Component)]
pub struct SlideAnimation {
    pub waypoints: Vec<Vec3>,
}

#[derive(Resource)]
pub struct AnimationDelay {
    pub delay: f32,
}

#[derive(Resource)]
pub struct SpriteSheetAtlas {
    pub handle: Handle<TextureAtlasLayout>,
}

impl FromWorld for SpriteSheetAtlas {
    fn from_world(world: &mut World) -> Self {
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 120, 2, None, None);
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

#[derive(Bundle)]
pub struct HealthIndicatorBundle {
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
    pub marker: HealthIndicator,
}

#[derive(Component)]
pub struct HealthIndicator;

#[derive(Bundle)]
pub struct AxiomCrateIconBundle {
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
    pub marker: AxiomCrateIcon,
}

#[derive(Component)]
pub struct AxiomCrateIcon;

#[derive(Event)]
pub struct PlaceMagicVfx {
    pub targets: Vec<Position>,
    pub sequence: EffectSequence,
    pub effect: EffectType,
    pub decay: f32,
    pub appear: f32,
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
    XCross,
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
        EffectType::XCross => 1,
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
                        EffectSequence::Simultaneous => {
                            Timer::from_seconds(event.appear, TimerMode::Once)
                        }
                        EffectSequence::Sequential { duration } => Timer::from_seconds(
                            (i as f32 / event.targets.len() as f32) * duration + event.appear,
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
) -> bool {
    magic_vfx.iter().len() == 0 && sliding.iter().len() == 0
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });
}

#[derive(Resource)]
pub struct WaypointSystem {
    pub add_waypoint: SystemId<Waypoint>,
}

impl FromWorld for WaypointSystem {
    fn from_world(world: &mut World) -> Self {
        WaypointSystem {
            add_waypoint: world.register_system(add_waypoint),
        }
    }
}

pub struct Waypoint {
    entity: Entity,
    destination: Vec3,
}

impl Waypoint {
    pub fn new(entity: Entity, destination: Vec3) -> Self {
        Waypoint {
            entity,
            destination,
        }
    }
}

pub fn add_waypoint(
    In(waypoint): In<Waypoint>,
    mut commands: Commands,
    mut animation: Query<&mut SlideAnimation>,
) {
    if let Ok(mut anim) = animation.get_mut(waypoint.entity) {
        anim.waypoints.push(waypoint.destination);
    } else {
        commands.entity(waypoint.entity).insert(SlideAnimation {
            waypoints: vec![waypoint.destination],
        });
    }
}

/// Each frame, adjust every entity's display location to match
/// their position on the grid, and make the camera follow the player.
pub fn adjust_transforms(
    mut creatures: Query<(
        Entity,
        &Position,
        &mut Transform,
        Option<&mut SlideAnimation>,
        Has<Player>,
    )>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Position>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, pos, mut trans, anim, is_player) in creatures.iter_mut() {
        // If this creature is affected by an animation...
        if let Some(mut anim) = anim {
            // The sprite approaches its destination.
            let current_translation = trans.translation;
            let target_translation = anim.waypoints.first().unwrap();
            // The creature is more than 0.5 pixels away from its destination - smooth animation.
            if ((target_translation.x - current_translation.x).abs()
                + (target_translation.y - current_translation.y).abs())
                > 0.5
            {
                trans.translation = trans
                    .translation
                    .lerp(*target_translation, 10. * time.delta_seconds());
            // Otherwise, the animation is over - clip the creature onto the grid.
            } else if anim.waypoints.len() > 1 {
                anim.waypoints.remove(0);
            } else {
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
