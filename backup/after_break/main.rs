mod creature;
mod events;
mod graphics;
mod input;
mod map;
mod sets;
mod spells;

use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use events::EventPlugin;
use graphics::GraphicsPlugin;
use input::InputPlugin;
use map::{MapPlugin, Position};
use sets::SetsPlugin;
use spells::SpellPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            SetsPlugin,
            SpellPlugin,
            EventPlugin,
            GraphicsPlugin,
            MapPlugin,
            InputPlugin,
        ))
        .edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        .run();
}

#[derive(Component, PartialEq, Eq, Copy, Clone, Debug)]
pub enum OrdDir {
    Up,
    Right,
    Down,
    Left,
}

impl OrdDir {
    pub fn as_offset(self) -> (i32, i32) {
        let (x, y) = match self {
            OrdDir::Up => (0, 1),
            OrdDir::Right => (1, 0),
            OrdDir::Down => (0, -1),
            OrdDir::Left => (-1, 0),
        };
        (x, y)
    }
    pub fn as_variant(dx: i32, dy: i32) -> Self {
        match (dx, dy) {
            (0, 1) => OrdDir::Up,
            (0, -1) => OrdDir::Down,
            (1, 0) => OrdDir::Right,
            (-1, 0) => OrdDir::Left,
            _ => panic!("Invalid offset provided: {dx}, {dy}"),
        }
    }

    pub fn direction_towards_adjacent_tile(source: Position, destination: Position) -> Self {
        OrdDir::as_variant(destination.x - source.x, destination.y - source.y)
    }
}
