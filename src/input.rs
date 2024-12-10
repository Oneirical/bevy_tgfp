use bevy::prelude::*;

use crate::{
    creature::Player,
    events::CreatureStep,
    spells::{Axiom, CastSpell, Spell},
    OrdDir,
};

/// Each frame, if a button is pressed, move the player 1 tile.
pub fn keyboard_input(
    player: Query<Entity, With<Player>>,
    mut spell: EventWriter<CastSpell>,
    mut events: EventWriter<CreatureStep>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        spell.send(CastSpell {
            caster: player.get_single().unwrap(),
            spell: Spell {
                axioms: vec![
                    Axiom::Ego,
                    Axiom::Dash { max_distance: 5 },
                    Axiom::Halo { radius: 3 },
                    Axiom::Ego,
                    Axiom::Dash { max_distance: 3 },
                    Axiom::Halo { radius: 3 },
                    Axiom::Dash { max_distance: 3 },
                ],
            },
        });
    }
    if input.just_pressed(KeyCode::KeyW) {
        events.send(CreatureStep {
            direction: OrdDir::Up,
            entity: player.get_single().unwrap(),
        });
    }
    if input.just_pressed(KeyCode::KeyD) {
        events.send(CreatureStep {
            direction: OrdDir::Right,
            entity: player.get_single().unwrap(),
        });
    }
    if input.just_pressed(KeyCode::KeyA) {
        events.send(CreatureStep {
            direction: OrdDir::Left,
            entity: player.get_single().unwrap(),
        });
    }
    if input.just_pressed(KeyCode::KeyS) {
        events.send(CreatureStep {
            direction: OrdDir::Down,
            entity: player.get_single().unwrap(),
        });
    }
}
