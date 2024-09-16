use bevy::prelude::*;
use std::time::Duration;

use crate::{events::PlayerStep, OrdDir};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputDelay {
            timer: Timer::new(Duration::from_millis(120), TimerMode::Once),
        });
        app.add_systems(Update, keyboard_input);
    }
}

/// How long to wait until input is accepted again.
#[derive(Resource)]
pub struct InputDelay {
    pub timer: Timer,
}

fn keyboard_input(
    time: Res<Time>,

    mut delay: ResMut<InputDelay>,
    mut events: EventWriter<PlayerStep>,
    input: Res<ButtonInput<KeyCode>>,
) {
    delay.timer.tick(time.delta());
    if !delay.timer.finished() {
        return;
    }

    if input.pressed(KeyCode::KeyW) {
        events.send(PlayerStep {
            direction: OrdDir::Up,
        });
    }
    if input.pressed(KeyCode::KeyD) {
        events.send(PlayerStep {
            direction: OrdDir::Right,
        });
    }
    if input.pressed(KeyCode::KeyA) {
        events.send(PlayerStep {
            direction: OrdDir::Left,
        });
    }
    if input.pressed(KeyCode::KeyS) {
        events.send(PlayerStep {
            direction: OrdDir::Down,
        });
    }
}
