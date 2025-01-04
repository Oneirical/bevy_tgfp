use bevy::prelude::*;

use crate::{
    creature::{Player, StatusEffect},
    events::{CreatureStep, DrawSoul, EndTurn, PlayerAction, TurnManager},
    spells::{Axiom, CastSpell, Spell},
    OrdDir,
};

/// Each frame, if a button is pressed, move the player 1 tile.
pub fn keyboard_input(
    player: Query<Entity, With<Player>>,
    mut spell: EventWriter<CastSpell>,
    mut draw_soul: EventWriter<DrawSoul>,
    mut events: EventWriter<CreatureStep>,
    input: Res<ButtonInput<KeyCode>>,
    mut turn_manager: ResMut<TurnManager>,
    mut turn_end: EventWriter<EndTurn>,
) {
    if input.just_pressed(KeyCode::Digit1) {
        spell.send(CastSpell {
            caster: player.get_single().unwrap(),
            spell: Spell {
                axioms: vec![Axiom::Ego, Axiom::Plus, Axiom::HealOrHarm { amount: 2 }],
            },
        });
        turn_manager.action_this_turn = PlayerAction::Spell;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::Digit2) {
        spell.send(CastSpell {
            caster: player.get_single().unwrap(),
            spell: Spell {
                axioms: vec![
                    Axiom::Ego,
                    Axiom::StatusEffect {
                        effect: StatusEffect::Invincible,
                        potency: 1,
                        stacks: 2,
                    },
                ],
            },
        });
        turn_manager.action_this_turn = PlayerAction::Spell;
        // No end_turn for the shield.
    }
    if input.just_pressed(KeyCode::Digit3) {
        spell.send(CastSpell {
            caster: player.get_single().unwrap(),
            spell: Spell {
                axioms: vec![
                    Axiom::Ego,
                    Axiom::PlaceStepTrap,
                    Axiom::PiercingBeams,
                    Axiom::PlusBeam,
                    Axiom::Ego,
                    Axiom::HealOrHarm { amount: -2 },
                ],
            },
        });
        turn_manager.action_this_turn = PlayerAction::Spell;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::Digit4) {
        spell.send(CastSpell {
            caster: player.get_single().unwrap(),
            spell: Spell {
                axioms: vec![Axiom::XBeam, Axiom::HealOrHarm { amount: -2 }],
            },
        });
        turn_manager.action_this_turn = PlayerAction::Spell;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::Digit5) {
        spell.send(CastSpell {
            caster: player.get_single().unwrap(),
            spell: Spell {
                axioms: vec![
                    Axiom::Ego,
                    Axiom::Trace,
                    Axiom::Dash { max_distance: 5 },
                    Axiom::Spread,
                    Axiom::UntargetCaster,
                    Axiom::HealOrHarm { amount: -1 },
                    Axiom::PurgeTargets,
                    Axiom::Touch,
                    Axiom::StatusEffect {
                        effect: StatusEffect::Dizzy,
                        potency: 1,
                        stacks: 2,
                    },
                    Axiom::Dash { max_distance: 1 },
                ],
            },
        });
        turn_manager.action_this_turn = PlayerAction::Spell;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::Digit6) {
        spell.send(CastSpell {
            caster: player.get_single().unwrap(),
            spell: Spell {
                axioms: vec![
                    Axiom::Ego,
                    Axiom::StatusEffect {
                        effect: StatusEffect::Stab,
                        potency: 5,
                        stacks: 20,
                    },
                ],
            },
        });
        turn_manager.action_this_turn = PlayerAction::Spell;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::KeyQ) {
        draw_soul.send(DrawSoul { amount: 1 });
        turn_manager.action_this_turn = PlayerAction::Draw;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::KeyW) {
        events.send(CreatureStep {
            direction: OrdDir::Up,
            entity: player.get_single().unwrap(),
        });
        turn_manager.action_this_turn = PlayerAction::Step;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::KeyD) {
        events.send(CreatureStep {
            direction: OrdDir::Right,
            entity: player.get_single().unwrap(),
        });
        turn_manager.action_this_turn = PlayerAction::Step;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::KeyA) {
        events.send(CreatureStep {
            direction: OrdDir::Left,
            entity: player.get_single().unwrap(),
        });
        turn_manager.action_this_turn = PlayerAction::Step;
        turn_end.send(EndTurn);
    }
    if input.just_pressed(KeyCode::KeyS) {
        events.send(CreatureStep {
            direction: OrdDir::Down,
            entity: player.get_single().unwrap(),
        });
        turn_manager.action_this_turn = PlayerAction::Step;
        turn_end.send(EndTurn);
    }
}
