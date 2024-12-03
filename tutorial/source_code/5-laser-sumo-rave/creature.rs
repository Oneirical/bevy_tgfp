use bevy::prelude::*;

use crate::{map::Position, OrdDir};

#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub momentum: OrdDir,
    pub sprite: Sprite,
}

/// Marker for the player
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Hunt;
