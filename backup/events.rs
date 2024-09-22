use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    creature::{Creature, DamageResult, Hunt, Ipseity, Player, Soul, Soulless, Species},
    graphics::{AnimationOffset, PlaceEffect, Scale, SpriteSheetAtlas, VisualEffectType},
    input::InputDelay,
    map::Map,
    OrdDir, Position,
};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerStep>();
        app.add_event::<TeleportEntity>();
        app.add_event::<EndTurn>();
        app.add_event::<RepressionDamage>();
        app.add_event::<BuildRoom>();
        app.add_event::<OpenDoor>();
        app.add_systems(Update, player_step);
        app.add_systems(Update, teleport_entity);
        app.add_systems(Update, end_turn);
        app.add_systems(Update, repression_damage);
        app.add_systems(Update, build_room_from_airlock);
        app.add_systems(Update, open_door);
    }
}

#[derive(Event)]
pub struct PlayerStep {
    pub direction: OrdDir,
}

#[derive(Event)]
struct EndTurn;

#[derive(Event)]
struct RepressionDamage {
    damage: usize,
    entity: Entity,
}

#[derive(Event)]
struct BuildRoom {
    direction: OrdDir,
    position: Position,
}

#[derive(Event)]
struct OpenDoor {
    entity: Entity,
}

#[derive(Event)]
struct TeleportEntity {
    destination: Position,
    entity: Entity,
}

impl TeleportEntity {
    fn new(entity: Entity, x: i32, y: i32) -> Self {
        Self {
            destination: Position::new(x, y),
            entity,
        }
    }
}

fn player_step(
    mut events: EventReader<PlayerStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut end_turn: EventWriter<EndTurn>,
    player: Query<(Entity, &Position), With<Player>>,
    mut delay: ResMut<InputDelay>,
) {
    let (player_entity, player_pos) = player.get_single().expect("0 or 2+ players");
    for event in events.read() {
        delay.timer.reset();
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportEntity::new(
            player_entity,
            player_pos.x + off_x,
            player_pos.y + off_y,
        ));
        // NOTE: This will end the turn even if the move (and collision)
        // does nothing.
        end_turn.send(EndTurn);
    }
}

fn end_turn(
    npcs: Query<(Entity, &Position), (With<Hunt>, Without<Player>)>,
    player: Query<&Position, With<Player>>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut events: EventReader<EndTurn>,
    map: Res<Map>,
) {
    for _event in events.read() {
        let player_pos = player.get_single().expect("0 or 2+ players");
        for (npc_entity, npc_pos) in npcs.iter() {
            if let Some(move_target) = map.best_manhattan_move(*npc_pos, *player_pos) {
                teleporter.send(TeleportEntity {
                    destination: move_target,
                    entity: npc_entity,
                });
            }
        }
    }
}

fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut melee_attack: EventWriter<RepressionDamage>,
    mut build_room: EventWriter<BuildRoom>,
    mut door_opener: EventWriter<OpenDoor>,
    mut creature: Query<(&mut Position, &mut AnimationOffset)>,
    species: Query<&Species>,
    mut map: ResMut<Map>,
    scale: Res<Scale>,
) {
    for event in events.read() {
        let (mut creature_position, mut creature_anim) = creature
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        if !map.is_passable(event.destination.x, event.destination.y) {
            // Check the type of the collided entity.
            let collided_entity = *map
                .get_entity_at(event.destination.x, event.destination.y)
                .unwrap();
            // If it's an airlock, spawn a new room.
            if let Species::Airlock { orientation } = species.get(collided_entity).unwrap() {
                build_room.send(BuildRoom {
                    direction: *orientation,
                    position: Position::new(event.destination.x, event.destination.y),
                });
                door_opener.send(OpenDoor {
                    entity: collided_entity,
                });
                continue;
            }
            // Otherwise, strike it.
            melee_attack.send(RepressionDamage {
                damage: 1,
                entity: collided_entity,
            });
            // Play the attack animation.
            creature_anim.initiate_offset_f32(
                (creature_position.x - event.destination.x) as f32 * -0.3,
                (creature_position.y - event.destination.y) as f32 * -0.3,
                scale.tile_size,
            );
            continue;
        }

        creature_anim.initiate_offset(
            creature_position.x - event.destination.x,
            creature_position.y - event.destination.y,
            scale.tile_size,
        );
        map.update_map(event.entity, creature_position.clone(), event.destination);
        creature_position.update(event.destination.x, event.destination.y);
    }
}

fn repression_damage(
    mut commands: Commands,
    mut events: EventReader<RepressionDamage>,
    mut ipseity: Query<&mut Ipseity>,
) {
    for event in events.read() {
        let mut ipseity = ipseity
            .get_mut(event.entity)
            .expect("A RepressionDamage was given an invalid entity");
        if ipseity.harvest_random_souls(event.damage) == DamageResult::Drained {
            // This creature can't take any more damage!
            commands.entity(event.entity).insert(Soulless);
        }
    }
}

fn open_door(
    mut events: EventReader<OpenDoor>,
    mut effect_place: EventWriter<PlaceEffect>,
    mut query: Query<(&mut Visibility, &Position, &Species)>,
    mut map: ResMut<Map>,
) {
    for event in events.read() {
        let (mut door_vis, door_pos, door_species) = query.get_mut(event.entity).unwrap();
        *door_vis = Visibility::Hidden;
        map.make_entity_passable(event.entity);
        let orientation = if let Species::Airlock { orientation } = door_species {
            orientation
        } else {
            panic!("Something that isn't a door is being opened!");
        };
        let pane_set = match orientation {
            OrdDir::Up | OrdDir::Down => [
                Position::new(door_pos.x + 1, door_pos.y),
                Position::new(door_pos.x - 1, door_pos.y),
            ],
            OrdDir::Left | OrdDir::Right => [
                Position::new(door_pos.x, door_pos.y + 1),
                Position::new(door_pos.x, door_pos.y - 1),
            ],
        };
        for pane in pane_set {
            effect_place.send(PlaceEffect {
                position: pane,
                effect_type: VisualEffectType::SlidingDoor {
                    orientation: *orientation,
                    source_door: *door_pos,
                },
            });
        }
    }
}

fn build_room_from_airlock(
    scale: Res<Scale>,
    mut commands: Commands,
    mut events: EventReader<BuildRoom>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    map: Res<Map>,
) {
    for event in events.read() {
        let seed =
            "####V#####.......##.......##.......#<.......>#.......##.......##.......#####^####";
        for (idx, tile_char) in seed.char_indices() {
            let mut position = Position::new(
                idx as i32 % 9 + event.position.x,
                idx as i32 / 9 + event.position.y,
            );
            // Align the room with the airlock that spawned it.
            match event.direction {
                OrdDir::Up => position.shift(-4, 0),
                OrdDir::Down => position.shift(-4, -8),
                OrdDir::Left => position.shift(-8, -4),
                OrdDir::Right => position.shift(0, -4),
            }
            // Do not build if it would overlap with the access wall.
            if !map.is_empty(position.x, position.y) {
                continue;
            }
            let (index, species) = match tile_char {
                '#' => (3, Species::Wall),
                'S' => (4, Species::Scion),
                'V' => (
                    17,
                    Species::Airlock {
                        orientation: crate::OrdDir::Down,
                    },
                ),
                '^' => (
                    17,
                    Species::Airlock {
                        orientation: crate::OrdDir::Up,
                    },
                ),
                '<' => (
                    17,
                    Species::Airlock {
                        orientation: crate::OrdDir::Left,
                    },
                ),
                '>' => (
                    17,
                    Species::Airlock {
                        orientation: crate::OrdDir::Right,
                    },
                ),
                '.' => continue,
                _ => panic!(),
            };
            let mut transform =
                Transform::from_scale(Vec3::new(scale.tile_size, scale.tile_size, 0.));
            // Out of sight, out of mind.
            transform.translation.x = 1000.;
            if let Species::Airlock { orientation } = species {
                match orientation {
                    OrdDir::Down => transform.rotate_z(0.),
                    OrdDir::Right => transform.rotate_z(PI / 2.),
                    OrdDir::Up => transform.rotate_z(PI),
                    OrdDir::Left => transform.rotate_z(3. * PI / 2.),
                }
            }

            commands.spawn((Creature {
                position,
                sprite: SpriteBundle {
                    texture: asset_server.load("spritesheet.png"),
                    transform,
                    ..default()
                },
                atlas: TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index,
                },
                ipseity: Ipseity::new(&[(Soul::Immutable, 1)]),
                animation: AnimationOffset::from_tile(20, 20, scale.tile_size),
                species,
            },));
        }
    }
}
