use bevy::prelude::*;

use crate::{
    creature::Player,
    events::{
        CreatureStep, DrawSoul, EndTurn, PlayerAction, RespawnPlayer, TurnManager, UseWheelSoul,
    },
    ui::{AddMessage, Message},
    OrdDir,
};

/// Each frame, if a button is pressed, move the player 1 tile.
pub fn keyboard_input(
    player: Query<Entity, With<Player>>,
    mut use_wheel_soul: EventWriter<UseWheelSoul>,
    mut draw_soul: EventWriter<DrawSoul>,
    mut events: EventWriter<CreatureStep>,
    input: Res<ButtonInput<KeyCode>>,
    mut turn_manager: ResMut<TurnManager>,
    mut turn_end: EventWriter<EndTurn>,
    mut respawn: EventWriter<RespawnPlayer>,
    mut text: EventWriter<AddMessage>,
) {
    let soul_keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
    ];
    if input.any_just_pressed(soul_keys) {
        for (i, key) in soul_keys.iter().enumerate() {
            if input.just_pressed(*key) {
                use_wheel_soul.send(UseWheelSoul { index: i });
                turn_manager.action_this_turn = PlayerAction::Spell;
                turn_end.send(EndTurn);
            }
        }
    }
    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::KeyQ) {
        draw_soul.send(DrawSoul { amount: 1 });
        turn_manager.action_this_turn = PlayerAction::Draw;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::ArrowUp) || input.just_pressed(KeyCode::KeyW) {
        events.send(CreatureStep {
            direction: OrdDir::Up,
            entity: player.get_single().unwrap(),
        });
        turn_manager.action_this_turn = PlayerAction::Step;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::ArrowRight) || input.just_pressed(KeyCode::KeyD) {
        events.send(CreatureStep {
            direction: OrdDir::Right,
            entity: player.get_single().unwrap(),
        });
        turn_manager.action_this_turn = PlayerAction::Step;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::KeyA) {
        events.send(CreatureStep {
            direction: OrdDir::Left,
            entity: player.get_single().unwrap(),
        });
        turn_manager.action_this_turn = PlayerAction::Step;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::ArrowDown) || input.just_pressed(KeyCode::KeyS) {
        events.send(CreatureStep {
            direction: OrdDir::Down,
            entity: player.get_single().unwrap(),
        });
        turn_manager.action_this_turn = PlayerAction::Step;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::KeyZ) || input.just_pressed(KeyCode::KeyX) {
        respawn.send(RespawnPlayer { victorious: false });
    }

    if input.just_pressed(KeyCode::KeyO) || input.just_pressed(KeyCode::KeyP) {
        text.send(AddMessage {
            message: Message::WASD,
        });
    }
}
