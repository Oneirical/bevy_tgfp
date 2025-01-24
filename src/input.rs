use bevy::prelude::*;

use crate::{
    creature::{Player, Soul},
    cursor::CursorStep,
    events::{
        CreatureStep, DrawSoul, EndTurn, PlayerAction, RespawnPlayer, TurnManager, UseWheelSoul,
    },
    sets::ControlState,
    ui::LargeCastePanel,
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
    state: Res<State<ControlState>>,
    mut next_state: ResMut<NextState<ControlState>>,
    mut cursor: EventWriter<CursorStep>,
    mut caste_menu: Query<&mut LargeCastePanel>,
    mut scale: ResMut<UiScale>,
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
                match state.get() {
                    ControlState::Player => {
                        use_wheel_soul.send(UseWheelSoul { index: i });
                        turn_manager.action_this_turn = PlayerAction::Spell;
                        turn_end.send(EndTurn);
                    }
                    ControlState::CasteMenu => {
                        let mut caste_menu = caste_menu.single_mut();
                        let current_soul = caste_menu.0;
                        caste_menu.0 = match i {
                            0 => Soul::Saintly,
                            1 => Soul::Ordered,
                            2 => Soul::Artistic,
                            3 => Soul::Unhinged,
                            4 => Soul::Feral,
                            5 => Soul::Vile,
                            _ => current_soul,
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::KeyQ) {
        draw_soul.send(DrawSoul { amount: 1 });
        turn_manager.action_this_turn = PlayerAction::Draw;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::ArrowUp) || input.just_pressed(KeyCode::KeyW) {
        match state.get() {
            ControlState::Cursor => {
                cursor.send(CursorStep {
                    direction: OrdDir::Up,
                });
            }
            ControlState::Player => {
                events.send(CreatureStep {
                    direction: OrdDir::Up,
                    entity: player.get_single().unwrap(),
                });
                turn_manager.action_this_turn = PlayerAction::Step;
                turn_end.send(EndTurn);
            }
            ControlState::CasteMenu => todo!(),
        }
    }
    if input.just_pressed(KeyCode::ArrowRight) || input.just_pressed(KeyCode::KeyD) {
        match state.get() {
            ControlState::Cursor => {
                cursor.send(CursorStep {
                    direction: OrdDir::Right,
                });
            }
            ControlState::Player => {
                events.send(CreatureStep {
                    direction: OrdDir::Right,
                    entity: player.get_single().unwrap(),
                });
                turn_manager.action_this_turn = PlayerAction::Step;
                turn_end.send(EndTurn);
            }
            ControlState::CasteMenu => todo!(),
        }
    }
    if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::KeyA) {
        match state.get() {
            ControlState::Cursor => {
                cursor.send(CursorStep {
                    direction: OrdDir::Left,
                });
            }
            ControlState::Player => {
                events.send(CreatureStep {
                    direction: OrdDir::Left,
                    entity: player.get_single().unwrap(),
                });
                turn_manager.action_this_turn = PlayerAction::Step;
                turn_end.send(EndTurn);
            }
            ControlState::CasteMenu => todo!(),
        }
    }
    if input.just_pressed(KeyCode::ArrowDown) || input.just_pressed(KeyCode::KeyS) {
        match state.get() {
            ControlState::Cursor => {
                cursor.send(CursorStep {
                    direction: OrdDir::Down,
                });
            }
            ControlState::Player => {
                events.send(CreatureStep {
                    direction: OrdDir::Down,
                    entity: player.get_single().unwrap(),
                });
                turn_manager.action_this_turn = PlayerAction::Step;
                turn_end.send(EndTurn);
            }
            ControlState::CasteMenu => todo!(),
        }
    }
    if input.just_pressed(KeyCode::KeyZ) || input.just_pressed(KeyCode::KeyX) {
        respawn.send(RespawnPlayer { victorious: false });
    }

    if input.just_pressed(KeyCode::KeyC) {
        match state.get() {
            ControlState::Cursor => next_state.set(ControlState::Player),
            _ => next_state.set(ControlState::Cursor),
        }
    }
    if input.just_pressed(KeyCode::KeyE) {
        match state.get() {
            ControlState::CasteMenu => next_state.set(ControlState::Player),
            _ => next_state.set(ControlState::CasteMenu),
        }
    }
    if input.pressed(KeyCode::KeyO) {
        scale.0 += 0.02;
    }
    if input.pressed(KeyCode::KeyP) {
        scale.0 -= 0.02;
    }
}
