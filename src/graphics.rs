use bevy::prelude::*;

use crate::{creature::Player, map::Position};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.insert_resource(Msaa::Off);
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
        app.add_systems(Update, decay_offsets);
    }
}

#[derive(Resource)]
pub struct SpriteSheetAtlas {
    // Note the pub!
    pub handle: Handle<TextureAtlasLayout>,
}

impl FromWorld for SpriteSheetAtlas {
    fn from_world(world: &mut World) -> Self {
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 1, None, None);
        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();
        Self {
            handle: texture_atlases.add(layout),
        }
    }
}

#[derive(Component)]
pub struct VisualOffset {
    x: f32,
    y: f32,
    convergence: Convergence,
}

impl VisualOffset {
    pub fn from_tile_displacement(old_pos: Position, new_pos: Position) -> Self {
        VisualOffset {
            x: (old_pos.x - new_pos.x) as f32 * 64.,
            y: (old_pos.y - new_pos.y) as f32 * 64.,
            convergence: Convergence::Zero,
        }
    }

    pub fn from_tile_attack(old_pos: Position, new_pos: Position) -> Self {
        VisualOffset {
            x: (new_pos.x - old_pos.x) as f32 * 16.,
            y: (new_pos.y - old_pos.y) as f32 * 16.,
            convergence: Convergence::Zero,
        }
    }

    fn converge_to_zero(&mut self) {
        (self.x, self.y) = (
            converge_to_zero(self.x, quadratic_adjustment(self.x)),
            converge_to_zero(self.y, quadratic_adjustment(self.y)),
        );
    }

    fn converge_towards(&mut self, x: f32, y: f32) {
        (self.x, self.y) = (
            converge_towards(self.x, quadratic_adjustment(self.x), x),
            converge_towards(self.y, quadratic_adjustment(self.y), y),
        );
    }

    fn is_finished(&self) -> bool {
        match self.convergence {
            Convergence::Zero => self.x == 0. && self.y == 0.,
            Convergence::Target { x, y } => self.x == x && self.y == y,
        }
    }
}

fn converge_towards(value: f32, adjustment: f32, target: f32) -> f32 {
    if value > target {
        (value - adjustment).max(target)
    } else if value < target {
        (value + adjustment).min(target)
    } else {
        target
    }
}

fn converge_to_zero(value: f32, adjustment: f32) -> f32 {
    converge_towards(value, adjustment, 0.)
}

fn quadratic_adjustment(value: f32) -> f32 {
    10.0
}

pub enum Convergence {
    Zero,
    Target { x: f32, y: f32 },
}

fn decay_offsets(mut commands: Commands, mut offsets: Query<(Entity, &mut VisualOffset)>) {
    for (entity, mut offset) in offsets.iter_mut() {
        match offset.convergence {
            Convergence::Zero => offset.converge_to_zero(),
            Convergence::Target { x, y } => offset.converge_towards(x, y),
        }
        if offset.is_finished() {
            commands.entity(entity).remove::<VisualOffset>();
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
    player: Query<(&Position, Option<&VisualOffset>), With<Player>>,
    mut npcs: Query<(&Position, &mut Transform, Option<&VisualOffset>), Without<Player>>,
) {
    // There should only be one player on any given frame.
    let (player_pos, player_offset) = player.get_single().expect("0 or 2+ players");
    // Get the player's position.
    let (px, py) = (player_pos.x, player_pos.y);
    // For each Position and Transform of each non-player creature...
    for (npc_pos, mut npc_tran, npc_offset) in npcs.iter_mut() {
        // Measure their offset distance from the player's location.
        let (off_x, off_y) = (npc_pos.x - px, npc_pos.y - py);
        // Adjust their visual position to match this offset.
        let (mut vis_off_x, mut vis_off_y) = (0., 0.);
        if let Some(player_offset) = player_offset {
            vis_off_x -= player_offset.x;
            vis_off_y -= player_offset.y;
        }
        if let Some(npc_offset) = npc_offset {
            vis_off_x += npc_offset.x;
            vis_off_y += npc_offset.y;
        }
        (npc_tran.translation.x, npc_tran.translation.y) = (
            // Multiplied by the graphical size of a tile, which is 64x64.
            off_x as f32 * 4. * 16. + vis_off_x,
            off_y as f32 * 4. * 16. + vis_off_y,
        );
    }
}
