use bevy::prelude::*;

use crate::{
    events::{
        become_intangible, creature_step, end_turn, repression_damage, summon_creature,
        teleport_entity,
    },
    graphics::{all_animations_complete, decay_magic_effects, place_magic_effects},
    map::register_creatures,
    spells::{dispatch_events, gather_effects},
};

pub struct SetsPlugin;

impl Plugin for SetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                ((creature_step, gather_effects, dispatch_events).chain()).in_set(ActionPhase),
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
