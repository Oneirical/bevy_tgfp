+++
title = "Bevy Traditional Roguelike Quick-Start - 2. Tiles to Frolic Around In"
date = 2024-09-18
authors = ["Julien Robert"]
[taxonomies]
tags = ["rust", "bevy", "tutorial"]
+++

Motionless floating in the void is getting old. Let's remedy this.

Our player might have a `Transform` translation of 0, making it appear in the centre of the screen, but that is merely a visual position. Roguelikes take place on a grid, and if a spell starts summoning magical rainbow clouds, it will need to know where to place those pretty vapours. This is where a new component, `Position`, comes in.

```rust
/// A position on the map.
#[derive(Component, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    /// Create a new Position instance.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Edit an existing Position with new coordinates.
    pub fn update(&mut self, x: i32, y: i32) {
        (self.x, self.y) = (x, y);
    }
}
```

This is, quite literally, a glorified `(i32, i32)` tuple with some functions to help manage its fields. The vast list of `#[derive]` macros is mostly self-explanatory, aside from the `Hash` which will be relevant later.

Not only do `Creature`s have a visual apperance, they also have a place where they exist on that grid. That is why they now obtain this new `Position` Component:

```rust
#[derive(Bundle)]
pub struct Creature {
    pub position: Position, // NEW!
    pub sprite: Sprite,
}

// SNIP

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    commands.spawn(Creature {
        Creature {
            position: Position { x: 4, y: 4 }, // NEW!
            sprite: Sprite {
                image: asset_server.load("spritesheet.png"),
                custom_size: Some(Vec2::new(64., 64.)),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index: 0,
                }),
                ..default()
            },
        },
    );
}
```

The choice of (4, 4) as the player's starting coordinates is arbitrary, but will be useful imminently to show off the visual effect of this offset from (0, 0).

Right now, `Position` does absolutely nothing. Even if it did do something, it would be quite difficult to tell, as there is only a single creature in this entire gray plane of nothingness and no other reference points. Let us fix that by placing the player into a 9x9 white cage of walls:

```rust
fn spawn_cage(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    let cage = "#########\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #.......#\
                #########";
    for (idx, tile_char) in cage.char_indices() {
        let position = Position::new(idx as i32 % 9, idx as i32 / 9);
        let index = match tile_char {
            '#' => 3,
            _ => continue,
        };
        commands.spawn(Creature {
            position,
            sprite: Sprite {
                image: asset_server.load("spritesheet.png"),
                custom_size: Some(Vec2::new(64., 64.)),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index,
                }),
                ..default()
            },
        });
    }
}
```

For each character within the `cage` string, the (x, y) `Position` is derived using modulo and division, respectively (every 9 tiles, the y coordinate increments by 1, and the remainder of that division is the x coordinate). Note that this will cause a mirror flip (as this code starts counting from the top, whereas Bevy's coordinate system starts counting from the bottom). This will not be an issue when the map generator is refactored in a future chapter.

As for the `#` being proper walls, we simply abort the loop for any character that is not a `#`, and assign sprite index "3" for those that are. This will go fetch the third sprite in our spritesheet!

Finally, the walls can be spawned one by one. Note the `Transform::from_scale(Vec3::new(4., 4., 0.))̀`, which is the exact same as the player - currently, every creature is drawn in the centre of the screen with a size of 64x64 (4 times 16 x 4 times 16).

Yes, the walls are `Creature`s. You can imagine them as really big, lazy snails if that floats your boat.

Don't forget to add this new system:

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_resource::<SpriteSheetAtlas>()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_player)
        .add_systems(Startup, spawn_cage) // NEW!
        .run();
}
```

Running `cargo run` will prove unimpressive.

{{ image(src="https://raw.githubusercontent.com/Oneirical/oneirical.github.io/main/2-tiles-to-run-around-in/stack.png", alt="A Bevy app with a single wall tile in the centre.",
         position="center", style="border-radius: 8px;") }}

The player is still there, drawn under a pile of 32 walls. `Position` is still completely ineffectual. Disappointing! It is time to remedy this. First, we'll need a way to quickly tell Bevy which of these 33 creatures is the `Player`:

```rust
/// Marker for the player
#[derive(Component)]
pub struct Player;
```

And, of course, to assign this new component to said player:

```rust
fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
) {
    commands.spawn(( // CHANGED - Note the added parentheses.
        Creature {
            position: Position { x: 4, y: 4 },
            sprite: Sprite {
                image: asset_server.load("spritesheet.png"),
                custom_size: Some(Vec2::new(64., 64.)),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index: 0,
                }),
                ..default()
            },
        },
        Player, // NEW!
    )); // CHANGED - Note the added parentheses.
}
```

Indeed, `commands.spawn()` doesn't only accept a single `Bundle` (`Creature`), but rather any set of `Component`s and `Bundle`s, arranged in a tuple. This `(Creature, Player)` tuple is therefore a valid argument!

Just like `Position`, `Player` also currently does nothing. However, it's time to unify everything with our very first `Update` system:

```rust
/// Each frame, adjust every entity's display location to match
/// their position on the grid, and make the camera follow the player.
fn adjust_transforms(
    mut creatures: Query<(&Position, &mut Transform, Has<Player>)>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Position>)>,
) {
    for (pos, mut trans, is_player) in creatures.iter_mut() {
        // Multiplied by the graphical size of a tile, which is 64x64.
        trans.translation.x = pos.x as f32 * 64.;
        trans.translation.y = pos.y as f32 * 64.;
        if is_player {
            // The camera follows the player.
            let mut camera_trans = camera.get_single_mut().unwrap();
            (camera_trans.translation.x, camera_trans.translation.y) =
                (trans.translation.x, trans.translation.y);
        }
    }
}
```

This introduces a major Bevy feature: `Query`. A query will go fetch *all* Entities in the game that match their Component list and added filters.

Queries are always structured `Query<QueryData, QueryFilter>` where both of these are either a single parameter or multiple ones bundled in a tuple. Each one is also optional - you can have zero filters or only filters, should you desire that.

- `Query<(&Position, &mut Transform, Has<Player>)>` is trying to fetch every creature in the grid, checking if it's the player or not, see where it is located and edit its visual position.

It grants us access to all Entities with *both* `Position` and `Transform`. The `Position` component is exposed for read-only access, while `Transform` is allowed to be modified. There is also an optional `Has<Player>`, which is a boolean determining if the current Entity being accessed has the `Player` component or not.

Note that this first query only has a tuple of these 3 elements. If we wanted, we could have added additional `QueryFilter` after a comma, which is the case of the second ̀`Query`:

- `Query<&mut Transform, (With<Camera>, Without<Position>)>` is trying to fetch the camera viewing the scene, and to move it around.

It grants us access to all Entities with `Transform` exposed for modification, with the added condition that `Camera` must be possessed by the Entity. There is, however, something peculiar here: `Without<Position>`.

Bevy doesn't know that a Camera and a Creature are two different things - it only sees the components. As far as it's concerned, nothing stops us from spawning in a creature that also has a `Camera` installed. In this case, this poor mutated chimera Creature would be caught by both queries at the same time, and have its `Transform` component mutably accessed twice in a row.

Those familiar with Rust's holy commandments will know that this is **unacceptable**. Try it without the `Without<Position>` filter, and the game will panic on startup.

The first loop fetches the player, as well as every wall. We loop through all of the matched entities with `iter_mut()`, then multiply their `Position` x and y coordinates by 64.0. This is the size of one tile (64 pixels x 64 pixels), letting this value become a graphical offset once it is assigned to their `Transform` component's `translation` field, moving them across the screen!

Then, if that Entity was the player, the camera should be displaced to follow it. As the ̀`Query<&mut Transform, (With<Camera>, Without<Position>)>` only fetches a single Entity, we can use the risky `get_single_mut` which will panic should there ever be more than one `Entity` fetched by the ̀`Querỳ`. Match the Camera's translation with the Player's translation, and the system is complete!

Don't forget to register this new system.

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_resource::<SpriteSheetAtlas>()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_player)
        .add_systems(Startup, spawn_cage)
        .add_systems(Update, adjust_transforms) // NEW!
        .run();
}
```

Compile once more with `cargo run`. It will reveal the player in its little cage, with no more visual superposition of entities!

{{ image(src="https://raw.githubusercontent.com/Oneirical/oneirical.github.io/main/2-tiles-to-run-around-in/walls.png", alt="A Bevy app with the player in the centre, surrounded by 9x9 walls.",
         position="center", style="border-radius: 8px;") }}

Since this new system is `Update`, it runs every frame and readjusts all `Creature`s where they need to be relative to the player. This isn't very useful as everyone here is cursed with eternal paralysis... Let's fix that.

```rust
/// Each frame, if a button is pressed, move the player 1 tile.
fn keyboard_input(
    input: Res<ButtonInput<KeyCode>>,
    mut player: Query<&mut Position, With<Player>>,
) {
    let mut player = player.get_single_mut().expect("0 or 2+ players");
    // WASD keys are used here. If your keyboard uses a different layout
    // (such as AZERTY), change the KeyCodes.
    if input.pressed(KeyCode::KeyW) {
        player.y += 1;
    }
    if input.pressed(KeyCode::KeyD) {
        player.x += 1;
    }
    if input.pressed(KeyCode::KeyA) {
        player.x -= 1;
    }
    if input.pressed(KeyCode::KeyS) {
        player.y -= 1;
    }
}
```

`Res<ButtonInput<KeyCode>>` is a Bevy resource to manage all flavours of button mashing, from gentle taps to bulldozing over the keyboard. It contains some subtly different functions - for example, `pressed` triggers every frame throughout a maintained press, whereas `just_pressed` only triggers once on the initial press.

The player is once again fetched - mutably, this time around - and its coordinates are changed, which will result in the walls visually moving to represent this new arrangement!

Register the new system.

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_resource::<SpriteSheetAtlas>()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_player)
        .add_systems(Startup, spawn_cage)
        .add_systems(Update, adjust_transforms)
        .add_systems(Update, keyboard_input) // NEW!
        .run();
}
```

`cargo run`. You can now move around the cage... and escape it with zero difficulty by phasing through the walls, running at the speed of light into the far reaches of reality itself. Note that despite the ludicrous speed, it is impossible to stop "clipping" to the grid - you will never be in between two walls!

{{ image(src="https://raw.githubusercontent.com/Oneirical/oneirical.github.io/main/2-tiles-to-run-around-in/moving.gif", alt="A Bevy app with the player moving frantically, ignoring all walls.",
         position="center", style="border-radius: 8px;") }}

Enforcing basic physical principles will be the topic of the next tutorial!
