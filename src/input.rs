use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    caste::{EquipSpell, UnequipSpell},
    creature::{EffectDuration, Player, Soul, StatusEffect},
    cursor::CursorStep,
    events::{CreatureStep, EndTurn, PlayerAction, RespawnPlayer, TurnManager, UseWheelSoul},
    graphics::PortalCamera,
    map::ConveyorTracker,
    sets::ControlState,
    spells::{Axiom, CastSpell, Spell},
    ui::{CastePanelColumn, CastePanelRow, LargeCastePanel},
    OrdDir,
};

/// Each frame, if a button is pressed, move the player 1 tile.
pub fn keyboard_input(
    player: Query<Entity, With<Player>>,
    mut use_wheel_soul: EventWriter<UseWheelSoul>,
    // mut draw_soul: EventWriter<DrawSoul>,
    mut events: EventWriter<CreatureStep>,
    input: Res<ButtonInput<KeyCode>>,
    mut turn_manager: ResMut<TurnManager>,
    mut turn_end: EventWriter<EndTurn>,
    mut respawn: EventWriter<RespawnPlayer>,
    state: Res<State<ControlState>>,
    mut next_state: ResMut<NextState<ControlState>>,
    mut cursor: EventWriter<CursorStep>,
    mut caste_menu: Query<&mut LargeCastePanel>,
    mut equip: EventWriter<EquipSpell>,
    mut unequip: EventWriter<UnequipSpell>,
    mut spell: EventWriter<CastSpell>,
    mut tracker: ResMut<ConveyorTracker>,
) -> Result {
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
                        use_wheel_soul.write(UseWheelSoul { index: i });
                        turn_manager.action_this_turn = PlayerAction::Spell;
                        turn_end.write(EndTurn);
                    }
                    _ => (),
                }
            }
        }
    }
    // if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::KeyQ) {
    //     draw_soul.write(DrawSoul { amount: 1 });
    //     turn_manager.action_this_turn = PlayerAction::Draw;
    //     turn_end.write(EndTurn);
    // }
    if input.just_pressed(KeyCode::ArrowUp) || input.just_pressed(KeyCode::KeyW) {
        match state.get() {
            ControlState::Cursor => {
                cursor.write(CursorStep {
                    direction: OrdDir::Up,
                });
            }
            ControlState::Player => {
                events.write(CreatureStep {
                    direction: OrdDir::Up,
                    entity: player.single()?,
                });
                turn_manager.action_this_turn = PlayerAction::Step;
                turn_end.write(EndTurn);
            }
            ControlState::CasteMenu => {
                let mut caste_menu = caste_menu.single_mut()?;
                let column = caste_menu.selected_column;
                caste_menu.selected_row.shift(-1, &column);
            }
            ControlState::QuestMenu => (),
        }
    }
    if input.just_pressed(KeyCode::ArrowRight) || input.just_pressed(KeyCode::KeyD) {
        match state.get() {
            ControlState::Cursor => {
                cursor.write(CursorStep {
                    direction: OrdDir::Right,
                });
            }
            ControlState::Player => {
                events.write(CreatureStep {
                    direction: OrdDir::Right,
                    entity: player.single()?,
                });
                turn_manager.action_this_turn = PlayerAction::Step;
                turn_end.write(EndTurn);
            }
            ControlState::CasteMenu => {
                let mut caste_menu = caste_menu.single_mut()?;
                caste_menu.selected_column.shift(1);
                if matches!(
                    caste_menu.selected_column,
                    CastePanelColumn::LibraryLeft | CastePanelColumn::LibraryRight,
                ) && !matches!(caste_menu.selected_row, CastePanelRow::Library(_))
                {
                    caste_menu.selected_row = CastePanelRow::Library(0);
                } else if !matches!(
                    caste_menu.selected_column,
                    CastePanelColumn::LibraryLeft | CastePanelColumn::LibraryRight,
                ) {
                    caste_menu.selected_row = CastePanelRow::Top;
                }
            }
            ControlState::QuestMenu => (),
        }
    }
    if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::KeyA) {
        match state.get() {
            ControlState::Cursor => {
                cursor.write(CursorStep {
                    direction: OrdDir::Left,
                });
            }
            ControlState::Player => {
                events.write(CreatureStep {
                    direction: OrdDir::Left,
                    entity: player.single()?,
                });
                turn_manager.action_this_turn = PlayerAction::Step;
                turn_end.write(EndTurn);
            }
            ControlState::CasteMenu => {
                let mut caste_menu = caste_menu.single_mut()?;
                caste_menu.selected_column.shift(-1);
                if matches!(
                    caste_menu.selected_column,
                    CastePanelColumn::LibraryLeft | CastePanelColumn::LibraryRight,
                ) && !matches!(caste_menu.selected_row, CastePanelRow::Library(_))
                {
                    caste_menu.selected_row = CastePanelRow::Library(0);
                } else if !matches!(
                    caste_menu.selected_column,
                    CastePanelColumn::LibraryLeft | CastePanelColumn::LibraryRight,
                ) {
                    caste_menu.selected_row = CastePanelRow::Top;
                }
            }
            ControlState::QuestMenu => (),
        }
    }
    if input.just_pressed(KeyCode::ArrowDown) || input.just_pressed(KeyCode::KeyS) {
        match state.get() {
            ControlState::Cursor => {
                cursor.write(CursorStep {
                    direction: OrdDir::Down,
                });
            }
            ControlState::Player => {
                events.write(CreatureStep {
                    direction: OrdDir::Down,
                    entity: player.single()?,
                });
                turn_manager.action_this_turn = PlayerAction::Step;
                turn_end.write(EndTurn);
            }
            ControlState::CasteMenu => {
                let mut caste_menu = caste_menu.single_mut()?;
                let column = caste_menu.selected_column;
                caste_menu.selected_row.shift(1, &column);
            }
            ControlState::QuestMenu => (),
        }
    }
    if input.just_pressed(KeyCode::KeyZ) || input.just_pressed(KeyCode::KeyX) {
        respawn.write(RespawnPlayer { victorious: false });
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
    if input.just_pressed(KeyCode::KeyQ) {
        match state.get() {
            ControlState::QuestMenu => next_state.set(ControlState::Player),
            _ => next_state.set(ControlState::QuestMenu),
        }
    }

    if input.just_pressed(KeyCode::Escape) {
        next_state.set(ControlState::Player);
    }

    if input.just_pressed(KeyCode::KeyP) {
        tracker.open_doors_next = false;
    }

    #[cfg(debug_assertions)]
    if input.pressed(KeyCode::KeyR) {
        spell.write(CastSpell {
            caster: player.single()?,
            spell: Spell {
                axioms: vec![
                    Axiom::Ego,
                    Axiom::StatusEffect {
                        effect: StatusEffect::Invincible,
                        potency: 1,
                        stacks: EffectDuration::Finite { stacks: 2 },
                    },
                    Axiom::Spread,
                    Axiom::Spread,
                    Axiom::Spread,
                    Axiom::Spread,
                    Axiom::Spread,
                    Axiom::HealOrHarm { amount: -6 },
                ],
                caste: Soul::Saintly,
                icon: 10,
                id: Uuid::new_v4(),
                description: String::new(),
            },
            starting_step: 0,
            soul_caste: Soul::Saintly,
        });
    }

    if input.just_pressed(KeyCode::Enter) {
        let caste_menu = caste_menu.single()?;
        if let CastePanelRow::Library(depth) = caste_menu.selected_row {
            equip.write(EquipSpell {
                index: match caste_menu.selected_column {
                    CastePanelColumn::LibraryLeft => depth * 2,
                    CastePanelColumn::LibraryRight => depth * 2 + 1,
                    _ => panic!(),
                },
            });
        } else {
            unequip.write(UnequipSpell {
                caste: match (caste_menu.selected_column, caste_menu.selected_row) {
                    (CastePanelColumn::Left, CastePanelRow::Top) => Soul::Saintly,
                    (CastePanelColumn::Left, CastePanelRow::Middle) => Soul::Artistic,
                    (CastePanelColumn::Left, CastePanelRow::Bottom) => Soul::Feral,
                    (CastePanelColumn::Right, CastePanelRow::Top) => Soul::Ordered,
                    (CastePanelColumn::Right, CastePanelRow::Middle) => Soul::Unhinged,
                    (CastePanelColumn::Right, CastePanelRow::Bottom) => Soul::Vile,
                    _ => panic!(),
                },
            });
        }
    }
    Ok(())
}
