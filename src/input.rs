use bevy::{ecs::system::SystemId, prelude::*};

use crate::{
    creature::Player,
    events::CreatureStep,
    graphics::{MagicVfx, SlideAnimation},
    spells::{Axiom, CastSpell, Spell, SpellStack},
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

/// If any key is pressed during animations, skip the rest of the animations
/// and execute that action.
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
            // All sliding creatures snap to their destination.
            commands.entity(entity).remove::<SlideAnimation>();
        }
        for mut visibility in magic_vfx.iter_mut() {
            // All visual magic effects become visible.
            *visibility = Visibility::Inherited;
        }
        commands.run_system(keyboard_repeat.id);
    }
}

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
                axioms: vec![Axiom::MomentumBeam, Axiom::Dash { max_distance: 5 }],
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
