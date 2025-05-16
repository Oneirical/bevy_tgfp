use std::{f32::consts::PI, time::Duration};

use uuid::Uuid;

use bevy::{prelude::*, text::TextLayoutInfo, window::WindowResized};

use crate::{
    caste::{on_click_equip_unequip, on_hover_move_caste_cursor},
    creature::{Soul, Species},
    events::mouse_use_wheel_soul,
    graphics::SpriteSheetAtlas,
    text::{split_text, LORE},
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, on_resize_system);
        app.add_event::<AnnounceGameOver>();
        app.add_event::<AddMessage>();
        app.add_event::<SlideMessages>();
    }
}

pub const SOUL_WHEEL_CONTAINER_SIZE: f32 = 33.;
const SOUL_WHEEL_RADIUS: f32 = 8.;
const SOUL_WHEEL_SLOT_SPRITE_SIZE: f32 = 4.;
const CHAIN_SIZE: f32 = 2.;
const TITLE_FADE_TIME: f32 = 3.;

#[derive(Component)]
pub struct SoulSlot {
    pub index: usize,
}

#[derive(Component)]
pub struct FadingTitle {
    timer: Timer,
}

#[derive(Event)]
pub struct AnnounceGameOver {
    pub victorious: bool,
}

fn on_resize_system(mut resize_reader: EventReader<WindowResized>, mut scale: ResMut<UiScale>) {
    for e in resize_reader.read() {
        scale.0 = e.height * 16. / 1080.;
    }
}

impl FadingTitle {
    pub fn new(delay: f32) -> Self {
        Self {
            timer: Timer::from_seconds(delay, TimerMode::Once),
        }
    }
}

pub fn despawn_fading_title(fading: Query<(Entity, &FadingTitle)>, mut commands: Commands) {
    for (entity, title) in fading.iter() {
        if title.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn decay_fading_title(
    mut set: ParamSet<(
        Query<(&mut ImageNode, &mut FadingTitle)>,
        Query<(&mut TextColor, &mut FadingTitle)>,
        Query<(&mut BackgroundColor, &mut FadingTitle)>,
    )>,
    time: Res<Time>,
) {
    for (mut chain, mut fade) in set.p0().iter_mut() {
        fade.timer.tick(time.delta());
        chain.color.set_alpha(fade.timer.fraction_remaining());
    }
    for (mut text, mut fade) in set.p1().iter_mut() {
        fade.timer.tick(time.delta());
        text.0.set_alpha(fade.timer.fraction_remaining());
    }
    for (mut text_box, mut fade) in set.p2().iter_mut() {
        fade.timer.tick(time.delta());
        text_box.0.set_alpha(fade.timer.fraction_remaining());
    }
}

pub fn spawn_fading_title(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    mut events: EventReader<AnnounceGameOver>,
) {
    for event in events.read() {
        // root node
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            width: Val::Px(47.),
                            height: Val::Px(10.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0., 0., 0.)),
                        FadingTitle::new(TITLE_FADE_TIME / 2.),
                    ))
                    .with_children(|parent| {
                        let chains = 6;
                        for i in 0..chains {
                            // right chains
                            if i != chains - 1 {
                                parent.spawn((
                                    ImageNode {
                                        image: asset_server.load("spritesheet.png"),
                                        texture_atlas: Some(TextureAtlas {
                                            layout: atlas_layout.handle.clone(),
                                            index: if i == 0 || i == chains - 1 {
                                                140
                                            } else {
                                                139
                                            },
                                        }),
                                        ..Default::default()
                                    },
                                    FadingTitle::new(TITLE_FADE_TIME),
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
                            // left chains
                            if i != chains - 1 {
                                parent.spawn((
                                    ImageNode {
                                        image: asset_server.load("spritesheet.png"),
                                        texture_atlas: Some(TextureAtlas {
                                            layout: atlas_layout.handle.clone(),
                                            index: if i == 0 || i == chains - 1 {
                                                140
                                            } else {
                                                139
                                            },
                                        }),
                                        ..Default::default()
                                    },
                                    FadingTitle::new(TITLE_FADE_TIME),
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
                        let chains = 24;
                        for i in 0..chains {
                            // top chains
                            parent.spawn((
                                ImageNode {
                                    image: asset_server.load("spritesheet.png"),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: atlas_layout.handle.clone(),
                                        index: if i == 0 || i == chains - 1 { 140 } else { 139 },
                                    }),
                                    ..Default::default()
                                },
                                FadingTitle::new(TITLE_FADE_TIME),
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
                            // bottom chains
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
                                    FadingTitle::new(TITLE_FADE_TIME),
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
                        }
                        parent.spawn((
                            Text::new(if event.victorious {
                                "VICTORIOUS"
                            } else {
                                "DEFEATED"
                            }),
                            TextLayout {
                                justify: JustifyText::Center,
                                linebreak: LineBreak::WordBoundary,
                            },
                            FadingTitle::new(TITLE_FADE_TIME),
                            TextFont {
                                font: asset_server.load("fonts/Play-Regular.ttf"),
                                font_size: 8.,
                                ..default()
                            },
                            TextColor(if event.victorious {
                                Color::srgb(0.31, 0.99, 0.25)
                            } else {
                                Color::srgb(0.97, 0.28, 0.25)
                            }),
                            Label,
                            Node { ..default() },
                        ));
                    });
            });
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        Pickable::IGNORE,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            children![
                (
                    RecipebookUI,
                    ChainBox,
                    Visibility::Hidden,
                    Node {
                        width: Val::Px(19.),
                        height: Val::Px(65.),
                        min_width: Val::Px(19.),
                        max_width: Val::Px(19.),
                        min_height: Val::Px(65.),
                        max_height: Val::Px(65.),
                        row_gap: Val::Px(1.),
                        column_gap: Val::Px(1.),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::new(Val::Px(2.), Val::Px(0.), Val::Px(2.), Val::Px(0.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0., 0., 0.)),
                ),
                children![
                    (
                        Node {
                            width: Val::Px(16.),
                            height: Val::Px(15.),
                            row_gap: Val::Px(1.),
                            column_gap: Val::Px(1.),
                            top: Val::Px(1.),
                            left: Val::Px(1.),
                            position_type: PositionType::Relative,
                            flex_wrap: FlexWrap::Wrap,
                            align_content: AlignContent::FlexStart,
                            ..default()
                        },
                        CraftingPredictor,
                    ),
                    (
                        Node {
                            width: Val::Px(16.),
                            height: Val::Px(46.),
                            row_gap: Val::Px(1.),
                            column_gap: Val::Px(1.),
                            top: Val::Px(1.),
                            left: Val::Px(1.),
                            position_type: PositionType::Relative,
                            flex_wrap: FlexWrap::Wrap,
                            align_content: AlignContent::FlexStart,
                            ..default()
                        },
                        CraftingPatterns,
                    )
                ]
            ]
        )],
    ));
    // TODO add this linear chain separator to recipe book ui
    // let chains = 9;
    // for i in 0..chains {
    //     if i != chains - 1 && i != 0 {
    //         parent.spawn((
    //             ImageNode {
    //                 image: asset_server.load("spritesheet.png"),
    //                 texture_atlas: Some(TextureAtlas {
    //                     layout: atlas_layout.handle.clone(),
    //                     index: 139,
    //                 }),
    //                 ..Default::default()
    //             },
    //             Node {
    //                 top: Val::Px(14.3),
    //                 left: Val::Px(-0.5 + i as f32 * 2.),
    //                 width: Val::Px(2.),
    //                 height: Val::Px(2.),
    //                 position_type: PositionType::Absolute,
    //                 ..default()
    //             },
    //             Transform::from_rotation(Quat::from_rotation_z(3. * PI / 2.)),
    //         ));
    //     }
    // }
    // root node
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        })
        .insert(Pickable::IGNORE)
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            ChainBox,
                            Node {
                                width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                min_width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                max_width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                min_height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                max_height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                border: UiRect::new(
                                    Val::Px(0.),
                                    Val::Px(2.),
                                    Val::Px(2.),
                                    Val::Px(0.),
                                ),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0., 0., 0.)),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                PatternBox,
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                    height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                    min_width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                    max_width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                    min_height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                    max_height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                    ..default()
                                },
                                Visibility::Hidden,
                            ));
                            parent
                                .spawn((
                                    SoulWheelBox,
                                    Node {
                                        position_type: PositionType::Absolute,
                                        width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                        height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                        min_width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                        max_width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                        min_height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                        max_height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                        ..default()
                                    },
                                ))
                                .with_children(|parent| {
                                    let rot = PI / 4.;
                                    // Soul slots, arranged in a circle formation.
                                    for i in 0..8 {
                                        parent
                                            .spawn((
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
                                                        ((i + 6) as f32 * rot).cos()
                                                            * SOUL_WHEEL_RADIUS
                                                            + SOUL_WHEEL_CONTAINER_SIZE / 2.
                                                            - SOUL_WHEEL_SLOT_SPRITE_SIZE
                                                            + 1.,
                                                    ),
                                                    top: Val::Px(
                                                        ((i + 6) as f32 * rot).sin()
                                                            * SOUL_WHEEL_RADIUS
                                                            + SOUL_WHEEL_CONTAINER_SIZE / 2.
                                                            - SOUL_WHEEL_SLOT_SPRITE_SIZE
                                                            + 1.,
                                                    ),
                                                    position_type: PositionType::Absolute,
                                                    width: Val::Px(SOUL_WHEEL_SLOT_SPRITE_SIZE),
                                                    height: Val::Px(SOUL_WHEEL_SLOT_SPRITE_SIZE),
                                                    ..default()
                                                },
                                            ))
                                            .observe(mouse_use_wheel_soul);
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
                                                    SOUL_WHEEL_RADIUS / 1.9
                                                        * ((i + 6) as f32 * rot).cos()
                                                        + SOUL_WHEEL_CONTAINER_SIZE / 2.
                                                        - SOUL_WHEEL_SLOT_SPRITE_SIZE / 1.7
                                                        + 1.,
                                                ),
                                                top: Val::Px(
                                                    SOUL_WHEEL_RADIUS / 1.9
                                                        * ((i + 6) as f32 * rot).sin()
                                                        + SOUL_WHEEL_CONTAINER_SIZE / 2.
                                                        - SOUL_WHEEL_SLOT_SPRITE_SIZE / 1.7
                                                        + 0.7,
                                                ),
                                                position_type: PositionType::Absolute,
                                                ..default()
                                            },
                                        ));
                                    }
                                });
                        });
                    parent.spawn((
                        ChainBox,
                        Node {
                            width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                            height: Val::Px(25.),
                            min_height: Val::Px(25.),
                            max_height: Val::Px(25.),
                            border: UiRect::new(Val::Px(0.), Val::Px(2.), Val::Px(2.), Val::Px(0.)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0., 0., 0.)),
                        children![
                            (
                                MessageLog,
                                Node {
                                    width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE - 3.),
                                    height: Val::Px(23.),
                                    left: Val::Px(0.5),
                                    min_height: Val::Px(23.),
                                    max_height: Val::Px(23.),
                                    overflow: Overflow::clip(),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                Visibility::Inherited,
                            ),
                            (
                                CursorBox,
                                Node {
                                    width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE - 3.),
                                    height: Val::Px(23.),
                                    left: Val::Px(0.5),
                                    min_height: Val::Px(23.),
                                    max_height: Val::Px(23.),
                                    overflow: Overflow::clip(),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                Visibility::Hidden,
                            ),
                            (
                                AxiomBox,
                                Node {
                                    width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE - 3.),
                                    height: Val::Px(23.),
                                    left: Val::Px(0.5),
                                    min_height: Val::Px(23.),
                                    max_height: Val::Px(23.),
                                    overflow: Overflow::clip(),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                Visibility::Hidden,
                            ),
                            (
                                CasteBox,
                                Node {
                                    width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE - 3.),
                                    height: Val::Px(23.),
                                    left: Val::Px(0.5),
                                    min_height: Val::Px(23.),
                                    max_height: Val::Px(23.),
                                    overflow: Overflow::clip(),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                Visibility::Hidden,
                            ),
                            (
                                QuestBox,
                                Node {
                                    width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE - 3.),
                                    height: Val::Px(23.),
                                    left: Val::Px(0.5),
                                    min_height: Val::Px(23.),
                                    max_height: Val::Px(23.),
                                    overflow: Overflow::clip(),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                Visibility::Hidden,
                            )
                        ],
                    ));
                    parent
                        .spawn((
                            ChainBox,
                            Node {
                                width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                height: Val::Px(7.),
                                min_height: Val::Px(7.),
                                max_height: Val::Px(7.),
                                border: UiRect::new(
                                    Val::Px(0.),
                                    Val::Px(2.),
                                    Val::Px(2.),
                                    Val::Px(0.),
                                ),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0., 0., 0.)),
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn((
                                    ImageNode {
                                        image: asset_server.load("spritesheet.png"),
                                        texture_atlas: Some(TextureAtlas {
                                            layout: atlas_layout.handle.clone(),
                                            index: 169,
                                        }),
                                        ..Default::default()
                                    },
                                    Node {
                                        width: Val::Px(SOUL_WHEEL_SLOT_SPRITE_SIZE),
                                        height: Val::Px(SOUL_WHEEL_SLOT_SPRITE_SIZE),
                                        left: Val::Px(0.5),
                                        top: Val::Px(0.5),
                                        ..default()
                                    },
                                ))
                                .with_child((
                                    Text::new("C"),
                                    TextLayout {
                                        justify: JustifyText::Center,
                                        linebreak: LineBreak::NoWrap,
                                    },
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 1.5,
                                        ..default()
                                    },
                                    Label,
                                    Node {
                                        left: Val::Px(4.1),
                                        top: Val::Px(1.),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                ));
                            parent
                                .spawn((
                                    ImageNode {
                                        image: asset_server.load("spritesheet.png"),
                                        texture_atlas: Some(TextureAtlas {
                                            layout: atlas_layout.handle.clone(),
                                            index: 166,
                                        }),
                                        ..Default::default()
                                    },
                                    Node {
                                        width: Val::Px(SOUL_WHEEL_SLOT_SPRITE_SIZE),
                                        height: Val::Px(SOUL_WHEEL_SLOT_SPRITE_SIZE),
                                        left: Val::Px(6.5),
                                        top: Val::Px(0.5),
                                        ..default()
                                    },
                                ))
                                .with_child((
                                    Text::new("E"),
                                    TextLayout {
                                        justify: JustifyText::Center,
                                        linebreak: LineBreak::NoWrap,
                                    },
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 1.5,
                                        ..default()
                                    },
                                    Label,
                                    Node {
                                        left: Val::Px(4.1),
                                        top: Val::Px(1.),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                ));

                            parent
                                .spawn((
                                    ImageNode {
                                        image: asset_server.load("spritesheet.png"),
                                        texture_atlas: Some(TextureAtlas {
                                            layout: atlas_layout.handle.clone(),
                                            index: 227,
                                        }),
                                        ..Default::default()
                                    },
                                    Node {
                                        width: Val::Px(SOUL_WHEEL_SLOT_SPRITE_SIZE),
                                        height: Val::Px(SOUL_WHEEL_SLOT_SPRITE_SIZE),
                                        left: Val::Px(12.5),
                                        top: Val::Px(0.5),
                                        ..default()
                                    },
                                ))
                                .with_child((
                                    Text::new("Q"),
                                    TextLayout {
                                        justify: JustifyText::Center,
                                        linebreak: LineBreak::NoWrap,
                                    },
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 1.5,
                                        ..default()
                                    },
                                    Label,
                                    Node {
                                        left: Val::Px(4.1),
                                        top: Val::Px(1.),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                ));
                        });
                    parent
                        .spawn((
                            ChainBox,
                            LargeQuestPanel { selected: None },
                            QuestBox,
                            Visibility::Hidden,
                            Node {
                                width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE + 32.),
                                height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE + 32.),
                                min_width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE + 32.),
                                max_width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE + 32.),
                                min_height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE + 32.),
                                max_height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE + 32.),
                                border: UiRect::new(
                                    Val::Px(0.),
                                    Val::Px(2.),
                                    Val::Px(2.),
                                    Val::Px(0.),
                                ),
                                right: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0., 0., 0.)),
                        ))
                        .with_children(|parent| {
                            for _i in 0..3 {
                                parent.spawn((
                                    ImageNode {
                                        image: asset_server.load("spritesheet.png"),
                                        texture_atlas: Some(TextureAtlas {
                                            layout: atlas_layout.handle.clone(),
                                            index: 0,
                                        }),
                                        ..Default::default()
                                    },
                                    Node {
                                        width: Val::Px(4.),
                                        height: Val::Px(4.),
                                        left: Val::Px(1.),
                                        top: Val::Px(1.),
                                        ..default()
                                    },
                                ));
                            }
                        });

                    parent
                        .spawn((
                            ChainBox,
                            LargeCastePanel {
                                selected_column: CastePanelColumn::Left,
                                selected_row: CastePanelRow::Top,
                            },
                            CasteBox,
                            Visibility::Hidden,
                            Node {
                                width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE + 32.),
                                height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE + 32.),
                                min_width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE + 32.),
                                max_width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE + 32.),
                                min_height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE + 32.),
                                max_height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE + 32.),
                                border: UiRect::new(
                                    Val::Px(0.),
                                    Val::Px(2.),
                                    Val::Px(2.),
                                    Val::Px(0.),
                                ),
                                right: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                                position_type: PositionType::Absolute,
                                align_items: AlignItems::Center,
                                justify_items: JustifyItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0., 0., 0.)),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                ImageNode {
                                    image: asset_server.load("spritesheet.png"),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: atlas_layout.handle.clone(),
                                        index: 166,
                                    }),
                                    ..Default::default()
                                },
                                Node {
                                    width: Val::Px(64.),
                                    height: Val::Px(64.),
                                    right: Val::Px(0.5),
                                    ..default()
                                },
                            ));

                            // Cursor
                            parent.spawn((
                                ImageNode {
                                    image: asset_server.load("spritesheet.png"),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: atlas_layout.handle.clone(),
                                        index: 18,
                                    }),
                                    ..Default::default()
                                },
                                CasteCursor,
                                Node {
                                    width: Val::Px(11.),
                                    height: Val::Px(11.),
                                    left: Val::Px(14.),
                                    top: Val::Px(6.),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                            ));
                            let castes = [Soul::Saintly, Soul::Artistic, Soul::Feral];
                            for i in 0..3 {
                                parent
                                    .spawn((
                                        EquipSlot(castes[i]),
                                        ImageNode {
                                            image: asset_server.load("spritesheet.png"),
                                            texture_atlas: Some(TextureAtlas {
                                                layout: atlas_layout.handle.clone(),
                                                index: 160 + i * 2,
                                            }),
                                            ..Default::default()
                                        },
                                        Node {
                                            width: Val::Px(7.),
                                            height: Val::Px(7.),
                                            left: Val::Px(if i == 1 { 8. } else { 16. }),
                                            top: Val::Px(i as f32 * 20. + 8.),
                                            position_type: PositionType::Absolute,
                                            ..default()
                                        },
                                    ))
                                    .observe(on_hover_move_caste_cursor)
                                    .observe(on_click_equip_unequip);
                            }
                            let castes = [Soul::Ordered, Soul::Unhinged, Soul::Vile];
                            for i in 0..3 {
                                parent
                                    .spawn((
                                        EquipSlot(castes[i]),
                                        ImageNode {
                                            image: asset_server.load("spritesheet.png"),
                                            texture_atlas: Some(TextureAtlas {
                                                layout: atlas_layout.handle.clone(),
                                                index: 161 + i * 2,
                                            }),
                                            ..Default::default()
                                        },
                                        Node {
                                            width: Val::Px(7.),
                                            height: Val::Px(7.),
                                            right: Val::Px(if i == 1 { 8. } else { 16. }),
                                            top: Val::Px(i as f32 * 20. + 8.),
                                            position_type: PositionType::Absolute,
                                            ..default()
                                        },
                                    ))
                                    .observe(on_hover_move_caste_cursor)
                                    .observe(on_click_equip_unequip);
                            }
                            parent.spawn((
                                Node {
                                    width: Val::Px(7.),
                                    height: Val::Px(31.),
                                    left: Val::Px(28.),
                                    top: Val::Px(16.),
                                    row_gap: Val::Px(1.),
                                    column_gap: Val::Px(1.),
                                    position_type: PositionType::Absolute,
                                    flex_wrap: FlexWrap::Wrap,
                                    align_content: AlignContent::FlexStart,
                                    ..default()
                                },
                                SpellLibraryUI,
                            ));
                        });
                });
        });
    commands.run_system_cached(decorate_with_chains);
}

#[derive(Component)]
struct ChainBox;

#[derive(Component)]
struct ChainUI;

#[derive(Component)]
pub struct SpellLibraryUI;

fn decorate_with_chains(
    query: Query<(Entity, &Node), With<ChainBox>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    for (chain_box, node) in query.iter() {
        let (width, height) = if let Val::Px(width) = node.width {
            if let Val::Px(height) = node.height {
                (width as usize, height as usize)
            } else {
                panic!();
            }
        } else {
            panic!();
        };
        let number_of_chains_left_right = height / CHAIN_SIZE as usize;
        let number_of_chains_top_bottom = width / CHAIN_SIZE as usize;
        commands.entity(chain_box).with_children(|parent| {
            for i in 0..number_of_chains_top_bottom {
                // top chains
                parent.spawn((
                    ChainUI,
                    ImageNode {
                        image: asset_server.load("spritesheet.png"),
                        texture_atlas: Some(TextureAtlas {
                            layout: atlas_layout.handle.clone(),
                            index: if i == 0 || i == number_of_chains_top_bottom - 1 {
                                140
                            } else {
                                139
                            },
                        }),
                        ..Default::default()
                    },
                    Node {
                        left: Val::Px(-0.5 + i as f32 * CHAIN_SIZE),
                        top: Val::Px(-0.5),
                        width: Val::Px(CHAIN_SIZE),
                        height: Val::Px(CHAIN_SIZE),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    Transform::from_rotation(if i != 0 {
                        Quat::from_rotation_z(PI / 2.)
                    } else {
                        Quat::from_rotation_z(0.)
                    }),
                ));

                // bottom chains
                if i != number_of_chains_top_bottom - 1 && i != 0 {
                    parent.spawn((
                        ChainUI,
                        ImageNode {
                            image: asset_server.load("spritesheet.png"),
                            texture_atlas: Some(TextureAtlas {
                                layout: atlas_layout.handle.clone(),
                                index: 139,
                            }),
                            ..Default::default()
                        },
                        Node {
                            left: Val::Px(-0.5 + i as f32 * CHAIN_SIZE),
                            bottom: Val::Px(-0.5),
                            width: Val::Px(CHAIN_SIZE),
                            height: Val::Px(CHAIN_SIZE),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        Transform::from_rotation(Quat::from_rotation_z(3. * PI / 2.)),
                    ));
                }
            }

            for i in 0..number_of_chains_left_right {
                // right chains
                if i != number_of_chains_left_right - 1 {
                    parent.spawn((
                        ImageNode {
                            image: asset_server.load("spritesheet.png"),
                            texture_atlas: Some(TextureAtlas {
                                layout: atlas_layout.handle.clone(),
                                index: if i == 0 || i == number_of_chains_left_right - 1 {
                                    140
                                } else {
                                    139
                                },
                            }),
                            ..Default::default()
                        },
                        Node {
                            bottom: Val::Px(-0.5 + i as f32 * CHAIN_SIZE),
                            right: Val::Px(-0.5),
                            width: Val::Px(CHAIN_SIZE),
                            height: Val::Px(CHAIN_SIZE),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        Transform::from_rotation(if i == number_of_chains_left_right - 1 {
                            Quat::from_rotation_z(3. * PI / 2.)
                        } else {
                            Quat::from_rotation_z(PI)
                        }),
                    ));
                }
                // left chains
                if i != number_of_chains_left_right - 1 {
                    parent.spawn((
                        ImageNode {
                            image: asset_server.load("spritesheet.png"),
                            texture_atlas: Some(TextureAtlas {
                                layout: atlas_layout.handle.clone(),
                                index: if i == 0 || i == number_of_chains_left_right - 1 {
                                    140
                                } else {
                                    139
                                },
                            }),
                            ..Default::default()
                        },
                        Node {
                            bottom: Val::Px(-0.5 + i as f32 * CHAIN_SIZE),
                            left: Val::Px(-0.5),
                            width: Val::Px(CHAIN_SIZE),
                            height: Val::Px(CHAIN_SIZE),
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
    }
}

#[derive(Component)]
/// To list game events in the text box
pub struct MessageLog;

#[derive(Component)]
/// To describe what Creatures do in the text box
pub struct CursorBox;

#[derive(Component)]
/// To describe what Spells do in the text box
pub struct CasteBox;

#[derive(Component)]
/// To describe what Axioms do in the text box
pub struct AxiomBox;

#[derive(Component)]
/// To preview crafting recipes in the soul wheel box
pub struct PatternBox;

#[derive(Component)]
/// The wheel where the player casts spells
pub struct SoulWheelBox;

#[derive(Component)]
/// A preview of the spell that will result from a soul cage
pub struct CraftingPredictor;

#[derive(Component)]
/// A list of possible axioms available for crafting
pub struct CraftingPatterns;

#[derive(Component)]
pub struct RecipebookUI;

#[derive(Component)]
/// The blue cursor in the caste equip/unequip menu
pub struct CasteCursor;

#[derive(Component)]
pub struct EquipSlot(pub Soul);

#[derive(Component)]
pub struct LibrarySlot(pub Uuid);

#[derive(Component)]
pub struct LargeCastePanel {
    pub selected_column: CastePanelColumn,
    pub selected_row: CastePanelRow,
}

#[derive(Component)]
pub struct LargeQuestPanel {
    pub selected: Option<usize>,
}

#[derive(Component)]
/// To describe what Quests do in the text box
pub struct QuestBox;

#[derive(PartialEq, Clone, Copy)]
pub enum CastePanelColumn {
    Left,
    LibraryLeft,
    LibraryRight,
    Right,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CastePanelRow {
    Top,
    Middle,
    Bottom,
    Library(usize),
}

const ROWS: [CastePanelRow; 3] = [
    CastePanelRow::Top,
    CastePanelRow::Middle,
    CastePanelRow::Bottom,
];

impl CastePanelRow {
    pub fn shift(&mut self, dir: i8, column: &CastePanelColumn) {
        if column == &CastePanelColumn::Left || column == &CastePanelColumn::Right {
            let index = ROWS.iter().position(|c| c == self).unwrap() as i8 + dir;
            let index = match index {
                -1 => 2,
                3 => 0,
                _ => index,
            } as usize;
            *self = ROWS[index];
        } else if let CastePanelRow::Library(index) = self {
            let index = *index as i8 + dir;
            let index = match index {
                -1 => 7,
                8 => 0,
                _ => index,
            } as usize;
            *self = CastePanelRow::Library(index);
        }
    }
}

const COLUMNS: [CastePanelColumn; 4] = [
    CastePanelColumn::Left,
    CastePanelColumn::LibraryLeft,
    CastePanelColumn::LibraryRight,
    CastePanelColumn::Right,
];

impl CastePanelColumn {
    pub fn shift(&mut self, dir: i8) {
        let index = COLUMNS.iter().position(|c| c == self).unwrap() as i8 + dir;
        let index = match index {
            -1 => 3,
            4 => 0,
            _ => index,
        } as usize;
        *self = COLUMNS[index];
    }
}

#[derive(Component)]
pub struct LogEntry;

#[derive(Event)]
pub struct AddMessage {
    pub message: Message,
}

#[derive(Event)]
pub struct SlideMessages;

#[derive(Component)]
pub struct LogSlide {
    timer: Timer,
    curve: EasingCurve<f32>,
    target: f32,
}

pub enum InvalidAction {
    WheelFull,
    NoSoulsInPile,
    CannotMelee(Species),
    EmptySlotCast,
    NoSpellForCaste,
}

pub enum Message {
    Tutorial,
    HostileAttack(Species, isize),
    PlayerAttack(Species, isize),
    NoPlayerAttack(Species, Species, isize),
    PlayerIsInvincible(Species),
    HealSelf(isize),
    HealOther(Species, isize),
    CreatureHealsItself(Species, isize),
    InvalidAction(InvalidAction),
}

pub fn print_message_in_log(
    mut events: EventReader<AddMessage>,
    mut slide: EventWriter<SlideMessages>,
    log: Query<Entity, With<MessageLog>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    for (i, event) in events.read().enumerate() {
        let new_string = match &event.message {
            Message::Tutorial => LORE[18],
            Message::HostileAttack(species, damage) => &format!(
                "The {} hits you for [r]{}[w] damage.",
                match_species_with_string(species),
                damage
            ),
            Message::PlayerIsInvincible(species) => &format!(
                "The {} fails to hit you.",
                match_species_with_string(species)
            ),
            Message::PlayerAttack(species, damage) => &format!(
                "You hit the {} for [r]{}[w] damage.",
                match_species_with_string(species),
                damage
            ),
            Message::HealSelf(damage) => {
                &format!("You heal yourself for [l]{}[w] health points.", damage)
            }
            Message::HealOther(species, damage) => &format!(
                "You heal the {} for [l]{}[w] health points.",
                match_species_with_string(species),
                damage
            ),
            Message::CreatureHealsItself(species, damage) => &format!(
                "The {} heals itself for [l]{}[w] health points.",
                match_species_with_string(species),
                damage
            ),
            Message::NoPlayerAttack(culprit_species, victim_species, damage) => &format!(
                "The {} hits the {} for [r]{}[w] damage.",
                match_species_with_string(culprit_species),
                match_species_with_string(victim_species),
                damage
            ),
            Message::InvalidAction(action) => match action {
                InvalidAction::WheelFull => {
                    "[y]Your Soul Wheel is already full, cast some with 1-8 before drawing more![w]"
                }
                InvalidAction::NoSoulsInPile => {
                    "[y]You have no Souls left in your pile, and must slay more creatures before drawing more![w]"
                }
                InvalidAction::CannotMelee(species) => {
                    &format!(
                    "[y]You cannot hope to breach the {}[y]'s defenses![w]",
                    match_species_with_string(species)
                    )
                }
                InvalidAction::EmptySlotCast => {
                    "[y]That slot has nothing in it, you cannot cast it as a spell![w]"
                }
                InvalidAction::NoSpellForCaste => {
                    "[y]Souls of this caste have no spell assigned, you cannot use them![w]"
                }
            },
        };
        let mut new_text = Entity::PLACEHOLDER;
        commands.entity(log.single()?).with_children(|parent| {
            new_text = spawn_split_text(new_string, parent, &asset_server);
        });
        // Necessary to prevent a "flash" of the text before it is moved by
        // slide_message_log.
        commands.entity(new_text).insert(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(-200.),
            ..default()
        });

        // This should only happen once.
        if i == 0 {
            slide.write(SlideMessages);
        }
    }
    Ok(())
}

pub fn dispense_sliding_components_log(
    mut events: EventReader<SlideMessages>,
    mut commands: Commands,
    new_log: Query<(Entity, &TextLayoutInfo), Added<LogEntry>>,
    old_log: Query<(Entity, &LogSlide)>,
) {
    for _event in events.read() {
        let mut total_slide_distance = 0.;
        for (entity, layout) in new_log.iter() {
            commands.entity(entity).insert(LogSlide {
                timer: Timer::new(Duration::from_millis(300), TimerMode::Once),
                curve: EasingCurve::new(
                    -layout.size.y,
                    0.5 + total_slide_distance,
                    EaseFunction::QuadraticInOut,
                ),
                target: 0.5 + total_slide_distance,
            });
            total_slide_distance += layout.size.y;
        }
        for (entity, slide) in old_log.iter() {
            commands.entity(entity).insert(LogSlide {
                timer: Timer::new(Duration::from_millis(300), TimerMode::Once),
                curve: EasingCurve::new(
                    slide.target,
                    slide.target + total_slide_distance,
                    EaseFunction::QuadraticInOut,
                ),
                target: slide.target + total_slide_distance,
            });
            // Despawn text that has scrolled out of view.
            if slide.target > 50. {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn slide_message_log(mut messages: Query<(&mut Node, &mut LogSlide)>, time: Res<Time>) {
    for (mut message, mut log) in messages.iter_mut() {
        {
            log.timer.tick(time.delta());
            let new_height = log.curve.sample_clamped(log.timer.fraction());
            message.bottom = Val::Px(new_height);
        }
    }
}

pub fn match_species_with_string(species: &Species) -> String {
    let string = match species {
        Species::Hunter => "[l]Scion of the Old World[w]",
        Species::Apiarist => "[m]Brass Apiarist[w]",
        Species::Tinker => "[d]Frenzied Dreamtinker[w]",
        Species::Oracle => "[r]Anisychic Oracle[w]",
        Species::Shrike => "[y]Jade Shrike[w]",
        Species::Second => "[b]Emblem of Sin[w]",
        Species::Trap => "[c]Psychic Prism[w]",
        Species::Abazon => "[s]Terracotta Sentry[w]",
        Species::Wall => "[a]Rampart of Nacre[w]",
        Species::WeakWall => "[a]Rampart of Nacre[w]",
        Species::Airlock => "[a]Quicksilver Curtains[w]",
        Species::Player => "[p]Reality Anchor[w]",
        Species::EpsilonTail => "[y]Rubberized Mecha-Segment[w]",
        Species::EpsilonHead => "[y]Epsilon, Crowned by Truth[w]",
        Species::AxiomaticSeal => "[a]Axiomatic Seal[w]",
        _ => &format!("{:?}", species),
    };
    string.to_owned()
}

pub fn spawn_split_text(
    new_string: &str,
    parent: &mut ChildSpawnerCommands,
    asset_server: &Res<AssetServer>,
) -> Entity {
    let split_string = split_text(new_string);
    parent
        .spawn((
            LogEntry,
            Text::new(&split_string[0].0),
            TextLayout {
                justify: JustifyText::Left,
                linebreak: LineBreak::WordBoundary,
            },
            TextFont {
                font: asset_server.load("fonts/Play-Regular.ttf"),
                font_size: 1.5,
                ..default()
            },
            TextColor(*split_string[0].1),
            Label,
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .with_children(|parent| {
            for (section, color) in split_string.iter().skip(1) {
                if section.is_empty() {
                    continue;
                }
                let (section, font_size) = if section.chars().next().unwrap() == '@' {
                    (&section[1..], 1.0)
                } else {
                    (&section[0..], 1.5)
                };
                parent.spawn((
                    LogEntry,
                    TextSpan::new(section),
                    TextFont {
                        font: asset_server.load("fonts/Play-Regular.ttf"),
                        font_size,
                        ..default()
                    },
                    *color,
                ));
            }
        })
        .id()
}
