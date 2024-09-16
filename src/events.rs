use bevy::prelude::*;

use crate::{input::InputDelay, map::Map, OrdDir, Player, Position};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerStep>();
        app.add_event::<TeleportEntity>();
        app.add_systems(Update, player_step);
        app.add_systems(Update, teleport_entity);
    }
}

#[derive(Event)]
pub struct PlayerStep {
    pub direction: OrdDir,
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
    }
}

fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut creature: Query<&mut Position>,
    map: Res<Map>,
) {
    for event in events.read() {
        let mut creature = creature
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        if map
            .creatures
            .get(&Position::new(event.destination.x, event.destination.y))
            .is_some()
        {
            // TODO: Raise a collision event here.
            continue;
        }
        (creature.x, creature.y) = (event.destination.x, event.destination.y);
    }
}
