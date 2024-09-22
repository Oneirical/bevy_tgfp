use bevy::prelude::*;

use crate::map::Position;

#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
}

/// Marker for the player
#[derive(Component)]
pub struct Player;
