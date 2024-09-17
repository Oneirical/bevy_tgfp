use bevy::prelude::*;

use crate::{input::InputDelay, map::Map, Hunt, OrdDir, Player, Position};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerStep>();
        app.add_event::<TeleportEntity>();
        app.add_event::<EndTurn>();
        app.add_systems(Update, player_step);
        app.add_systems(Update, teleport_entity);
        app.add_systems(Update, end_turn);
    }
}

#[derive(Event)]
pub struct PlayerStep {
    pub direction: OrdDir,
}

#[derive(Event)]
struct EndTurn;

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
    mut creature: Query<&mut Position>,
    mut map: ResMut<Map>,
) {
    for event in events.read() {
        let mut creature_position = creature
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        if !map.is_empty(event.destination.x, event.destination.y) {
            // TODO: Raise a collision event here.
            continue;
        }

        map.update_map(event.entity, creature_position.clone(), event.destination);
        creature_position.update(event.destination.x, event.destination.y);
    }
}
