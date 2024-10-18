Thrilling to be jumping, sliding and bashing in fancy acrobatics, but quite lacking in eye candy. Creatures merely blink from one point to another, without any style or intrigue. Animation is a complex topic, but making creatures properly "dash" from one point to another is certainly doable with as little as one new resource, and a rework of `adjust_transforms`.

```rust
// graphics.rs
#[derive(Resource)]
pub struct SlideAnimation {
    pub elapsed: Timer,
}
```

```rust
// graphics.rs
impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        // NEW!
        app.insert_resource(SlideAnimation {
            elapsed: Timer::from_seconds(0.4, TimerMode::Once),
        });
        // End NEW.
        app.insert_resource(Msaa::Off);
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
    }
}
```

This `Resource` will be used to add a 0.4 second delay after each creature motion, during which the entities will slide from their origin point to their destination. Each time a `TeleportEntity` event occurs, this timer will reset, allowing the animation to unfold for each move.

```rust
// events.rs
fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut creature: Query<&mut Position>,
    mut map: ResMut<Map>,
    mut animation_timer: ResMut<SlideAnimation>, // NEW!
) {
    for event in events.read() {
        let mut creature_position = creature
            // Get the Position of the Entity targeted by TeleportEntity.
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        // If motion is possible...
        if map.is_passable(event.destination.x, event.destination.y) {
            // ...update the Map to reflect this...
            map.move_creature(*creature_position, event.destination);
            // NEW!
            // ...begin the sliding animation...
            animation_timer.elapsed.reset();
            // End NEW.
            // ...and move that Entity to TeleportEntity's destination tile.
            creature_position.update(event.destination.x, event.destination.y);
        } else {
            // Nothing here just yet, but this is where collisions between creatures
            // will be handled.
            continue;
        }
    }
}
```

Now... for the main course.

```rust
fn adjust_transforms(
    mut creatures: Query<(&Position, &mut Transform, Has<Player>)>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Position>)>,
    // NEW!
    mut animation_timer: ResMut<SlideAnimation>,
    time: Res<Time>,
    // End NEW.
) {
    // NEW!
    let fraction_before_tick = animation_timer.elapsed.fraction();
    animation_timer.elapsed.tick(time.delta());
    // Calculate what % of the animation has elapsed during this tick.
    let fraction_ticked = animation_timer.elapsed.fraction() - fraction_before_tick;
    // End NEW.
    for (pos, mut trans, is_player) in creatures.iter_mut() {
        // NEW!
        // The distance between where a creature CURRENTLY is,
        // and the destination of a creature's movement.
        // Multiplied by the graphical size of a tile, which is 64x64.
        let (dx, dy) = (
            pos.x as f32 * 64. - trans.translation.x,
            pos.y as f32 * 64. - trans.translation.y,
        );
        // The distance between the original position and the destination position.
        let (ori_dx, ori_dy) = (
            dx / animation_timer.elapsed.fraction_remaining(),
            dy / animation_timer.elapsed.fraction_remaining(),
        );
        // The sprite approaches its destination.
        trans.translation.x = bring_closer_to_target_value(
            trans.translation.x,
            ori_dx * fraction_ticked,
            pos.x as f32 * 64.,
        );
        trans.translation.y = bring_closer_to_target_value(
            trans.translation.y,
            ori_dy * fraction_ticked,
            pos.y as f32 * 64.,
        );
        // End NEW.
        if is_player {
            // The camera follows the player.
            let mut camera_trans = camera.get_single_mut().unwrap();
            (camera_trans.translation.x, camera_trans.translation.y) =
                (trans.translation.x, trans.translation.y);
        }
    }
}
```

Each tick, this system runs... but we cannot know for sure how long a tick is! A computer being turned into a localized micro-Sun due to compiling Bevy in the background while playing our game will see its Frames-Per-Seconds drop, and increase the time elapsed per tick. Therefore, the new first three lines calculate which % of the animation has been processed this tick - stored within `fraction_ticked`.

Let's say that our hero `@` is moving to `X`.

```
...
@.X
...
```

Each tile is 64x64 pixels. Right now, ̀`@` is `(128, 0)` pixels away from its destination, which is the tuple `(dx, dy)`. We need to keep track of this original value! As it approaches its goal, the distance will decrease, but our calculations must be based on the original distance.

Later on, when we reach this point, 0.2 seconds later:

```
...
.@X
...
```

`(dx, dy)` is now `(64, 0)̀`. The fraction elapsed of the timer is 50%. `64 / 0.5 = 128`, meaning the original distance is restored - stored in `(ori_dx, ori_dy)`.

Finally, the `Transform` component is adjusted. If the original distance was 128 and the fraction elapsed this tick is 3%, then the creature will move 3.84 pixels to the right this tick!

In order to avoid little visual "bumps" (in the cases where a creature is, say, at 127.84 pixels, and moves 5 pixels to the right, overshooting its objective), I also added the `bring_closer_to_target_value` function, preventing any increases past the limit no matter if that limit is negative or positive.

```rust
// graphics.rs
fn bring_closer_to_target_value(value: f32, adjustment: f32, target_value: f32) -> f32 {
    let adjustment = adjustment.abs();
    if value > target_value {
        (value - adjustment).max(target_value)
    } else if value < target_value {
        (value + adjustment).min(target_value)
    } else {
        target_value // Value is already at target.
    }
}
```

Finally, `cargo run`, and behold these smooth and graceful motions!

// TODO gif
