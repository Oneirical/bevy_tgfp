use bevy::prelude::*;

use crate::{
    events::{
        become_intangible, creature_step, end_turn, repression_damage, summon_creature,
        teleport_entity,
    },
    graphics::{all_animations_complete, decay_magic_effects, place_magic_effects},
    map::register_creatures,
    spells::{process_axiom, queue_up_spell, spell_stack_is_not_empty},
};

pub struct SetsPlugin;

impl Plugin for SetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, process_axiom);
        app.add_systems(
            FixedUpdate,
            (
                ((
                    creature_step,
                    queue_up_spell,
                    // FIXME: This run condition is broken. It is replaced by the "if let Some".
                    // This might be because of the system sets.
                    process_axiom.run_if(spell_stack_is_not_empty),
                )
                    .chain())
                .in_set(ActionPhase),
                ((
                    summon_creature,
                    repression_damage,
                    become_intangible,
                    register_creatures,
                    teleport_entity,
                )
                    .chain())
                .in_set(ResolutionPhase),
                ((place_magic_effects, decay_magic_effects).chain()).in_set(AnimationPhase),
                ((end_turn).chain()).in_set(TurnPhase),
            ),
        );
        app.configure_sets(
            FixedUpdate,
            (
                ActionPhase,
                ResolutionPhase,
                AnimationPhase,
                TurnPhase.run_if(all_animations_complete),
            )
                .chain(),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct ActionPhase;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct ResolutionPhase;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct AnimationPhase;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct TurnPhase;
