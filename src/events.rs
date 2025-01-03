use std::f32::consts::PI;

use bevy::{prelude::*, utils::HashMap};

use crate::{
    creature::{
        get_species_sprite, is_naturally_intangible, Creature, Dizzy, Door, Fragile, Health,
        HealthIndicator, Hunt, Intangible, Invincible, Meleeproof, Player, PotencyAndStacks,
        Random, Species, Speed, Spellproof, Stab, StatusEffect, StatusEffectsList, Summoned, Wall,
        WhenSteppedOn,
    },
    graphics::{
        get_effect_sprite, EffectSequence, EffectType, MagicEffect, MagicVfx, PlaceMagicVfx,
        SlideAnimation, SpriteSheetAtlas,
    },
    map::{Map, Position},
    spells::{Axiom, CastSpell, Spell, SpellStack},
    OrdDir, TILE_SIZE,
};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SummonCreature>();
        app.init_resource::<Events<EndTurn>>();
        app.add_event::<TeleportEntity>();
        app.add_event::<SteppedOnTile>();
        app.add_event::<CreatureCollision>();
        app.add_event::<AlterMomentum>();
        app.add_event::<DamageOrHealCreature>();
        app.add_event::<OpenDoor>();
        app.add_event::<RemoveCreature>();
        app.add_event::<EchoSpeed>();
        app.add_event::<DistributeNpcActions>();
        app.add_event::<AddStatusEffect>();
        app.init_resource::<Events<CreatureStep>>();
        app.insert_resource(TurnManager {
            turn_count: 0,
            action_this_turn: PlayerAction::Invalid,
        });
    }
}

#[derive(Resource)]
pub struct TurnManager {
    pub turn_count: usize,
    /// Whether the player took a step, cast a spell, or did something useless (like step into a wall) this turn.
    pub action_this_turn: PlayerAction,
}

pub enum PlayerAction {
    Step,
    Spell,
    Invalid,
}

#[derive(Event)]
pub struct AddStatusEffect {
    pub entity: Entity,
    pub effect: StatusEffect,
    pub potency: usize,
    pub stacks: usize,
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
    pub spell: Option<Spell>,
}

/// Place a new Creature on the map of Species and at Position.
pub fn summon_creature(
    mut commands: Commands,
    mut events: EventReader<SummonCreature>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    map: Res<Map>,
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
            Species::Hunter => 2,
            Species::Spawner => 3,
            Species::Apiarist => 3,
            Species::Shrike => 1,
            Species::Second => 1,
            Species::Tinker => 2,
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
                spell: if let Some(spell) = event.spell.clone() {
                    spell
                } else {
                    Spell { axioms: Vec::new() }
                },
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
            Species::Wall => {
                new_creature.insert((Meleeproof, Spellproof, Wall, Invincible));
            }
            Species::Trap => {
                new_creature.insert((
                    Meleeproof,
                    Spellproof,
                    Intangible,
                    WhenSteppedOn,
                    Fragile,
                    Invincible,
                ));
            }
            Species::WeakWall => {
                new_creature.insert((Meleeproof, Wall, Invincible));
            }
            Species::Airlock => {
                new_creature.insert((Meleeproof, Spellproof, Door, Invincible));
            }
            Species::Hunter | Species::Spawner | Species::Second => {
                new_creature.insert(Hunt);
            }
            Species::Tinker => {
                new_creature.insert(Random);
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
                    5 => 178,
                    4 => 179,
                    3 => 180,
                    2 => 181,
                    1 => 182,
                    _ => 183,
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
    mut creature: Query<(&mut Position, Has<Intangible>)>,
    mut map: ResMut<Map>,
    mut commands: Commands,
    mut collision: EventWriter<CreatureCollision>,
    mut stepped: EventWriter<SteppedOnTile>,
    is_player: Query<Has<Player>>,
) {
    for event in events.read() {
        let (mut creature_position, is_intangible) = creature
            // Get the Position of the Entity targeted by TeleportEntity.
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        // If motion is possible...
        if map.is_passable(event.destination.x, event.destination.y) || is_intangible {
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
        } else {
            // A creature collides with another entity.
            let collided_with = map
                .get_entity_at(event.destination.x, event.destination.y)
                .unwrap();
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
    mut spell: EventWriter<CastSpell>,
    mut remove: EventWriter<RemoveCreature>,
    traps: Query<(&Position, &Spell), With<WhenSteppedOn>>,
    fragile: Query<(Entity, &Position), With<Fragile>>,
) {
    for event in events.read() {
        // Traps trigger their spell effect when stepped on.
        for (position, axiom_sequence) in traps.iter() {
            if event.position == *position {
                spell.send(CastSpell {
                    caster: event.entity,
                    spell: axiom_sequence.clone(),
                });
            }
        }
        // Fragile floor entities are destroyed when stepped on.
        for (entity, position) in fragile.iter() {
            if event.position == *position {
                remove.send(RemoveCreature { entity });
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
    mut open: EventWriter<OpenDoor>,
    attacker_flags: Query<&Stab>,
    defender_flags: Query<(Has<Door>, Has<Meleeproof>)>,
    mut turn_manager: ResMut<TurnManager>,
    mut creature: Query<(&OrdDir, &mut Transform, Has<Player>)>,
    mut commands: Commands,
    mut effects: Query<&mut StatusEffectsList>,
    position: Query<&Position>,
) {
    for event in events.read() {
        if event.culprit == event.collided_with {
            // No colliding with yourself.
            continue;
        }
        let (is_door, cannot_be_melee_attacked) = defender_flags.get(event.collided_with).unwrap();
        let (attacker_orientation, mut attacker_transform, is_player) =
            creature.get_mut(event.culprit).unwrap();
        if is_door {
            // Open doors.
            open.send(OpenDoor {
                entity: event.collided_with,
            });
        } else if !cannot_be_melee_attacked {
            let damage = if let Ok(stab) = attacker_flags.get(event.culprit) {
                // Attacking something with Stab active resets the Stab bonus.
                let mut status_effects = effects.get_mut(event.culprit).unwrap();
                status_effects
                    .effects
                    .get_mut(&StatusEffect::Stab)
                    .unwrap()
                    .stacks = 0;
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
                health.hp = health.hp.saturating_sub((event.hp_mod * -1) as usize)
            } // Damage
            1 => health.hp = health.hp.saturating_add(event.hp_mod as usize), // Healing
            _ => (),                                                          // 0 values do nothing
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
pub struct OpenDoor {
    entity: Entity,
}

pub fn open_door(
    mut events: EventReader<OpenDoor>,
    mut commands: Commands,
    mut door: Query<(&mut Visibility, &Position, &OrdDir)>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    for event in events.read() {
        // Gather component values of the door.
        let (mut visibility, position, orientation) = door.get_mut(event.entity).unwrap();
        // The door becomes intangible, and can be walked through.
        commands.entity(event.entity).insert(Intangible);
        // The door is no longer visible, as it is open.
        *visibility = Visibility::Hidden;
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
                    position: Position::new(position.x + offset.0, position.y + offset.1),
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
                        decay: Timer::from_seconds(3., TimerMode::Once),
                    },
                },
                // Ensure the panes are sliding.
                SlideAnimation,
                Transform {
                    translation: Vec3 {
                        x: position.x as f32 * TILE_SIZE,
                        y: position.y as f32 * TILE_SIZE,
                        // The pane needs to hide under actual tiles, such as walls.
                        z: -1.,
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

#[derive(Event)]
pub struct RemoveCreature {
    pub entity: Entity,
}

pub fn remove_creature(
    mut events: EventReader<RemoveCreature>,
    mut commands: Commands,
    mut map: ResMut<Map>,
    creature: Query<(&Position, Has<Player>)>,
    mut spell_stack: ResMut<SpellStack>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
) {
    for event in events.read() {
        let (position, is_player) = creature.get(event.entity).unwrap();
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
            // Remove the creature from Map
            if let Some(preexisting_entity) = map.creatures.get(position) {
                // Check that the entity being removed is actually the dead entity.
                // REASON: Dying intangible creatures which are on top of a tangible
                // creature will remove the tangible creature from the map instead
                // of themselves.
                if *preexisting_entity == event.entity {
                    map.creatures.remove(position);
                }
            }
            // Remove the creature AND its children (health bar)
            commands.entity(event.entity).despawn_recursive();
            // Remove all spells cast by this creature
            // (this entity doesn't exist anymore, casting its spells would crash the game)
            spell_stack
                .spells
                .retain(|spell| spell.caster != event.entity);
        } else {
            panic!("You have been slain");
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
) {
    for _event in events.read() {
        // The player shouldn't be allowed to "wait" turns by stepping into walls.
        if matches!(turn_manager.action_this_turn, PlayerAction::Invalid) {
            return;
        }

        // The turncount increases.
        turn_manager.turn_count += 1;
        // Tick down status effects.
        for (entity, mut effect_list, species) in effects.iter_mut() {
            for (effect, potency_and_stacks) in effect_list.effects.iter_mut() {
                potency_and_stacks.stacks = potency_and_stacks.stacks.saturating_sub(1);
                if potency_and_stacks.stacks == 0 {
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
                    }
                    // HACK: Transforming the entity into its own species has no effect on
                    // the creature, but it does trigger assign_species_components's change
                    // detection, which will prevent walls from losing Invincible due to
                    // running out of the Invincible status effect, for example.
                    commands.entity(entity).insert(*species);
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
            Option<&Speed>,
            Has<Hunt>,
            Has<Random>,
            Has<Dizzy>,
        ),
        Without<Player>,
    >,
    map: Res<Map>,
) {
    for event in events.read() {
        let player_pos = player.get_single().unwrap();
        let mut send_echo = false;
        for (npc_entity, npc_pos, npc_species, speed, is_hunter, is_random, is_too_dizzy_to_act) in
            npcs.iter()
        {
            if is_too_dizzy_to_act {
                continue;
            }
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
            // Occasionally cast a spell.
            if turn_manager.turn_count % 500 == 0 {
                match npc_species {
                    Species::Second => {
                        spell.send(CastSpell {
                            caster: npc_entity,
                            spell: Spell {
                                axioms: vec![Axiom::Plus, Axiom::DevourWall],
                            },
                        });
                    }
                    _ => (),
                }
            } else if is_random {
                if let Some(move_direction) = map.random_adjacent_passable_direction(*npc_pos) {
                    // If it is found, cause a CreatureStep event.
                    step.send(CreatureStep {
                        direction: move_direction,
                        entity: npc_entity,
                    });
                }
            } else if is_hunter {
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
