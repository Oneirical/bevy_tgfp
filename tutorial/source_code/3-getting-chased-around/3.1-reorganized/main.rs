mod creature;
mod graphics;
mod input;
mod map;

use bevy::prelude::*;
use graphics::GraphicsPlugin;
use input::InputPlugin;
use map::MapPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((GraphicsPlugin, MapPlugin, InputPlugin))
        .run();
}
