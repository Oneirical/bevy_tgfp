use bevy::prelude::*;

use crate::{
    caste::{
        equip_spell, hide_caste_menu, show_caste_menu, update_caste_box, EquipSpell, UnequipSpell,
    },
    crafting::{
        craft_with_axioms, take_or_drop_soul, CraftWithAxioms, CraftingRecipes, TakeOrDropSoul,
    },
    creature::SpellLibrary,
    cursor::{cursor_step, despawn_cursor, spawn_cursor, teleport_cursor, update_cursor_box},
    events::{
        add_status_effects, alter_momentum, assign_species_components, creature_collision,
        creature_step, distribute_npc_actions, draw_soul, echo_speed, end_turn, harm_creature,
        is_painting, magnet_follow, magnetize_tail_segments, open_close_door, remove_creature,
        remove_designated_creatures, render_closing_doors, respawn_cage, respawn_player,
        stepped_on_tile, summon_creature, swap_current_paint, teleport_entity, transform_creature,
        use_wheel_soul,
    },
    graphics::{adjust_transforms, decay_magic_effects, place_magic_effects},
    input::keyboard_input,
    map::register_creatures,
    spells::{
        cast_new_spell, cleanup_synapses, process_axiom, spell_stack_is_empty, trigger_contingency,
    },
    ui::{
        decay_fading_title, despawn_fading_title, dispense_sliding_components_log,
        print_message_in_log, slide_message_log, spawn_fading_title,
    },
};

pub struct SetsPlugin;

impl Plugin for SetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ControlState>();
        app.add_systems(OnEnter(ControlState::Cursor), spawn_cursor);
        app.add_systems(OnExit(ControlState::Cursor), despawn_cursor);
        app.add_systems(OnEnter(ControlState::CasteMenu), show_caste_menu);
        app.add_systems(OnExit(ControlState::CasteMenu), hide_caste_menu);
        app.add_systems(Update, magnetize_tail_segments.before(teleport_entity));
        app.add_systems(Update, magnet_follow.after(teleport_entity));
        app.add_systems(Update, take_or_drop_soul.after(stepped_on_tile));
        app.add_systems(Update, craft_with_axioms);
        app.add_systems(Update, swap_current_paint.run_if(is_painting));

        // app.add_systems(Update, dispense_sliding_components_caste);
        // app.add_systems(Update, slide_caste_spells);
        // app.add_event::<SlideCastes>();

        app.add_event::<TakeOrDropSoul>();
        app.add_event::<CraftWithAxioms>();
        app.add_event::<EquipSpell>();
        app.add_event::<UnequipSpell>();
        app.add_systems(Update, equip_spell);
        app.init_resource::<CraftingRecipes>();
        app.insert_resource(SpellLibrary {
            library: Vec::new(),
        });
        app.add_systems(
            Update,
            (cursor_step, teleport_cursor, update_cursor_box)
                .run_if(in_state(ControlState::Cursor)),
        );
        app.add_systems(
            Update,
            update_caste_box.run_if(in_state(ControlState::CasteMenu)),
        );
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
                use_wheel_soul.run_if(not(is_painting)),
                process_axiom,
                cleanup_synapses,
                draw_soul.run_if(not(is_painting)),
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
                open_close_door,
                respawn_player,
                remove_creature,
                // Last chance to add spells to the spell stack before the end-of-turn check.
                trigger_contingency,
                cast_new_spell,
                remove_designated_creatures.run_if(spell_stack_is_empty),
                end_turn.run_if(spell_stack_is_empty),
                distribute_npc_actions,
                echo_speed,
                respawn_cage.run_if(spell_stack_is_empty),
            )
                .chain())
            .in_set(ResolutionPhase),
        );
        app.add_systems(
            Update,
            ((
                render_closing_doors,
                place_magic_effects,
                adjust_transforms,
                decay_magic_effects,
                spawn_fading_title,
                decay_fading_title,
                despawn_fading_title,
                // NOTE: This must go before print_message_in_log,
                // or else TextLayoutInfo has no time to compute.
                dispense_sliding_components_log,
                print_message_in_log,
                slide_message_log,
            )
                .chain())
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

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ControlState {
    #[default]
    Player,
    Cursor,
    CasteMenu,
}
