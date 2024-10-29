use bevy::prelude::*;

use crate::{map::Position, OrdDir};

#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub momentum: OrdDir,
    pub species: Species,
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
    pub health: HealthBar,
}

/// Marker for the player
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Hunt;

// This creature has no collisions with other entities.
#[derive(Component)]
pub struct Intangible;

#[derive(Component)]
pub struct HealthBar {
    pub deck: Vec<HealthPoint>,
    pub repressed: Vec<HealthPoint>,
}

impl HealthBar {
    /// Create a new health container with a certain amount of points in its deck.
    pub fn new(max_hp: i32) -> Self {
        let mut deck = Vec::new();
        for _i in 0..max_hp {
            deck.push(HealthPoint);
        }
        Self {
            deck,
            repressed: Vec::new(),
        }
    }

    /// Deal damage, shifting HealthPoints from the deck to the repressed discard.
    /// The bool return is true if a creature was brought to 0 HP.
    pub fn repress(&mut self, damage: i32) -> bool {
        for _i in 0..damage {
            let lost = self.deck.pop();
            if let Some(lost) = lost {
                self.repressed.push(lost);
            } else {
                return true;
            }
            if self.deck.is_empty() {
                return true;
            }
        }
        false
    }
}

pub struct HealthPoint;

#[derive(Debug, Component, Clone, Copy)]
pub enum Species {
    Player,
    Wall,
    Hunter,
    Spawner,
    Trap,
    Crate,
}

/// Get the appropriate texture from the spritesheet depending on the species type.
pub fn get_species_sprite(species: &Species) -> usize {
    match species {
        Species::Player => 0,
        Species::Wall => 3,
        Species::Hunter => 4,
        Species::Trap => 12,
        Species::Spawner => 75,
        Species::Crate => 19,
    }
}
