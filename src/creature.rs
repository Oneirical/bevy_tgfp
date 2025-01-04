use bevy::{prelude::*, utils::HashMap};

use crate::{map::Position, spells::Spell, OrdDir};

#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub momentum: OrdDir,
    pub sprite: Sprite,
    pub species: Species,
    pub health: Health,
    pub effects: StatusEffectsList,
    pub spell: Spell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Soul {
    Saintly,
    Ordered,
    Artistic,
    Unhinged,
    Feral,
    Vile,
}

/// Get the appropriate texture from the spritesheet depending on the soul type.
pub fn get_soul_sprite(soul: &Soul) -> usize {
    match soul {
        Soul::Saintly => 160,
        Soul::Ordered => 161,
        Soul::Artistic => 162,
        Soul::Unhinged => 163,
        Soul::Feral => 164,
        Soul::Vile => 165,
    }
}

// The graphical representation of Health: a health bar.
#[derive(Bundle)]
pub struct HealthIndicator {
    pub sprite: Sprite,
    pub visibility: Visibility,
    pub transform: Transform,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatusEffect {
    // Cannot take damage.
    Invincible,
    // Bonus melee damage, dispels on melee attack.
    Stab,
    // Stun, no action.
    Dizzy,
}

pub struct PotencyAndStacks {
    pub potency: usize,
    pub stacks: usize,
}

#[derive(Component)]
pub struct StatusEffectsList {
    pub effects: HashMap<StatusEffect, PotencyAndStacks>,
}

#[derive(Component)]
pub enum Speed {
    Slow { wait_turns: usize },
    Fast { actions_per_turn: usize },
}

/// Marker for the player
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Hunt;

#[derive(Component)]
pub struct Stab {
    pub bonus_damage: isize,
}

#[derive(Component)]
pub struct Invincible;

#[derive(Component)]
pub struct Dizzy;

// Performs random actions on its turn.
#[derive(Component)]
pub struct Random;

#[derive(Component)]
pub struct Summoned {
    pub summoner: Entity,
}

#[derive(Component)]
pub struct Door;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Spellproof;

#[derive(Component)]
pub struct Meleeproof;

#[derive(Component)]
pub struct Intangible;

#[derive(Component)]
pub struct WhenSteppedOn;

// Breaks when stepped on.
#[derive(Component)]
pub struct Fragile;

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
    Trap,
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
        Species::Trap => 12,
    }
}

pub fn is_naturally_intangible(species: &Species) -> bool {
    match species {
        Species::Trap => true,
        _ => false,
    }
}
