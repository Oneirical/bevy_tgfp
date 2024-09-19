use bevy::prelude::*;

use crate::{
    creature::{DamageResult, Hunt, Ipseity, Player, Soulless},
    graphics::{AnimationOffset, Scale},
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
        app.add_systems(Update, player_step);
        app.add_systems(Update, teleport_entity);
        app.add_systems(Update, end_turn);
        app.add_systems(Update, repression_damage);
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
    mut creature: Query<(&mut Position, &mut AnimationOffset)>,
    mut map: ResMut<Map>,
    scale: Res<Scale>,
) {
    for event in events.read() {
        let (mut creature_position, mut creature_anim) = creature
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        if !map.is_empty(event.destination.x, event.destination.y) {
            melee_attack.send(RepressionDamage {
                damage: 1,
                entity: *map
                    .get_entity_at(event.destination.x, event.destination.y)
                    .unwrap(),
            });
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
