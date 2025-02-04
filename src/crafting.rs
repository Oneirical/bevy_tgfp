use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use rand::{seq::IteratorRandom, thread_rng};
use uuid::Uuid;

use crate::{
    caste::{on_click_equip_unequip, on_hover_move_caste_cursor},
    creature::{
        get_soul_sprite, EffectDuration, Player, Soul, Species, SpellLibrary, Spellbook,
        StatusEffect,
    },
    graphics::SpriteSheetAtlas,
    map::Position,
    spells::{Axiom, Spell},
    ui::{
        spawn_split_text, AxiomBox, CraftingPatterns, CraftingPredictor, LibrarySlot, MessageLog,
        PatternBox, SoulWheelBox, SpellLibraryUI,
    },
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

pub fn match_axiom_with_icon(axiom: &Axiom) -> usize {
    match axiom {
        Axiom::Ego => 189,
        Axiom::MomentumBeam => 226,
        Axiom::XBeam => 198,
        Axiom::PlusBeam => 180,
        Axiom::Transform { species: _ } => 28,
        Axiom::WhenMoved => 183,
        Axiom::PlaceStepTrap => 12,
        Axiom::StatusEffect {
            effect,
            potency: _,
            stacks: _,
        } => match effect {
            StatusEffect::Stab => 40,
            StatusEffect::Invincible => 201,
            _ => 1,
        },
        Axiom::HealOrHarm { amount } => match amount.signum() {
            -1 => 188,
            1 => 184,
            _ => 1,
        },
        Axiom::ForceCast => 200,
        Axiom::Dash { max_distance: _ } => 187,
        Axiom::WhenTakingDamage => 173,
        Axiom::WhenDealingDamage => 174,
        Axiom::Touch => 177,
        Axiom::Trace => 231,
        _ => 1,
    }
}

#[derive(Event)]
pub struct PredictCraft {
    pub boundaries: (Position, Position),
}

#[derive(Component)]
pub struct CraftingVeil {
    pub boundaries: (Position, Position),
    pub pattern: Vec<Position>,
}

#[derive(Component)]
pub struct AxiomUI {
    pub axiom: Axiom,
}

pub fn predict_craft(
    mut events: EventReader<PredictCraft>,
    dropped_souls: Query<(&Position, &DroppedSoul)>,
    crafting_recipes: Res<CraftingRecipes>,
    mut commands: Commands,
    ui: Query<Entity, With<CraftingPredictor>>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
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
        commands.entity(ui.single()).try_despawn_descendants();
        for (positions, axiom) in matches {
            commands.entity(ui.single()).with_children(|parent| {
                parent
                    .spawn((
                        CraftingVeil {
                            boundaries: event.boundaries,
                            pattern: positions,
                        },
                        AxiomUI {
                            axiom: axiom.clone(),
                        },
                        ImageNode {
                            image: asset_server.load("spritesheet.png"),
                            texture_atlas: Some(TextureAtlas {
                                layout: atlas_layout.handle.clone(),
                                index: match_axiom_with_icon(axiom),
                            }),
                            ..Default::default()
                        },
                        Node {
                            width: Val::Px(3.),
                            height: Val::Px(3.),
                            ..default()
                        },
                    ))
                    .observe(on_hover_crafting_predictor)
                    .observe(on_exit_crafting_predictor)
                    .observe(on_hover_display_axiom)
                    .observe(on_exit_hover_axiom);
            });
        }
    }
}

fn on_exit_hover_axiom(
    _out: Trigger<Pointer<Out>>,
    mut message: Query<&mut Visibility, (With<MessageLog>, Without<AxiomBox>)>,
    mut axiom_box: Query<&mut Visibility, (With<AxiomBox>, Without<MessageLog>)>,
) {
    *message.single_mut() = Visibility::Inherited;
    *axiom_box.single_mut() = Visibility::Hidden;
}

fn on_hover_display_axiom(
    hover: Trigger<Pointer<Over>>,
    mut message: Query<&mut Visibility, (With<MessageLog>, Without<AxiomBox>)>,
    mut axiom_box: Query<(Entity, &mut Visibility), (With<AxiomBox>, Without<MessageLog>)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    axiom: Query<&AxiomUI>,
) {
    *message.single_mut() = Visibility::Hidden;
    let (axiom_box_entity, mut vis) = axiom_box.single_mut();
    *vis = Visibility::Inherited;

    let axiom = axiom.get(hover.entity()).unwrap();
    let axiom = &axiom.axiom;
    // TODO: Instead of multiple entities, would it be interesting to
    // have these merged into a single string with \n to space them out?
    // This would be good in case there's a ton of "effects flags".
    let (mut axiom_name, mut axiom_description) = (Entity::PLACEHOLDER, Entity::PLACEHOLDER);
    commands.entity(axiom_box_entity).despawn_descendants();
    commands.entity(axiom_box_entity).with_children(|parent| {
        axiom_name = spawn_split_text("ayo", parent, &asset_server);
        axiom_description = spawn_split_text("wao", parent, &asset_server);
        parent.spawn((
            ImageNode {
                image: asset_server.load("spritesheet.png"),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index: match_axiom_with_icon(axiom),
                }),
                ..Default::default()
            },
            Node {
                width: Val::Px(3.),
                height: Val::Px(3.),
                right: Val::Px(0.3),
                top: Val::Px(0.5),
                position_type: PositionType::Absolute,
                ..default()
            },
        ));
    });
    commands.entity(axiom_name).insert(Node {
        position_type: PositionType::Absolute,
        top: Val::Px(0.5),
        ..default()
    });
    commands.entity(axiom_description).insert(Node {
        position_type: PositionType::Absolute,
        top: Val::Px(3.5),
        ..default()
    });
}

#[derive(Component)]
pub struct BlackVeil;

fn on_hover_crafting_predictor(
    hover: Trigger<Pointer<Over>>,
    mut commands: Commands,
    veil: Query<&CraftingVeil>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    black: Query<&BlackVeil>,
) {
    if !black.is_empty() {
        return;
    }
    let veil = veil.get(hover.entity()).unwrap();
    for x in veil.boundaries.0.x..=veil.boundaries.1.x {
        for y in veil.boundaries.0.y..=veil.boundaries.1.y {
            let position = Position::new(x, y);
            if !veil.pattern.contains(&position) {
                commands.spawn((
                    BlackVeil,
                    position,
                    Transform::from_translation(Vec3::new(
                        position.x as f32 * TILE_SIZE,
                        position.y as f32 * TILE_SIZE,
                        -1.,
                    )),
                    Sprite {
                        image: asset_server.load("spritesheet.png"),
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        texture_atlas: Some(TextureAtlas {
                            layout: atlas_layout.handle.clone(),
                            index: 131,
                        }),
                        color: Color::Srgba(Srgba::new(0., 0., 0., 0.95)),
                        ..default()
                    },
                ));
            }
        }
    }
}

fn on_exit_crafting_predictor(
    _out: Trigger<Pointer<Out>>,
    mut commands: Commands,
    veils: Query<Entity, With<BlackVeil>>,
) {
    for veil in veils.iter() {
        commands.entity(veil).despawn();
    }
}

#[derive(Event)]
pub struct LearnNewAxiom {
    pub axiom: Axiom,
}

#[derive(Component)]
pub struct KnownPattern {
    pub recipe: Recipe,
}

pub fn learn_new_axiom(
    ui: Query<Entity, With<CraftingPatterns>>,
    known_patterns: Query<&AxiomUI, With<KnownPattern>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    all_patterns: Res<CraftingRecipes>,
    mut events: EventReader<LearnNewAxiom>,
) {
    for event in events.read() {
        // Do not learn duplicate axioms.
        for known in known_patterns.iter() {
            if known.axiom == event.axiom {
                continue;
            }
        }
        commands.entity(ui.single()).with_children(|parent| {
            parent
                .spawn((
                    AxiomUI {
                        axiom: event.axiom.clone(),
                    },
                    KnownPattern {
                        recipe: {
                            all_patterns
                                .sorted_recipes
                                .iter()
                                .find_map(|(key, value)| {
                                    if value == &event.axiom {
                                        Some(key)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap()
                                .clone()
                        },
                    },
                    ImageNode {
                        image: asset_server.load("spritesheet.png"),
                        texture_atlas: Some(TextureAtlas {
                            layout: atlas_layout.handle.clone(),
                            index: match_axiom_with_icon(&event.axiom),
                        }),
                        ..Default::default()
                    },
                    Node {
                        width: Val::Px(3.),
                        height: Val::Px(3.),
                        ..default()
                    },
                ))
                .observe(on_hover_display_axiom)
                .observe(on_exit_hover_axiom)
                .observe(on_hover_pattern_display)
                .observe(on_exit_pattern_display);
        });
    }
}

fn on_exit_pattern_display(
    _out: Trigger<Pointer<Out>>,
    mut wheel: Query<&mut Visibility, (With<SoulWheelBox>, Without<PatternBox>)>,
    mut pattern_box: Query<&mut Visibility, (With<PatternBox>, Without<SoulWheelBox>)>,
) {
    *wheel.single_mut() = Visibility::Inherited;
    *pattern_box.single_mut() = Visibility::Hidden;
}

fn on_hover_pattern_display(
    hover: Trigger<Pointer<Over>>,
    mut wheel: Query<&mut Visibility, (With<SoulWheelBox>, Without<PatternBox>)>,
    mut pattern_box: Query<(Entity, &mut Visibility), (With<PatternBox>, Without<SoulWheelBox>)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    pattern: Query<&KnownPattern>,
) {
    *wheel.single_mut() = Visibility::Hidden;
    let (pattern_box_entity, mut vis) = pattern_box.single_mut();
    *vis = Visibility::Inherited;

    let pattern = pattern.get(hover.entity()).unwrap();
    let pattern = &pattern.recipe;
    // TODO: Instead of multiple entities, would it be interesting to
    // have these merged into a single string with \n to space them out?
    // This would be good in case there's a ton of "effects flags".
    commands.entity(pattern_box_entity).despawn_descendants();
    commands.entity(pattern_box_entity).with_children(|parent| {
        for x in 0..pattern.dimensions.x {
            for y in 0..pattern.dimensions.y {
                parent.spawn((
                    ImageNode {
                        image: asset_server.load("spritesheet.png"),
                        texture_atlas: Some(TextureAtlas {
                            layout: atlas_layout.handle.clone(),
                            index: 167,
                        }),
                        ..Default::default()
                    },
                    Node {
                        width: Val::Px(3.),
                        height: Val::Px(3.),
                        left: Val::Px(x as f32 * 3.),
                        top: Val::Px(y as f32 * 3.),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                ));
            }
        }
    });
}

pub fn take_or_drop_soul(
    mut events: EventReader<TakeOrDropSoul>,
    mut predict: EventWriter<PredictCraft>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    dropped_souls: Query<(Entity, &Position), With<DroppedSoul>>,
    veils: Query<Entity, With<BlackVeil>>,
) {
    for event in events.read() {
        // A quick purge of crafting pattern veils to avoid
        // janky graphics when placing souls while veils
        // are active.
        for veil in veils.iter() {
            commands.entity(veil).despawn();
        }
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
                Transform::from_translation(Vec3::new(
                    event.position.x as f32 * TILE_SIZE,
                    event.position.y as f32 * TILE_SIZE,
                    -7.,
                )),
            ));
        }
        predict.send(PredictCraft {
            boundaries: (Position::new(6, 7), Position::new(8, 9)),
        });
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
    mut spell_library: ResMut<SpellLibrary>,
    mut spellbook: Query<(&mut Spellbook, Has<Player>)>,
    mut commands: Commands,
    ui: Query<Entity, With<SpellLibraryUI>>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
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

        // Do not create empty spells.
        if matches.is_empty() {
            continue;
        }
        let axioms: Vec<Axiom> = matches
            .into_iter()
            .map(|(_positions, axiom)| axiom.clone())
            .collect();

        if let Some(caste) = most_common_soul(soul_types) {
            let icon = 172;
            let id = Uuid::new_v4();
            let spell = Spell {
                axioms,
                caste,
                icon,
                id,
            };
            let (mut book, is_player) = spellbook.get_mut(event.receiver).unwrap();
            if is_player {
                spell_library.library.push(spell);
                commands.entity(ui.single()).with_children(|parent| {
                    parent
                        .spawn((
                            LibrarySlot(id),
                            ImageNode {
                                image: asset_server.load("spritesheet.png"),
                                texture_atlas: Some(TextureAtlas {
                                    layout: atlas_layout.handle.clone(),
                                    index: icon,
                                }),
                                ..Default::default()
                            },
                            Node {
                                width: Val::Px(3.),
                                height: Val::Px(3.),
                                ..default()
                            },
                        ))
                        .observe(on_click_equip_unequip)
                        .observe(on_hover_move_caste_cursor);
                });
            } else {
                book.spells.insert(caste, spell);
            }
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
        sorted_positions.sort_by(|a, b| b.y.cmp(&a.y).then(a.x.cmp(&b.x)));

        let cage_dimension_x = ingredients.keys().map(|p| p.x).max().unwrap_or(0)
            - ingredients.keys().map(|p| p.x).min().unwrap_or(0)
            + 1;
        let cage_dimension_y = ingredients.keys().map(|p| p.y).max().unwrap_or(0)
            - ingredients.keys().map(|p| p.y).min().unwrap_or(0)
            + 1;

        for pos in &sorted_positions {
            // avoid already used souls
            if used_positions.contains(*pos) {
                continue;
            }

            let soul = ingredients.get(pos).unwrap();
            // starting from the most complex recipes...
            for (recipe, axiom) in &self.sorted_recipes {
                // only scan recipes with this soul type
                if &recipe.soul_type != soul {
                    continue;
                }

                // ban recipes that are too large for this cage
                if recipe.dimensions.x > cage_dimension_x || recipe.dimensions.y > cage_dimension_y
                {
                    continue;
                }

                // locate potential axioms
                let mut is_match = true;
                let mut recipe_positions = Vec::new();
                for rel_pos in &recipe.souls {
                    let abs_pos = Position {
                        x: pos.x + rel_pos.x,
                        y: pos.y + rel_pos.y,
                    };
                    // This soul is incorrect or already used by another recipe
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

#[derive(Hash, PartialEq, Eq, Clone)]
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
        let mut first_encountered = None;

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
                        first_encountered = Some(Position::new(x as i32, y as i32));
                    }
                    positions.push(Position::new(
                        x as i32 - first_encountered.unwrap().x,
                        -(y as i32 - first_encountered.unwrap().y),
                    ));
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
                O\
                ",
            ),
            Axiom::StatusEffect {
                effect: StatusEffect::Invincible,
                potency: 1,
                stacks: EffectDuration::Finite { stacks: 3 },
            },
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
                V\
                ",
            ),
            Axiom::Touch,
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
                FF\
                ",
            ),
            Axiom::Dash { max_distance: 5 },
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
                SS\n\
                S.\
                ",
            ),
            Axiom::ForceCast,
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
        recipes.insert(
            Recipe::from_string(
                "\
                AA\n\
                ..\n\
                A.\
                ",
            ),
            Axiom::Transform {
                species: Species::Abazon,
            },
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
                F.F\n\
                .F.\n\
                .F.\
                ",
            ),
            Axiom::Trace,
        );
        recipes.insert(
            Recipe::from_string(
                "\
                V..\n\
                ...\n\
                VVV\
                ",
            ),
            Axiom::WhenDealingDamage,
        );
        recipes.insert(
            Recipe::from_string(
                "\
                O..\n\
                ...\n\
                OOO\
                ",
            ),
            Axiom::WhenTakingDamage,
        );
        let mut sorted_recipes: Vec<(Recipe, Axiom)> = recipes.into_iter().collect();
        sorted_recipes.sort_by(|(a, _), (b, _)| b.souls.len().cmp(&a.souls.len()));

        CraftingRecipes { sorted_recipes }
    }
}
