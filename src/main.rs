mod creature;
mod events;
mod graphics;
mod input;
mod map;
mod sets;
mod spells;
mod ui;

use bevy::{
    asset::AssetMetaCheck,
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
    window::WindowResolution,
};
use events::EventPlugin;
use graphics::GraphicsPlugin;
use map::{MapPlugin, Position};
use sets::SetsPlugin;
use spells::SpellPlugin;
use ui::UIPlugin;

pub const TILE_SIZE: f32 = 3.;

fn main() {
    let app_window = Some(Window {
        title: "The Games Foxes Play".into(),
        resolution: WindowResolution::new(5120., 2880.).with_scale_factor_override(16.),
        mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
        // mode: bevy::window::WindowMode::Windowed,
        ..default()
    });
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: app_window,
                    ..default()
                }),
        )
        .add_plugins((
            SetsPlugin,
            SpellPlugin,
            EventPlugin,
            GraphicsPlugin,
            MapPlugin,
            UIPlugin,
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

    pub fn as_variant(dx: i32, dy: i32) -> Option<Self> {
        match (dx, dy) {
            (0, 1) => Some(OrdDir::Up),
            (0, -1) => Some(OrdDir::Down),
            (1, 0) => Some(OrdDir::Right),
            (-1, 0) => Some(OrdDir::Left),
            _ => None,
        }
    }

    pub fn direction_towards_adjacent_tile(
        source: Position,
        destination: Position,
    ) -> Option<Self> {
        OrdDir::as_variant(destination.x - source.x, destination.y - source.y)
    }
}
