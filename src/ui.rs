use std::f32::consts::PI;

use bevy::prelude::*;

use crate::graphics::SpriteSheetAtlas;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

const SOUL_WHEEL_CONTAINER_SIZE: f32 = 33.;
const SOUL_WHEEL_RADIUS: f32 = 8.;
const SOUL_WHEEL_SLOT_SPRITE_SIZE: f32 = 4.;

#[derive(Component)]
pub struct SoulSlot {
    pub index: usize,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    // root node
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        })
        .insert(PickingBehavior::IGNORE)
        .with_children(|parent| {
            // left vertical fill (border)
            parent
                .spawn((
                    Node {
                        width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                        height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                        display: Display::Flex,
                        justify_content: JustifyContent::FlexEnd,
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0., 0., 0.)),
                ))
                .with_children(|parent| {
                    let rot = PI / 4.;
                    // Soul slots, arranged in a circle formation.
                    for i in 0..8 {
                        parent.spawn((
                            SoulSlot { index: i },
                            ImageNode {
                                image: asset_server.load("spritesheet.png"),
                                texture_atlas: Some(TextureAtlas {
                                    layout: atlas_layout.handle.clone(),
                                    index: 167,
                                }),
                                ..Default::default()
                            },
                            Node {
                                left: Val::Px(
                                    ((i + 6) as f32 * rot).cos() * SOUL_WHEEL_RADIUS
                                        + SOUL_WHEEL_CONTAINER_SIZE / 2.
                                        - SOUL_WHEEL_SLOT_SPRITE_SIZE,
                                ),
                                top: Val::Px(
                                    ((i + 6) as f32 * rot).sin() * SOUL_WHEEL_RADIUS
                                        + SOUL_WHEEL_CONTAINER_SIZE / 2.
                                        - SOUL_WHEEL_SLOT_SPRITE_SIZE,
                                ),
                                position_type: PositionType::Absolute,
                                width: Val::Px(SOUL_WHEEL_SLOT_SPRITE_SIZE),
                                height: Val::Px(SOUL_WHEEL_SLOT_SPRITE_SIZE),
                                ..default()
                            },
                        ));
                        parent.spawn((
                            Text::new((i + 1).to_string()),
                            TextFont {
                                font: asset_server.load("fonts/Play-Regular.ttf"),
                                font_size: 1.,
                                ..default()
                            },
                            Label,
                            Node {
                                left: Val::Px(
                                    SOUL_WHEEL_RADIUS / 1.9 * ((i + 6) as f32 * rot).cos()
                                        + SOUL_WHEEL_CONTAINER_SIZE / 2.
                                        - SOUL_WHEEL_SLOT_SPRITE_SIZE / 1.7,
                                ),
                                top: Val::Px(
                                    SOUL_WHEEL_RADIUS / 1.9 * ((i + 6) as f32 * rot).sin()
                                        + SOUL_WHEEL_CONTAINER_SIZE / 2.
                                        - SOUL_WHEEL_SLOT_SPRITE_SIZE / 1.7
                                        - 0.3,
                                ),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                        ));
                    }

                    let chains = (SOUL_WHEEL_CONTAINER_SIZE / 2. - 1.) as usize;
                    for i in 0..chains {
                        parent.spawn((
                            ImageNode {
                                image: asset_server.load("spritesheet.png"),
                                texture_atlas: Some(TextureAtlas {
                                    layout: atlas_layout.handle.clone(),
                                    index: if i == 0 || i == chains - 1 { 140 } else { 139 },
                                }),
                                ..Default::default()
                            },
                            Node {
                                top: Val::Px(-0.5),
                                left: Val::Px(-0.5 + i as f32 * 2.),
                                width: Val::Px(2.),
                                height: Val::Px(2.),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            Transform::from_rotation(if i != 0 {
                                Quat::from_rotation_z(PI / 2.)
                            } else {
                                Quat::from_rotation_z(0.)
                            }),
                        ));
                        if i != chains - 1 && i != 0 {
                            parent.spawn((
                                ImageNode {
                                    image: asset_server.load("spritesheet.png"),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: atlas_layout.handle.clone(),
                                        index: 139,
                                    }),
                                    ..Default::default()
                                },
                                Node {
                                    bottom: Val::Px(-0.5),
                                    left: Val::Px(-0.5 + i as f32 * 2.),
                                    width: Val::Px(2.),
                                    height: Val::Px(2.),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                Transform::from_rotation(Quat::from_rotation_z(3. * PI / 2.)),
                            ));
                        }
                        if i != chains - 1 {
                            parent.spawn((
                                ImageNode {
                                    image: asset_server.load("spritesheet.png"),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: atlas_layout.handle.clone(),
                                        index: if i == 0 || i == chains - 1 { 140 } else { 139 },
                                    }),
                                    ..Default::default()
                                },
                                Node {
                                    bottom: Val::Px(-0.5 + i as f32 * 2.),
                                    right: Val::Px(-0.5),
                                    width: Val::Px(2.),
                                    height: Val::Px(2.),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                Transform::from_rotation(if i == chains - 1 {
                                    Quat::from_rotation_z(3. * PI / 2.)
                                } else {
                                    Quat::from_rotation_z(PI)
                                }),
                            ));
                        }
                        if i != chains - 1 {
                            parent.spawn((
                                ImageNode {
                                    image: asset_server.load("spritesheet.png"),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: atlas_layout.handle.clone(),
                                        index: if i == 0 || i == chains - 1 { 140 } else { 139 },
                                    }),
                                    ..Default::default()
                                },
                                Node {
                                    bottom: Val::Px(-0.5 + i as f32 * 2.),
                                    left: Val::Px(-0.5),
                                    width: Val::Px(2.),
                                    height: Val::Px(2.),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                Transform::from_rotation(if i == 0 {
                                    Quat::from_rotation_z(3. * PI / 2.)
                                } else {
                                    Quat::from_rotation_z(0.)
                                }),
                            ));
                        }
                    }
                });
        });
}
