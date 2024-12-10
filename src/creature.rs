use bevy::prelude::*;

use crate::{map::Position, OrdDir};

#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub momentum: OrdDir,
    pub sprite: Sprite,
    pub species: Species,
    pub health: Health,
}

#[derive(Bundle)]
pub struct HealthIndicator {
    pub sprite: Sprite,
    pub visibility: Visibility,
    pub transform: Transform,
}

/// Marker for the player
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Hunt;

#[derive(Component)]
pub struct Health {
    pub hp: usize,
    pub max_hp: usize,
}

#[derive(Debug, Component, Clone, Copy)]
pub enum Species {
    Player,
    Wall,
    Hunter,
    Spawner,
}

/// Get the appropriate texture from the spritesheet depending on the species type.
pub fn get_species_sprite(species: &Species) -> usize {
    match species {
        Species::Player => 0,
        Species::Wall => 3,
        Species::Hunter => 4,
        Species::Spawner => 5,
    }
}
