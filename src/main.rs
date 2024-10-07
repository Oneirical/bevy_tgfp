mod creature;
mod events;
mod graphics;
mod input;
mod map;
mod spells;

use bevy::prelude::*;
use events::EventPlugin;
use graphics::GraphicsPlugin;
use input::InputPlugin;
use map::MapPlugin;
use spells::SpellPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            SpellPlugin,
            EventPlugin,
            GraphicsPlugin,
            MapPlugin,
            InputPlugin,
        ))
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
        match self {
            OrdDir::Up => (0, 1),
            OrdDir::Right => (1, 0),
            OrdDir::Down => (0, -1),
            OrdDir::Left => (-1, 0),
        }
    }

    pub fn as_variant(dx: i32, dy: i32) -> Self {
        match (dx, dy) {
            (0, 1) => OrdDir::Up,
            (0, -1) => OrdDir::Down,
            (1, 0) => OrdDir::Right,
            (-1, 0) => OrdDir::Left,
            _ => panic!("Invalid offset provided."),
        }
    }
}
