use crate::{
    creature::{get_species_sprite, Player, Species},
    events::CagePainter,
    graphics::{SlideAnimation, SpriteSheetAtlas},
    map::{Map, Position},
    text::match_species_with_description,
    ui::{
        match_species_with_string, spawn_split_text, AxiomBox, CursorBox, MessageLog, RecipebookUI,
    },
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
pub struct Cursor(Entity);

pub fn spawn_cursor(
    player: Query<(Entity, &Position), With<Player>>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    mut commands: Commands,
    mut set: ParamSet<(
        Query<&mut Visibility, With<MessageLog>>,
        Query<&mut Visibility, With<RecipebookUI>>,
        Query<&mut Visibility, With<AxiomBox>>,
        Query<&mut Visibility, With<CursorBox>>,
    )>,
    painter: Res<CagePainter>,
) -> Result {
    let (entity, player_position) = player.single()?;
    commands.spawn((
        *player_position,
        Cursor(entity),
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
    if painter.is_painting {
        *set.p1().single_mut()? = Visibility::Hidden;
        *set.p2().single_mut()? = Visibility::Hidden;
    }
    *set.p0().single_mut()? = Visibility::Hidden;
    *set.p3().single_mut()? = Visibility::Inherited;
    Ok(())
}

pub fn despawn_cursor(
    mut commands: Commands,
    cursor: Query<Entity, With<Cursor>>,
    mut set: ParamSet<(
        Query<&mut Visibility, With<MessageLog>>,
        Query<&mut Visibility, With<RecipebookUI>>,
        Query<&Visibility, With<AxiomBox>>,
        Query<&mut Visibility, With<CursorBox>>,
    )>,
    painter: Res<CagePainter>,
) -> Result {
    commands.entity(cursor.single()?).despawn();
    if painter.is_painting {
        *set.p1().single_mut()? = Visibility::Inherited;
    }
    if matches!(set.p2().single()?, Visibility::Hidden) {
        *set.p0().single_mut()? = Visibility::Inherited;
    }
    *set.p3().single_mut()? = Visibility::Hidden;
    Ok(())
}

#[derive(Event)]
pub struct CursorStep {
    pub direction: OrdDir,
}

pub fn cursor_step(
    mut events: EventReader<CursorStep>,
    mut teleporter: EventWriter<TeleportCursor>,
    cursor: Query<&Position, With<Cursor>>,
) -> Result {
    for event in events.read() {
        let cursor_pos = cursor.single()?;
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.write(TeleportCursor {
            destination: Position::new(cursor_pos.x + off_x, cursor_pos.y + off_y),
        });
    }
    Ok(())
}

#[derive(Event)]
pub struct TeleportCursor {
    pub destination: Position,
}

pub fn teleport_cursor(
    mut events: EventReader<TeleportCursor>,
    mut cursor: Query<(Entity, &mut Position, &mut Cursor)>,
    mut commands: Commands,
    map: Res<Map>,
) -> Result {
    for event in events.read() {
        let (entity, mut cursor_position, mut cursor_target) = cursor.single_mut()?;
        cursor_position.update(event.destination.x, event.destination.y);
        if let Some(new_creature) = map.get_entity_at(cursor_position.x, cursor_position.y) {
            cursor_target.0 = *new_creature;
        }
        commands.entity(entity).insert(SlideAnimation);
    }
    Ok(())
}

pub fn update_cursor_box(
    cursor: Query<&Cursor, Changed<Cursor>>,
    creature_query: Query<&Species>,
    cursor_box: Query<Entity, With<CursorBox>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) -> Result {
    if let Ok(examined_entity) = cursor.single() {
        let examined_entity = examined_entity.0;
        let species = creature_query.get(examined_entity).unwrap();
        let cursor_box = cursor_box.single()?;
        // TODO: Instead of multiple entities, would it be interesting to
        // have these merged into a single string with \n to space them out?
        // This would be good in case there's a ton of "effects flags".
        let (mut species_name, mut species_description) =
            (Entity::PLACEHOLDER, Entity::PLACEHOLDER);
        commands.entity(cursor_box).despawn_related::<Children>();
        commands.entity(cursor_box).with_children(|parent| {
            species_name =
                spawn_split_text(&match_species_with_string(species), parent, &asset_server);
            species_description = spawn_split_text(
                match_species_with_description(species),
                parent,
                &asset_server,
            );
            parent.spawn((
                ImageNode {
                    image: asset_server.load("spritesheet.png"),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_layout.handle.clone(),
                        index: get_species_sprite(species),
                    }),
                    ..Default::default()
                },
                Node {
                    width: Val::Px(3.),
                    height: Val::Px(3.),
                    right: Val::Px(0.3),
                    top: Val::Px(0.5),
                    position_type: PositionType::Absolute,
                    ..default()
                },
            ));
        });
        commands.entity(species_name).insert(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.5),
            ..default()
        });
        commands.entity(species_description).insert(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(3.5),
            ..default()
        });
    }
    Ok(())
}
