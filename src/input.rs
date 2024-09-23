use bevy::prelude::*;

use crate::{events::PlayerStep, OrdDir};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, keyboard_input);
    }
}

/// Each frame, if a button is pressed, move the player 1 tile.
fn keyboard_input(mut events: EventWriter<PlayerStep>, input: Res<ButtonInput<KeyCode>>) {
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
