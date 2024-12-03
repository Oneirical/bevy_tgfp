use bevy::prelude::*;

use crate::{
    events::{creature_step, end_turn, summon_creature, teleport_entity},
    graphics::{
        adjust_transforms, all_animations_finished, decay_magic_effects, place_magic_effects,
    },
    input::{accelerate_animations, keyboard_input, KeyboardInputId},
    map::register_creatures,
    spells::{cast_new_spell, process_axiom, spell_stack_is_empty},
};

pub struct SetsPlugin;

impl Plugin for SetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeyboardInputId>();
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
                end_turn,
            )
                .chain())
            .in_set(ResolutionPhase),
        );
        app.add_systems(
            Update,
            ((
                accelerate_animations
                    .run_if(not(all_animations_finished))
                    .run_if(spell_stack_is_empty),
                place_magic_effects,
                adjust_transforms,
                decay_magic_effects,
            )
                .chain())
            .in_set(AnimationPhase),
        );
        app.configure_sets(
            Update,
            (ActionPhase, AnimationPhase, ResolutionPhase).chain(),
        );
        app.configure_sets(
            Update,
            (ActionPhase, ResolutionPhase).run_if(all_animations_finished),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct ActionPhase;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct ResolutionPhase;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct AnimationPhase;
