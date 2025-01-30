use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use rand::{seq::IteratorRandom, thread_rng};

use crate::{
    creature::{get_soul_sprite, EffectDuration, Soul, Spellbook, StatusEffect},
    graphics::SpriteSheetAtlas,
    map::Position,
    spells::{Axiom, Spell},
    TILE_SIZE,
};

#[derive(Resource)]
pub struct CraftingRecipes {
    sorted_recipes: Vec<(Recipe, Axiom)>,
}

#[derive(Component)]
pub struct DroppedSoul(Soul);

#[derive(Event)]
pub struct TakeOrDropSoul {
    pub position: Position,
    pub soul: Option<Soul>,
}

pub fn take_or_drop_soul(
    mut events: EventReader<TakeOrDropSoul>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    dropped_souls: Query<(Entity, &Position), With<DroppedSoul>>,
) {
    for event in events.read() {
        for (soul, pos) in dropped_souls.iter() {
            if pos == &event.position {
                commands.entity(soul).despawn();
            }
        }
        if let Some(soul) = event.soul {
            commands.spawn((
                event.position,
                DroppedSoul(soul),
                Sprite {
                    image: asset_server.load("spritesheet.png"),
                    custom_size: Some(Vec2::new(TILE_SIZE - 1., TILE_SIZE - 1.)),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_layout.handle.clone(),
                        index: get_soul_sprite(&soul),
                    }),
                    ..default()
                },
            ));
        }
    }
}

#[derive(Event)]
pub struct CraftWithAxioms {
    pub boundaries: (Position, Position),
    pub receiver: Entity,
}

pub fn craft_with_axioms(
    mut events: EventReader<CraftWithAxioms>,
    dropped_souls: Query<(&Position, &DroppedSoul)>,
    crafting_recipes: Res<CraftingRecipes>,
    mut spellbook: Query<&mut Spellbook>,
) {
    for event in events.read() {
        let mut ingredients = HashMap::new();
        let mut soul_types = Vec::new();
        for (pos, soul_type) in dropped_souls.iter() {
            if pos.is_within_range(&event.boundaries.0, &event.boundaries.1) {
                ingredients.insert(pos, soul_type.0);
                soul_types.push(soul_type.0);
            }
        }
        let matches = crafting_recipes.find_all_matching_axioms(&ingredients);

        dbg!(&matches);
        // Do not create empty spells.
        if matches.is_empty() {
            continue;
        }
        let axioms: Vec<Axiom> = matches
            .into_iter()
            .map(|(_positions, axiom)| axiom.clone())
            .collect();

        let spell = Spell { axioms };
        let mut spellbook = spellbook.get_mut(event.receiver).unwrap();
        if let Some(caste) = most_common_soul(soul_types) {
            *spellbook.spells.entry(caste).or_insert(spell) = spell.clone();
        }
    }
}

pub fn most_common_soul(souls: Vec<Soul>) -> Option<Soul> {
    if souls.is_empty() {
        return None;
    }

    // count how many souls there are of each type
    let counts: HashMap<Soul, usize> = souls.into_iter().fold(HashMap::new(), |mut map, soul| {
        *map.entry(soul).or_insert(0) += 1;
        map
    });

    // locate the number of souls in the most numerous castes
    let max_count = counts.values().max().cloned().unwrap_or(0);

    // if there are multiple candidates, pick one at random
    let mut rng = thread_rng();
    counts
        .into_iter()
        .filter_map(|(soul, count)| if count == max_count { Some(soul) } else { None })
        .choose(&mut rng)
}

impl CraftingRecipes {
    pub fn find_all_matching_axioms(
        &self,
        ingredients: &HashMap<&Position, Soul>,
    ) -> Vec<(Vec<Position>, &Axiom)> {
        // This will accumulate the discovered axioms
        let mut matches = Vec::new();
        // This will ban souls from being used in 2 recipes
        let mut used_positions = HashSet::new();

        // Sort right-to-left, top-to-bottom (reading order) in the cage.
        let mut sorted_positions: Vec<&Position> = ingredients.keys().copied().collect();
        sorted_positions.sort_by(|a, b| a.y.cmp(&b.y).then(a.x.cmp(&b.x)));

        // starting from the most complex recipes...
        for pos in sorted_positions {
            // avoid already used souls
            if used_positions.contains(pos) {
                continue;
            }

            let soul = ingredients.get(pos).unwrap();

            for (recipe, axiom) in &self.sorted_recipes {
                // only scan recipes with this soul type
                if &recipe.soul_type != soul {
                    continue;
                }

                // // ban recipes that are too large for this cage
                // if recipe.dimensions.x > ingredients.keys().map(|p| p.x).max().unwrap_or(0) - pos.x
                //     || recipe.dimensions.y
                //         > ingredients.keys().map(|p| p.y).max().unwrap_or(0) - pos.y
                // {
                //     continue;
                // }

                // locate potential axioms
                let mut is_match = true;
                let mut recipe_positions = Vec::new();
                for rel_pos in &recipe.souls {
                    let abs_pos = Position {
                        x: pos.x + rel_pos.x,
                        y: pos.y + rel_pos.y,
                    };
                    if used_positions.contains(&abs_pos) || ingredients.get(&abs_pos) != Some(soul)
                    {
                        is_match = false;
                        break;
                    }
                    recipe_positions.push(abs_pos);
                }

                if is_match {
                    matches.push((recipe_positions.clone(), axiom));
                    used_positions.extend(recipe_positions);
                    break;
                }
            }
        }

        matches
    }
}

#[derive(Hash, PartialEq, Eq)]
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
        let mut recipes = HashMap::new();
        recipes.insert(
            Recipe::from_string(
                "\
                S\
                ",
            ),
            Axiom::Ego,
        );
        recipes.insert(
            Recipe::from_string(
                "\
                F\n\
                F\
                ",
            ),
            Axiom::MomentumBeam,
        );
        recipes.insert(
            Recipe::from_string(
                "\
                .U\n\
                U\
                ",
            ),
            Axiom::XBeam,
        );
        recipes.insert(
            Recipe::from_string(
                "\
                U\n\
                U\
                ",
            ),
            Axiom::PlusBeam,
        );
        recipes.insert(
            Recipe::from_string(
                "\
                O\n\
                O\
                ",
            ),
            Axiom::Plus,
        );
        recipes.insert(
            Recipe::from_string(
                "\
                V\
                ",
            ),
            Axiom::Touch,
        );
        recipes.insert(
            Recipe::from_string(
                "\
                .U.\n\
                U.U\n\
                .U.\
                ",
            ),
            Axiom::Halo { radius: 4 },
        );
        recipes.insert(
            Recipe::from_string(
                "\
                FF\
                ",
            ),
            Axiom::Dash { max_distance: 5 },
        );
        recipes.insert(
            Recipe::from_string(
                "\
                U\
                ",
            ),
            Axiom::HealOrHarm { amount: -1 },
        );
        recipes.insert(
            Recipe::from_string(
                "\
                A.A\
                ",
            ),
            Axiom::PlaceStepTrap,
        );
        recipes.insert(
            Recipe::from_string(
                "\
                .V.\n\
                V.V\
                ",
            ),
            Axiom::StatusEffect {
                effect: StatusEffect::Stab,
                potency: 5,
                stacks: EffectDuration::Finite { stacks: 20 },
            },
        );
        let mut sorted_recipes: Vec<(Recipe, Axiom)> = recipes.into_iter().collect();
        sorted_recipes.sort_by(|(a, _), (b, _)| b.souls.len().cmp(&a.souls.len()));

        CraftingRecipes { sorted_recipes }
    }
}
