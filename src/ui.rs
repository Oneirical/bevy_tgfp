use std::f32::consts::PI;

use bevy::{prelude::*, window::Monitor};

use crate::{
    creature::{get_species_sprite, Species},
    graphics::SpriteSheetAtlas,
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_event::<AnnounceGameOver>();
    }
}

const SOUL_WHEEL_CONTAINER_SIZE: f32 = 33.;
const SOUL_WHEEL_RADIUS: f32 = 8.;
const SOUL_WHEEL_SLOT_SPRITE_SIZE: f32 = 4.;
const CHAIN_SIZE: f32 = 4.;
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
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .insert(PickingBehavior::IGNORE)
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
    mut scale: ResMut<UiScale>,
    window: Query<&Monitor>,
) {
    let win_height = window.iter().next().unwrap().physical_height;
    scale.0 = 0.75;
    // root node
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        })
        .insert(PickingBehavior::IGNORE)
        .with_children(|parent|{parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            ..default()
        }).with_children(|parent| {
            // left vertical fill (border)
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
                        border: UiRect::new(Val::Px(0.), Val::Px(2.), Val::Px(2.), Val::Px(0.)),
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
                                        - SOUL_WHEEL_SLOT_SPRITE_SIZE + 1.,
                                ),
                                top: Val::Px(
                                    ((i + 6) as f32 * rot).sin() * SOUL_WHEEL_RADIUS
                                        + SOUL_WHEEL_CONTAINER_SIZE / 2.
                                        - SOUL_WHEEL_SLOT_SPRITE_SIZE + 1.,
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
                                        - SOUL_WHEEL_SLOT_SPRITE_SIZE / 1.7 + 1.,
                                ),
                                top: Val::Px(
                                    SOUL_WHEEL_RADIUS / 1.9 * ((i + 6) as f32 * rot).sin()
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
            dbg!(win_height);
            parent.spawn((
                    ChainBox,
                    Node {
                        width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                        height: text_box_height(win_height),
                        min_height: text_box_height(win_height),
                        max_height: text_box_height(win_height),
                        border: UiRect::new(Val::Px(0.), Val::Px(2.), Val::Px(2.), Val::Px(0.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0., 0., 0.)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Stay alive, and slay every creature in the tower to win!\n\n\
                            Bump into creatures to attack them in melee. Slain creatures drop their "),
                        TextLayout {
                            justify: JustifyText::Left,
                            linebreak: LineBreak::WordBoundary,
                        },
                        TextFont {
                            font: asset_server.load("fonts/Play-Regular.ttf"),
                            font_size: 0.9,
                            ..default()
                        },
                        Label,
                        Node {
                            left: Val::Px(0.5),
                            top: Val::Px(1.5),
                            width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE / 1.2),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                    )).with_children(|parent| {
                            parent.spawn((
                                    TextSpan::new("Soul"),
                                    TextColor(Color::srgb(0.31, 0.99, 0.25)),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new(".\n\n\
                            Draw these "),
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new("Souls"),
                                    TextColor(Color::srgb(0.31, 0.99, 0.25)),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new(" on the "),
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new("Soul Wheel"),
                                    TextColor(Color::srgb(0.31, 0.99, 0.25)),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new(", and cast them for one of 6 special spells.\n\n\
                            The effects of all 6 spells, as well as the peculiarities of each creature type, \
                            are written on the left sidebar.\n\n\nControls:\n\n"),
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new("Arrow Keys"),
                                    TextColor(Color::srgb(0.31, 0.99, 0.25)),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new(" or "),
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new("WASD"),
                                    TextColor(Color::srgb(0.31, 0.99, 0.25)),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new(": Move or melee attack one step in the cardinal directions.\n"),
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new("Space"),
                                    TextColor(Color::srgb(0.31, 0.99, 0.25)),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new(" or "),
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new("Q"),
                                    TextColor(Color::srgb(0.31, 0.99, 0.25)),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new(": Draw one Soul on the Soul Wheel.\n"),
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new("1-8"),
                                    TextColor(Color::srgb(0.31, 0.99, 0.25)),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new(": Cast a spell corresponding to the chosen slot on the Soul Wheel.\n"),
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new("Z"),
                                    TextColor(Color::srgb(0.31, 0.99, 0.25)),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new(" or "),
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new("X"),
                                    TextColor(Color::srgb(0.31, 0.99, 0.25)),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            parent.spawn((
                                    TextSpan::new(": Reset the game.\n"),
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                        });

                });
            parent.spawn((
                    ChainBox,
                    Node {
                        width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                        height: Val::Px(7.),
                        min_height: Val::Px(7.),
                        max_height: Val::Px(7.),
                        border: UiRect::new(Val::Px(0.), Val::Px(2.), Val::Px(2.), Val::Px(0.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0., 0., 0.)),
                ));
        }); });

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexStart,
            ..default()
        })
        .insert(PickingBehavior::IGNORE)
        .with_children(|parent| {
            parent
                .spawn((
                    ChainBox,
                    Node {
                        width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE),
                        height: Val::Px(SOUL_WHEEL_CONTAINER_SIZE * 2. + 1.),
                        display: Display::Flex,
                        justify_content: JustifyContent::SpaceAround,
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0., 0., 0.)),
                ))
                .with_children(|parent| {
                    for i in 0..12 {
                        parent
                            .spawn((
                                ImageNode {
                                    image: asset_server.load("spritesheet.png"),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: atlas_layout.handle.clone(),
                                        index: match i {
                                            6 => get_species_sprite(&Species::Hunter),
                                            7 => get_species_sprite(&Species::Apiarist),
                                            8 => get_species_sprite(&Species::Tinker),
                                            9 => get_species_sprite(&Species::Oracle),
                                            10 => get_species_sprite(&Species::Shrike),
                                            11 => get_species_sprite(&Species::Second),
                                            _ => 160 + i,
                                        },
                                    }),
                                    ..Default::default()
                                },
                                Node {
                                    width: Val::Px(SOUL_WHEEL_SLOT_SPRITE_SIZE),
                                    height: Val::Px(SOUL_WHEEL_SLOT_SPRITE_SIZE),
                                    left: Val::Px(1.),
                                    ..default()
                                },
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(match i {
                                        0 => "Saintly",
                                        1 => "Ordered",
                                        2 => "Artistic",
                                        3 => "Unhinged",
                                        4 => "Feral",
                                        5 => "Vile",
                                        6 => "Scion (Saintly Soul)",
                                        7 => "Apiarist (Ordered Soul)",
                                        8 => "Tinker (Artistic Soul)",
                                        9 => "Oracle (Unhinged Soul)",
                                        10 => "Shrike (Feral Soul)",
                                        _ => "Emblem (Vile Soul)",
                                    }),
                                    TextLayout {
                                        justify: JustifyText::Left,
                                        linebreak: LineBreak::WordBoundary,
                                    },
                                    TextColor( match i {
                                        0 | 6 => Color::srgb(0.31, 0.99, 0.25),
                                        1 | 7 => Color::srgb(0.97, 0.28, 0.25),
                                        2 | 8 => Color::srgb(0.94, 0.55, 0.38),
                                        3 | 9 => Color::srgb(0.97, 0.99, 0.),
                                        4 | 10 => Color::srgb(0.66, 0.82, 0.11),
                                        _ => Color::srgb(0.87, 0.67, 0.89),
                                    }),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                    Label,
                                    Node {
                                        left: Val::Px(5.5),
                                        width: Val::Px(SOUL_WHEEL_CONTAINER_SIZE / 1.5),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    )).with_child((
                                    TextSpan::new(match i {
                                        0 => " - You, and all adjacent creatures, heal for 2 HP.",
                                        1 => " - You cannot take damage next turn. Instantaneous.",
                                        2 => " - Places a trap at your feet. The next creature to step on it will cause it to fire 2 damage beams in all 4 cardinal directions.",
                                        3 => " - Fires 4 beams in all diagonal directions, dealing 2 damage.",
                                        4 => " - Dashes 5 tiles in the direction you are facing, attacking all creatures adjacent to your path with 1 damage. Creatures struck at the end are knocked backwards.",
                                        5 => " - The next time you strike with a melee attack, deal 6 damage.",
                                        6 => " - Its melee attacks cause it to heal itself for 1 HP.",
                                        7 => " - Resilient, yet slow, acting once every two turns.",
                                        8 => " - It moves erratically, and sculpts sentries from walls. These crumble into dust once their creator is slain.",
                                        9 => " - It charges up as it moves, empowering its next melee attack with 1 bonus damage every 5 steps.",
                                        10 => " - Frail, but fast, acting twice every turn.",
                                        _ => " - It hungers, devouring nearby walls to regenerate.",
                                    }),
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font: asset_server.load("fonts/Play-Regular.ttf"),
                                        font_size: 0.9,
                                        ..default()
                                    },
                                ));
                            });
                    }
                });
        });
    commands.run_system_cached(decorate_with_chains);
}

#[derive(Component)]
struct ChainBox;

#[derive(Component)]
struct ChainUI;

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

fn text_box_height(window_size: u32) -> Val {
    let mut height = window_size * 28 / 1080;
    if height % 2 != 0 {
        height -= 1;
    }
    Val::Px(height as f32)
}
