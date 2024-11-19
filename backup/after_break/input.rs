use bevy::prelude::*;

use crate::{
    creature::Player,
    events::{CreatureStep, EndTurn},
    graphics::{all_animations_complete, AnimationDelay},
    sets::TurnProgression,
    spells::{Axiom, CastSpell, Spell},
    OrdDir,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            keyboard_input.run_if(in_state(TurnProgression::Animating)),
        );
    }
}

/// Each frame, if a button is pressed, move the player 1 tile.
pub fn keyboard_input(
    player: Query<Entity, With<Player>>,
    mut events: EventWriter<CreatureStep>,
    input: Res<ButtonInput<KeyCode>>,
    mut spell: EventWriter<CastSpell>,
    mut turn_end: EventWriter<EndTurn>,
    mut next_state: ResMut<NextState<TurnProgression>>,
    mut animation_delay: ResMut<AnimationDelay>,
) {
    if let Ok(player) = player.get_single() {
        if input.any_just_pressed([
            KeyCode::Space,
            KeyCode::KeyW,
            KeyCode::KeyS,
            KeyCode::KeyA,
            KeyCode::KeyD,
            KeyCode::Enter,
        ]) {
            next_state.set(TurnProgression::PlayerTurn);
            animation_delay.delay = 0.;
        }
        if input.just_pressed(KeyCode::Space) {
            spell.send(CastSpell {
                caster: player,
                spell: Spell {
                    axioms: vec![
                        Axiom::Halo { radius: 4 },
                        Axiom::RandomCasterTeleport,
                        Axiom::PurgeTargets,
                        Axiom::Plus,
                        Axiom::RepressionDamage { damage: 1 },
                        Axiom::PurgeTargets,
                        Axiom::LoopBack { steps: 6 },
                        Axiom::LoopBack { steps: 6 },
                        Axiom::LoopBack { steps: 6 },
                        Axiom::LoopBack { steps: 6 },
                    ],
                },
            });
            turn_end.send(EndTurn);
        }
        if input.just_pressed(KeyCode::Enter) {
            spell.send(CastSpell {
                caster: player,
                spell: Spell {
                    axioms: vec![Axiom::Ego, Axiom::Dash { max_distance: 4 }],
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
