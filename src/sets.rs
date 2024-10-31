use bevy::prelude::*;

use crate::{
    events::{
        become_intangible, creature_collision, creature_step, end_turn, repression_damage,
        summon_creature, teleport_entity,
    },
    graphics::{
        adjust_transforms, all_animations_complete, decay_magic_effects, place_magic_effects,
    },
    map::register_creatures,
    spells::{all_spells_complete, process_axiom, queue_up_spell},
};

pub struct SetsPlugin;

impl Plugin for SetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(TurnProgression::Animating);
        app.add_systems(Update, process_axiom);
        app.add_systems(
            FixedUpdate,
            (
                ((
                    creature_step,
                    queue_up_spell,
                    // FIXME: This run condition is broken. It is replaced by the "if let Some".
                    // This might be because of the system sets.
                    process_axiom.run_if(not(all_spells_complete)),
                )
                    .chain())
                .in_set(ActionPhase),
                ((
                    summon_creature,
                    register_creatures,
                    teleport_entity,
                    creature_collision,
                    repression_damage,
                    become_intangible,
                )
                    .chain())
                .in_set(ResolutionPhase),
                ((adjust_transforms, place_magic_effects, decay_magic_effects).chain())
                    .in_set(AnimationPhase),
                ((end_turn).chain()).in_set(TurnPhase),
            ),
        );
        app.configure_sets(
            FixedUpdate,
            (
                ActionPhase,
                ResolutionPhase,
                TurnPhase.run_if(all_spells_complete),
                AnimationPhase.run_if(in_state(TurnProgression::Animating)),
            )
                .chain(),
        );
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TurnProgression {
    PlayerTurn,
    NpcTurn,
    Animating,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct ActionPhase;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct ResolutionPhase;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct AnimationPhase;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct TurnPhase;
