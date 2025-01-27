use bevy::{prelude::*, utils::HashMap};

use crate::{
    creature::{EffectDuration, Soul, StatusEffect},
    map::Position,
    spells::Axiom,
};

#[derive(Resource)]
pub struct CraftingRecipes {
    pub recipes: HashMap<Axiom, Recipe>,
}

pub struct Recipe {
    pub dimensions: Position,
    pub souls: Vec<Position>,
    pub soul_type: Soul,
}

impl Recipe {
    pub fn from_string(pattern: &str) -> Self {
        // number of lines
        let height = pattern.lines().count();
        // length of the first line
        let width = pattern.lines().next().unwrap_or("").len();

        let mut positions: Vec<Position> = Vec::new();

        let mut soul = None;

        for (y, line) in pattern.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ['S', 'O', 'A', 'U', 'F', 'V'].contains(&ch) {
                    if soul.is_none() {
                        soul = Some(match &ch {
                            'S' => Soul::Saintly,
                            'O' => Soul::Ordered,
                            'A' => Soul::Artistic,
                            'U' => Soul::Unhinged,
                            'F' => Soul::Feral,
                            'V' => Soul::Vile,
                            _ => panic!("Invalid crafting pattern: {}", ch),
                        });
                    }
                    positions.push(Position::new(x as i32, y as i32));
                }
            }
        }

        Recipe {
            dimensions: Position::new(width as i32, height as i32),
            souls: positions,
            soul_type: soul.unwrap(),
        }
    }
}

impl FromWorld for CraftingRecipes {
    fn from_world(_world: &mut World) -> Self {
        let mut crafting = CraftingRecipes {
            recipes: HashMap::new(),
        };
        crafting.recipes.insert(
            Axiom::Ego,
            Recipe::from_string(
                "\
                S\
                ",
            ),
        );
        crafting.recipes.insert(
            Axiom::MomentumBeam,
            Recipe::from_string(
                "\
                F\n\
                F\
                ",
            ),
        );
        crafting.recipes.insert(
            Axiom::XBeam,
            Recipe::from_string(
                "\
                .U\n\
                U\
                ",
            ),
        );
        crafting.recipes.insert(
            Axiom::PlusBeam,
            Recipe::from_string(
                "\
                U\n\
                U\
                ",
            ),
        );
        crafting.recipes.insert(
            Axiom::Plus,
            Recipe::from_string(
                "\
                O\n\
                O\
                ",
            ),
        );
        crafting.recipes.insert(
            Axiom::Touch,
            Recipe::from_string(
                "\
                V\
                ",
            ),
        );
        crafting.recipes.insert(
            Axiom::Halo { radius: 4 },
            Recipe::from_string(
                "\
                .U.\n\
                U.U\n\
                .U.\
                ",
            ),
        );
        crafting.recipes.insert(
            Axiom::Dash { max_distance: 5 },
            Recipe::from_string(
                "\
                FF\
                ",
            ),
        );
        crafting.recipes.insert(
            Axiom::HealOrHarm { amount: -1 },
            Recipe::from_string(
                "\
                U\
                ",
            ),
        );
        crafting.recipes.insert(
            Axiom::PlaceStepTrap,
            Recipe::from_string(
                "\
                A.A\
                ",
            ),
        );
        crafting.recipes.insert(
            Axiom::StatusEffect {
                effect: StatusEffect::Stab,
                potency: 5,
                stacks: EffectDuration::Finite { stacks: 20 },
            },
            Recipe::from_string(
                "\
                .V.\n\
                V.V\
                ",
            ),
        );
        crafting
    }
}
