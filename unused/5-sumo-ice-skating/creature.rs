use bevy::prelude::*;

use crate::{map::Position, OrdDir};

#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub momentum: OrdDir,
    pub species: Species,
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
}

/// Marker for the player
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Hunt;

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
        Species::Spawner => 75,
    }
}
