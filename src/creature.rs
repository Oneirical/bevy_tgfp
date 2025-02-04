use bevy::{prelude::*, utils::HashMap};

use uuid::Uuid;

use crate::{
    map::Position,
    spells::{Axiom, CounterCondition, Spell},
    OrdDir,
};

#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub momentum: OrdDir,
    pub sprite: Sprite,
    pub species: Species,
    pub health: Health,
    pub effects: StatusEffectsList,
    pub spellbook: Spellbook,
    pub soul: Soul,
    pub flags: CreatureFlags,
}

#[derive(Component, Clone)]
pub struct CreatureFlags {
    pub effects_flags: Entity,
    pub species_flags: Entity,
}

#[derive(Component, Clone)]
pub struct FlagEntity {
    pub parent_creature: Entity,
}

#[derive(Resource)]
pub struct SpellLibrary {
    pub library: Vec<Spell>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Soul {
    Saintly,
    Ordered,
    Artistic,
    Unhinged,
    Feral,
    Vile,
    Serene,
    // Its sole purpose is to display a tutorial tooltip in the Caste UI menu.
    Empty,
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
        Soul::Serene => 181,
        Soul::Empty => 166,
    }
}

#[derive(Component, Clone)]
pub struct Spellbook {
    pub spells: HashMap<Soul, Spell>,
}

impl Spellbook {
    pub fn new(slots: [Option<Vec<Axiom>>; 6]) -> Self {
        let mut book = HashMap::new();
        let souls = [
            Soul::Saintly,
            Soul::Ordered,
            Soul::Artistic,
            Soul::Unhinged,
            Soul::Feral,
            Soul::Vile,
        ];
        for (i, soul) in souls.iter().enumerate() {
            if let Some(spell) = &slots[i] {
                book.insert(
                    *soul,
                    Spell {
                        axioms: spell.to_vec(),
                        caste: *soul,
                        icon: get_soul_sprite(soul),
                        id: Uuid::new_v4(),
                    },
                );
            }
        }
        Spellbook { spells: book }
    }
    pub fn empty() -> Self {
        Spellbook {
            spells: HashMap::new(),
        }
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
    // The creature acts as if it was summoned by whoever cursed it.
    DimensionBond,
}

#[derive(Debug)]
pub struct PotencyAndStacks {
    pub potency: usize,
    pub stacks: EffectDuration,
}

impl PotencyAndStacks {
    pub fn is_active(&self) -> bool {
        self.potency > 0
            && match self.stacks {
                EffectDuration::Finite { stacks } => stacks > 0,
                EffectDuration::Infinite => true,
            }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectDuration {
    Finite { stacks: usize },
    Infinite,
}

impl EffectDuration {
    pub fn add(&self, amount: Self) -> Self {
        match self {
            EffectDuration::Finite { stacks } => match amount {
                EffectDuration::Infinite => EffectDuration::Infinite,
                EffectDuration::Finite { stacks: add_stacks } => EffectDuration::Finite {
                    stacks: stacks + add_stacks,
                },
            },
            EffectDuration::Infinite => EffectDuration::Infinite,
        }
    }
}

#[derive(Component, Debug)]
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

#[derive(Component)]
pub struct Sleeping {
    pub cage_idx: usize,
}

#[derive(Component)]
pub struct Awake;

// Performs random actions on its turn.
#[derive(Component)]
pub struct Random;

// Vulnerable to Abjuration.
#[derive(Component)]
pub struct Summoned {
    pub summoner: Entity,
}

// Will start dragging along creatures of this species.
#[derive(Component)]
pub struct Magnetic {
    pub species: Species,
    pub conductor: Option<Entity>,
}

#[derive(Component)]
pub struct Magnetized {
    pub train: Vec<Entity>,
    pub species: Species,
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
pub struct Immobile;

#[derive(Component)]
pub struct NoDropSoul;

#[derive(Component)]
pub struct Intangible;

#[derive(Component)]
pub struct CraftingSlot;

#[derive(Component)]
pub struct DesignatedForRemoval;

// Breaks when stepped on.
#[derive(Component)]
pub struct Fragile;

#[derive(Component)]
pub struct Health {
    pub hp: usize,
    pub max_hp: usize,
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash)]
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
    Oracle,
    Abazon,
    EpsilonHead,
    EpsilonTail,
    CageBorder,
    CageSlot,
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
        Species::Oracle => 40,
        Species::Abazon => 28,
        Species::EpsilonHead => 67,
        Species::EpsilonTail => 68,
        Species::CageBorder => 108,
        Species::CageSlot => 167,
    }
}

pub fn get_species_spellbook(species: &Species) -> Spellbook {
    match species {
        Species::Oracle => Spellbook::new([
            None,
            None,
            None,
            Some(vec![
                Axiom::WhenMoved,
                Axiom::IncrementCounter {
                    amount: 1,
                    count: 0,
                },
                Axiom::TerminateIfCounter {
                    condition: CounterCondition::NotModuloOf { modulo: 5 },
                    threshold: 0,
                },
                Axiom::Ego,
                Axiom::StatusEffect {
                    effect: StatusEffect::Stab,
                    potency: 0,
                    stacks: EffectDuration::Infinite,
                },
                Axiom::UpgradeStatusEffect {
                    effect: StatusEffect::Stab,
                    potency: 1,
                    stacks: EffectDuration::Infinite,
                },
            ]),
            None,
            None,
        ]),
        Species::EpsilonHead => Spellbook::new([
            None,
            None,
            None,
            Some(vec![
                Axiom::WhenMoved,
                Axiom::IncrementCounter {
                    amount: 1,
                    count: 0,
                },
                Axiom::TerminateIfCounter {
                    condition: CounterCondition::NotModuloOf { modulo: 5 },
                    threshold: 0,
                },
                Axiom::Ego,
                Axiom::Dash { max_distance: 5 },
            ]),
            None,
            None,
        ]),
        Species::Second => Spellbook::new([
            None,
            None,
            None,
            None,
            None,
            Some(vec![Axiom::Plus, Axiom::DevourWall]),
        ]),
        Species::Hunter => Spellbook::new([
            Some(vec![
                Axiom::WhenDealingDamage,
                Axiom::Ego,
                Axiom::HealOrHarm { amount: 1 },
            ]),
            None,
            None,
            None,
            None,
            None,
        ]),
        Species::Tinker => Spellbook::new([
            None,
            None,
            Some(vec![
                Axiom::WhenMoved,
                Axiom::IncrementCounter {
                    amount: 1,
                    count: 0,
                },
                Axiom::TerminateIfCounter {
                    condition: CounterCondition::NotModuloOf { modulo: 5 },
                    threshold: 0,
                },
                Axiom::Plus,
                Axiom::FilterBySpecies {
                    species: Species::WeakWall,
                },
                Axiom::Transform {
                    species: Species::Abazon,
                },
                Axiom::StatusEffect {
                    effect: StatusEffect::DimensionBond,
                    potency: 1,
                    stacks: EffectDuration::Infinite,
                },
                Axiom::Terminate,
                Axiom::WhenRemoved,
                Axiom::Ego,
                Axiom::Abjuration,
            ]),
            None,
            None,
            None,
        ]),
        Species::Player => Spellbook::new([
            Some(vec![
                Axiom::Ego,
                Axiom::Plus,
                Axiom::HealOrHarm { amount: 2 },
            ]),
            Some(vec![
                Axiom::Ego,
                Axiom::StatusEffect {
                    effect: StatusEffect::Invincible,
                    potency: 1,
                    stacks: EffectDuration::Finite { stacks: 2 },
                },
            ]),
            Some(vec![
                Axiom::Ego,
                Axiom::PlaceStepTrap,
                Axiom::PiercingBeams,
                Axiom::PlusBeam,
                Axiom::Ego,
                Axiom::HealOrHarm { amount: -2 },
            ]),
            Some(vec![
                Axiom::PiercingBeams,
                Axiom::XBeam,
                Axiom::HealOrHarm { amount: -2 },
            ]),
            Some(vec![
                Axiom::Ego,
                Axiom::Trace,
                Axiom::Dash { max_distance: 5 },
                Axiom::Spread,
                Axiom::UntargetCaster,
                Axiom::HealOrHarm { amount: -1 },
                Axiom::PurgeTargets,
                Axiom::Touch,
                Axiom::StatusEffect {
                    effect: StatusEffect::Dizzy,
                    potency: 1,
                    stacks: EffectDuration::Finite { stacks: 2 },
                },
                Axiom::Dash { max_distance: 1 },
            ]),
            Some(vec![
                Axiom::Ego,
                Axiom::StatusEffect {
                    effect: StatusEffect::Stab,
                    potency: 5,
                    stacks: EffectDuration::Infinite,
                },
            ]),
        ]),
        _ => Spellbook::empty(),
    }
}

pub fn is_naturally_intangible(species: &Species) -> bool {
    match species {
        Species::Trap => true,
        _ => false,
    }
}
