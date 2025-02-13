use std::{cmp::min, f32::consts::PI};

use bevy::{
    prelude::*,
    render::view::RenderLayers,
    utils::{HashMap, HashSet},
};
use rand::{seq::IteratorRandom, thread_rng};

use crate::{
    crafting::{
        BagOfLoot, CraftWithAxioms, DroppedSoul, KnownPattern, LearnNewAxiom, PredictCraft,
        TakeOrDropSoul,
    },
    creature::{
        get_soul_sprite, get_species_spellbook, get_species_sprite, is_naturally_intangible, Awake,
        Charm, ConveyorBelt, CraftingSlot, Creature, CreatureFlags, DesignatedForRemoval, Dizzy,
        Door, EffectDuration, FlagEntity, Fragile, Health, HealthIndicator, Hunt, Immobile,
        Intangible, Invincible, Magnetic, Magnetized, Meleeproof, NoDropSoul, Player, Possessed,
        Possessing, PotencyAndStacks, Random, Sleeping, Soul, Species, Speed, SpellLibrary,
        Spellbook, Spellproof, Stab, StatusEffect, StatusEffectsList, Summoned, Wall,
    },
    graphics::{
        get_effect_sprite, EffectSequence, EffectType, MagicEffect, MagicVfx, PlaceMagicVfx,
        SlideAnimation, SpriteSheetAtlas,
    },
    map::{is_soul_cage_room, manhattan_distance, spawn_cage, FaithsEnd, Map, Position},
    sets::ControlState,
    spells::{walk_grid, AntiContingencyLoop, Axiom, CastSpell, TriggerContingency},
    ui::{
        AddMessage, AnnounceGameOver, EquipSlot, InvalidAction, LibrarySlot, Message, RecipebookUI,
        SoulSlot,
    },
    OrdDir, TILE_SIZE,
};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SummonCreature>();
        app.init_resource::<Events<EndTurn>>();
        app.add_event::<TeleportEntity>();
        app.add_event::<TransformCreature>();
        app.add_event::<SteppedOnTile>();
        app.add_event::<CreatureCollision>();
        app.add_event::<AlterMomentum>();
        app.add_event::<DamageOrHealCreature>();
        app.add_event::<OpenCloseDoor>();
        app.add_event::<RemoveCreature>();
        app.add_event::<EchoSpeed>();
        app.add_event::<DistributeNpcActions>();
        app.add_event::<RespawnPlayer>();
        app.add_event::<AddStatusEffect>();
        app.add_event::<DrawSoul>();
        app.add_event::<UseWheelSoul>();
        app.add_event::<MagnetFollow>();
        app.init_resource::<Events<CreatureStep>>();
        app.init_resource::<Events<RespawnCage>>();
        app.insert_resource(TurnManager {
            turn_count: 0,
            action_this_turn: PlayerAction::Invalid,
        });
        app.insert_resource(CagePainter {
            is_painting: false,
            current_paint: None,
        });
        app.init_resource::<SoulWheel>();
    }
}

#[derive(Resource)]
pub struct TurnManager {
    pub turn_count: usize,
    /// Whether the player took a step, cast a spell, or did something useless (like step into a wall) this turn.
    pub action_this_turn: PlayerAction,
}

#[derive(Resource)]
pub struct CagePainter {
    pub is_painting: bool,
    pub current_paint: Option<Soul>,
}

pub fn is_painting(painter: Res<CagePainter>) -> bool {
    painter.is_painting
}

pub fn swap_current_paint(
    mut events: EventReader<UseWheelSoul>,
    mut painter: ResMut<CagePainter>,
    mut ui_soul_slots: Query<(&mut ImageNode, &SoulSlot)>,
    mut turn_manager: ResMut<TurnManager>,
) {
    for event in events.read() {
        painter.current_paint = match event.index {
            0 => None,
            1 => Some(Soul::Saintly),
            2 => Some(Soul::Ordered),
            3 => Some(Soul::Artistic),
            4 => Some(Soul::Unhinged),
            5 => Some(Soul::Feral),
            6 => Some(Soul::Vile),
            7 => Some(Soul::Serene),
            _ => panic!("There should only be 8 soul wheel slots!"),
        };
        for (mut ui_slot_node, ui_slot_marker) in ui_soul_slots.iter_mut() {
            if ui_slot_marker.index == event.index {
                ui_slot_node.color.set_alpha(1.);
            } else {
                ui_slot_node.color.set_alpha(0.1);
            }
        }
        // Changing the paint type does not take a turn.
        turn_manager.action_this_turn = PlayerAction::Skipped;
    }
}

pub fn toggle_paint_mode(
    mut painter: ResMut<CagePainter>,
    player: Query<&Position, With<Player>>,
    slots: Query<&FlagEntity, With<CraftingSlot>>,
    position: Query<&Position>,
    mut ui_soul_slots: Query<(&mut ImageNode, &SoulSlot)>,
    soul_wheel: Res<SoulWheel>,
    mut recipe_book: Query<&mut Visibility, With<RecipebookUI>>,
    mut predict: EventWriter<PredictCraft>,
) {
    if painter.is_painting {
        let player_pos = player.single();
        let mut out_of_range = true;
        for slot in slots.iter() {
            let slot = position.get(slot.parent_creature).unwrap();
            if player_pos.is_within_range(
                &Position {
                    x: slot.x - 1,
                    y: slot.y - 1,
                },
                &Position {
                    x: slot.x + 1,
                    y: slot.y + 1,
                },
            ) {
                out_of_range = false;
            }
        }
        if out_of_range {
            painter.is_painting = false;
            *recipe_book.single_mut() = Visibility::Hidden;
            for (mut ui_slot_node, ui_slot_marker) in ui_soul_slots.iter_mut() {
                ui_slot_node.texture_atlas.as_mut().unwrap().index =
                    if let Some(wheel_soul) = soul_wheel.souls.get(ui_slot_marker.index).unwrap() {
                        get_soul_sprite(wheel_soul)
                    } else {
                        167
                    };
                ui_slot_node.color.set_alpha(1.);
            }
        }
    } else {
        let player_pos = player.single();
        for slot in slots.iter() {
            let slot = position.get(slot.parent_creature).unwrap();
            if player_pos.is_within_range(
                &Position {
                    x: slot.x - 1,
                    y: slot.y - 1,
                },
                &Position {
                    x: slot.x + 1,
                    y: slot.y + 1,
                },
            ) {
                painter.is_painting = true;
                predict.send(PredictCraft {
                    impact_point: *slot,
                });
                *recipe_book.single_mut() = Visibility::Inherited;
                let full_alpha_index = match painter.current_paint {
                    None => 0,
                    Some(Soul::Saintly) => 1,
                    Some(Soul::Ordered) => 2,
                    Some(Soul::Artistic) => 3,
                    Some(Soul::Unhinged) => 4,
                    Some(Soul::Feral) => 5,
                    Some(Soul::Vile) => 6,
                    Some(Soul::Serene) => 7,
                    _ => panic!("There should only be 8 soul wheel slots!"),
                };
                for (mut ui_slot_node, ui_slot_marker) in ui_soul_slots.iter_mut() {
                    ui_slot_node.texture_atlas.as_mut().unwrap().index = match ui_slot_marker.index
                    {
                        0 => 167,
                        1 => get_soul_sprite(&Soul::Saintly),
                        2 => get_soul_sprite(&Soul::Ordered),
                        3 => get_soul_sprite(&Soul::Artistic),
                        4 => get_soul_sprite(&Soul::Unhinged),
                        5 => get_soul_sprite(&Soul::Feral),
                        6 => get_soul_sprite(&Soul::Vile),
                        7 => get_soul_sprite(&Soul::Serene),
                        _ => panic!("There should only be 8 soul wheel slots!"),
                    };

                    if ui_slot_marker.index == full_alpha_index {
                        ui_slot_node.color.set_alpha(1.);
                    } else {
                        ui_slot_node.color.set_alpha(0.1);
                    }
                }
                return;
            }
        }
    }
}

#[derive(Resource)]
pub struct SoulWheel {
    pub souls: [Option<Soul>; 8],
    pub draw_pile: HashMap<Soul, usize>,
    pub discard_pile: HashMap<Soul, usize>,
}

impl FromWorld for SoulWheel {
    fn from_world(_world: &mut World) -> Self {
        let mut soul_wheel = Self {
            souls: [None; 8],
            draw_pile: HashMap::new(),
            discard_pile: HashMap::new(),
        };
        soul_wheel.draw_pile.insert(Soul::Saintly, 100);
        soul_wheel.draw_pile.insert(Soul::Ordered, 1);
        soul_wheel.draw_pile.insert(Soul::Artistic, 1);
        soul_wheel.draw_pile.insert(Soul::Unhinged, 1);
        soul_wheel.draw_pile.insert(Soul::Feral, 1);
        soul_wheel.draw_pile.insert(Soul::Vile, 1);
        soul_wheel.discard_pile.insert(Soul::Saintly, 0);
        soul_wheel.discard_pile.insert(Soul::Ordered, 0);
        soul_wheel.discard_pile.insert(Soul::Artistic, 0);
        soul_wheel.discard_pile.insert(Soul::Unhinged, 0);
        soul_wheel.discard_pile.insert(Soul::Feral, 0);
        soul_wheel.discard_pile.insert(Soul::Vile, 0);
        soul_wheel
    }
}

impl SoulWheel {
    fn castes_with_non_zero_souls(&self) -> HashSet<Soul> {
        let mut output = HashSet::new();
        for (caste, amount) in &self.draw_pile {
            if amount != &0 {
                output.insert(*caste);
            }
        }
        output
    }

    fn draw_random_caste(&mut self) -> Option<Soul> {
        let possible_castes = self.castes_with_non_zero_souls();
        let mut rng = thread_rng();
        if let Some(drawn_soul) = possible_castes.iter().choose(&mut rng) {
            self.draw_pile
                .entry(*drawn_soul)
                .and_modify(|count| *count -= 1);
            return Some(*drawn_soul);
        }
        None
    }
}

#[derive(Event)]
pub struct DrawSoul {
    pub amount: usize,
}

pub fn draw_soul(
    mut events: EventReader<DrawSoul>,
    mut soul_wheel: ResMut<SoulWheel>,
    mut ui_soul_slots: Query<(&mut ImageNode, &SoulSlot)>,
    mut turn_manager: ResMut<TurnManager>,
    mut text: EventWriter<AddMessage>,
) {
    for event in events.read() {
        for _i in 0..event.amount {
            let mut index_to_fill = None;

            // Find an empty slot in the Soul Wheel.
            for (index, soul_slot) in soul_wheel.souls.iter().enumerate() {
                if soul_slot.is_none() {
                    index_to_fill = Some(index);
                    break;
                }
            }

            if let Some(index) = index_to_fill {
                // Draw a new soul from the deck.
                if let Some(new_soul) = soul_wheel.draw_random_caste() {
                    soul_wheel.souls[index] = Some(new_soul);
                    // Reflect this new soul in the UI wheel.
                    for (mut ui_slot_node, ui_slot_marker) in ui_soul_slots.iter_mut() {
                        if ui_slot_marker.index == index {
                            ui_slot_node.texture_atlas.as_mut().unwrap().index =
                                get_soul_sprite(&new_soul);
                        }
                    }
                } else {
                    // NOTE: Commented out to match the new "draw on melee" rework
                    // There is nothing left in the draw pile!
                    // text.send(AddMessage {
                    //     message: Message::InvalidAction(InvalidAction::NoSoulsInPile),
                    // });
                    // turn_manager.action_this_turn = PlayerAction::Invalid;
                }
            } else {
                // NOTE: Commented out to match the new "draw on melee" rework
                // There is no empty space in the Wheel!
                // text.send(AddMessage {
                //     message: Message::InvalidAction(InvalidAction::WheelFull),
                // });
                // turn_manager.action_this_turn = PlayerAction::Invalid;
            }
        }
    }
}

#[derive(Event)]
pub struct UseWheelSoul {
    pub index: usize,
}

pub fn mouse_use_wheel_soul(
    trigger: Trigger<Pointer<Click>>,
    mut use_wheel_soul: EventWriter<UseWheelSoul>,
    state: Res<State<ControlState>>,
    slot: Query<&SoulSlot>,
    mut turn_manager: ResMut<TurnManager>,
    mut turn_end: EventWriter<EndTurn>,
) {
    if matches!(state.get(), ControlState::Player) {
        use_wheel_soul.send(UseWheelSoul {
            index: slot.get(trigger.entity()).unwrap().index,
        });
        turn_manager.action_this_turn = PlayerAction::Spell;
        turn_end.send(EndTurn);
    }
}

pub fn use_wheel_soul(
    mut events: EventReader<UseWheelSoul>,
    mut soul_wheel: ResMut<SoulWheel>,
    mut spell: EventWriter<CastSpell>,
    mut ui_soul_slots: Query<(&mut ImageNode, &SoulSlot)>,
    mut turn_manager: ResMut<TurnManager>,
    player: Query<(Entity, &Spellbook), With<Player>>,
    mut text: EventWriter<AddMessage>,
) {
    for event in events.read() {
        let mut newly_discarded = None;
        if let Some(soul) = soul_wheel.souls.get(event.index).unwrap() {
            // Cast the spell corresponding to this soul type.
            let (player_entity, spellbook) = player.get_single().unwrap();
            if let Some(chosen_spell) = spellbook.spells.get(soul) {
                spell.send(CastSpell {
                    caster: player_entity,
                    spell: chosen_spell.clone(),
                    starting_step: 0,
                    soul_caste: *soul,
                });
                // Discard the soul into the discard pile.
                newly_discarded = Some(*soul);
                // Empty this soul slot.
                soul_wheel.souls[event.index] = None;
                // Update the UI accordingly.
                for (mut ui_slot_node, ui_slot_marker) in ui_soul_slots.iter_mut() {
                    if ui_slot_marker.index == event.index {
                        ui_slot_node.texture_atlas.as_mut().unwrap().index = 167;
                    }
                }
            } else {
                // That caste has no spell attached!
                text.send(AddMessage {
                    message: Message::InvalidAction(InvalidAction::NoSpellForCaste),
                });
                turn_manager.action_this_turn = PlayerAction::Invalid;
            }
        } else {
            // That soul slot is empty!
            text.send(AddMessage {
                message: Message::InvalidAction(InvalidAction::EmptySlotCast),
            });
            turn_manager.action_this_turn = PlayerAction::Invalid;
        }
        // The spent soul is sent to the discard pile.
        if let Some(newly_discarded) = newly_discarded {
            soul_wheel
                .discard_pile
                .entry(newly_discarded)
                .and_modify(|amount| *amount += 1);
        }
    }
}

pub enum PlayerAction {
    Step,
    Spell,
    Draw,
    Invalid,
    Skipped,
}

#[derive(Event)]
pub struct AddStatusEffect {
    pub entity: Entity,
    pub effect: StatusEffect,
    pub potency: usize,
    pub stacks: EffectDuration,
    pub culprit: Entity,
}

pub fn add_status_effects(
    mut events: EventReader<AddStatusEffect>,
    mut effects: Query<(&mut StatusEffectsList, &CreatureFlags)>,
    mut commands: Commands,
) {
    for event in events.read() {
        let (mut effects_list, flags) = effects.get_mut(event.entity).unwrap();
        if let Some(effect) = effects_list.effects.get(&event.effect) {
            // Re-applying a status effect which is already possessed does not work
            // if the new effect has a lesser potency.
            if event.potency < effect.potency {
                continue;
            }
        }
        // Mark the creature as possessing that status effect.
        effects_list.effects.insert(
            event.effect,
            PotencyAndStacks {
                potency: event.potency,
                stacks: event.stacks,
            },
        );
        let effects_flags = flags.effects_flags;
        // Insert the components corresponding to the new status effect.
        match event.effect {
            StatusEffect::Invincible => {
                commands.entity(effects_flags).insert(Invincible);
            }
            StatusEffect::Stab => {
                commands.entity(effects_flags).insert(Stab {
                    bonus_damage: event.potency as isize,
                });
            }
            StatusEffect::Dizzy => {
                commands.entity(effects_flags).insert(Dizzy);
            }
            StatusEffect::DimensionBond => {
                commands.entity(effects_flags).insert(Summoned {
                    summoner: event.culprit,
                });
            }
            StatusEffect::Possessed => {
                commands.entity(effects_flags).insert(Possessed {
                    original: event.culprit,
                });
            }
            StatusEffect::Haste => {
                commands.entity(effects_flags).insert(Speed::Fast {
                    actions_per_turn: event.potency + 1,
                });
            }
            StatusEffect::Charm => {
                commands.entity(effects_flags).insert(Charm);
            }
            StatusEffect::Magnetize => {
                commands.entity(effects_flags).insert(Magnetic {
                    species: Species::WeakWall,
                    conductor: None,
                });
            }
        }
    }
}

#[derive(Event)]
pub struct SummonCreature {
    pub position: Position,
    pub species: Species,
    pub properties: Vec<SummonProperties>,
}

pub enum SummonProperties {
    Spellbook(Spellbook),
    ConveyorBelt,
    Summoned {
        summoner_tile: Position,
        summoner: Entity,
    },
    Momentum(OrdDir),
    Sleeping,
}

/// Place a new Creature on the map of Species and at Position.
pub fn summon_creature(
    mut commands: Commands,
    mut events: EventReader<SummonCreature>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    map: Res<Map>,
    faiths_end: Res<FaithsEnd>,
) {
    for event in events.read() {
        // Avoid summoning if the tile is already occupied.
        // Intangible creatures are allowed to spawn.
        if !map.is_passable(event.position.x, event.position.y)
            && !is_naturally_intangible(&event.species)
        {
            continue;
        }
        let max_hp = 6;
        let hp = match &event.species {
            Species::Player => 6,
            Species::Hunter => 1,
            Species::Spawner => 3,
            Species::Apiarist => 3,
            Species::Shrike => 1,
            Species::Second => 1,
            Species::Tinker => 1,
            Species::Oracle => 2,
            Species::AxiomaticSeal => 4,
            // Wall-type creatures just get full HP to avoid displaying
            // their healthbar.
            _ => max_hp,
        };

        let (effects_flags, species_flags) = (
            commands
                .spawn_empty()
                .observe(start_possessing_creature)
                .observe(stop_possessing_creature)
                .id(),
            commands
                .spawn_empty()
                .observe(start_possessing_creature)
                .observe(stop_possessing_creature)
                .id(),
        );

        let mut new_creature = commands.spawn_empty();
        let parent_creature = new_creature.id();

        new_creature.insert((
            Creature {
                position: event.position,
                species: event.species,
                sprite: Sprite {
                    image: asset_server.load("spritesheet.png"),
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_layout.handle.clone(),
                        index: get_species_sprite(&event.species),
                    }),
                    ..default()
                },
                momentum: OrdDir::Down,
                health: Health { max_hp, hp },
                effects: StatusEffectsList {
                    effects: HashMap::new(),
                },
                soul: match &event.species {
                    Species::Player => Soul::Saintly,
                    Species::Wall | Species::WeakWall => Soul::Ordered,
                    Species::Hunter => Soul::Saintly,
                    Species::Shrike => Soul::Feral,
                    Species::Apiarist => Soul::Ordered,
                    Species::Tinker => Soul::Artistic,
                    Species::Second => Soul::Vile,
                    Species::Oracle => Soul::Unhinged,
                    Species::EpsilonHead | Species::EpsilonTail => Soul::Ordered,
                    Species::CageSlot => Soul::Saintly,
                    Species::AxiomaticSeal => Soul::Vile,
                    _ => Soul::Unhinged,
                },
                spellbook: get_species_spellbook(&event.species),
                flags: CreatureFlags {
                    effects_flags,
                    species_flags,
                },
            },
            SlideAnimation,
        ));

        let mut transform = Transform {
            translation: Vec3 {
                x: event.position.x as f32 * TILE_SIZE,
                y: event.position.y as f32 * TILE_SIZE,
                z: 0.,
            },
            rotation: Quat::from_rotation_z(0.),
            ..Default::default()
        };

        let mut summoner_flag_insertion = None;

        for property in event.properties.iter() {
            match property {
                SummonProperties::Spellbook(book) => {
                    new_creature.insert(book.clone());
                }
                SummonProperties::Momentum(momentum) => {
                    transform.rotation = Quat::from_rotation_z(match momentum {
                        OrdDir::Down => 0.,
                        OrdDir::Right => PI / 2.,
                        OrdDir::Up => PI,
                        OrdDir::Left => 3. * PI / 2.,
                    });
                    new_creature.insert(*momentum);
                }
                SummonProperties::Summoned {
                    summoner_tile,
                    summoner,
                } => {
                    transform.translation.x = summoner_tile.x as f32 * TILE_SIZE;
                    transform.translation.y = summoner_tile.y as f32 * TILE_SIZE;
                    // Summoned creatures are marked with their summoner.
                    summoner_flag_insertion = Some(*summoner);
                }
                SummonProperties::ConveyorBelt => {
                    new_creature.insert(ConveyorBelt);
                }
                SummonProperties::Sleeping => {
                    new_creature.insert(Sleeping);
                }
            };
        }

        new_creature.insert(transform);

        // TODO: This will have to be removed when creating player clones
        // becomes possible.
        if event.species == Species::Player {
            new_creature.insert(Player);
        }

        // Creatures which start out damaged show their HP bar in advance.
        let (visibility, index) = hp_bar_visibility_and_index(hp, max_hp);

        // Free the borrow on Commands.
        let new_creature_entity = new_creature.id();

        // Inform the effects and species flags that this creature
        // is their parent.
        commands
            .entity(effects_flags)
            .insert(FlagEntity { parent_creature });
        commands
            .entity(species_flags)
            .insert(FlagEntity { parent_creature });

        if let Some(summoner) = summoner_flag_insertion {
            commands.entity(effects_flags).insert(Summoned { summoner });
        }

        let hp_bar = commands
            .spawn(HealthIndicator {
                sprite: Sprite {
                    image: asset_server.load("spritesheet.png"),
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_layout.handle.clone(),
                        index,
                    }),
                    ..default()
                },
                visibility,
                transform: Transform::from_xyz(0., 0., 1.),
            })
            .id();
        commands.entity(new_creature_entity).add_child(hp_bar);
    }
}

#[derive(Event)]
pub struct TransformCreature {
    pub entity: Entity,
    pub new_species: Species,
}

pub fn transform_creature(
    mut transform: EventReader<TransformCreature>,
    mut creature_query: Query<(&mut Species, &mut Sprite, &CreatureFlags)>,
    mut commands: Commands,
) {
    for event in transform.read() {
        let (mut species_of_creature, mut sprite, flags) =
            creature_query.get_mut(event.entity).unwrap();
        // Change the species.
        *species_of_creature = event.new_species;
        sprite.texture_atlas.as_mut().unwrap().index = get_species_sprite(&event.new_species);
        // Remove all components except for its knowledge of its parent.
        // The appropriate ones will be readded by assign_species_components.
        commands.entity(flags.species_flags).retain::<FlagEntity>();
    }
}

/// Add any species-specific components.
pub fn assign_species_components(
    changed_species: Query<(&CreatureFlags, &Species), Changed<Species>>,
    mut commands: Commands,
) {
    for (flags, species) in changed_species.iter() {
        let mut new_creature = commands.entity(flags.species_flags);
        match species {
            Species::Player => {
                new_creature.insert(Intangible);
            }
            Species::Trap => {
                new_creature.insert((
                    Meleeproof, Spellproof, Intangible, Fragile, Invincible, NoDropSoul,
                ));
            }
            Species::CageBorder => {
                new_creature.insert((Meleeproof, Spellproof, Intangible, Invincible, NoDropSoul));
            }
            Species::CageSlot => {
                new_creature.insert((
                    Meleeproof,
                    Spellproof,
                    Intangible,
                    Invincible,
                    NoDropSoul,
                    CraftingSlot,
                ));
            }
            Species::Wall => {
                new_creature.insert((Meleeproof, Spellproof, Wall, Invincible, Dizzy, NoDropSoul));
            }
            Species::WeakWall => {
                new_creature.insert((Meleeproof, Wall, Invincible, Dizzy, NoDropSoul));
            }
            Species::Airlock => {
                new_creature.insert((Meleeproof, Spellproof, Door, Invincible, Dizzy, NoDropSoul));
            }
            Species::Hunter | Species::Spawner | Species::Second | Species::Oracle => {
                new_creature.insert(Hunt);
            }
            Species::Tinker => {
                new_creature.insert(Random);
            }
            Species::Abazon => {
                new_creature.insert((Immobile, Hunt));
            }
            Species::AxiomaticSeal => {
                new_creature.insert((Immobile, Hunt, Dizzy, NoDropSoul, Spellproof));
            }
            Species::EpsilonHead => {
                new_creature.insert((
                    Magnetic {
                        species: Species::EpsilonTail,
                        conductor: None,
                    },
                    Hunt,
                ));
            }
            Species::Apiarist => {
                new_creature.insert((Speed::Slow { wait_turns: 1 }, Hunt));
            }
            Species::Shrike => {
                new_creature.insert((
                    Speed::Fast {
                        actions_per_turn: 2,
                    },
                    Hunt,
                ));
            }
            _ => (),
        }
    }
}

/// Determine whether to show or not, and at which sprite index, an HP bar.
fn hp_bar_visibility_and_index(hp: usize, max_hp: usize) -> (Visibility, usize) {
    (
        {
            if max_hp == hp {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            }
        },
        {
            if max_hp == hp {
                178
            } else {
                match hp {
                    5 => 238,
                    4 => 239,
                    3 => 240,
                    2 => 241,
                    1 => 242,
                    _ => 243,
                }
            }
        },
    )
}

#[derive(Event)]
pub struct CreatureStep {
    pub entity: Entity,
    pub direction: OrdDir,
}

pub fn creature_step(
    mut events: EventReader<CreatureStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut momentum: EventWriter<AlterMomentum>,
    mut creature: Query<&Position>,
) {
    for event in events.read() {
        let creature_pos = creature.get_mut(event.entity).unwrap();
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportEntity::new(
            event.entity,
            creature_pos.x + off_x,
            creature_pos.y + off_y,
        ));
        // Update the direction towards which this creature is facing.
        momentum.send(AlterMomentum {
            entity: event.entity,
            direction: event.direction,
        });
    }
}

pub fn magnetize_tail_segments(
    query: Query<(Entity, &Magnetic)>,
    conductor_query: Query<(Entity, &Position, &CreatureFlags)>,
    creature_flags: Query<&CreatureFlags>,
    mut magnetized_set: ParamSet<(
        Query<Entity, With<Magnetized>>,
        Query<&Magnetized>,
        Query<&mut Magnetized>,
    )>,
    species_query: Query<&Species>,
    map: Res<Map>,
    mut commands: Commands,
) {
    // Get the species/effects flags...
    for (conductor_entity, pos, flags) in conductor_query.iter() {
        // of a creature that is considered Magnetic.
        // species has override on effects.
        if let Ok((flag_entity, magnet)) = query
            .get(flags.species_flags)
            .or(query.get(flags.effects_flags))
        {
            let magnetized_finder = magnetized_set.p0();
            let flags_with_magnetized = if let Some(original_conductor) = magnet.conductor {
                if let Ok(conductor_flags) = creature_flags.get(original_conductor) {
                    if let Ok(flags_with_magnetized) = magnetized_finder
                        .get(conductor_flags.species_flags)
                        .or(magnetized_finder.get(conductor_flags.effects_flags))
                    {
                        Some(flags_with_magnetized)
                    } else {
                        None
                    }
                } else {
                    // The conductor has been removed, and this segment
                    // loses its magnetism.
                    commands.entity(flag_entity).remove::<Magnetic>();
                    return;
                }
            } else {
                None
            };
            // Find adjacent creatures to magnetize.
            // NOTE: This will ignore intangible creatures.
            let adjacent_tiles = map.get_adjacent_tiles(*pos);
            for tile in adjacent_tiles {
                // If a creature is found...
                if let Some(adjacent_creature) = map.creatures.get(&tile) {
                    // Make sure it has the correct species to match with the magnet,
                    // and that it is not already magnetized.
                    let mut is_part_of_tail = false;
                    let magnetized_finder = magnetized_set.p1();
                    for magnetized in magnetized_finder.iter() {
                        // No stealing from other snakes.
                        if magnetized.train.contains(adjacent_creature) {
                            is_part_of_tail = true;
                        }
                    }
                    if *species_query.get(*adjacent_creature).unwrap() == magnet.species
                        && !is_part_of_tail
                    {
                        // If so, enter its effects flags to start editing.
                        let new_tail_segment_flags =
                            creature_flags.get(*adjacent_creature).unwrap();
                        // Remove all instances of Magnetic from the original creature -
                        // it has found its fellow magnet.
                        commands.entity(flags.species_flags).remove::<Magnetic>();
                        commands.entity(flags.effects_flags).remove::<Magnetic>();
                        // The new tail segment receives Magnetic as it will seek out
                        // the next magnet.
                        commands
                            .entity(new_tail_segment_flags.effects_flags)
                            .insert(Magnetic {
                                species: magnet.species,
                                // If it is not the first to be magnetized, keep the
                                // conductor the same down the chain.
                                conductor: if let Some(original_conductor) = magnet.conductor {
                                    Some(original_conductor)
                                } else {
                                    Some(conductor_entity)
                                },
                            });
                        // Either add to the conductor's train, or create a new train
                        // if the original creature is starting a new tail.
                        if let Some(flags_with_magnetized) = flags_with_magnetized {
                            let mut magnetized_query = magnetized_set.p2();
                            let mut magnetized_component =
                                magnetized_query.get_mut(flags_with_magnetized).unwrap();
                            magnetized_component.train.push(*adjacent_creature);
                        } else {
                            commands.entity(flag_entity).insert(Magnetized {
                                train: vec![*adjacent_creature],
                                species: magnet.species,
                            });
                        }
                        // TODO: Rerun this system with recursion, or give it a "flood"
                        // loop to find more segments, as there might still be candidates.
                        // This currently runs every frame, so it might be barely noticeable.
                    }
                }
            }
        }
    }
}

#[derive(Event)]
pub struct TeleportEntity {
    pub destination: Position,
    pub entity: Entity,
}

impl TeleportEntity {
    pub fn new(entity: Entity, x: i32, y: i32) -> Self {
        Self {
            destination: Position::new(x, y),
            entity,
        }
    }
}

pub fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut creature: Query<(&mut Position, &CreatureFlags)>,
    intangible_query: Query<&Intangible>,
    immobile_query: Query<&Immobile>,
    magnet_query: Query<&Magnetized>,
    charm_query: Query<&Charm>,
    mut map: ResMut<Map>,
    mut commands: Commands,
    mut collision: EventWriter<CreatureCollision>,
    mut stepped: EventWriter<SteppedOnTile>,
    mut contingency: EventWriter<TriggerContingency>,
    mut magnet: EventWriter<MagnetFollow>,
    is_player: Query<Has<Player>>,
) {
    for event in events.read() {
        let (mut creature_position, creature_flags) = creature
            // Get the Position of the Entity targeted by TeleportEntity.
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        let (is_intangible, is_immobile, is_magnetized, is_charmed) = {
            (
                intangible_query.contains(creature_flags.species_flags)
                    || intangible_query.contains(creature_flags.effects_flags),
                immobile_query.contains(creature_flags.species_flags)
                    || immobile_query.contains(creature_flags.effects_flags),
                magnet_query.contains(creature_flags.species_flags)
                    || magnet_query.contains(creature_flags.effects_flags),
                charm_query.contains(creature_flags.species_flags)
                    || charm_query.contains(creature_flags.effects_flags),
            )
        };
        // If motion is possible...
        if !is_immobile
            && (map.is_passable(event.destination.x, event.destination.y) || is_intangible)
        {
            if !is_intangible {
                // ...update the Map to reflect this...
                map.move_creature(*creature_position, event.destination);
            }
            // Magnetized creatures will have their tail follow them.
            if is_magnetized {
                magnet.send(MagnetFollow {
                    old_pos: *creature_position,
                    conductor: event.entity,
                });
            }
            // ...and move that Entity to TeleportEntity's destination tile.
            creature_position.update(event.destination.x, event.destination.y);
            // Also, animate this creature, making its teleport action visible on the screen.
            commands.entity(event.entity).insert(SlideAnimation);
            // The creature steps on its destination tile, triggering traps there.
            stepped.send(SteppedOnTile {
                entity: event.entity,
                position: event.destination,
            });
            // This triggers the "when moved" contingency.
            contingency.send(TriggerContingency {
                caster: event.entity,
                contingency: Axiom::WhenMoved,
            });
            if is_player.get(event.entity).unwrap() {
                commands.run_system_cached(toggle_paint_mode);
            }
        } else if let Some(collided_with) =
            map.get_entity_at(event.destination.x, event.destination.y)
        {
            // A creature collides with another entity.
            let (culprit_is_player, collided_is_player) = (
                is_player.get(event.entity).unwrap(),
                is_player.get(*collided_with).unwrap(),
            );
            // Only collide if one of the two creature is the player, or if the
            // creature is charmed and is NOT attacking the player.
            if (culprit_is_player || collided_is_player) || (is_charmed && !collided_is_player) {
                collision.send(CreatureCollision {
                    culprit: event.entity,
                    collided_with: *collided_with,
                });
            }
        }
    }
}

#[derive(Event)]
pub struct MagnetFollow {
    pub old_pos: Position,
    pub conductor: Entity,
}

/// All creatures with a tail following them (magnetized entities)
/// will have their "train" follow along with their moves.
pub fn magnet_follow(
    mut magnet_set: ParamSet<(Query<Entity, With<Magnetized>>, Query<&mut Magnetized>)>,
    position: Query<&Position>,
    mut teleport: EventWriter<TeleportEntity>,
    mut events: EventReader<MagnetFollow>,
    mut commands: Commands,
    flags_query: Query<&CreatureFlags>,
) {
    for event in events.read() {
        // Get all train conductors (first in line)
        let conductor_flags = flags_query.get(event.conductor).unwrap();
        let magnet_finder = magnet_set.p0();
        let flags_with_magnetized = magnet_finder
            .get(conductor_flags.species_flags)
            .or(magnet_finder.get(conductor_flags.effects_flags))
            .unwrap();
        let mut magnet = magnet_set.p1();
        let mut magnet = magnet.get_mut(flags_with_magnetized).unwrap();
        let mut train_idx = 0;
        // new_pos is the position the conductor currently occupies,
        // and old_pos was before their last move.
        let mut new_pos = *position.get(event.conductor).unwrap();
        let mut old_pos = event.old_pos;
        // Every tail segment must be processed, stop when no more remain.
        let mut should_keep_looping = true;
        while should_keep_looping {
            // Predict the snake's movement to get to its new position.
            // Reversed as the snake starts at the new position.
            // TODO: This currently disregards walls, check if this is a problem,
            // otherwise, replace with an actual pathfinding function.
            let mut walk = walk_grid(new_pos, old_pos);
            // If the snake didn't move, do not proceed.
            if walk.len() <= 1 {
                break;
            }
            // Teleport each tail segment on the positions behind the
            // conductor, following the line of "walk".
            for tile in walk.iter().skip(1) {
                if position.get(magnet.train[train_idx]).is_err() {
                    // Do not allow non-existent tail segments to continue the train.

                    // If it wasn't the last tail segment, remove Magnetic
                    // from the last tail segment.
                    // If it was the last tail segment, then Magnetic is already
                    // removed as a result.
                    if train_idx != magnet.train.len() - 1 {
                        if let Ok(magnetic_tail) =
                            flags_query.get(magnet.train[magnet.train.len() - 1])
                        {
                            commands
                                .entity(magnetic_tail.effects_flags)
                                .remove::<Magnetic>();
                        }
                    }
                    // Cut off the tail.
                    magnet.train.truncate(train_idx);

                    // If the snake no longer has a tail, do not proceed.
                    if magnet.train.is_empty() {
                        commands
                            .entity(flags_with_magnetized)
                            .remove::<Magnetized>();
                        commands.entity(flags_with_magnetized).insert(Magnetic {
                            species: magnet.species,
                            conductor: None,
                        });
                        should_keep_looping = false;
                        break;
                    }

                    let new_magnetic_entity = flags_query
                        .get(magnet.train[magnet.train.len() - 1])
                        .unwrap()
                        .effects_flags;
                    commands.entity(new_magnetic_entity).insert(Magnetic {
                        species: magnet.species,
                        conductor: Some(event.conductor),
                    });
                    should_keep_looping = false;
                    break;
                }
                teleport.send(TeleportEntity {
                    destination: *tile,
                    entity: magnet.train[train_idx],
                });
                train_idx += 1;
                if train_idx >= magnet.train.len() {
                    should_keep_looping = false;
                    break;
                }
            }
            if should_keep_looping {
                // If the snake's movement wasn't big enough to fit in all
                // the segments, make the last moved segment into the new conductor.
                new_pos = walk.pop().unwrap();
                old_pos = *position.get(magnet.train[train_idx - 1]).unwrap();
            }
        }
    }
}

#[derive(Event)]
pub struct SteppedOnTile {
    entity: Entity,
    position: Position,
}

pub fn stepped_on_tile(
    mut events: EventReader<SteppedOnTile>,
    mut contingency: EventWriter<TriggerContingency>,
    mut drop: EventWriter<TakeOrDropSoul>,
    mut remove: EventWriter<RemoveCreature>,
    stepped_on_creatures: Query<(Entity, &Position, &CreatureFlags)>,
    fragile: Query<&Fragile>,
    crafting_slot: Query<&CraftingSlot>,
    paint: Res<CagePainter>,
    is_player: Query<Has<Player>>,
) {
    for event in events.read() {
        for (entity, position, flags) in stepped_on_creatures.iter() {
            let (is_fragile, is_crafting_slot) = (
                fragile.contains(flags.species_flags) || fragile.contains(flags.effects_flags),
                crafting_slot.contains(flags.species_flags)
                    || crafting_slot.contains(flags.effects_flags),
            ); // If an entity is at the Position that was stepped on and isn't the creature
               // responsible for stepping...
            if event.position == *position && entity != event.entity {
                // Traps trigger their spell effect when stepped on.
                contingency.send(TriggerContingency {
                    caster: entity,
                    contingency: Axiom::WhenSteppedOn,
                });
                // Fragile floor entities are destroyed when stepped on.
                if is_fragile {
                    remove.send(RemoveCreature { entity });
                }
                // If the player steps on a Soul Cage, start painting in it.
                if is_crafting_slot && is_player.get(event.entity).unwrap() {
                    drop.send(TakeOrDropSoul {
                        position: *position,
                        soul: paint.current_paint,
                    });
                }
            }
        }
    }
}

#[derive(Event)]
pub struct CreatureCollision {
    culprit: Entity,
    collided_with: Entity,
}

pub fn creature_collision(
    mut events: EventReader<CreatureCollision>,
    mut harm: EventWriter<DamageOrHealCreature>,
    mut text: EventWriter<AddMessage>,
    stab_query: Query<&Stab>,
    species_query: Query<&Species>,
    meleeproof_query: Query<&Meleeproof>,
    mut turn_manager: ResMut<TurnManager>,
    mut creature: Query<(&mut Transform, Has<Player>, &CreatureFlags)>,
    flags_query: Query<&CreatureFlags>,
    mut commands: Commands,
    mut effects: Query<&mut StatusEffectsList>,
    position: Query<&Position>,
    mut draw_soul: EventWriter<DrawSoul>,
) {
    for event in events.read() {
        if event.culprit == event.collided_with {
            // No colliding with yourself.
            continue;
        }
        let (mut attacker_transform, is_player, flags) = creature.get_mut(event.culprit).unwrap();
        let cannot_be_melee_attacked = {
            let defender_flags = flags_query.get(event.collided_with).unwrap();
            meleeproof_query.contains(defender_flags.species_flags)
                || meleeproof_query.contains(defender_flags.effects_flags)
        };
        // if is_door {
        // Open doors.
        // NOTE: Disabled as doors are currently automatic.
        // open.send(OpenCloseDoor {
        //     entity: event.collided_with,
        //     open: true,
        // });
        // }
        // else
        if !cannot_be_melee_attacked {
            let damage = if let Ok(stab) = {
                stab_query
                    .get(flags.species_flags)
                    .or(stab_query.get(flags.effects_flags))
            } {
                // Attacking something with Stab active resets the Stab bonus.
                let mut status_effects = effects.get_mut(event.culprit).unwrap();
                status_effects
                    .effects
                    .get_mut(&StatusEffect::Stab)
                    .unwrap()
                    .stacks = EffectDuration::Finite { stacks: 0 };
                -1 - stab.bonus_damage
            } else {
                -1
            };
            // Melee attack.
            harm.send(DamageOrHealCreature {
                entity: event.collided_with,
                culprit: event.culprit,
                hp_mod: damage,
            });
            if is_player {
                draw_soul.send(DrawSoul { amount: 1 });
            }
            // Melee attack animation.
            // This must be calculated and cannot be "momentum", it has not been altered yet.
            let atk_pos = position.get(event.culprit).unwrap();
            let def_pos = position.get(event.collided_with).unwrap();
            attacker_transform.translation.x += (def_pos.x - atk_pos.x) as f32 * TILE_SIZE / 4.;
            attacker_transform.translation.y += (def_pos.y - atk_pos.y) as f32 * TILE_SIZE / 4.;
            commands.entity(event.culprit).insert(SlideAnimation);
        } else if matches!(turn_manager.action_this_turn, PlayerAction::Step) && is_player {
            // The player spent their turn walking into a wall, disallow the turn from ending.
            text.send(AddMessage {
                message: Message::InvalidAction(InvalidAction::CannotMelee(
                    *species_query.get(event.collided_with).unwrap(),
                )),
            });
            turn_manager.action_this_turn = PlayerAction::Invalid;
        }
    }
}

#[derive(Event)]
pub struct AlterMomentum {
    pub entity: Entity,
    pub direction: OrdDir,
}

pub fn alter_momentum(
    mut events: EventReader<AlterMomentum>,
    mut creature: Query<(&mut OrdDir, &mut Transform, &Children)>,
    mut hp_bar: Query<&mut Transform, Without<OrdDir>>,
    turn_manager: Res<TurnManager>,
) {
    for event in events.read() {
        // Don't allow changing your momentum by stepping into walls.
        if matches!(turn_manager.action_this_turn, PlayerAction::Invalid) {
            return;
        }
        let (mut creature_momentum, mut creature_transform, children) =
            creature.get_mut(event.entity).unwrap();
        *creature_momentum = event.direction;
        match event.direction {
            OrdDir::Down => creature_transform.rotation = Quat::from_rotation_z(0.),
            OrdDir::Right => creature_transform.rotation = Quat::from_rotation_z(PI / 2.),
            OrdDir::Up => creature_transform.rotation = Quat::from_rotation_z(PI),
            OrdDir::Left => creature_transform.rotation = Quat::from_rotation_z(3. * PI / 2.),
        }
        // Keep the HP bar on the bottom.
        for child in children.iter() {
            let mut hp_transform = hp_bar.get_mut(*child).unwrap();
            match event.direction {
                OrdDir::Down => hp_transform.rotation = Quat::from_rotation_z(0.),
                OrdDir::Right => hp_transform.rotation = Quat::from_rotation_z(3. * PI / 2.),
                OrdDir::Up => hp_transform.rotation = Quat::from_rotation_z(PI),
                OrdDir::Left => hp_transform.rotation = Quat::from_rotation_z(PI / 2.),
            }
        }
    }
}

#[derive(Event)]
pub struct DamageOrHealCreature {
    pub entity: Entity,
    pub culprit: Entity,
    pub hp_mod: isize,
}

pub fn harm_creature(
    mut events: EventReader<DamageOrHealCreature>,
    mut remove: EventWriter<RemoveCreature>,
    mut creature: Query<(&mut Health, &Children, &CreatureFlags)>,
    mut hp_bar: Query<(&mut Visibility, &mut Sprite)>,
    defender_flags: Query<&Invincible>,
    mut contingency: EventWriter<TriggerContingency>,
    mut text: EventWriter<AddMessage>,
    text_query: Query<(&Species, Has<Player>)>,
) {
    for event in events.read() {
        let (mut health, children, flags) = creature.get_mut(event.entity).unwrap();
        let is_invincible = defender_flags.contains(flags.effects_flags)
            || defender_flags.contains(flags.species_flags);
        let (culprit_species, culprit_is_player) = text_query.get(event.culprit).unwrap();
        let (victim_species, victim_is_player) = text_query.get(event.entity).unwrap();
        // Apply damage or healing.
        match event.hp_mod.signum() {
            -1 => {
                if is_invincible {
                    if victim_is_player {
                        text.send(AddMessage {
                            message: Message::PlayerIsInvincible(*culprit_species),
                        });
                    }
                    continue;
                }

                if culprit_is_player {
                    text.send(AddMessage {
                        message: Message::PlayerAttack(*victim_species, -event.hp_mod),
                    });
                } else if victim_is_player {
                    text.send(AddMessage {
                        message: Message::HostileAttack(*culprit_species, -event.hp_mod),
                    });
                } else {
                    text.send(AddMessage {
                        message: Message::NoPlayerAttack(
                            *culprit_species,
                            *victim_species,
                            -event.hp_mod,
                        ),
                    });
                }

                health.hp = health.hp.saturating_sub((-event.hp_mod) as usize);
                contingency.send(TriggerContingency {
                    caster: event.culprit,
                    contingency: Axiom::WhenDealingDamage,
                });
                contingency.send(TriggerContingency {
                    caster: event.entity,
                    contingency: Axiom::WhenTakingDamage,
                });
            } // Damage
            1 => {
                // Do not heal above max HP.
                if health.hp == health.max_hp {
                    continue;
                }
                let health_difference = health.hp;
                health.hp = min(
                    health.hp.saturating_add(event.hp_mod as usize),
                    health.max_hp,
                );
                let health_difference = (health.hp - health_difference) as isize;
                if victim_is_player {
                    text.send(AddMessage {
                        message: Message::HealSelf(health_difference),
                    });
                } else if culprit_is_player {
                    text.send(AddMessage {
                        message: Message::HealOther(*victim_species, health_difference),
                    });
                } else {
                    text.send(AddMessage {
                        message: Message::CreatureHealsItself(*victim_species, health_difference),
                    });
                }
            } // Healing
            _ => (), // 0 values do nothing
        }
        // Update the healthbar.
        for child in children.iter() {
            let (mut hp_vis, mut hp_bar) = hp_bar.get_mut(*child).unwrap();
            // Don't show the healthbar at full hp.
            (*hp_vis, hp_bar.texture_atlas.as_mut().unwrap().index) =
                hp_bar_visibility_and_index(health.hp, health.max_hp);
        }
        // 0 hp creatures are removed.
        if health.hp == 0 {
            remove.send(RemoveCreature {
                entity: event.entity,
            });
        }
    }
}

#[derive(Event)]
pub struct OpenCloseDoor {
    pub entity: Entity,
    pub open: bool,
}

#[derive(Component)]
pub struct BecomingVisible {
    timer: Timer,
}

#[derive(Component)]
pub struct DoorPanel;

pub fn open_close_door(
    mut events: EventReader<OpenCloseDoor>,
    mut commands: Commands,
    mut door: Query<(&mut Visibility, &Position, &OrdDir, &CreatureFlags)>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    for event in events.read() {
        // Gather component values of the door.
        // NOTE: Wrapped in Ok - if the player defeats a room and dies at the same time,
        // the success check will try to open the door while the failure check is trying
        // to delete them.
        if let Ok((mut visibility, position, orientation, flags)) = door.get_mut(event.entity) {
            if event.open {
                // The door becomes intangible, and can be walked through.
                commands.entity(flags.species_flags).insert(Intangible);
                // The door is no longer visible, as it is open.
                *visibility = Visibility::Hidden;
            } else {
                commands.entity(flags.species_flags).remove::<Intangible>();
                commands.entity(event.entity).insert(BecomingVisible {
                    timer: Timer::from_seconds(0.4, TimerMode::Once),
                });
            }
            // Find the direction in which the door was facing to play its animation correctly.
            let (offset_1, offset_2) = match orientation {
                OrdDir::Up | OrdDir::Down => (OrdDir::Left.as_offset(), OrdDir::Right.as_offset()),
                OrdDir::Right | OrdDir::Left => (OrdDir::Down.as_offset(), OrdDir::Up.as_offset()),
            };
            // Loop twice: for each pane of the door.
            for offset in [offset_1, offset_2] {
                commands.spawn((
                    DoorPanel,
                    // The sliding panes are represented as a MagicEffect with a very slow decay.
                    MagicEffect {
                        // The panes slide into the adjacent walls to the door, hence the offset.
                        position: if event.open {
                            Position::new(position.x + offset.0, position.y + offset.1)
                        } else {
                            *position
                        },
                        sprite: Sprite {
                            image: asset_server.load("spritesheet.png"),
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                            texture_atlas: Some(TextureAtlas {
                                layout: atlas_layout.handle.clone(),
                                index: get_effect_sprite(&EffectType::Airlock),
                            }),
                            ..default()
                        },
                        visibility: Visibility::Inherited,
                        vfx: MagicVfx {
                            appear: Timer::from_seconds(0., TimerMode::Once),
                            // Very slow decay - the alpha shouldn't be reduced too much
                            // while the panes are still visible.
                            decay: Timer::from_seconds(5.0, TimerMode::Once),
                        },
                    },
                    // Ensure the panes are sliding.
                    SlideAnimation,
                    Transform {
                        translation: if event.open {
                            Vec3 {
                                x: position.x as f32 * TILE_SIZE,
                                y: position.y as f32 * TILE_SIZE,
                                // The pane needs to hide under actual tiles, such as walls.
                                z: -1.,
                            }
                        } else {
                            Vec3 {
                                x: (position.x + offset.0) as f32 * TILE_SIZE,
                                y: (position.y + offset.1) as f32 * TILE_SIZE,
                                // The pane needs to hide under actual tiles, such as walls.
                                z: -1.,
                            }
                        },
                        // Adjust the pane's rotation with its door.
                        rotation: Quat::from_rotation_z(match orientation {
                            OrdDir::Down => 0.,
                            OrdDir::Right => PI / 2.,
                            OrdDir::Up => PI,
                            OrdDir::Left => 3. * PI / 2.,
                        }),
                        scale: Vec3::new(1., 1., 1.),
                    },
                ));
            }
        }
    }
}

pub fn render_closing_doors(
    mut commands: Commands,
    mut becoming_visible: Query<(Entity, &mut BecomingVisible, &mut Visibility)>,
    time: Res<Time>,
    door_panes: Query<Entity, With<DoorPanel>>,
) {
    for (entity, mut door_timer, mut door_vis) in becoming_visible.iter_mut() {
        door_timer.timer.tick(time.delta());
        if door_timer.timer.finished() {
            *door_vis = Visibility::Inherited;
            commands.entity(entity).remove::<BecomingVisible>();
            for pane in door_panes.iter() {
                commands.entity(pane).try_despawn();
            }
        }
    }
}

#[derive(Event, Debug)]
pub struct RemoveCreature {
    pub entity: Entity,
}

pub fn remove_creature(
    mut events: EventReader<RemoveCreature>,
    mut commands: Commands,
    creature: Query<(&Position, &Soul, Has<Player>, &CreatureFlags)>,
    dying_flags: Query<&NoDropSoul>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut soul_wheel: ResMut<SoulWheel>,
    mut contingency: EventWriter<TriggerContingency>,
    mut respawn: EventWriter<RespawnPlayer>,
) {
    let mut seen = HashSet::new();
    // NOTE: This filter prevents double-removal of a single entity by removing duplicates.
    // As an example, this can happen if two Shrikes simultaneously attack the player.
    for event in events.read().filter(|e| seen.insert(e.entity)) {
        // HACK: This panicked once for seemingly no good reason. It has been changed
        // to if let Ok instead of unwrap(), hoping to see the weird behaviour in game.
        if let Ok((position, soul, is_player, flags)) = creature.get(event.entity) {
            // Visually flash an X where the creature was removed.
            magic_vfx.send(PlaceMagicVfx {
                targets: vec![*position],
                sequence: EffectSequence::Simultaneous,
                effect: EffectType::XCross,
                decay: 0.5,
                appear: 0.,
            });
            let cannot_drop_soul = dying_flags.contains(flags.effects_flags)
                || dying_flags.contains(flags.species_flags);
            // For now, avoid removing the player - the game panics without a player.
            if !is_player {
                // Add Dizzy to prevent this creature from taking any further actions.
                commands
                    .entity(event.entity)
                    .insert((DesignatedForRemoval, Dizzy));
                // This triggers the "when removed" contingency.
                contingency.send(TriggerContingency {
                    caster: event.entity,
                    contingency: Axiom::WhenRemoved,
                });
                if !cannot_drop_soul && soul != &Soul::Empty {
                    // Add this entity's soul to the soul wheel
                    soul_wheel
                        .draw_pile
                        .entry(*soul)
                        .and_modify(|amount| *amount += 1);
                }
            } else {
                respawn.send(RespawnPlayer { victorious: false });
            }
        } else {
            info!("A RemoveEntity failed to fetch components from its Entity.");
            dbg!(event);
        }
    }
}

#[derive(Event)]
pub struct RespawnPlayer {
    pub victorious: bool,
}

pub fn respawn_player(
    mut events: EventReader<RespawnPlayer>,
    npcs: Query<
        Entity,
        (
            With<Species>,
            Without<Player>,
            Without<DesignatedForRemoval>,
        ),
    >,
    mut player: Query<(Entity, &mut Spellbook), With<Player>>,
    souls_to_delete: Query<Entity, Or<(With<DroppedSoul>, With<LibrarySlot>, With<KnownPattern>)>>,
    mut slots: Query<(&mut ImageNode, &EquipSlot), Without<LibrarySlot>>,
    mut spell_library: ResMut<SpellLibrary>,
    mut loot: ResMut<BagOfLoot>,
    mut commands: Commands,
    mut remove: EventWriter<RemoveCreature>,
    mut heal: EventWriter<DamageOrHealCreature>,
    mut teleport: EventWriter<TeleportEntity>,
    mut title: EventWriter<AnnounceGameOver>,
    mut cage: EventWriter<RespawnCage>,
    mut transform: EventWriter<TransformCreature>,
    mut soul_wheel: ResMut<SoulWheel>,
    mut faiths_end: ResMut<FaithsEnd>,
) {
    for event in events.read() {
        for npc in npcs.iter() {
            remove.send(RemoveCreature { entity: npc });
        }
        for delete in souls_to_delete.iter() {
            commands.entity(delete).despawn();
        }
        spell_library.library.clear();
        *loot = BagOfLoot::get_initial();
        let (player, mut spellbook) = player.single_mut();
        // Reset the player's HP.
        heal.send(DamageOrHealCreature {
            entity: player,
            culprit: player,
            hp_mod: 6,
        });
        // Return the player to the start.
        teleport.send(TeleportEntity {
            destination: Position::new(4, 4),
            entity: player,
        });
        // Ensure the player is back to its initial form.
        transform.send(TransformCreature {
            entity: player,
            new_species: Species::Player,
        });
        // Reset the player's spellbook.
        *spellbook = get_species_spellbook(&Species::Player);
        for (mut node, slot) in slots.iter_mut() {
            node.texture_atlas.as_mut().unwrap().index = get_soul_sprite(&slot.0);
            node.color.set_alpha(1.);
        }
        soul_wheel.draw_pile.insert(Soul::Saintly, 1);
        soul_wheel.draw_pile.insert(Soul::Ordered, 1);
        soul_wheel.draw_pile.insert(Soul::Artistic, 1);
        soul_wheel.draw_pile.insert(Soul::Unhinged, 1);
        soul_wheel.draw_pile.insert(Soul::Feral, 1);
        soul_wheel.draw_pile.insert(Soul::Vile, 1);
        faiths_end.cage_address_position.clear();
        faiths_end.current_cage = 0;
        cage.send(RespawnCage);
        title.send(AnnounceGameOver {
            victorious: event.victorious,
        });
    }
}

/// This is done separately, and ONLY once the spell stack is empty, to avoid crashes
/// where the game tries to fetch a spellcaster's position that stopped existing.
// NOTE: If dead entities start having their sprite lingering during spell animations,
// set their visibility to hidden in remove_creature.
pub fn remove_designated_creatures(
    remove: Query<(Entity, &CreatureFlags), With<DesignatedForRemoval>>,
    mut commands: Commands,
    mut map: ResMut<Map>,
    position: Query<&Position>,
) {
    for (designated, designated_flags) in remove.iter() {
        // Remove the creature from Map
        let position = position.get(designated).unwrap();
        if let Some(preexisting_entity) = map.creatures.get(position) {
            // Check that the entity being removed is actually the dead entity.
            // REASON: Dying intangible creatures which are on top of a tangible
            // creature will remove the tangible creature from the map instead
            // of themselves.
            if *preexisting_entity == designated {
                map.creatures.remove(position);
            }
        }
        // Remove the creature AND its children (health bar)
        commands.entity(designated).despawn_recursive();
        commands
            .entity(designated_flags.species_flags)
            .despawn_recursive();
        commands
            .entity(designated_flags.effects_flags)
            .despawn_recursive();
    }
}

#[derive(Event)]
pub struct EndTurn;

pub fn end_turn(
    mut events: EventReader<EndTurn>,
    mut npc_actions: EventWriter<DistributeNpcActions>,
    mut turn_manager: ResMut<TurnManager>,
    mut commands: Commands,
    mut loop_protection: ResMut<AntiContingencyLoop>,
    player_flags: Query<&CreatureFlags, With<Player>>,
    speed_query: Query<&Speed>,
) {
    for _event in events.read() {
        // The player shouldn't be allowed to "wait" turns by stepping into walls.
        if matches!(
            turn_manager.action_this_turn,
            PlayerAction::Invalid | PlayerAction::Skipped
        ) {
            // NOTE: Disabled because I ended up disliking this effect.
            // if matches!(turn_manager.action_this_turn, PlayerAction::Invalid) {
            //     screenshake.intensity = 3;
            // }
            return;
        }

        let flags = player_flags.single();
        let speed_level = if let Ok(player_speed) = speed_query
            .get(flags.effects_flags)
            .or(speed_query.get(flags.species_flags))
        {
            match player_speed {
                // TODO: This is deceptive: if the player is perma-fast,
                // they don't get "2 actions per turn", they just get to
                // act while everything BUT creatures with the same or
                // superior speed level are frozen.
                Speed::Fast { actions_per_turn } => *actions_per_turn,
                // TODO: The player being slowed currently has no effect whatsoever.
                Speed::Slow { wait_turns: _ } => 1,
            }
        } else {
            1
        };
        // Clear the anti-contingency infinite loop filter.
        loop_protection.contingencies_this_turn.clear();

        // NOTE: This might have some strange behaviours due to how
        // effects are ticked down between player turn and npc turn.
        // Maybe I should find a way to make this happen after everyone's
        // turn.
        commands.run_system_cached(room_circulation_check);
        commands.run_system_cached(tick_down_status_effects);

        // The turncount increases.
        turn_manager.turn_count += 1;
        npc_actions.send(DistributeNpcActions { speed_level });
    }
}

fn start_possessing_creature(
    added: Trigger<OnAdd, Possessed>,
    possessed: Query<&Possessed>,
    flag_entity: Query<&FlagEntity>,
    possessing: Query<Entity, With<Possessing>>,
    player: Query<Has<Player>>,
    mut commands: Commands,
) {
    // Do not allow possessing multiple creatures at once.
    // NOTE: It could be fun to "sync" their movements if this
    // happens instead of just returning.
    if possessing.get_single().is_ok() {
        return;
    }
    let possessed_creature_flags = added.entity();
    let possessed_creature = flag_entity
        .get(possessed_creature_flags)
        .unwrap()
        .parent_creature;
    let culprit = possessed.get(possessed_creature_flags).unwrap().original;
    if player.get(culprit).unwrap() {
        commands.entity(culprit).remove::<Player>();
        commands.entity(possessed_creature).insert(Player);
        commands.entity(culprit).insert(Possessing);
    }
}

fn stop_possessing_creature(
    removed: Trigger<OnRemove, Possessed>,
    possessing: Query<Entity, With<Possessing>>,
    flag_entity: Query<&FlagEntity>,
    mut commands: Commands,
) {
    let possessed_creature_flags = removed.entity();
    let possessed_creature = flag_entity
        .get(possessed_creature_flags)
        .unwrap()
        .parent_creature;
    // HACK: Possession "chains" will not be pretty with this, as
    // it assumes there is only one possession happening at a time.
    if let Ok(culprit) = possessing.get_single() {
        commands.entity(possessed_creature).remove::<Player>();
        commands.entity(culprit).insert(Player);
    }
}

fn room_circulation_check(
    awake_creatures: Query<&Awake>,
    sleeping_creatures: Query<Entity, With<Sleeping>>,
    mut faiths_end: ResMut<FaithsEnd>,
    player_position: Query<&Position, With<Player>>,
    flags_query: Query<(Entity, &CreatureFlags)>,
    open_door_query: Query<&Door, With<Intangible>>,
    mut open: EventWriter<OpenCloseDoor>,
    mut respawn: EventWriter<RespawnPlayer>,
    mut status_effect: EventWriter<AddStatusEffect>,
    mut learn: EventWriter<LearnNewAxiom>,
    mut loot: ResMut<BagOfLoot>,
    mut commands: Commands,
) {
    // Victory check.
    if sleeping_creatures.is_empty() && awake_creatures.is_empty() {
        respawn.send(RespawnPlayer { victorious: true });
    }
    // If the player has cleared a cage inside of faith's end, awaken all the
    // creatures in the next cage.
    else if let Some((mut boundary_a, mut boundary_b)) = faiths_end
        .cage_dimensions
        .get(&(faiths_end.current_cage + 1))
    {
        boundary_a.shift(1, 1);
        boundary_b.shift(-1, -1);
        if awake_creatures.is_empty()
            && player_position
                .get_single()
                .unwrap()
                .is_within_range(&boundary_a, &boundary_b)
        {
            faiths_end.current_cage += 1;
            if is_soul_cage_room(faiths_end.current_cage) {
                let extracted_axioms = loot.extract_axioms();
                for axiom in extracted_axioms {
                    learn.send(LearnNewAxiom { axiom });
                }
            }
            for (door, flags) in flags_query.iter() {
                if open_door_query.contains(flags.species_flags)
                    || open_door_query.contains(flags.effects_flags)
                {
                    open.send(OpenCloseDoor {
                        entity: door,
                        open: false,
                    });
                }
            }
            for sleeping_entity in sleeping_creatures.iter() {
                commands.entity(sleeping_entity).insert(Awake);
                commands.entity(sleeping_entity).remove::<Sleeping>();
                // Give one turn for the player to act.
                // This also prevents them from immediately moving
                // inside the closing doors.
                commands
                    .entity(flags_query.get(sleeping_entity).unwrap().1.effects_flags)
                    .insert(Dizzy);
                status_effect.send(AddStatusEffect {
                    entity: sleeping_entity,
                    effect: StatusEffect::Dizzy,
                    potency: 1,
                    stacks: EffectDuration::Finite { stacks: 1 },
                    culprit: sleeping_entity,
                });
            }
        }
    }
}

pub fn tick_down_status_effects(
    mut effects: Query<(Entity, &mut StatusEffectsList)>,
    flags_query: Query<(Entity, &CreatureFlags)>,
    mut commands: Commands,
) {
    // Tick down status effects.
    for (entity, mut effect_list) in effects.iter_mut() {
        for (effect, potency_and_stacks) in effect_list.effects.iter_mut() {
            if let EffectDuration::Finite { stacks } = &mut potency_and_stacks.stacks {
                *stacks = stacks.saturating_sub(1);
                if *stacks == 0 {
                    // Disable this effect.
                    potency_and_stacks.potency = 0;
                    let effects_flags = flags_query.get(entity).unwrap().1.effects_flags;
                    match effect {
                        StatusEffect::Invincible => {
                            commands.entity(effects_flags).remove::<Invincible>();
                        }
                        StatusEffect::Stab => {
                            commands.entity(effects_flags).remove::<Stab>();
                        }
                        StatusEffect::Dizzy => {
                            commands.entity(effects_flags).remove::<Dizzy>();
                        }
                        StatusEffect::DimensionBond => {
                            commands.entity(effects_flags).remove::<Summoned>();
                        }
                        StatusEffect::Haste => {
                            commands.entity(effects_flags).remove::<Speed>();
                        }
                        StatusEffect::Charm => {
                            commands.entity(effects_flags).remove::<Charm>();
                        }
                        StatusEffect::Possessed => {
                            commands.entity(effects_flags).remove::<Possessed>();
                        }
                        StatusEffect::Magnetize => {
                            commands.entity(effects_flags).remove::<Magnetic>();
                            commands.entity(effects_flags).remove::<Magnetized>();
                        }
                    }
                }
            }
        }
    }
}

#[derive(Event)]
pub struct DistributeNpcActions {
    pub speed_level: usize,
}

pub fn distribute_npc_actions(
    mut step: EventWriter<CreatureStep>,
    mut spell: EventWriter<CastSpell>,
    mut echo: EventWriter<EchoSpeed>,
    mut events: EventReader<DistributeNpcActions>,
    turn_manager: Res<TurnManager>,
    player: Query<&Position, With<Player>>,
    // TODO: This will break if Awake becomes a flag component
    // and is no longer stitched directly onto the creature.
    charm_position_query: Query<&Position, (With<Awake>, Without<Player>)>,
    npcs: Query<(Entity, &Position, &Species, &Spellbook, &CreatureFlags), Without<Player>>,
    species: Query<&Species>,
    map: Res<Map>,

    hunt_query: Query<&Hunt>,
    random_query: Query<&Random>,
    speed_query: Query<&Speed>,
    charm_query: Query<&Charm>,
    stunned_query: Query<Entity, Or<(With<Dizzy>, With<Sleeping>)>>,
) {
    for event in events.read() {
        let player_pos = player.get_single().unwrap();
        let mut send_echo = false;
        for (npc_entity, npc_pos, npc_species, npc_spellbook, flags) in npcs.iter() {
            let (is_hunter, is_random, is_stunned, is_charmed, speed) = {
                (
                    hunt_query.contains(flags.species_flags)
                        || hunt_query.contains(flags.effects_flags),
                    random_query.contains(flags.species_flags)
                        || random_query.contains(flags.effects_flags),
                    stunned_query.contains(flags.species_flags)
                        || stunned_query.contains(flags.effects_flags)
                        // HACK: The "Sleeping" component currently appears
                        // on the creature itself and not the effects_flags.
                        || stunned_query.contains(npc_entity),
                    charm_query.contains(flags.species_flags)
                        || charm_query.contains(flags.effects_flags),
                    // NOTE: Currently, status effect speed overrides species speed.
                    // Maybe it would be interesting to have them cancel each other out.
                    speed_query
                        .get(flags.effects_flags)
                        .or(speed_query.get(flags.species_flags)),
                )
            };
            if is_stunned {
                continue;
            }
            if let Ok(speed) = speed {
                match speed {
                    Speed::Slow { wait_turns } => {
                        if turn_manager.turn_count % (wait_turns + 1) != 0 || event.speed_level > 1
                        {
                            continue;
                        }
                    }
                    Speed::Fast { actions_per_turn } => {
                        if event.speed_level > *actions_per_turn {
                            continue;
                        } else {
                            send_echo = true;
                        }
                    }
                }
            } else if event.speed_level > 1 {
                continue;
            }
            if is_random {
                if let Some(move_direction) = map.random_adjacent_passable_direction(*npc_pos) {
                    // If it is found, cause a CreatureStep event.
                    step.send(CreatureStep {
                        direction: move_direction,
                        entity: npc_entity,
                    });
                }
            } else if is_hunter {
                // Occasionally cast a spell.
                if *npc_species == Species::Second {
                    let mut found_wall = false;
                    for adj_pos in map.get_adjacent_tiles(*npc_pos) {
                        if let Some(adjacent_npc) = map.creatures.get(&adj_pos) {
                            if *species.get(*adjacent_npc).unwrap() == Species::WeakWall {
                                spell.send(CastSpell {
                                    caster: npc_entity,
                                    spell: npc_spellbook.spells.get(&Soul::Vile).unwrap().clone(),
                                    starting_step: 0,
                                    soul_caste: Soul::Vile,
                                });
                                found_wall = true;
                                break;
                            }
                        }
                    }
                    if found_wall {
                        continue;
                    }
                }
                let destination = if is_charmed {
                    let mut closest_slot: Option<Position> = None;
                    let mut min_distance = i32::MAX;
                    for position in charm_position_query.iter() {
                        let distance = manhattan_distance(npc_pos, position);
                        if distance < min_distance {
                            min_distance = distance;
                            closest_slot = Some(position.clone());
                        }
                    }
                    if let Some(closest_slot) = closest_slot {
                        closest_slot
                    } else {
                        *npc_pos
                    }
                } else {
                    *player_pos
                };
                // Try to find a tile that gets the hunter closer to its target.
                if let Some(move_direction) = map.best_manhattan_move(*npc_pos, destination) {
                    // If it is found, cause a CreatureStep event.
                    step.send(CreatureStep {
                        direction: move_direction,
                        entity: npc_entity,
                    });
                }
            }
        }
        if send_echo {
            echo.send(EchoSpeed {
                speed_level: event.speed_level + 1,
            });
        }
    }
}

#[derive(Event)]
pub struct EchoSpeed {
    pub speed_level: usize,
}

pub fn echo_speed(
    mut events: EventReader<EchoSpeed>,
    mut end_turn: EventWriter<DistributeNpcActions>,
) {
    for event in events.read() {
        end_turn.send(DistributeNpcActions {
            speed_level: event.speed_level,
        });
    }
}

#[derive(Event)]
pub struct RespawnCage;

/// This is necessary to come last, as to ensure everything has despawned
/// before spawning the next batch of creatures.
pub fn respawn_cage(mut events: EventReader<RespawnCage>, mut commands: Commands) {
    // HACK: If multiple RespawnCage events are processed, it will build multiple
    // levels on top of each other, making the game unplayable.
    if events.read().count() > 0 {
        commands.run_system_cached(spawn_cage);
    }
}
