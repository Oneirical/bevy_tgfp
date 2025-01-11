use bevy::prelude::*;

use crate::{
    events::{
        add_status_effects, alter_momentum, assign_species_components, creature_collision,
        creature_step, distribute_npc_actions, draw_soul, echo_speed, end_turn, harm_creature,
        open_door, remove_creature, remove_designated_creatures, stepped_on_tile, summon_creature,
        teleport_entity, transform_creature, use_wheel_soul,
    },
    graphics::{adjust_transforms, decay_magic_effects, place_magic_effects},
    input::keyboard_input,
    map::register_creatures,
    spells::{
        cast_new_spell, cleanup_synapses, process_axiom, spell_stack_is_empty, trigger_contingency,
    },
};

pub struct SetsPlugin;

impl Plugin for SetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                // When a creature loses a status effect,
                // it might lose a component (such as Spellproof)
                // which is innate to its species.
                // This will ensure entities keep their species-specific
                // components when a turn begins.
                assign_species_components,
                keyboard_input.run_if(spell_stack_is_empty),
                creature_step,
                use_wheel_soul,
                process_axiom,
                cleanup_synapses,
                draw_soul,
            )
                .chain())
            .in_set(ActionPhase),
        );
        app.add_systems(
            Update,
            ((
                summon_creature,
                transform_creature,
                assign_species_components,
                register_creatures,
                add_status_effects,
                teleport_entity,
                stepped_on_tile,
                creature_collision,
                alter_momentum,
                harm_creature,
                open_door,
                remove_designated_creatures.run_if(spell_stack_is_empty),
                remove_creature,
                // Last chance to add spells to the spell stack before the end-of-turn check.
                trigger_contingency,
                cast_new_spell,
                end_turn.run_if(spell_stack_is_empty),
                distribute_npc_actions,
                echo_speed,
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
