use bevy::prelude::*;

use crate::{
    events::CagePainter,
    ui::{AxiomBox, MessageLog, QuestBox, RecipebookUI},
};

pub fn show_quest_menu(
    mut set: ParamSet<(
        Query<&mut Visibility, With<MessageLog>>,
        Query<&mut Visibility, With<RecipebookUI>>,
        Query<&mut Visibility, With<AxiomBox>>,
        Query<&mut Visibility, With<QuestBox>>,
    )>,
    painter: Res<CagePainter>,
) {
    if painter.is_painting {
        *set.p1().single_mut() = Visibility::Hidden;
        *set.p2().single_mut() = Visibility::Hidden;
    }
    *set.p0().single_mut() = Visibility::Hidden;
    for mut vis in set.p3().iter_mut() {
        *vis = Visibility::Inherited;
    }
}

pub fn hide_quest_menu(
    mut set: ParamSet<(
        Query<&mut Visibility, With<MessageLog>>,
        Query<&mut Visibility, With<RecipebookUI>>,
        Query<&Visibility, With<AxiomBox>>,
        Query<&mut Visibility, With<QuestBox>>,
    )>,
    painter: Res<CagePainter>,
) {
    if painter.is_painting {
        *set.p1().single_mut() = Visibility::Inherited;
    }
    if matches!(set.p2().single(), Visibility::Hidden) {
        *set.p0().single_mut() = Visibility::Inherited;
    }
    for mut vis in set.p3().iter_mut() {
        *vis = Visibility::Hidden;
    }
}
