use bevy::prelude::*;

use crate::map::Position;

#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub sprite: Sprite,
}

/// Marker for the player
#[derive(Component)]
pub struct Player;
