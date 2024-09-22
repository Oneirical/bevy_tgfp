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

Note that this reorganization comes with the necessity of many import (`use`) statements. In the future of this tutorial, inter-file imports will no longer be represented in the code snippets. `rust-analyzer` offers auto-importing of unimported items as a code action, and compiler errors for this particular issue are clear and offer precise suggestions. Also remember to clean as you go, and remove unused imports marked by warnings.

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

Compile everything with `cargo run` to make sure all is neat and proper, and to fix potential still-private or unimported structs/struct fields.

If it works, you may notice strange black lines on the periphery of the walls:

TODO image

This can happen when working with a 2D spritesheet in Bevy. To fix it, disable Multi Sample Anti-aliasing:

```rust
impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteSheetAtlas>();
        app.insert_resource(Msaa::Off); // NEW!
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, adjust_transforms);
    }
}
```

TODO image

Much better. If you'd like to see how the fully reorganized code looks like, check in [`tutorial/source_code/3-getting-chased-around/3.1-reorganized`](TODO).

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
){ /* endless misery */ }
```

Yes, this is a real function, from one of my old (and bad) Bevy projects. We wish to avoid this. Enter: `Event`s!

This revolution will be neatly contained in a new plugin, `EventPlugin`, inside a new file, `events.rs`. It will serve as a repository of the "actions" being taken within our game. The player taking a step is one such action of interest.

```rust
// events.rs
pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerStep>();
    }
}

#[derive(Event)]
pub struct PlayerStep {
    pub direction: OrdDir,
}
```

Don't forget to link all this to `main.rs`.

```rust
mod creature;
mod events; // NEW!
mod graphics;
mod input;
mod map;

use bevy::prelude::*;
use events::EventPlugin; // NEW!
use graphics::GraphicsPlugin;
use input::InputPlugin;
use map::MapPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((EventPlugin, GraphicsPlugin, MapPlugin, InputPlugin)) // CHANGED
        .run();
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

And, at last, the very first ̀`Event`-based system can be implemented:

```rust
// events.rs
fn player_step(
    // Incoming events must be read with an EventReader.
    mut events: EventReader<PlayerStep>,
    // Fetch the Position of the Player.
    mut player: Query<&mut Position, With<Player>>,
) {
    // There should only be one player.
    let mut player_pos = player.get_single_mut().expect("0 or 2+ players");
    // Unpack the event queue - not that it will be very long in this case!
    for event in events.read() {
        // Calculate how to modify the player's Position from the OrdDir.
        let (off_x, off_y) = event.direction.as_offset();
        // Change the player's position.
        player_pos.shift(off_x, off_y);
    }
}
```

Register it.

```rust
// events.rs
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerStep>();
        app.add_systems(Update, player_step); // NEW!
    }
}
```

First, note the `EventReader` argument, which is a requirement to unpack the contents of received `Event̀`s, which are getting produced by... nothing at the moment. An `EventReader`, of course, needs a companion `EventWriter`. This is how the previously unwieldy `keyboard_input` system can be reworked!

```rust
// input.rs
fn keyboard_input(
    mut events: EventWriter<PlayerStep>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.pressed(KeyCode::KeyW) {
        events.send(PlayerStep {
            direction: OrdDir::Up,
        });
    }
    if input.pressed(KeyCode::KeyD) {
        events.send(PlayerStep {
            direction: OrdDir::Right,
        });
    }
    if input.pressed(KeyCode::KeyA) {
        events.send(PlayerStep {
            direction: OrdDir::Left,
        });
    }
    if input.pressed(KeyCode::KeyS) {
        events.send(PlayerStep {
            direction: OrdDir::Down,
        });
    }
}
```

Instead of this system handling the player's motion - and being responsible for the implementation of all the subtleties that may imply, the heavy work is now all offshored to an `Event` specialized in handling this task!

`cargo ruǹ`'s results should be fairly disappointing - as, from a non-developer perspective, nothing about the game has fundamentally changed - at least not our ability to phase through walls at lightspeed. However, our codebase will be much more extensible for the near future - not to mention that this `Event` is only the first of many.

# Enforcing Basic Physics - Collisions & The Map

A wall should wall things. It's in the name.

There are multiple ways to implement this - the simplest would be to query every single creature with a `Position` on the player's move, check if any of them occupies the destination tile, and abort the move if that's the case. Computers today are decently fast, but that is still a very naive implementation.

The alternative is to keep a tidy phone book of everyone's location! Enter - the `Map` Resource.

```rust
// map.rs
/// The position of every creature, updated automatically.
#[derive(Resource)]
pub struct Map {
    pub creatures: HashMap<Position, Entity>,
}

impl Map {
    /// Which creature stands on a certain tile?
    pub fn get_entity_at(&self, x: i32, y: i32) -> Option<&Entity> {
        self.creatures.get(&Position::new(x, y))
    }

    /// Is this tile passable?
    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        self.get_entity_at(x, y).is_none()
    }
}
```

It's a `HashMap` which contains only entries where a creature exists, which gives it the ability to fetch whoever is standing on, say, (27, 4) in record time with no ̀`Query` or iterating over entities required!

When importing the `HashMap`, I suggest using the `use bevy::utils::HashMap` instead of Rust's `std` implementation. The Bevy version bases itself off of `hashbrown`, which is weaker to [flooding](https://en.wikipedia.org/wiki/Collision_attack) hacks but more performant - an interesting characteristic for game development, unless one is making the next CIA agent training simulator.

Don't forget to register this new `Resourcè`.

```rust
// map.rs
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        // NEW!
        app.insert_resource(Map {
            creatures: HashMap::new(),
        });
        // End NEW.
        app.add_systems(Startup, spawn_player);
        app.add_systems(Startup, spawn_cage);
    }
}
```

It's now possible to test the waters before venturing into a new tile, thus avoiding any further phasing incidents.

```rust
// events.rs
fn player_step(
    mut events: EventReader<PlayerStep>,
    mut player: Query<&mut Position, With<Player>>,
    map: Res<Map>,
) {
    let mut player_pos = player.get_single_mut().expect("0 or 2+ players");
    for event in events.read() {
        let (off_x, off_y) = event.direction.as_offset();
        // REPLACES player_pos.shift(off_x, off_y)
        // Get the destination tile.
        let destination = Position::new(player_pos.x + off_x, player_pos.y + off_y);
        // Check if the destination tile is empty.
        if map.is_passable(destination.x, destination.y) {
            // If yes, authorize the move.
            player_pos.shift(off_x, off_y);
        }
        // End REPLACES.
    }
}
```

Don't `cargo run` just yet! Our ̀`Map` is completely empty and unaware of the existence of walls. This can be fixed with a single new system.

```rust
// map.rs
/// Newly spawned creatures earn their place in the HashMap.
fn register_creatures(
    mut map: ResMut<Map>,
    // Any entity that has a Position that just got added to it -
    // currently only possible as a result of having just been spawned in.
    displaced_creatures: Query<(&Position, Entity), Added<Position>>,
) {
    for (position, entity) in displaced_creatures.iter() {
        // Insert the new creature in the Map. Position implements Copy,
        // so it can be dereferenced (*), but `.clone()` would have been
        // fine too.
        map.creatures.insert(*position, entity);
    }
}
```

[//]: # (It may be worthwhile to eventually mention that `commands.insert()` can be used to replace existing components, and that this does not trigger the Added filter.)

The most unique part about this new system is the ̀`Added` filter, which fetches only entities who have newly received the `Position` component and not been handled by this system yet. Right now, it means all newly created creatures will be processed by this system once, and then ignored afterwards.

Register it.

```rust
// map.rs
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map {
            creatures: HashMap::new(),
        });
        app.add_systems(Startup, spawn_player);
        app.add_systems(Startup, spawn_cage);
        app.add_systems(Update, register_creatures); // NEW!
    }
}
```

Activate `cargo run`... and the walls finally have tangibility!

TODO gif

# A Very Sticky Critter - The First NPC

It's about time the Player got some company. Not a particularly affable one, I must admit, but we all start from somewhere.

```rust
// map.rs, spawn_cage
let cage = "##########H......##.......##.......##.......##.......##.......##.......##########";
```

Edit the wall placement string to include a (H)unter. Yes, this is messy - a proper map generator will be the topic of a future chapter.

This Hunter also earns itself a separate sprite:

```rust
// map.rs, spawn_cage
let position = Position::new(idx as i32 % 9, idx as i32 / 9);
let index = match tile_char {
    '#' => 3,
    'H' => 4, // NEW!
    _ => continue,
};
```

And the ability to be differentiated from walls, with a new `Hunt` component...

```rust
// creature.rs
#[derive(Component)]
pub struct Hunt;
```

...added to any 'H' character in the initial spawn function.

```rust
// map.rs, spawn_cage
let mut creature = commands.spawn(Creature { // CHANGED - note the variable assignment
    position,
    sprite: SpriteBundle {
        texture: asset_server.load("spritesheet.png"),
        transform: Transform::from_scale(Vec3::new(4., 4., 0.)),
        ..default()
    },
    atlas: TextureAtlas {
        layout: atlas_layout.handle.clone(),
        index,
    },
});
if tile_char == 'H' {
    creature.insert(Hunt);
}
```

`cargo run`, and our new companion is here. Excellent. Now, to give it motion of its own...

TODO image

The first problem is that motion, in our game, is currently only supported by `player_step`, which solely refers to the player character and nothing else. There should be a more generic `Event`, capable of controlling absolutely any creature to move around...

```rust
// events.rs
#[derive(Event)]
struct TeleportEntity {
    destination: Position,
    entity: Entity,
}

impl TeleportEntity {
    fn new(entity: Entity, x: i32, y: i32) -> Self {
        Self {
            destination: Position::new(x, y),
            entity,
        }
    }
}
```

Its matching system has a lot of similarity to `player_step`.

```rust
// events.rs
fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut creature: Query<&mut Position>,
    map: Res<Map>,
) {
    for event in events.read() {
        let mut creature_position = creature
            // Get the Position of the Entity targeted by TeleportEntity.
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        // If motion is possible...
        if map.is_passable(event.destination.x, event.destination.y) {
            // ...move that Entity to TeleportEntity's destination tile.
            creature_position.update(event.destination.x, event.destination.y);
        } else {
            // Nothing here just yet, but this is where collisions between creatures
            // will be handled.
            continue;
        }
    }
}
```

Don't forget to register all of this...

```rust
// events.rs
impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerStep>();
        app.add_event::<TeleportEntity>(); // NEW!
        app.add_systems(Update, player_step);
        app.add_systems(Update, teleport_entity); // NEW!
    }
}
```

...and, of course, to actually use it in `player_step` so all entity motion of any kind is handled by this specialized system.

```rust
// events.rs
fn player_step(
    mut events: EventReader<PlayerStep>,
    mut teleporter: EventWriter<TeleportEntity>, // NEW!
    // CHANGED, no longer needs mutable access, and also fetches the Entity component.
    player: Query<(Entity, &Position), With<Player>>,
) {
    // CHANGED, no longer needs mutable access, and also fetches the Entity component.
    let (player_entity, player_pos) = player.get_single().expect("0 or 2+ players");
    for event in events.read() {
        let (off_x, off_y) = event.direction.as_offset();
        // CHANGED, Send the event to TeleportEntity instead of handling the motion directly.
        teleporter.send(TeleportEntity::new(
            player_entity,
            player_pos.x + off_x,
            player_pos.y + off_y,
        ));
    }
}
```

And there we go! `player_step` is now only an intermediate point leading to a central `teleport_entity` system, which can handle any and all creature motion. This means every creature will be on the same footing, with no repeated code!

Just like when `player_step` was first added, `cargo run` on this will not change gameplay whatsoever. However, all this has finally allowed us to gift motion to our new Hunter.

First, define a very naive "algorithm" to move towards a point on the map. Start with this helper function to calculate a distance between two points:

```rust
// map.rs
fn manhattan_distance(a: Position, b: Position) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}
```

And then, a way to find the best move among all four orthogonal options:

```rust
// map.rs
impl Map {

    // SNIP - all other impl Map functions
    
    /// Find all adjacent accessible tiles to start, and pick the one closest to end.
    pub fn best_manhattan_move(&self, start: Position, end: Position) -> Option<Position> {
        let mut options = [
            Position::new(start.x, start.y + 1),
            Position::new(start.x, start.y - 1),
            Position::new(start.x + 1, start.y),
            Position::new(start.x - 1, start.y),
        ];

        // Sort all candidate tiles by their distance to the `end` destination.
        options.sort_by(|&a, &b| manhattan_distance(a, end).cmp(&manhattan_distance(b, end)));

        options
            .iter()
            // Only keep either the destination or unblocked tiles.
            .filter(|&p| *p == end || self.is_passable(p.x, p.y))
            // Remove the borrow.
            .map(|p| *p)
            // Get the tile that manages to close the most distance to the destination.
            // If it exists, that is. Otherwise, this is just a None.
            .next()
    }
}
```

Finally, implement that `Hunt` implies chasing the player around.

```rust
// events.rs
fn player_step(
    mut events: EventReader<PlayerStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    player: Query<(Entity, &Position), With<Player>>,
    hunters: Query<(Entity, &Position), With<Hunt>>, // NEW!
    map: Res<Map>, // NEW! Bringing back the map, so "pathfinding" can be done.
) {
    let (player_entity, player_pos) = player.get_single().expect("0 or 2+ players");
    for event in events.read() {
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportEntity::new(
            player_entity,
            player_pos.x + off_x,
            player_pos.y + off_y,
        ));

        // NEW!
        for (hunter_entity, hunter_pos) in hunters.iter() {
            // Try to find a tile that gets the hunter closer to the player.
            if let Some(move_target) = map.best_manhattan_move(*hunter_pos, *player_pos) {
                // If it is found, cause another TeleportEntity event.
                teleporter.send(TeleportEntity {
                    destination: move_target,
                    entity: hunter_entity,
                });
            }
        }
        // End NEW.
    }
}
```

`cargo run`, and let the hunt begin!

TODO gif

There is only the slight issue that our Hunter is rather on the incorporeal side of things. Indeed, as it moves, the Map fails to update and the Hunter is still considered to have phantasmatically remained in its spawn location. Not to mention that the centre of the cage, where we spawned, is also mysteriously blocked by an invisible wall.

[//]: # (This part is really weird. Maybe there is a more elegant, Bevy specific way to do this.)

There exists another filter like `Added`, named `Changed`, which triggers whenever a specified component is not only added for the first time, but also when an already existing instance is modified - such as in the case of moving a creature around. However, using it would be unwise. Here is why - the following happen in order:

- The user presses a button on their keyboard to move.
- `PlayerStep` is triggered. Two `TeleportEntity` are sent out.
- The Player's `TeleportEntity` happens first, moving the Player onto coordinates (2, 3). The `Map` is NOT updated yet, because it is located in a different system (`register_creatures`), and ̀`teleport_entity` isn't done yet, as it has another event to get through.
- The Hunter's `TeleportEntity` happens, moving the Hunter onto coordinates (2, 3) too! This appears to be a legal move to the game, because the `Map̀` hadn't been updated yet.
- `teleport_entity` is done, and `register_creatures` triggers, editing `Map` to "knock out" the Player and leave only the Hunter, while the Player is now off the `Map` and completely untargetable.

To fix this, we need to modify the `Map` immediately after a creature moves. Leave `register_creatures` set to `Added`, and instead, modify `teleport_entity`:

```rust
// events.rs
fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut creature: Query<&mut Position>,
    mut map: ResMut<Map>, // CHANGED, this needs mutability now.
) {
    for event in events.read() {
        let mut creature_position = creature
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        if map.is_passable(event.destination.x, event.destination.y) {
            map.move_creature(*creature_position, event.destination); // NEW!
            creature_position.update(event.destination.x, event.destination.y);
        } else {
            continue;
        }
    }
}
```

`map.move_creature` is a new `impl Map` function.

```rust
// map.rs
impl Map {
    /// Move a pre-existing entity around the Map.
    pub fn move_creature(&mut self, old_pos: Position, new_pos: Position) {
        // As the entity already existed in the Map's records, remove it.
        let entity = self.creatures.remove(&old_pos).expect(&format!(
            "The map cannot move a nonexistent Entity from {:?} to {:?}.",
            old_pos, new_pos
        ));
        self.creatures.insert(new_pos, entity);
    }
}
```

And with that, everything is going according to plan.

TODO gif

The next chapter of this tutorial will introduce basic animation, as well as a cleaner way to generate the starting map, free of mega one-line `"#####H....#####"̀`-style strings and match statements!
