use bevy::{ecs::system::SystemId, prelude::*};

use crate::{
    creature::Player,
    events::PlayerStep,
    graphics::{MagicVfx, SlideAnimation},
    spells::{Axiom, CastSpell, Spell},
    OrdDir,
};

#[derive(Resource)]
pub struct KeyboardInputId {
    id: SystemId,
}

impl FromWorld for KeyboardInputId {
    fn from_world(world: &mut World) -> Self {
        KeyboardInputId {
            id: world.register_system(keyboard_input),
        }
    }
}

pub fn accelerate_animations(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    slide_skippers: Query<Entity, With<SlideAnimation>>,
    mut magic_vfx: Query<&mut Visibility, With<MagicVfx>>,
    keyboard_repeat: Res<KeyboardInputId>,
) {
    if input.any_just_pressed([
        KeyCode::Space,
        KeyCode::KeyW,
        KeyCode::KeyA,
        KeyCode::KeyS,
        KeyCode::KeyD,
    ]) {
        for entity in slide_skippers.iter() {
            commands.entity(entity).remove::<SlideAnimation>();
        }
        for mut visibility in magic_vfx.iter_mut() {
            *visibility = Visibility::Inherited;
        }
        dbg!("hai");
        commands.run_system(keyboard_repeat.id);
    }
}

/// Each frame, if a button is pressed, move the player 1 tile.
pub fn keyboard_input(
    player: Query<Entity, With<Player>>,
    mut spell: EventWriter<CastSpell>,
    mut events: EventWriter<PlayerStep>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        spell.send(CastSpell {
            caster: player.get_single().unwrap(),
            spell: Spell {
                axioms: vec![Axiom::MomentumBeam, Axiom::Dash { max_distance: 5 }],
            },
        });
    }
    if input.just_pressed(KeyCode::KeyW) {
        events.send(PlayerStep {
            direction: OrdDir::Up,
        });
    }
    if input.just_pressed(KeyCode::KeyD) {
        events.send(PlayerStep {
            direction: OrdDir::Right,
        });
    }
    if input.just_pressed(KeyCode::KeyA) {
        events.send(PlayerStep {
            direction: OrdDir::Left,
        });
    }
    if input.just_pressed(KeyCode::KeyS) {
        events.send(PlayerStep {
            direction: OrdDir::Down,
        });
    }
}
