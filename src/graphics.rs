use bevy::prelude::*;

use crate::{creature::Player, Position};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.insert_resource(Scale { tile_size: 3. });
        app.insert_resource(Msaa::Off);
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
        app.add_systems(Update, decay_animation_offsets);
    }
}

/// The scale of tiles. Non-round floats will cause artifacts.
#[derive(Resource)]
pub struct Scale {
    pub tile_size: f32,
}

/// The pixels offsetting a creature from its real position.
#[derive(Component)]
pub struct AnimationOffset {
    pub x: f32,
    pub y: f32,
}

impl AnimationOffset {
    pub fn new() -> Self {
        AnimationOffset { x: 0., y: 0. }
    }

    pub fn decay(&mut self) {
        self.x = bring_closer_to_zero(self.x);
        self.y = bring_closer_to_zero(self.y);
    }

    pub fn initiate_offset(&mut self, x: i32, y: i32, tile_scale: f32) {
        self.x = x as f32 * 16. * tile_scale;
        self.y = y as f32 * 16. * tile_scale;
    }

    pub fn initiate_offset_f32(&mut self, x: f32, y: f32, tile_scale: f32) {
        self.x = x * 16. * tile_scale;
        self.y = y * 16. * tile_scale;
    }
}

fn bring_closer_to_zero(value: f32) -> f32 {
    let abs_value = value.abs();
    let adjustment = 0.1 * abs_value + 0.3;

    if value > 0.0 {
        (value - adjustment).max(0.0)
    } else if value < 0.0 {
        (value + adjustment).min(0.0)
    } else {
        0.0 // value is already 0
    }
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

fn decay_animation_offsets(mut creatures: Query<&mut AnimationOffset>) {
    for mut creature_anim in creatures.iter_mut() {
        creature_anim.decay();
    }
}

/// Adjust every entity's display location to be offset according to the player.
fn adjust_transforms(
    player: Query<(&Position, &AnimationOffset), With<Player>>,
    scale: Res<Scale>,
    mut npcs: Query<(&Position, &mut Transform, &AnimationOffset), Without<Player>>,
) {
    let (player_pos, player_anim) = player.get_single().expect("0 or 2+ players");
    let (px, py) = (player_pos.x, player_pos.y);
    for (npc_pos, mut npc_tran, npc_anim) in npcs.iter_mut() {
        let (off_x, off_y) = (npc_pos.x - px, npc_pos.y - py);
        (npc_tran.translation.x, npc_tran.translation.y) = (
            off_x as f32 * scale.tile_size * 16. + npc_anim.x - player_anim.x,
            off_y as f32 * scale.tile_size * 16. + npc_anim.y - player_anim.y,
        );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });
}
