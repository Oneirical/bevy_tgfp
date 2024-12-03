use bevy::prelude::*;

use crate::{
    creature::{Creature, Player},
    graphics::SpriteSheetAtlas,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);
        app.add_systems(Startup, spawn_cage);
    }
}

/// A position on the map.
#[derive(Component, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    /// Create a new Position instance.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Edit an existing Position with new coordinates.
    pub fn update(&mut self, x: i32, y: i32) {
        (self.x, self.y) = (x, y);
    }

    /// Shift the position by a delta.
    pub fn shift(&mut self, dx: i32, dy: i32) {
        (self.x, self.y) = (self.x + dx, self.y + dy);
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    commands.spawn((
        Creature {
            position: Position { x: 4, y: 4 },
            sprite: Sprite {
                image: asset_server.load("spritesheet.png"),
                custom_size: Some(Vec2::new(64., 64.)),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index: 0,
                }),
                ..default()
            },
        },
        Player,
    ));
}

fn spawn_cage(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    let cage = "#########\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #########";
    for (idx, tile_char) in cage.char_indices() {
        let position = Position::new(idx as i32 % 9, idx as i32 / 9);
        let index = match tile_char {
            '#' => 3,
            _ => continue,
        };
        commands.spawn(Creature {
            position,
            sprite: Sprite {
                image: asset_server.load("spritesheet.png"),
                custom_size: Some(Vec2::new(64., 64.)),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index,
                }),
                ..default()
            },
        });
    }
}
