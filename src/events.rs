use bevy::prelude::*;

use crate::{
    creature::{get_species_sprite, Creature, Health, HealthIndicator, Hunt, Player, Species},
    graphics::{EffectSequence, EffectType, PlaceMagicVfx, SlideAnimation, SpriteSheetAtlas},
    map::{Map, Position},
    spells::{Axiom, CastSpell, Spell, SpellStack},
    OrdDir,
};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SummonCreature>();
        app.add_event::<EndTurn>();
        app.add_event::<TeleportEntity>();
        app.add_event::<HarmCreature>();
        app.add_event::<RemoveCreature>();
        app.init_resource::<Events<CreatureStep>>();
        app.insert_resource(TurnCount { turns: 0 });
    }
}

#[derive(Resource)]
pub struct TurnCount {
    turns: usize,
}

#[derive(Event)]
pub struct SummonCreature {
    pub position: Position,
    pub species: Species,
    pub summon_tile: Position,
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
                momentum: OrdDir::Up,
                health: {
                    let max_hp = match &event.species {
                        Species::Player => 6,
                        Species::Wall => 10,
                        Species::Hunter => 5,
                        Species::Spawner => 3,
                    };
                    // Start at full health.
                    let hp = max_hp;
                    Health { max_hp, hp }
                },
            },
            Transform::from_xyz(
                event.summon_tile.x as f32 * 64.,
                event.summon_tile.y as f32 * 64.,
                0.,
            ),
            SlideAnimation,
        ));
        // Add any species-specific components.
        match &event.species {
            Species::Player => {
                new_creature.insert(Player);
            }
            Species::Hunter | Species::Spawner => {
                new_creature.insert(Hunt);
            }
            _ => (),
        }

        // Free the borrow on Commands.
        let new_creature_entity = new_creature.id();
        let hp_bar = commands
            .spawn(HealthIndicator {
                sprite: Sprite {
                    image: asset_server.load("spritesheet.png"),
                    custom_size: Some(Vec2::new(64., 64.)),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_layout.handle.clone(),
                        index: 178,
                    }),
                    ..default()
                },
                visibility: Visibility::Hidden,
                transform: Transform::from_xyz(0., 0., 1.),
            })
            .id();
        commands.entity(new_creature_entity).add_child(hp_bar);
    }
}

#[derive(Event)]
pub struct CreatureStep {
    pub entity: Entity,
    pub direction: OrdDir,
}

pub fn creature_step(
    mut events: EventReader<CreatureStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut turn_end: EventWriter<EndTurn>,
    mut creature: Query<(&Position, Has<Player>, &mut OrdDir)>,
) {
    for event in events.read() {
        let (creature_pos, is_player, mut creature_momentum) =
            creature.get_mut(event.entity).unwrap();
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportEntity::new(
            event.entity,
            creature_pos.x + off_x,
            creature_pos.y + off_y,
        ));
        // Update the direction towards which this creature is facing.
        *creature_momentum = event.direction;
        // If this creature was the player, this will end the turn.
        if is_player {
            turn_end.send(EndTurn);
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
    mut creature: Query<&mut Position>,
    mut map: ResMut<Map>,
    mut commands: Commands,

    mut harm: EventWriter<HarmCreature>,
) {
    for event in events.read() {
        let mut creature_position = creature
            // Get the Position of the Entity targeted by TeleportEntity.
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        // If motion is possible...
        if map.is_passable(event.destination.x, event.destination.y) {
            // ...update the Map to reflect this...
            map.move_creature(*creature_position, event.destination);
            // ...and move that Entity to TeleportEntity's destination tile.
            creature_position.update(event.destination.x, event.destination.y);
            // Also, animate this creature, making its teleport action visible on the screen.
            commands.entity(event.entity).insert(SlideAnimation);
        } else {
            // A creature collides with another entity.
            harm.send(HarmCreature {
                entity: *map
                    .get_entity_at(event.destination.x, event.destination.y)
                    .unwrap(),
                culprit: event.entity,
                damage: 1,
            });
            continue;
        }
    }
}

#[derive(Event)]
pub struct HarmCreature {
    entity: Entity,
    culprit: Entity,
    damage: usize,
}

pub fn harm_creature(
    mut events: EventReader<HarmCreature>,
    mut remove: EventWriter<RemoveCreature>,
    mut creature: Query<(&mut Health, &Children)>,
    mut hp_bar: Query<(&mut Visibility, &mut Sprite)>,
) {
    for event in events.read() {
        let (mut health, children) = creature.get_mut(event.entity).unwrap();
        // Deduct damage from hp.
        health.hp = health.hp.saturating_sub(event.damage);
        // Update the healthbar.
        for child in children.iter() {
            let (mut hp_vis, mut hp_bar) = hp_bar.get_mut(*child).unwrap();
            // Don't show the healthbar at full hp.
            if health.max_hp == health.hp {
                *hp_vis = Visibility::Hidden;
            } else {
                *hp_vis = Visibility::Inherited;
                let hp_percent = health.hp as f32 / health.max_hp as f32;
                hp_bar.texture_atlas.as_mut().unwrap().index = match hp_percent {
                    0.86..1.00 => 178,
                    0.72..0.86 => 179,
                    0.58..0.72 => 180,
                    0.44..0.58 => 181,
                    0.30..0.44 => 182,
                    0.16..0.30 => 183,
                    0.00..0.16 => 184,
                    _ => panic!("That is not a possible HP %!"),
                }
            }
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
pub struct RemoveCreature {
    entity: Entity,
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
        magic_vfx.send(PlaceMagicVfx {
            targets: vec![*position],
            sequence: EffectSequence::Simultaneous,
            effect: EffectType::XCross,
            decay: 0.5,
            appear: 0.,
        });
        if !is_player {
            map.creatures.remove(position);
            commands.entity(event.entity).despawn_recursive();
            spell_stack
                .spells
                .retain(|spell| spell.caster != event.entity);
        }
    }
}

#[derive(Event)]
pub struct EndTurn;

pub fn end_turn(
    mut events: EventReader<EndTurn>,
    mut step: EventWriter<CreatureStep>,
    mut spell: EventWriter<CastSpell>,
    mut turn_count: ResMut<TurnCount>,
    player: Query<&Position, With<Player>>,
    hunters: Query<(Entity, &Position, &Species), (With<Hunt>, Without<Player>)>,
    map: Res<Map>,
) {
    for _event in events.read() {
        turn_count.turns += 1;
        let player_pos = player.get_single().unwrap();
        for (hunter_entity, hunter_pos, hunter_species) in hunters.iter() {
            // Occasionally cast a spell.
            if turn_count.turns % 5 == 0 {
                match hunter_species {
                    Species::Hunter => {
                        spell.send(CastSpell {
                            caster: hunter_entity,
                            spell: Spell {
                                axioms: vec![Axiom::MomentumBeam, Axiom::Dash { max_distance: 5 }],
                            },
                        });
                    }
                    Species::Spawner => {
                        spell.send(CastSpell {
                            caster: hunter_entity,
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
            }
            // Try to find a tile that gets the hunter closer to the player.
            else if let Some(move_direction) = map.best_manhattan_move(*hunter_pos, *player_pos) {
                // If it is found, cause a CreatureStep event.

                step.send(CreatureStep {
                    direction: move_direction,
                    entity: hunter_entity,
                });
            }
        }
    }
}
