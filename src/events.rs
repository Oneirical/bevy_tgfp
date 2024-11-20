use bevy::prelude::*;

use crate::{
    creature::{Hunt, Player},
    graphics::SlideAnimation,
    map::{Map, Position},
    OrdDir,
};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerStep>();
        app.add_event::<TeleportEntity>();
    }
}

#[derive(Event)]
pub struct PlayerStep {
    pub direction: OrdDir,
}

pub fn player_step(
    mut events: EventReader<PlayerStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut player: Query<(Entity, &Position, &mut OrdDir), With<Player>>,
    hunters: Query<(Entity, &Position), With<Hunt>>,
    map: Res<Map>,
) {
    let (player_entity, player_pos, mut player_momentum) =
        player.get_single_mut().expect("0 or 2+ players");
    for event in events.read() {
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportEntity::new(
            player_entity,
            player_pos.x + off_x,
            player_pos.y + off_y,
        ));

        // Update the direction towards which this creature is facing.
        *player_momentum = event.direction;

        for (hunter_entity, hunter_pos) in hunters.iter() {
            // Try to find a tile that gets the hunter closer to the player.
            if let Some(move_target) = map.best_manhattan_move(*hunter_pos, *player_pos) {
                // If it is found, cause another TeleportEntity event.
                teleporter.send(TeleportEntity {
                    destination: move_target,
                    entity: hunter_entity,
                });
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
    mut creature: Query<&mut Position>,
    mut map: ResMut<Map>,
    mut commands: Commands,
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
            // Nothing here just yet, but this is where collisions between creatures
            // will be handled.
            continue;
        }
    }
}
