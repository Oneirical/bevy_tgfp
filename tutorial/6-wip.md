Are your muscles crisped yet from our animation system preventing you from button mashing? Let us introduce something to alleviate that.

```rust
// input.rs
#[derive(Resource)]
pub struct KeyboardInputId {
    id: SystemId,
}

impl FromWorld for KeyboardInputId {
    fn from_world(world: &mut World) -> Self {
        KeyboardInputId {
            id: world.register_system(keyboard_input),
        }
    }
}

/// If any key is pressed during animations, skip the rest of the animations
/// and execute that action.
pub fn accelerate_animations(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    slide_skippers: Query<Entity, With<SlideAnimation>>,
    mut magic_vfx: Query<&mut Visibility, With<MagicVfx>>,
    keyboard_repeat: Res<KeyboardInputId>,
) {
    if input.any_just_pressed([
        KeyCode::Space,
        KeyCode::KeyW,
        KeyCode::KeyA,
        KeyCode::KeyS,
        KeyCode::KeyD,
    ]) {
        for entity in slide_skippers.iter() {
            // All sliding creatures snap to their destination.
            commands.entity(entity).remove::<SlideAnimation>();
        }
        for mut visibility in magic_vfx.iter_mut() {
            // All visual magic effects become visible.
            *visibility = Visibility::Inherited;
        }
        commands.run_system(keyboard_repeat.id);
    }
}
```

The pre-existing system `keyboard_input` is registered inside a One-Shot System, as seen previously through the spell system. Whenever an input is detected during the animation phase, this system will forcefully cause all magic effect and sliding animations to finish, and repeat that keyboard input to perform its associated action.

This new system is added as a part of the ̀`AnimationPhase`. It only runs if there are animations to be skipped at all.

```rust
        app.add_systems(
            Update,
            ((
              // NEW!
                accelerate_animations
                    .run_if(not(all_animations_finished))
              // End NEW.
                place_magic_effects,
                adjust_transforms,
                decay_magic_effects,
            )
                .chain())
            .in_set(AnimationPhase),
        );
```

If you `cargo run` now, you'll be able to sprint much faster by mashing your keyboard. However, this can lead to exploits - try casting **MomentumBeam, Dash** twice, then rapidly stepping one tile into the beam. As you were faster than Bevy's system rotation, your character is now on the beam's path, and you will be boosted forwards by your own beam, which wasn't supposed to hit you!

// TODO gif

To fix this, we'll allow animation-skipping only for steps, and let spells - which are a more important gameplay event - fully unfold.

We'll do this by checking if any spells still need to be executed this tick, and prevent animation-skipping if that is the case.

```rust
// spells.rs
pub fn spell_stack_is_empty(spell_stack: Res<SpellStack>) -> bool {
    spell_stack.spells.is_empty()
}
```

```rust
// sets.rs
app.add_systems(
    Update,
    ((
        keyboard_input.run_if(spell_stack_is_empty), // CHANGED
        player_step,
        cast_new_spell,
        process_axiom,
    )
        .chain())
    .in_set(ActionPhase),
);
// SNIP
app.add_systems(
    Update,
    ((
        accelerate_animations
            .run_if(not(all_animations_finished))
            .run_if(spell_stack_is_empty), // NEW!
        place_magic_effects,
        adjust_transforms,
        decay_magic_effects,
    )
        .chain())
    .in_set(AnimationPhase),
);
```

And there we go! Now, you should still be just as capable of rapidly stepping, but the game will properly pause to let you watch the fireworks.

// TODO gif
