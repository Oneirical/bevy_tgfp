use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    creature::{
        get_species_sprite, Attackproof, Creature, Door, Health, HealthIndicator, Hunt, Intangible,
        Player, Random, Species, Speed, Spellproof, Summoned, Wall,
    },
    graphics::{
        get_effect_sprite, EffectSequence, EffectType, MagicEffect, MagicVfx, PlaceMagicVfx,
        SlideAnimation, SpriteSheetAtlas,
    },
    map::{Map, Position},
    spells::{Axiom, CastSpell, Spell, SpellStack},
    OrdDir,
};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SummonCreature>();
        app.init_resource::<Events<EndTurn>>();
        app.add_event::<TeleportEntity>();
        app.add_event::<CreatureCollision>();
        app.add_event::<AlterMomentum>();
        app.add_event::<DamageOrHealCreature>();
        app.add_event::<OpenDoor>();
        app.add_event::<RemoveCreature>();
        app.add_event::<EchoSpeed>();
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
pub struct SummonCreature {
    pub position: Position,
    pub species: Species,
    pub momentum: OrdDir,
    pub summon_tile: Position,
    pub summoner: Option<Entity>,
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
        if !map.is_passable(event.position.x, event.position.y) {
            continue;
        }
        let max_hp = match &event.species {
            Species::Player => 7,
            Species::Wall => 10,
            Species::WeakWall => 10,
            Species::Hunter => 2,
            Species::Spawner => 3,
            Species::Airlock => 10,
            Species::Apiarist => 3,
            Species::Shrike => 1,
            Species::Second => 6,
            Species::Tinker => 2,
            Species::Architect => 3,
        };
        // Start at full health, in most cases.
        let hp = match &event.species {
            Species::Second => 1,
            _ => max_hp,
        };
        let mut new_creature = commands.spawn((
            Creature {
                position: event.position,
                species: event.species,
                sprite: Sprite {
                    image: asset_server.load("spritesheet.png"),
                    custom_size: Some(Vec2::new(64., 64.)),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_layout.handle.clone(),
                        index: get_species_sprite(&event.species),
                    }),
                    ..default()
                },
                momentum: event.momentum,
                health: Health { max_hp, hp },
            },
            Transform {
                translation: Vec3 {
                    x: event.summon_tile.x as f32 * 64.,
                    y: event.summon_tile.y as f32 * 64.,
                    z: 0.,
                },
                rotation: Quat::from_rotation_z(match event.momentum {
                    OrdDir::Down => 0.,
                    OrdDir::Right => PI / 2.,
                    OrdDir::Up => PI,
                    OrdDir::Left => 3. * PI / 2.,
                }),
                scale: Vec3::new(1., 1., 1.),
            },
            SlideAnimation,
        ));
        // Add any species-specific components.
        // TODO: Offshore this to a function when transformation axioms get added to avoid repetition?
        match &event.species {
            Species::Player => {
                new_creature.insert(Player);
            }
            Species::Wall => {
                new_creature.insert((Attackproof, Spellproof, Wall));
            }
            Species::WeakWall => {
                new_creature.insert((Attackproof, Wall));
            }
            Species::Airlock => {
                new_creature.insert((Attackproof, Spellproof, Door));
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
                    custom_size: Some(Vec2::new(64., 64.)),
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
                let hp_percent = hp as f32 / max_hp as f32;
                match hp_percent {
                    0.86..1.00 => 178,
                    0.72..0.86 => 179,
                    0.58..0.72 => 180,
                    0.44..0.58 => 181,
                    0.30..0.44 => 182,
                    0.16..0.30 => 183,
                    0.00..0.16 => 184,
                    _ => panic!("{hp}/{max_hp} - That is not a possible HP %!"),
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
        } else {
            // A creature collides with another entity.
            let collided_with = map
                .get_entity_at(event.destination.x, event.destination.y)
                .unwrap();
            collision.send(CreatureCollision {
                culprit: event.entity,
                collided_with: *collided_with,
            });
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
    flags: Query<(Has<Door>, Has<Attackproof>)>,
    mut turn_manager: ResMut<TurnManager>,
    mut creature: Query<(&OrdDir, &mut Transform, Has<Player>)>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.culprit == event.collided_with {
            // No colliding with yourself.
            continue;
        }
        let (is_door, cannot_be_attacked) = flags.get(event.collided_with).unwrap();
        let (attacker_orientation, mut attacker_transform, is_player) =
            creature.get_mut(event.culprit).unwrap();
        if is_door {
            // Open doors.
            open.send(OpenDoor {
                entity: event.collided_with,
            });
        } else if !cannot_be_attacked {
            // Melee attack.
            harm.send(DamageOrHealCreature {
                entity: event.collided_with,
                culprit: event.culprit,
                hp_mod: -1,
            });
            // Melee attack animation.
            attacker_transform.translation.x +=
                attacker_orientation.as_offset().0 as f32 * 64. / 4.;
            attacker_transform.translation.y +=
                attacker_orientation.as_offset().1 as f32 * 64. / 4.;
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
    mut spell: EventWriter<CastSpell>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut creature: Query<(&mut Health, &Children, &Species)>,
    mut hp_bar: Query<(&mut Visibility, &mut Sprite)>,
) {
    for event in events.read() {
        let (mut health, children, species) = creature.get_mut(event.entity).unwrap();
        // Apply damage or healing.
        match event.hp_mod.signum() {
            -1 => health.hp = health.hp.saturating_sub((event.hp_mod * -1) as usize), // Damage
            1 => health.hp = health.hp.saturating_add(event.hp_mod as usize),         // Healing
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
        } else if matches!(species, Species::Architect) {
            teleporter.send(TeleportEntity::new(event.culprit, 22, 1));
            spell.send(CastSpell {
                caster: event.entity,
                spell: Spell {
                    axioms: vec![Axiom::Ego, Axiom::Abjuration],
                },
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
                        custom_size: Some(Vec2::new(64., 64.)),
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
                        x: position.x as f32 * 64.,
                        y: position.y as f32 * 64.,
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
            map.creatures.remove(position);
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
pub struct EndTurn {
    pub speed_level: usize,
}

pub fn end_turn(
    mut events: EventReader<EndTurn>,
    mut step: EventWriter<CreatureStep>,
    mut spell: EventWriter<CastSpell>,
    mut echo: EventWriter<EchoSpeed>,
    mut turn_manager: ResMut<TurnManager>,
    player: Query<&Position, With<Player>>,
    npcs: Query<
        (
            Entity,
            &Position,
            &Species,
            Option<&Speed>,
            Has<Hunt>,
            Has<Random>,
        ),
        Without<Player>,
    >,
    map: Res<Map>,
) {
    for event in events.read() {
        // The player shouldn't be allowed to "wait" turns by stepping into walls.
        if matches!(turn_manager.action_this_turn, PlayerAction::Invalid) {
            return;
        }
        turn_manager.turn_count += 1;
        let player_pos = player.get_single().unwrap();
        for (npc_entity, npc_pos, npc_species, speed, is_hunter, is_random) in npcs.iter() {
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
                            echo.send(EchoSpeed {
                                speed_level: event.speed_level + 1,
                            });
                        }
                    }
                }
            } else if event.speed_level > 1 {
                continue;
            }
            // Occasionally cast a spell.
            if turn_manager.turn_count % 10 == 0 {
                match npc_species {
                    Species::Hunter => {
                        spell.send(CastSpell {
                            caster: npc_entity,
                            spell: Spell {
                                axioms: vec![Axiom::MomentumBeam, Axiom::Dash { max_distance: 5 }],
                            },
                        });
                    }
                    Species::Architect => {
                        spell.send(CastSpell {
                            caster: npc_entity,
                            spell: Spell {
                                axioms: vec![Axiom::Player, Axiom::ArchitectCage],
                            },
                        });
                    }
                    Species::Second => {
                        spell.send(CastSpell {
                            caster: npc_entity,
                            spell: Spell {
                                axioms: vec![Axiom::Plus, Axiom::DevourWall],
                            },
                        });
                    }
                    Species::Spawner => {
                        spell.send(CastSpell {
                            caster: npc_entity,
                            spell: Spell {
                                axioms: vec![
                                    Axiom::Halo { radius: 3 },
                                    Axiom::SummonCreature {
                                        species: Species::Hunter,
                                    },
                                ],
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
    }
}

#[derive(Event)]
pub struct EchoSpeed {
    pub speed_level: usize,
}

pub fn echo_speed(mut events: EventReader<EchoSpeed>, mut end_turn: EventWriter<EndTurn>) {
    for event in events.read() {
        end_turn.send(EndTurn {
            speed_level: event.speed_level,
        });
    }
}
