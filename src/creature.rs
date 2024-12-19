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

// The graphical representation of Health: a health bar.
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

// Performs random actions on its turn.
#[derive(Component)]
pub struct Random;

#[derive(Component)]
pub struct Door;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Spellproof;

#[derive(Component)]
pub struct Attackproof;

#[derive(Component)]
pub struct Intangible;

#[derive(Component)]
pub struct Health {
    pub hp: usize,
    pub max_hp: usize,
}

#[derive(Debug, Component, Clone, Copy)]
pub enum Species {
    Player,
    Wall,
    WeakWall,
    Hunter,
    Apiarist,
    Shrike,
    Tinker,
    Second,
    Spawner,
    Airlock,
}

/// Get the appropriate texture from the spritesheet depending on the species type.
pub fn get_species_sprite(species: &Species) -> usize {
    match species {
        Species::Player => 0,
        Species::Wall => 3,
        Species::WeakWall => 3,
        Species::Hunter => 4,
        Species::Spawner => 5,
        Species::Airlock => 17,
        Species::Shrike => 5,
        Species::Apiarist => 6,
        Species::Second => 7,
        Species::Tinker => 8,
    }
}
