use bevy::prelude::*;

use crate::{
    creature::Player,
    events::{CreatureStep, EndTurn, PlayerAction, TurnManager},
    spells::{Axiom, CastSpell, Spell},
    OrdDir,
};

/// Each frame, if a button is pressed, move the player 1 tile.
pub fn keyboard_input(
    player: Query<Entity, With<Player>>,
    mut spell: EventWriter<CastSpell>,
    mut events: EventWriter<CreatureStep>,
    input: Res<ButtonInput<KeyCode>>,
    mut turn_manager: ResMut<TurnManager>,
    mut turn_end: EventWriter<EndTurn>,
) {
    if input.just_pressed(KeyCode::Space) {
        spell.send(CastSpell {
            caster: player.get_single().unwrap(),
            spell: Spell {
                axioms: vec![Axiom::Ego, Axiom::ArchitectCage],
            },
        });
        turn_manager.action_this_turn = PlayerAction::Spell;
        turn_end.send(EndTurn { speed_level: 1 });
    }
    if input.just_pressed(KeyCode::KeyW) {
        events.send(CreatureStep {
            direction: OrdDir::Up,
            entity: player.get_single().unwrap(),
        });
        turn_manager.action_this_turn = PlayerAction::Step;
        turn_end.send(EndTurn { speed_level: 1 });
    }
    if input.just_pressed(KeyCode::KeyD) {
        events.send(CreatureStep {
            direction: OrdDir::Right,
            entity: player.get_single().unwrap(),
        });
        turn_manager.action_this_turn = PlayerAction::Step;
        turn_end.send(EndTurn { speed_level: 1 });
    }
    if input.just_pressed(KeyCode::KeyA) {
        events.send(CreatureStep {
            direction: OrdDir::Left,
            entity: player.get_single().unwrap(),
        });
        turn_manager.action_this_turn = PlayerAction::Step;
        turn_end.send(EndTurn { speed_level: 1 });
    }
    if input.just_pressed(KeyCode::KeyS) {
        events.send(CreatureStep {
            direction: OrdDir::Down,
            entity: player.get_single().unwrap(),
        });
        turn_manager.action_this_turn = PlayerAction::Step;
        turn_end.send(EndTurn { speed_level: 1 });
    }
}
