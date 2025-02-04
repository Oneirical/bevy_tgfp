use bevy::prelude::*;

use crate::{
    creature::{get_soul_sprite, Player, Soul, SpellLibrary, Spellbook},
    graphics::SpriteSheetAtlas,
    text::match_soul_with_description,
    ui::{
        spawn_split_text, CasteBox, CasteCursor, CastePanelColumn, CastePanelRow, EquipSlot,
        LargeCastePanel, LibrarySlot, MessageLog, SpellLibraryUI,
    },
};

pub fn show_caste_menu(
    mut message: Query<&mut Visibility, (With<MessageLog>, Without<CasteBox>)>,
    mut caste_box: Query<&mut Visibility, (With<CasteBox>, Without<MessageLog>)>,
) {
    *message.single_mut() = Visibility::Hidden;
    for mut vis in caste_box.iter_mut() {
        *vis = Visibility::Inherited;
    }
}

pub fn hide_caste_menu(
    mut message: Query<&mut Visibility, (With<MessageLog>, Without<CasteBox>)>,
    mut caste_box: Query<&mut Visibility, (With<CasteBox>, Without<MessageLog>)>,
) {
    *message.single_mut() = Visibility::Inherited;
    for mut vis in caste_box.iter_mut() {
        *vis = Visibility::Hidden;
    }
}

#[derive(Event)]
pub struct EquipSpell {
    pub index: usize,
}

#[derive(Event)]
pub struct UnequipSpell {
    pub caste: Soul,
}

pub fn equip_spell(
    mut events: EventReader<EquipSpell>,
    mut unequips: EventReader<UnequipSpell>,
    mut spell_library: ResMut<SpellLibrary>,
    mut spellbook: Query<&mut Spellbook, With<Player>>,
    mut slots: Query<(&mut ImageNode, &EquipSlot), Without<LibrarySlot>>,
    mut ui_library: Query<(Entity, &mut ImageNode, &mut LibrarySlot), Without<EquipSlot>>,
    mut commands: Commands,
    ui: Query<Entity, With<SpellLibraryUI>>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    // NOTE: Instead of this entire charade with matching Uuids, it might
    // have been better to let the spell library sprite BE the spell library
    // - no resource, just entities with a sprite and a spell component.
    for event in events.read() {
        // Do not equip empty slots in the library.
        if spell_library.library.get(event.index).is_none() {
            continue;
        }
        let equipped_spell = spell_library.library.remove(event.index);
        let mut spellbook = spellbook.single_mut();
        // If a spell was in the equipped slot before, remove it and add it
        // back to the library.
        if let Some(old_spell) = spellbook.spells.remove(&equipped_spell.caste) {
            for (_entity, mut node, mut lib_slot) in ui_library.iter_mut() {
                if lib_slot.0 == equipped_spell.id {
                    node.texture_atlas.as_mut().unwrap().index = old_spell.icon;
                    lib_slot.0 = old_spell.id;
                    break;
                }
            }
            spell_library.library.insert(event.index, old_spell);
        // If there was no spell in the equipped slot, despawn the library
        // icon (which will go into the equipment slot).
        } else {
            for (entity, _node, lib_slot) in ui_library.iter() {
                if lib_slot.0 == equipped_spell.id {
                    commands.entity(entity).despawn();
                    break;
                }
            }
        }
        // Add the new spell on its equipment slot.
        for (mut node, slot) in slots.iter_mut() {
            if slot.0 == equipped_spell.caste {
                node.texture_atlas.as_mut().unwrap().index = equipped_spell.icon;
                node.color.set_alpha(1.);
                break;
            }
        }
        spellbook
            .spells
            .insert(equipped_spell.caste, equipped_spell);
    }
    for unequip in unequips.read() {
        let mut spellbook = spellbook.single_mut();
        if let Some(old_spell) = spellbook.spells.remove(&unequip.caste) {
            // Add the unequipped spell back into the library.
            commands.entity(ui.single()).with_children(|parent| {
                parent
                    .spawn((
                        LibrarySlot(old_spell.id),
                        ImageNode {
                            image: asset_server.load("spritesheet.png"),
                            texture_atlas: Some(TextureAtlas {
                                layout: atlas_layout.handle.clone(),
                                index: old_spell.icon,
                            }),
                            ..Default::default()
                        },
                        Node {
                            width: Val::Px(3.),
                            height: Val::Px(3.),
                            ..default()
                        },
                    ))
                    .observe(on_click_equip_unequip)
                    .observe(on_hover_move_caste_cursor);
            });
            spell_library.library.push(old_spell);
        }
        // Revert the old equip slot back to the default caste icon,
        // slightly transparent.
        for (mut node, slot) in slots.iter_mut() {
            if slot.0 == unequip.caste {
                node.texture_atlas.as_mut().unwrap().index = get_soul_sprite(&unequip.caste);
                node.color.set_alpha(0.1);
            }
        }
    }
}

pub fn update_caste_box(
    caste_panel: Query<&LargeCastePanel, Changed<LargeCastePanel>>,
    caste_box: Query<Entity, (With<CasteBox>, Without<LargeCastePanel>)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    mut cursor: Query<&mut Node, With<CasteCursor>>,
) {
    if let Ok(caste) = caste_panel.get_single() {
        let mut cursor = cursor.single_mut();
        if matches!(
            caste.selected_column,
            CastePanelColumn::Left | CastePanelColumn::Right,
        ) {
            let caste = match (caste.selected_column, caste.selected_row) {
                (CastePanelColumn::Left, CastePanelRow::Top) => Soul::Saintly,
                (CastePanelColumn::Left, CastePanelRow::Middle) => Soul::Artistic,
                (CastePanelColumn::Left, CastePanelRow::Bottom) => Soul::Feral,
                (CastePanelColumn::Right, CastePanelRow::Top) => Soul::Ordered,
                (CastePanelColumn::Right, CastePanelRow::Middle) => Soul::Unhinged,
                (CastePanelColumn::Right, CastePanelRow::Bottom) => Soul::Vile,
                _ => Soul::Empty,
            };
            cursor.width = Val::Px(11.);
            cursor.height = Val::Px(11.);
            cursor.left = match caste {
                Soul::Saintly | Soul::Feral => Val::Px(14.),
                Soul::Artistic => Val::Px(6.),
                _ => Val::Auto,
            };
            cursor.right = match caste {
                Soul::Ordered | Soul::Vile => Val::Px(14.),
                Soul::Unhinged => Val::Px(6.),
                _ => Val::Auto,
            };
            cursor.top = match caste {
                Soul::Ordered | Soul::Saintly => Val::Px(6.),
                Soul::Unhinged | Soul::Artistic => Val::Px(26.),
                Soul::Feral | Soul::Vile => Val::Px(46.),
                _ => Val::Auto,
            };
            let caste_box = caste_box.single();
            // TODO: Instead of multiple entities, would it be interesting to
            // have these merged into a single string with \n to space them out?
            // This would be good in case there's a ton of "effects flags".
            let (mut caste_name, mut caste_description) =
                (Entity::PLACEHOLDER, Entity::PLACEHOLDER);
            commands.entity(caste_box).despawn_descendants();
            commands.entity(caste_box).with_children(|parent| {
                caste_name =
                    spawn_split_text(&match_soul_with_string(&caste), parent, &asset_server);
                caste_description =
                    spawn_split_text(match_soul_with_description(&caste), parent, &asset_server);
                parent.spawn((
                    ImageNode {
                        image: asset_server.load("spritesheet.png"),
                        texture_atlas: Some(TextureAtlas {
                            layout: atlas_layout.handle.clone(),
                            index: get_soul_sprite(&caste),
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
            commands.entity(caste_name).insert(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.5),
                ..default()
            });
            commands.entity(caste_description).insert(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(3.5),
                ..default()
            });
        } else {
            cursor.width = Val::Px(5.);
            cursor.height = Val::Px(5.);
            cursor.left = match caste.selected_column {
                CastePanelColumn::LibraryLeft => Val::Px(27.),
                CastePanelColumn::LibraryRight => Val::Px(31.),
                _ => Val::Auto,
            };
            cursor.top = match caste.selected_row {
                CastePanelRow::Library(depth) => Val::Px(15. + 4. * depth as f32),
                _ => Val::Auto,
            };
        }
    }
}

pub fn match_soul_with_string(soul: &Soul) -> String {
    let string = match soul {
        Soul::Saintly => "[l]Saintly Soul[w]",
        Soul::Ordered => "[r]Ordered Soul[w]",
        Soul::Artistic => "[o]Artistic Soul[w]",
        Soul::Unhinged => "[y]Unhinged Soul[w]",
        Soul::Feral => "[g]Feral Soul[w]",
        Soul::Vile => "[p]Vile Soul[w]",
        Soul::Empty => "[w]Spell Menu[w]",
        _ => &format!("{:?}", soul),
    };
    string.to_owned()
}

pub fn on_hover_move_caste_cursor(
    hover: Trigger<Pointer<Over>>,
    mut caste_menu: Query<&mut LargeCastePanel>,
    equip: Query<&EquipSlot>,
    library: Query<&LibrarySlot>,
    spell_storage: Res<SpellLibrary>,
) {
    let mut caste_menu = caste_menu.single_mut();
    if let Ok(slot) = equip.get(hover.entity()) {
        (caste_menu.selected_column, caste_menu.selected_row) = match slot.0 {
            Soul::Saintly => (CastePanelColumn::Left, CastePanelRow::Top),
            Soul::Artistic => (CastePanelColumn::Left, CastePanelRow::Middle),
            Soul::Feral => (CastePanelColumn::Left, CastePanelRow::Bottom),
            Soul::Ordered => (CastePanelColumn::Right, CastePanelRow::Top),
            Soul::Unhinged => (CastePanelColumn::Right, CastePanelRow::Middle),
            Soul::Vile => (CastePanelColumn::Right, CastePanelRow::Bottom),
            _ => panic!(),
        };
    } else if let Ok(library) = library.get(hover.entity()) {
        let index = spell_storage
            .library
            .iter()
            .position(|r| r.id == library.0)
            .unwrap();
        caste_menu.selected_row = CastePanelRow::Library(index / 2);
        caste_menu.selected_column = if index % 2 == 0 {
            CastePanelColumn::LibraryLeft
        } else {
            CastePanelColumn::LibraryRight
        };
    }
}

pub fn on_click_equip_unequip(
    click: Trigger<Pointer<Click>>,
    mut equip: EventWriter<EquipSpell>,
    mut unequip: EventWriter<UnequipSpell>,
    equip_slot: Query<&EquipSlot>,
    library: Query<&LibrarySlot>,
    spell_storage: Res<SpellLibrary>,
) {
    if let Ok(slot) = equip_slot.get(click.entity()) {
        unequip.send(UnequipSpell { caste: slot.0 });
    } else if let Ok(library) = library.get(click.entity()) {
        let index = spell_storage
            .library
            .iter()
            .position(|r| r.id == library.0)
            .unwrap();
        equip.send(EquipSpell { index });
    }
}

// #[derive(Component)]
// pub struct CasteSlide {
//     timer: Timer,
//     curve: EasingCurve<Vec3>,
// }

// #[derive(Component)]
// pub struct AnimatedCasteIcon;

// enum CasteDestination {
//     Equip(Soul),
//     Unequip(usize),
// }

// #[derive(Event)]
// pub struct SlideCastes {
//     destination: CasteDestination,
// }

// pub fn dispense_sliding_components_caste(
//     mut events: EventReader<SlideCastes>,
//     mut commands: Commands,
//     node: Query<(Entity, &Node), With<AnimatedCasteIcon>>,
// ) {
//     for event in events.read() {
//         for (entity, node) in node.iter() {
//             let curve_start = Vec3::new(
//                 extract_from_val(node.left),
//                 extract_from_val(node.top),
//                 extract_from_val(node.width),
//             );
//             let curve_end = match event.destination {
//                 CasteDestination::Equip(caste) => Vec3::new(
//                     match caste {
//                         Soul::Saintly | Soul::Feral => 16.,
//                         Soul::Artistic => 8.,
//                         Soul::Unhinged => 57.,
//                         Soul::Vile | Soul::Ordered => 49.,
//                         _ => 0.,
//                     },
//                     match caste {
//                         Soul::Ordered | Soul::Saintly => 8.,
//                         Soul::Unhinged | Soul::Artistic => 28.,
//                         Soul::Feral | Soul::Vile => 48.,
//                         _ => 0.,
//                     },
//                     7.,
//                 ),
//                 CasteDestination::Unequip(library) => Vec3::splat(3.),
//             };

//             commands.entity(entity).insert(CasteSlide {
//                 timer: Timer::new(Duration::from_millis(3000), TimerMode::Once),
//                 curve: EasingCurve::new(curve_start, curve_end, EaseFunction::QuadraticInOut),
//             });
//         }
//     }
// }

// pub fn slide_caste_spells(mut spells: Query<(&mut Node, &mut CasteSlide)>, time: Res<Time>) {
//     for (mut node, mut spell) in spells.iter_mut() {
//         {
//             spell.timer.tick(time.delta());
//             let new_dimensions = spell.curve.sample_clamped(spell.timer.fraction());
//             node.top = Val::Px(new_dimensions.x);
//             node.left = Val::Px(new_dimensions.y);
//             node.width = Val::Px(new_dimensions.z);
//             node.height = Val::Px(new_dimensions.z);
//         }
//     }
// }

// fn extract_from_val(val: Val) -> f32 {
//     if let Val::Px(val) = val {
//         val
//     } else {
//         panic!();
//     }
// }
