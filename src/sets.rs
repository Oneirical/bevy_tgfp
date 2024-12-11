use bevy::prelude::*;

use crate::{
    events::{
        alter_momentum, creature_collision, creature_step, end_turn, harm_creature, open_door,
        remove_creature, summon_creature, teleport_entity,
    },
    graphics::{adjust_transforms, decay_magic_effects, place_magic_effects},
    input::keyboard_input,
    map::register_creatures,
    spells::{cast_new_spell, process_axiom, spell_stack_is_empty},
};

pub struct SetsPlugin;

impl Plugin for SetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                keyboard_input.run_if(spell_stack_is_empty),
                creature_step,
                cast_new_spell,
                process_axiom,
            )
                .chain())
            .in_set(ActionPhase),
        );
        app.add_systems(
            Update,
            ((
                summon_creature,
                register_creatures,
                teleport_entity,
                creature_collision,
                alter_momentum,
                harm_creature,
                open_door,
                remove_creature,
                end_turn.run_if(spell_stack_is_empty),
            )
                .chain())
            .in_set(ResolutionPhase),
        );
        app.add_systems(
            Update,
            ((place_magic_effects, adjust_transforms, decay_magic_effects).chain())
                .in_set(AnimationPhase),
        );
        app.configure_sets(
            Update,
            (ActionPhase, AnimationPhase, ResolutionPhase).chain(),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct ActionPhase;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct ResolutionPhase;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct AnimationPhase;
