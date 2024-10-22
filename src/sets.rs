use bevy::prelude::*;

use crate::{
    events::{creature_step, end_turn, summon_creature, teleport_entity},
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
                ((summon_creature, register_creatures, teleport_entity).chain())
                    .in_set(ResolutionPhase),
                ((end_turn).chain()).in_set(TurnPhase),
            ),
        );
        app.configure_sets(
            FixedUpdate,
            (ActionPhase, ResolutionPhase, TurnPhase).chain(),
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
