# Cleaning Our Room

Before continuing, it must be noted that the `main.rs` file is slowly reaching critical mass with its 161 lines of code. Before it swallows the Sun, it would be wise to divide it into multiple files, using `Plugins`.

As an example, let's bundle up everything that has something to do with displaying things on screen into a single `GraphicsPlugin`.

Create a new file in `src/graphics.rs`. Write within:

```rust
// graphics.rs

use bevy::prelude::*;
// Note the imports from main.rs
use crate::{Player, OrdDir, Position};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
    }
}
```

Then, add the resource and the two systems, as they appeared in Part 2 of the tutorial:

```rust
// graphics.rs
#[derive(Resource)]
pub struct SpriteSheetAtlas { // Note the pub!
    handle: Handle<TextureAtlasLayout>,
}

impl FromWorld for SpriteSheetAtlas {
    fn from_world(world: &mut World) -> Self {
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 1, None, None);
        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();
        Self {
            handle: texture_atlases.add(layout),
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });
}

/// Each frame, adjust every entity's display location to be offset
/// according to the player's location.
fn adjust_transforms(
    player: Query<&Position, With<Player>>,
    mut npcs: Query<(&Position, &mut Transform), Without<Player>>,
) {
    // There should only be one player on any given frame.
    let player_pos = player.get_single().expect("0 or 2+ players");
    // Get the player's position.
    let (px, py) = (player_pos.x, player_pos.y);
    // For each Position and Transform of each non-player creature...
    for (npc_pos, mut npc_tran) in npcs.iter_mut() {
        // Measure their offset distance from the player's location.
        let (off_x, off_y) = (npc_pos.x - px, npc_pos.y - py);
        // Adjust their visual position to match this offset.
        (npc_tran.translation.x, npc_tran.translation.y) = (
            // Multiplied by the graphical size of a tile, which is 64x64.
            off_x as f32 * 4. * 16.,
            off_y as f32 * 4. * 16.,
        );
    }
}
```

This can finally be connected to `main.rs`:

```rust
// main.rs
mod graphics; // NEW!

use graphics::GraphicsPlugin; // NEW!

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(GraphicsPlugin) // NEW!
        // Note that the following have been removed:
        // - SpriteSheetAtlas
        // - setup_camera
        // - adjust_transforms
        .add_systems(Startup, spawn_player)
        .add_systems(Startup, spawn_cage)
        .add_systems(Update, keyboard_input)
        .run();
}
```

Note that this reorganization comes with the necessity of many import (`use`) statements. In the future of this tutorial, inter-file imports will no longer be represented in the code snippets. `rust-analyzer` offers auto-importing of unimported items as a code action, and compiler errors for this particular issue are clear and offer precise suggestions.

I have organized the rest of the Part 2 components, bundles, systems and resources in the following way:

`creature.rs` (**No plugin! Only struct definitions.**)
- Player
- Creature

`input.rs`
- keyboard_input

`map.rs`
- Position
- spawn_player
- spawn_cage

And, as it was only just done:

`graphics.rs`
- SpriteSheetAtlas
- setup_camera
- adjust_transforms

We will also add `pub` markers to the structs and enums moved over (but not the systems). As `Component`s and `Resourcè`s tend to travel around quite a bit, they will often need to be imported across other `Plugin`s. Not to worry, missing a `pub` will simply have the compiler complain a bit and provide a helpful error message to correct the issue, mentioning that "this struct is inaccessible".

This leads to this `main()` function:

```rust
// main.rs
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((GraphicsPlugin, MapPlugin, InputPlugin))
        .run();
}
```

Note the tuple in the second `add_plugins̀`. Just as it was shown in Part 2 for `commands.spawn()`, many Bevy functions can take either a single item or a tuple of items as an argument!

# Detecting the Happening of Things - Events

You may remember `keyboard_input` and how it adjusts the `Player`'s position:

```rust
// input.rs
// SNIP
if input.pressed(KeyCode::KeyW) {
    player.y += 1;
}
// SNIP
```

This is very weak programming! As the game expands, we might need to detect when the player steps on slippery goo or when it collides with another entity. We'll need to implement these checks on each possible direction to step in, have error-prone repeated code blocks, and end up with a towering heap of function arguments that looks like this:

```rust
fn dispense_functions(
    mut creatures: ParamSet<(
        Query<(&Transform, &mut Species, &mut SoulBreath, &mut AxiomEffects, 
        	&mut Animator<Transform>, &mut Position, Has<RealityAnchor>)>,
        Query<&Position>,
        Query<&Species>,
        Query<&SoulBreath>,
        Query<(&Position, &Transform), With<RealityAnchor>>,
    )>,
    mut plant: Query<&mut Plant>,
    faction: Query<&Faction>,
    check_wound: Query<Entity, With<Wounded>>,
    mut next_state: ResMut<NextState<TurnState>>,
    mut world_map: ResMut<WorldMap>,
    mut souls: Query<(&mut Animator<Transform>, &Transform, 
    	&mut TextureAtlasSprite, &mut Soul), Without<Position>>,
    ui_center: Res<CenterOfWheel>,
    time: Res<SoulRotationTimer>,
    mut events: EventWriter<LogMessage>,
    mut zoom: ResMut<ZoomInEffect>,
    mut commands: Commands,
    mut current_crea_display: ResMut<CurrentEntityInUI>,
    texture_atlas_handle: Res<SpriteSheetHandle>,
){
```

Yes, this is a real function, from one of my old (and bad) Bevy projects. We wish to avoid this. Enter: `Event`s!

This revolution will be neatly contained in a new plugin, `EventPlugin`, inside `events.rs`. The creation of the new plugin, and the import into `main.rs`, will be left as an exercise to the reader. Or, left to the cheeky snooping around the [final reference files](TODO) of this Part 3. 

It will serve as a repository of the "actions" being taken within our game. The player taking a step is one such action of interest.

```rust
// events.rs
#[derive(Event)]
pub struct PlayerStep {
    pub direction: OrdDir,
}
```

Note the new struct: `OrdDir`, short for "Ordinal Direction". This will be a very common enum throughout the game's code - so common, in fact, that I have opted to place it within ̀`main.rs̀̀̀`. This is personal preference and it could have very well been integrated into one of the plugins.

```rust
// main.rs
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum OrdDir {
    Up,
    Right,
    Down,
    Left,
}

impl OrdDir {
    pub fn as_offset(self) -> (i32, i32) {
        let (x, y) = match self {
            OrdDir::Up => (0, 1),
            OrdDir::Right => (1, 0),
            OrdDir::Down => (0, -1),
            OrdDir::Left => (-1, 0),
        };
        (x, y)
    }
}
```


```rust
// events.rs
fn player_step(
    mut events: EventReader<PlayerStep>,
    mut player: Query<&mut Position, With<Player>>,
) {
    let mut player_pos = player.get_single_mut().expect("0 or 2+ players");
    for event in events.read() {
        let (off_x, off_y) = event.direction.as_offset();
        player_pos.shift(off_x, off_y);
    }
}
```
