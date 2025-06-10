use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

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

#[derive(Component, Clone, Copy)]
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
                        description: String::from(match soul {
                            Soul::Saintly => "You, and all adjacent creatures, heal for 2 HP.",
                            Soul::Ordered => "You cannot take damage next turn. Instantaneous.",
                            Soul::Artistic => "Places a trap at your feet. The next creature to step on it will cause it to fire 2 damage beams in all 4 cardinal directions.",
                            Soul::Unhinged => "Fires 4 beams in all diagonal directions, dealing 2 damage.",
                            Soul::Feral => "Dashes 5 tiles in the direction you are facing, attacking all creatures adjacent to your path with 1 damage. Creatures struck at the end are knocked backwards.",
                            Soul::Vile => "The next time you strike with a melee attack, deal 6 damage.",
                            _ => panic!()
                        })
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
    // The creature gains additional turns.
    Haste,
    // The creature attacks its allies.
    Charm,
    // The creature starts being able to drag walls behind it.
    Magnetize,
    // The creature is controlled by the player.
    Possessed,
    // The creature will return to its original Species when this expires.
    ReturnOriginalForm,
}

#[derive(Debug)]
pub struct PotencyAndStacks {
    pub potency: usize,
    pub stacks: EffectDuration,
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

#[derive(Component, Default)]
pub struct Targeting(pub HashSet<Species>);

#[derive(Component)]
pub struct Stab {
    pub bonus_damage: isize,
}

#[derive(Component)]
pub struct Invincible;

#[derive(Component)]
pub struct ReturnOriginalForm {
    pub original_form: Species,
}

#[derive(Component)]
pub struct Dizzy;

#[derive(Component)]
pub struct Sleeping;

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

// Controlled by another creature.
#[derive(Component)]
pub struct Possessed {
    pub original: Entity,
}

// Currently away, controlling another creature.
#[derive(Component)]
pub struct Possessing;

#[derive(Component)]
pub struct Charm;

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
pub struct Meleeproof;

#[derive(Component)]
pub struct Immobile;

#[derive(Component)]
pub struct NoDropSoul;

#[derive(Component)]
pub struct DoesNotLockInput;

/// This creature ignores Immobile when it teleports
/// others.
#[derive(Component)]
pub struct RealityBreak(pub usize);

/// This creature ignores others' RealityBreak.
#[derive(Component)]
pub struct RealityShield(pub usize);

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
    Scion,
    Apiarist,
    Shrike,
    Tinker,
    Second,
    Airlock,
    Trap,
    Oracle,
    Abazon,
    EpsilonHead,
    EpsilonTail,
    CageBorder,
    CageSlot,
    AxiomaticSeal,
    ConveyorBelt,
    Grinder,
    Hechaton,
    Grappler,
    Exploder,
}

/// Get the appropriate texture from the spritesheet depending on the species type.
pub fn get_species_sprite(species: &Species) -> usize {
    match species {
        Species::Player => 0,
        Species::Wall => 3,
        Species::WeakWall => 3,
        Species::Scion => 4,
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
        Species::AxiomaticSeal => 25,
        Species::ConveyorBelt => 23,
        Species::Grinder => 20,
        Species::Hechaton => 61,
        Species::Grappler => 62,
        Species::Exploder => 63,
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
        Species::Exploder => Spellbook::new([
            None,
            None,
            None,
            Some(vec![
                Axiom::WhenRemoved,
                Axiom::XBeam,
                Axiom::Spread,
                Axiom::HealOrHarm { amount: -1 },
            ]),
            None,
            None,
        ]),
        Species::Hechaton => Spellbook::new([
            None,
            None,
            Some(vec![
                Axiom::MomentumBeam,
                Axiom::StatusEffect {
                    effect: StatusEffect::ReturnOriginalForm,
                    potency: 0,
                    stacks: EffectDuration::Finite { stacks: 5 },
                },
                Axiom::Transform {
                    species: Species::Abazon,
                },
            ]),
            None,
            None,
            None,
        ]),
        Species::Grappler => Spellbook::new([
            None,
            None,
            None,
            None,
            Some(vec![
                Axiom::MomentumBeam,
                Axiom::ToggleUntarget,
                Axiom::Touch,
                Axiom::Dash { max_distance: -5 },
            ]),
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
        Species::Scion => Spellbook::new([
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
        Species::ConveyorBelt => Spellbook::new([
            None,
            Some(vec![
                Axiom::DisableVfx,
                Axiom::Ego,
                Axiom::TeleportDash { distance: 1 },
            ]),
            None,
            None,
            None,
            None,
        ]),
        Species::Grinder => Spellbook::new([
            None,
            Some(vec![
                Axiom::DisableVfx,
                Axiom::Ego,
                Axiom::HealOrHarm { amount: -99 },
            ]),
            None,
            None,
            None,
            None,
        ]),
        Species::Tinker => Spellbook::new([
            None,
            None,
            Some(vec![
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
                    stacks: EffectDuration::Finite { stacks: 3 },
                },
                Axiom::StatusEffect {
                    effect: StatusEffect::Haste,
                    potency: 1,
                    stacks: EffectDuration::Finite { stacks: 1 },
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
                Axiom::ToggleUntarget,
                Axiom::Ego,
                Axiom::ToggleUntarget,
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
