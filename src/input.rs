use bevy::prelude::*;

use crate::{
    creature::Player,
    events::{CreatureStep, EndTurn},
    spells::{Axiom, CastSpell, Spell},
    OrdDir,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, keyboard_input);
    }
}

/// Each frame, if a button is pressed, move the player 1 tile.
fn keyboard_input(
    player: Query<Entity, With<Player>>,
    mut events: EventWriter<CreatureStep>,
    input: Res<ButtonInput<KeyCode>>,
    mut spell: EventWriter<CastSpell>,
    mut turn_end: EventWriter<EndTurn>,
) {
    if let Ok(player) = player.get_single() {
        if input.just_pressed(KeyCode::Space) {
            spell.send(CastSpell {
                caster: player,
                spell: Spell {
                    axioms: vec![Axiom::MomentumBeam, Axiom::Dash],
                },
            });
            turn_end.send(EndTurn);
        }
        if input.just_pressed(KeyCode::KeyW) {
            events.send(CreatureStep {
                entity: player,
                direction: OrdDir::Up,
            });
        }
        if input.just_pressed(KeyCode::KeyD) {
            events.send(CreatureStep {
                entity: player,
                direction: OrdDir::Right,
            });
        }
        if input.just_pressed(KeyCode::KeyA) {
            events.send(CreatureStep {
                entity: player,
                direction: OrdDir::Left,
            });
        }
        if input.just_pressed(KeyCode::KeyS) {
            events.send(CreatureStep {
                entity: player,
                direction: OrdDir::Down,
            });
        }
    }
}
