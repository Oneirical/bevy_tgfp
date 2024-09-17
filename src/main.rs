use bevy::prelude::*;
use events::EventPlugin;
use graphics::GraphicsPlugin;
use input::InputPlugin;
use map::{MapPlugin, Position};

mod creature;
mod events;
mod graphics;
mod input;
mod map;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((EventPlugin, GraphicsPlugin, MapPlugin, InputPlugin))
        .run();
}

#[derive(Copy, Clone, Debug)]
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
}
