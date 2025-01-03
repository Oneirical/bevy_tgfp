use std::f32::consts::PI;

use bevy::prelude::*;

use crate::graphics::SpriteSheetAtlas;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
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
                        width: Val::Px(40.),
                        height: Val::Px(40.),
                        display: Display::Flex,
                        justify_content: JustifyContent::FlexEnd,
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0., 0., 0.)),
                ))
                .with_children(|parent| {
                    let rot = PI / 4.;
                    for i in 0..8 {
                        let i = i as f32;
                        parent.spawn((
                            ImageNode {
                                image: asset_server.load("spritesheet.png"),
                                texture_atlas: Some(TextureAtlas {
                                    layout: atlas_layout.handle.clone(),
                                    index: 107,
                                }),
                                ..Default::default()
                            },
                            Node {
                                left: Val::Px((i * rot).cos()),
                                top: Val::Px((i * rot).sin()),
                                width: Val::Px(5.),
                                height: Val::Px(5.),
                                ..default()
                            },
                        ));
                    }
                });
        });
}
