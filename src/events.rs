use std::{cmp::min, f32::consts::PI};

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use rand::{seq::IteratorRandom, thread_rng};

use crate::{
    creature::{
        get_soul_sprite, get_species_spellbook, get_species_sprite, is_naturally_intangible, Awake,
        Creature, DesignatedForRemoval, Dizzy, Door, EffectDuration, Fragile, Health,
        HealthIndicator, Hunt, Immobile, Intangible, Invincible, Meleeproof, NoDropSoul, Player,
        PotencyAndStacks, Random, Sleeping, Soul, Species, Speed, Spellbook, Spellproof, Stab,
        StatusEffect, StatusEffectsList, Summoned, Wall,
    },
    graphics::{
        get_effect_sprite, EffectSequence, EffectType, MagicEffect, MagicVfx, PlaceMagicVfx,
        SlideAnimation, SpriteSheetAtlas,
    },
    map::{spawn_cage, FaithsEnd, Map, Position},
    spells::{Axiom, CastSpell, TriggerContingency},
    ui::{AnnounceGameOver, SoulSlot},
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
        app.init_resource::<Events<CreatureStep>>();
        app.init_resource::<Events<RespawnCage>>();
        app.insert_resource(TurnManager {
            turn_count: 0,
            action_this_turn: PlayerAction::Invalid,
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
        soul_wheel.draw_pile.insert(Soul::Saintly, 1);
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
                    // There is nothing left in the draw pile!
                    turn_manager.action_this_turn = PlayerAction::Invalid;
                }
            } else {
                // There is no empty space in the Wheel!
                turn_manager.action_this_turn = PlayerAction::Invalid;
            }
        }
    }
}

#[derive(Event)]
pub struct UseWheelSoul {
    pub index: usize,
}

pub fn use_wheel_soul(
    mut events: EventReader<UseWheelSoul>,
    mut soul_wheel: ResMut<SoulWheel>,
    mut spell: EventWriter<CastSpell>,
    mut ui_soul_slots: Query<(&mut ImageNode, &SoulSlot)>,
    mut turn_manager: ResMut<TurnManager>,
    player: Query<(Entity, &Spellbook), With<Player>>,
) {
    for event in events.read() {
        let mut newly_discarded = None;
        if let Some(soul) = soul_wheel.souls.get(event.index).unwrap() {
            // Cast the spell corresponding to this soul type.
            let (player_entity, spellbook) = player.get_single().unwrap();
            spell.send(CastSpell {
                caster: player_entity,
                spell: spellbook.spells.get(soul).unwrap().clone(),
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
            // That soul slot is empty!
            turn_manager.action_this_turn = PlayerAction::Invalid;
        }
        // The spent soul is sent to the discard pile.
        if let Some(newly_discarded) = newly_discarded {
            soul_wheel
                .discard_pile
                .entry(newly_discarded)
                .and_modify(|amount| *amount += 1);
            if newly_discarded == Soul::Ordered {
                // TODO HACK: This makes the shield not take a turn. It should
                // probably be a "Timeless" axiom instead.
                turn_manager.action_this_turn = PlayerAction::Invalid;
            }
        }
    }
}

pub enum PlayerAction {
    Step,
    Spell,
    Draw,
    Invalid,
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
    mut effects: Query<&mut StatusEffectsList>,
    mut commands: Commands,
) {
    for event in events.read() {
        let mut effects_list = effects.get_mut(event.entity).unwrap();
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
        // Insert the components corresponding to the new status effect.
        match event.effect {
            StatusEffect::Invincible => {
                commands.entity(event.entity).insert(Invincible);
            }
            StatusEffect::Stab => {
                commands.entity(event.entity).insert(Stab {
                    bonus_damage: event.potency as isize,
                });
            }
            StatusEffect::Dizzy => {
                commands.entity(event.entity).insert(Dizzy);
            }
            StatusEffect::DimensionBond => {
                commands.entity(event.entity).insert(Summoned {
                    summoner: event.culprit,
                });
            }
        }
    }
}

#[derive(Event)]
pub struct SummonCreature {
    pub position: Position,
    pub species: Species,
    pub momentum: OrdDir,
    pub summoner_tile: Position,
    pub summoner: Option<Entity>,
    pub spellbook: Option<Spellbook>,
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
            // Wall-type creatures just get full HP to avoid displaying
            // their healthbar.
            _ => max_hp,
        };
        let mut new_creature = commands.spawn((
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
                momentum: event.momentum,
                health: Health { max_hp, hp },
                effects: StatusEffectsList {
                    effects: HashMap::new(),
                },
                soul: match &event.species {
                    Species::Player => Soul::Saintly,
                    Species::Wall => Soul::Ordered,
                    Species::WeakWall => Soul::Ordered,
                    Species::Hunter => Soul::Saintly,
                    Species::Shrike => Soul::Feral,
                    Species::Apiarist => Soul::Ordered,
                    Species::Tinker => Soul::Artistic,
                    Species::Second => Soul::Vile,
                    Species::Oracle => Soul::Unhinged,
                    _ => Soul::Unhinged,
                },
                spellbook: event
                    .spellbook
                    .clone()
                    .unwrap_or(get_species_spellbook(&event.species)),
            },
            Transform {
                translation: Vec3 {
                    x: event.summoner_tile.x as f32 * TILE_SIZE,
                    y: event.summoner_tile.y as f32 * TILE_SIZE,
                    z: 0.,
                },
                rotation: Quat::from_rotation_z(match event.momentum {
                    OrdDir::Down => 0.,
                    OrdDir::Right => PI / 2.,
                    OrdDir::Up => PI,
                    OrdDir::Left => 3. * PI / 2.,
                }),
                ..Default::default()
            },
            SlideAnimation,
        ));

        if let Some(summoner) = event.summoner {
            new_creature.insert(Summoned { summoner });
        }

        // If the map is "faith's end", log the cage address # of this creature.
        if let Some(cage_idx) = faiths_end
            .cage_address_position
            .get(&event.position)
            .copied()
        {
            // HACK: Walls being marked as Awake prevents the cage clear check,
            // as they must then be cleared as well to open the doors (this is impossible).
            if cage_idx != 0
                && [
                    Species::Shrike,
                    Species::Tinker,
                    Species::Oracle,
                    Species::Second,
                    Species::Hunter,
                    Species::Apiarist,
                ]
                .contains(&event.species)
            {
                new_creature.insert(Sleeping { cage_idx });
            } else if [
                Species::Shrike,
                Species::Tinker,
                Species::Oracle,
                Species::Second,
                Species::Hunter,
                Species::Apiarist,
            ]
            .contains(&event.species)
            {
                new_creature.insert(Awake);
            }
        }

        // Creatures which start out damaged show their HP bar in advance.
        let (visibility, index) = hp_bar_visibility_and_index(hp, max_hp);

        // Free the borrow on Commands.
        let new_creature_entity = new_creature.id();

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
    pub old_species: Species,
    pub new_species: Species,
}

pub fn transform_creature(
    mut transform: EventReader<TransformCreature>,
    mut creature_query: Query<(&mut Species, &mut Sprite, &StatusEffectsList)>,
    mut commands: Commands,
) {
    for event in transform.read() {
        let (mut species_of_creature, mut sprite, status_effects_list) =
            creature_query.get_mut(event.entity).unwrap();
        // Change the species.
        *species_of_creature = event.new_species;
        sprite.texture_atlas.as_mut().unwrap().index = get_species_sprite(&event.new_species);
        // Remove all components except for the basics of a Creature.
        // The appropriate ones will be readded by assign_species_components.
        // Refresh all status effects (without changing them), as to re-apply all
        // pertinent components.
        // HACK: This hardcoding (alongside assign_species_components) is very bad, but
        // I am not sure how to make it better.
        // NOTE: Maybe 2 Child entities, one with the species flag and the other with the
        // effect flags.
        match event.old_species {
            Species::Player => {
                commands.entity(event.entity).remove::<Player>();
            }
            Species::Trap => {
                commands.entity(event.entity).remove::<(
                    Meleeproof,
                    Spellproof,
                    Intangible,
                    Fragile,
                    Invincible,
                    NoDropSoul,
                )>();
            }
            Species::Wall => {
                commands.entity(event.entity).remove::<(
                    Meleeproof,
                    Spellproof,
                    Wall,
                    Invincible,
                    Dizzy,
                    NoDropSoul,
                )>();
            }
            Species::WeakWall => {
                commands
                    .entity(event.entity)
                    .remove::<(Meleeproof, Wall, Invincible, Dizzy, NoDropSoul)>();
            }
            Species::Airlock => {
                commands.entity(event.entity).remove::<(
                    Meleeproof,
                    Spellproof,
                    Door,
                    Invincible,
                    Dizzy,
                    NoDropSoul,
                )>();
            }
            Species::Hunter | Species::Spawner | Species::Second | Species::Oracle => {
                commands.entity(event.entity).remove::<Hunt>();
            }
            Species::Tinker => {
                commands.entity(event.entity).remove::<Random>();
            }
            Species::Abazon => {
                commands.entity(event.entity).remove::<(Immobile, Hunt)>();
            }
            Species::Apiarist => {
                commands.entity(event.entity).remove::<(Speed, Hunt)>();
            }
            Species::Shrike => {
                commands.entity(event.entity).remove::<(Speed, Hunt)>();
            }
        }
        if let Some(potency_and_stacks) = status_effects_list.effects.get(&StatusEffect::Invincible)
        {
            if potency_and_stacks.is_active() {
                commands.entity(event.entity).insert(Invincible);
            }
        }
        if let Some(potency_and_stacks) = status_effects_list.effects.get(&StatusEffect::Dizzy) {
            if potency_and_stacks.is_active() {
                commands.entity(event.entity).insert(Dizzy);
            }
        }
    }
}

/// Add any species-specific components.
pub fn assign_species_components(
    changed_species: Query<(Entity, &Species), Changed<Species>>,
    mut commands: Commands,
) {
    for (entity, species) in changed_species.iter() {
        let mut new_creature = commands.entity(entity);
        match species {
            Species::Player => {
                new_creature.insert(Player);
            }
            Species::Trap => {
                new_creature.insert((
                    Meleeproof, Spellproof, Intangible, Fragile, Invincible, NoDropSoul,
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
    mut creature: Query<(&mut Position, Has<Intangible>, Has<Immobile>)>,
    mut map: ResMut<Map>,
    mut commands: Commands,
    mut collision: EventWriter<CreatureCollision>,
    mut stepped: EventWriter<SteppedOnTile>,
    mut contingency: EventWriter<TriggerContingency>,
    is_player: Query<Has<Player>>,
) {
    for event in events.read() {
        let (mut creature_position, is_intangible, is_immobile) = creature
            // Get the Position of the Entity targeted by TeleportEntity.
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        // If motion is possible...
        if !is_immobile
            && (map.is_passable(event.destination.x, event.destination.y) || is_intangible)
        {
            if !is_intangible {
                // ...update the Map to reflect this...
                map.move_creature(*creature_position, event.destination);
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
        } else if let Some(collided_with) =
            map.get_entity_at(event.destination.x, event.destination.y)
        {
            // A creature collides with another entity.
            let (culprit_is_player, collided_is_player) = (
                is_player.get(event.entity).unwrap(),
                is_player.get(*collided_with).unwrap(),
            );
            // Only collide if one of the two creature is the player.
            // TODO: This will prevent allied creatures from attacking.
            if culprit_is_player || collided_is_player {
                collision.send(CreatureCollision {
                    culprit: event.entity,
                    collided_with: *collided_with,
                });
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
    mut remove: EventWriter<RemoveCreature>,
    stepped_on_creatures: Query<(Entity, &Position, Has<Fragile>), With<Fragile>>,
) {
    for event in events.read() {
        for (entity, position, is_fragile) in stepped_on_creatures.iter() {
            // If an entity is at the Position that was stepped on and isn't the creature
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
    attacker_flags: Query<&Stab>,
    defender_flags: Query<Has<Meleeproof>>,
    mut turn_manager: ResMut<TurnManager>,
    mut creature: Query<(&mut Transform, Has<Player>)>,
    mut commands: Commands,
    mut effects: Query<&mut StatusEffectsList>,
    position: Query<&Position>,
) {
    for event in events.read() {
        if event.culprit == event.collided_with {
            // No colliding with yourself.
            continue;
        }
        let cannot_be_melee_attacked = defender_flags.get(event.collided_with).unwrap();
        let (mut attacker_transform, is_player) = creature.get_mut(event.culprit).unwrap();
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
            let damage = if let Ok(stab) = attacker_flags.get(event.culprit) {
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
            // Melee attack animation.
            // This must be calculated and cannot be "momentum", it has not been altered yet.
            let atk_pos = position.get(event.culprit).unwrap();
            let def_pos = position.get(event.collided_with).unwrap();
            attacker_transform.translation.x += (def_pos.x - atk_pos.x) as f32 * TILE_SIZE / 4.;
            attacker_transform.translation.y += (def_pos.y - atk_pos.y) as f32 * TILE_SIZE / 4.;
            commands.entity(event.culprit).insert(SlideAnimation);
        } else if matches!(turn_manager.action_this_turn, PlayerAction::Step) && is_player {
            // The player spent their turn walking into a wall, disallow the turn from ending.
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
    mut creature: Query<(&mut Health, &Children)>,
    mut hp_bar: Query<(&mut Visibility, &mut Sprite)>,
    defender_flags: Query<Has<Invincible>>,
    mut contingency: EventWriter<TriggerContingency>,
) {
    for event in events.read() {
        let (mut health, children) = creature.get_mut(event.entity).unwrap();
        let is_invincible = defender_flags.get(event.entity).unwrap();
        // Apply damage or healing.
        match event.hp_mod.signum() {
            -1 => {
                if is_invincible {
                    continue;
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
                health.hp = min(
                    health.hp.saturating_add(event.hp_mod as usize),
                    health.max_hp,
                )
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
    entity: Entity,
    open: bool,
}

#[derive(Component)]
pub struct BecomingVisible {
    timer: Timer,
}

pub fn open_close_door(
    mut events: EventReader<OpenCloseDoor>,
    mut commands: Commands,
    mut door: Query<(&mut Visibility, &Position, &OrdDir)>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    for event in events.read() {
        // Gather component values of the door.
        let (mut visibility, position, orientation) = door.get_mut(event.entity).unwrap();
        if event.open {
            // The door becomes intangible, and can be walked through.
            commands.entity(event.entity).insert(Intangible);
            // The door is no longer visible, as it is open.
            *visibility = Visibility::Hidden;
        } else {
            commands.entity(event.entity).remove::<Intangible>();
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

pub fn render_closing_doors(
    mut commands: Commands,
    mut becoming_visible: Query<(Entity, &mut BecomingVisible, &mut Visibility)>,
    time: Res<Time>,
) {
    for (entity, mut door_timer, mut door_vis) in becoming_visible.iter_mut() {
        door_timer.timer.tick(time.delta());
        if door_timer.timer.finished() {
            *door_vis = Visibility::Inherited;
            commands.entity(entity).remove::<BecomingVisible>();
        }
    }
}

#[derive(Event)]
pub struct RemoveCreature {
    pub entity: Entity,
}

pub fn remove_creature(
    mut events: EventReader<RemoveCreature>,
    mut commands: Commands,
    creature: Query<(&Position, &Soul, Has<Player>, Has<NoDropSoul>)>,
    // mut spell_stack: ResMut<SpellStack>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut soul_wheel: ResMut<SoulWheel>,
    mut contingency: EventWriter<TriggerContingency>,
    mut respawn: EventWriter<RespawnPlayer>,
) {
    for event in events.read() {
        let (position, soul, is_player, cannot_drop_soul) = creature.get(event.entity).unwrap();
        // Visually flash an X where the creature was removed.
        magic_vfx.send(PlaceMagicVfx {
            targets: vec![*position],
            sequence: EffectSequence::Simultaneous,
            effect: EffectType::XCross,
            decay: 0.5,
            appear: 0.,
        });
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
            if !cannot_drop_soul {
                // Add this entity's soul to the soul wheel
                soul_wheel
                    .draw_pile
                    .entry(*soul)
                    .and_modify(|amount| *amount += 1);
            }
        } else {
            respawn.send(RespawnPlayer);
        }
    }
}

#[derive(Event)]
pub struct RespawnPlayer;

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
    player: Query<Entity, With<Player>>,
    mut remove: EventWriter<RemoveCreature>,
    mut heal: EventWriter<DamageOrHealCreature>,
    mut teleport: EventWriter<TeleportEntity>,
    mut cage: EventWriter<RespawnCage>,
    mut title: EventWriter<AnnounceGameOver>,
    mut soul_wheel: ResMut<SoulWheel>,
    mut faiths_end: ResMut<FaithsEnd>,
) {
    for _event in events.read() {
        for npc in npcs.iter() {
            remove.send(RemoveCreature { entity: npc });
        }
        let player = player.get_single().unwrap();
        heal.send(DamageOrHealCreature {
            entity: player,
            culprit: player,
            hp_mod: 6,
        });
        teleport.send(TeleportEntity {
            destination: Position::new(4, 4),
            entity: player,
        });
        soul_wheel.draw_pile.insert(Soul::Saintly, 1);
        soul_wheel.draw_pile.insert(Soul::Ordered, 1);
        soul_wheel.draw_pile.insert(Soul::Artistic, 1);
        soul_wheel.draw_pile.insert(Soul::Unhinged, 1);
        soul_wheel.draw_pile.insert(Soul::Feral, 1);
        soul_wheel.draw_pile.insert(Soul::Vile, 1);
        faiths_end.cage_address_position.clear();
        faiths_end.current_cage = 0;
        cage.send(RespawnCage);
        title.send(AnnounceGameOver { victorious: false });
    }
}

/// This is done separately, and ONLY once the spell stack is empty, to avoid crashes
/// where the game tries to fetch a spellcaster's position that stopped existing.
// NOTE: If dead entities start having their sprite lingering during spell animations,
// set their visibility to hidden in remove_creature.
pub fn remove_designated_creatures(
    remove: Query<Entity, With<DesignatedForRemoval>>,
    mut commands: Commands,
    mut map: ResMut<Map>,
    position: Query<&Position>,

    awake: Query<&Awake>,
    doors: Query<Entity, (With<Door>, Without<Intangible>)>,
    mut open: EventWriter<OpenCloseDoor>,
) {
    for designated in remove.iter() {
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
    }

    if awake.is_empty() {
        for door in doors.iter() {
            open.send(OpenCloseDoor {
                entity: door,
                open: true,
            });
        }
    }
}

#[derive(Event)]
pub struct EndTurn;

pub fn end_turn(
    mut events: EventReader<EndTurn>,
    mut npc_actions: EventWriter<DistributeNpcActions>,
    mut turn_manager: ResMut<TurnManager>,
    mut effects: Query<(Entity, &mut StatusEffectsList, &Species)>,
    mut commands: Commands,
    awake_creatures: Query<&Awake>,
    sleeping_creatures: Query<(Entity, &Sleeping), (Without<Wall>, Without<Door>)>,
    mut faiths_end: ResMut<FaithsEnd>,
    player_position: Query<&Position, With<Player>>,
    doors: Query<Entity, With<Door>>,
    mut open: EventWriter<OpenCloseDoor>,
) {
    for _event in events.read() {
        // The player shouldn't be allowed to "wait" turns by stepping into walls.
        if matches!(turn_manager.action_this_turn, PlayerAction::Invalid) {
            return;
        }

        // If the player has cleared a cage inside of faith's end, awaken all the
        // creatures in the next cage.
        if let Some((mut boundary_a, mut boundary_b)) = faiths_end
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
                for airlock in doors.iter() {
                    open.send(OpenCloseDoor {
                        entity: airlock,
                        open: false,
                    });
                }
                for (sleeping_entity, sleeping_component) in sleeping_creatures.iter() {
                    if sleeping_component.cage_idx == faiths_end.current_cage {
                        commands.entity(sleeping_entity).insert(Awake);
                        commands.entity(sleeping_entity).remove::<Sleeping>();
                    }
                }
            }
        }

        // The turncount increases.
        turn_manager.turn_count += 1;
        // Tick down status effects.
        for (entity, mut effect_list, species) in effects.iter_mut() {
            for (effect, potency_and_stacks) in effect_list.effects.iter_mut() {
                if let EffectDuration::Finite { stacks } = &mut potency_and_stacks.stacks {
                    *stacks = stacks.saturating_sub(1);
                    if *stacks == 0 {
                        // Disable this effect.
                        potency_and_stacks.potency = 0;
                        match effect {
                            StatusEffect::Invincible => {
                                commands.entity(entity).remove::<Invincible>();
                            }
                            StatusEffect::Stab => {
                                commands.entity(entity).remove::<Stab>();
                            }
                            StatusEffect::Dizzy => {
                                commands.entity(entity).remove::<Dizzy>();
                            }
                            StatusEffect::DimensionBond => {
                                commands.entity(entity).remove::<Summoned>();
                            }
                        }
                        // HACK: Transforming the entity into its own species has no effect on
                        // the creature, but it does trigger assign_species_components's change
                        // detection, which will prevent walls from losing Invincible due to
                        // running out of the Invincible status effect, for example.
                        commands.entity(entity).insert(*species);
                    }
                }
            }
        }
        npc_actions.send(DistributeNpcActions { speed_level: 1 });
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
    npcs: Query<
        (
            Entity,
            &Position,
            &Species,
            &Spellbook,
            Option<&Speed>,
            Has<Hunt>,
            Has<Random>,
        ),
        (Without<Player>, Without<Dizzy>, Without<Sleeping>),
    >,
    species: Query<&Species>,
    map: Res<Map>,
) {
    for event in events.read() {
        let player_pos = player.get_single().unwrap();
        let mut send_echo = false;
        for (npc_entity, npc_pos, npc_species, npc_spellbook, speed, is_hunter, is_random) in
            npcs.iter()
        {
            if let Some(speed) = speed {
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
                // Try to find a tile that gets the hunter closer to the player.
                if let Some(move_direction) = map.best_manhattan_move(*npc_pos, *player_pos) {
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

pub fn respawn_cage(mut events: EventReader<RespawnCage>, mut commands: Commands) {
    for _event in events.read() {
        commands.run_system_cached(spawn_cage);
    }
}
