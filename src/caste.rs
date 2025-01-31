use bevy::prelude::*;

use crate::{
    creature::{get_soul_sprite, Player, Soul, SpellLibrary, Spellbook},
    graphics::SpriteSheetAtlas,
    text::match_soul_with_description,
    ui::{spawn_split_text, CasteBox, LargeCastePanel, MessageLog},
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
    index: usize,
}

pub fn equip_spell(
    mut events: EventReader<EquipSpell>,
    mut spell_library: ResMut<SpellLibrary>,
    mut spellbook: Query<&mut Spellbook, With<Player>>,
) {
    for event in events.read() {
        let equipped_spell = spell_library.library.remove(event.index);
        let mut spellbook = spellbook.single_mut();
        if let Some(old_spell) = spellbook.spells.remove(&equipped_spell.caste) {
            spell_library.library.push(old_spell);
        }
        spellbook
            .spells
            .insert(equipped_spell.caste, equipped_spell);
    }
}

pub fn update_caste_box(
    caste_panel: Query<&LargeCastePanel, Changed<LargeCastePanel>>,
    caste_box: Query<Entity, (With<CasteBox>, Without<LargeCastePanel>)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    if let Ok(caste) = caste_panel.get_single() {
        let caste = caste.0;
        let caste_box = caste_box.single();
        // TODO: Instead of multiple entities, would it be interesting to
        // have these merged into a single string with \n to space them out?
        // This would be good in case there's a ton of "effects flags".
        let (mut caste_name, mut caste_description) = (Entity::PLACEHOLDER, Entity::PLACEHOLDER);
        commands.entity(caste_box).despawn_descendants();
        commands.entity(caste_box).with_children(|parent| {
            caste_name = spawn_split_text(&match_soul_with_string(&caste), parent, &asset_server);
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
