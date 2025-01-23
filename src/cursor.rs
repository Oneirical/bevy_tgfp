use crate::{
    creature::Player,
    graphics::{SlideAnimation, SpriteSheetAtlas},
    map::Position,
    OrdDir, TILE_SIZE,
};
use bevy::prelude::*;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CursorStep>();
        app.add_event::<TeleportCursor>();
    }
}

#[derive(Component)]
pub struct Cursor;

pub fn spawn_cursor(
    player: Query<&Position, With<Player>>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    mut commands: Commands,
) {
    let player_position = player.single();
    commands.spawn((
        *player_position,
        Cursor,
        Sprite {
            image: asset_server.load("spritesheet.png"),
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            texture_atlas: Some(TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index: 18,
            }),
            ..default()
        },
        Transform::from_translation(Vec3::new(0., 0., 3.)),
    ));
}

pub fn despawn_cursor(mut commands: Commands, cursor: Query<Entity, With<Cursor>>) {
    commands.entity(cursor.single()).despawn();
}

#[derive(Event)]
pub struct CursorStep {
    pub direction: OrdDir,
}

pub fn cursor_step(
    mut events: EventReader<CursorStep>,
    mut teleporter: EventWriter<TeleportCursor>,
    mut cursor: Query<&Position, With<Cursor>>,
) {
    for event in events.read() {
        let cursor_pos = cursor.single();
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportCursor {
            destination: Position::new(cursor_pos.x + off_x, cursor_pos.y + off_y),
        });
    }
}

#[derive(Event)]
pub struct TeleportCursor {
    pub destination: Position,
}

pub fn teleport_cursor(
    mut events: EventReader<TeleportCursor>,
    mut cursor: Query<(Entity, &mut Position), With<Cursor>>,
    mut commands: Commands,
) {
    for event in events.read() {
        let (entity, mut cursor_position) = cursor
            // Get the Position of the Entity targeted by TeleportEntity.
            .single_mut();
        cursor_position.update(event.destination.x, event.destination.y);
        commands.entity(entity).insert(SlideAnimation);
    }
}
