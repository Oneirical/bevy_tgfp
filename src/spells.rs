use std::mem::{discriminant, Discriminant};

use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};

use crate::{
    creature::{Player, Species, Spellproof, Summoned, Wall},
    events::{DamageOrHealCreature, RemoveCreature, SummonCreature, TeleportEntity},
    graphics::{EffectSequence, EffectType, PlaceMagicVfx},
    map::{Map, Position},
    OrdDir,
};

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<CastSpell>>();
        app.init_resource::<SpellStack>();
        app.init_resource::<AxiomLibrary>();
    }
}

#[derive(Resource)]
/// All available Axioms and their corresponding systems.
pub struct AxiomLibrary {
    pub library: HashMap<Discriminant<Axiom>, SystemId>,
}

impl FromWorld for AxiomLibrary {
    fn from_world(world: &mut World) -> Self {
        let mut axioms = AxiomLibrary {
            library: HashMap::new(),
        };
        axioms.library.insert(
            discriminant(&Axiom::Ego),
            world.register_system(axiom_form_ego),
        );
        axioms.library.insert(
            discriminant(&Axiom::Player),
            world.register_system(axiom_form_player),
        );
        axioms.library.insert(
            discriminant(&Axiom::MomentumBeam),
            world.register_system(axiom_form_momentum_beam),
        );
        axioms.library.insert(
            discriminant(&Axiom::Plus),
            world.register_system(axiom_form_plus),
        );
        axioms.library.insert(
            discriminant(&Axiom::Halo { radius: 1 }),
            world.register_system(axiom_form_halo),
        );
        axioms.library.insert(
            discriminant(&Axiom::Touch),
            world.register_system(axiom_form_touch),
        );
        axioms.library.insert(
            discriminant(&Axiom::Dash { max_distance: 1 }),
            world.register_system(axiom_function_dash),
        );
        axioms.library.insert(
            discriminant(&Axiom::SummonCreature {
                species: Species::Player,
            }),
            world.register_system(axiom_function_summon_creature),
        );
        axioms.library.insert(
            discriminant(&Axiom::DevourWall),
            world.register_system(axiom_function_devour_wall),
        );
        axioms.library.insert(
            discriminant(&Axiom::ArchitectCage),
            world.register_system(axiom_function_architect_cage),
        );
        axioms.library.insert(
            discriminant(&Axiom::Abjuration),
            world.register_system(axiom_function_abjuration),
        );
        axioms
    }
}

#[derive(Resource)]
/// The current spells being executed.
pub struct SpellStack {
    /// The stack of spells, last in, first out.
    pub spells: Vec<SynapseData>,
    /// A system used to clean up the last spells after each Axiom is processed.
    cleanup_id: SystemId,
}

impl FromWorld for SpellStack {
    fn from_world(world: &mut World) -> Self {
        SpellStack {
            spells: Vec::new(),
            cleanup_id: world.register_system(cleanup_last_axiom),
        }
    }
}

#[derive(Event)]
/// Triggered when a creature (the `caster`) casts a `spell`.
pub struct CastSpell {
    pub caster: Entity,
    pub spell: Spell,
}

#[derive(Component, Clone)]
/// A spell is composed of a list of "Axioms", which will select tiles or execute an effect onto
/// those tiles, in the order they are listed.
pub struct Spell {
    pub axioms: Vec<Axiom>,
}

#[derive(Debug, Clone)]
/// There are Form axioms, which target certain tiles, and Function axioms, which execute an effect
/// onto those tiles.
pub enum Axiom {
    // FORMS
    /// Target the caster's tile.
    Ego,
    /// Target the player's tile.
    Player,
    /// Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
    /// including the first solid tile encountered, which stops the beam.
    MomentumBeam,
    /// Target all orthogonally adjacent tiles to the caster.
    Plus,
    /// Target the tile adjacent to the caster, towards the caster's last move.
    Touch,
    /// Target a ring of `radius` around the caster.
    Halo {
        radius: i32,
    },

    // FUNCTIONS
    /// The targeted creatures dash in the direction of the caster's last move.
    Dash {
        max_distance: i32,
    },
    /// The targeted passable tiles summon a new instance of species.
    SummonCreature {
        species: Species,
    },
    /// Any targeted creature with the Wall component is removed.
    /// Each removed wall heals the caster +1.
    DevourWall,
    ArchitectCage,
    /// All creatures summoned by targeted creatures are removed.
    Abjuration,
}

/// The tracker of everything which determines how a certain spell will act.
pub struct SynapseData {
    /// Where a spell will act.
    targets: Vec<Position>,
    /// How a spell will act.
    pub axioms: Vec<Axiom>,
    /// The nth axiom currently being executed.
    pub step: usize,
    /// Who cast the spell.
    pub caster: Entity,
}

impl SynapseData {
    /// Create a blank SynapseData.
    fn new(caster: Entity, axioms: Vec<Axiom>) -> Self {
        SynapseData {
            targets: Vec::new(),
            axioms,
            step: 0,
            caster,
        }
    }

    /// Get the Entity of each creature standing on a tile inside `targets` and its position.
    fn get_all_targeted_entity_pos_pairs(&self, map: &Map) -> Vec<(Entity, Position)> {
        let mut targeted_pairs = Vec::new();
        for target in &self.targets {
            if let Some(creature) = map.get_entity_at(target.x, target.y) {
                targeted_pairs.push((*creature, *target));
            }
        }
        targeted_pairs
    }

    /// Get the Entity of each creature standing on a tile inside `targets`.
    fn get_all_targeted_entities(&self, map: &Map) -> Vec<Entity> {
        self.get_all_targeted_entity_pos_pairs(map)
            .into_iter()
            .map(|(entity, _)| entity)
            .collect()
    }
}

pub fn cast_new_spell(
    mut cast_spells: EventReader<CastSpell>,
    mut spell_stack: ResMut<SpellStack>,
) {
    for cast_spell in cast_spells.read() {
        // First, get the list of Axioms.
        let axioms = cast_spell.spell.axioms.clone();
        // Create a new synapse to start "rolling down the hill" accumulating targets and
        // dispatching events.
        let synapse_data = SynapseData::new(cast_spell.caster, axioms);
        // Send it off for processing - right away, for the spell stack is "last in, first out."
        spell_stack.spells.push(synapse_data);
    }
}

/// Get the most recently added spell (re-adding it at the end if it's not complete yet).
/// Get the next axiom, and runs its effects.
pub fn process_axiom(
    mut commands: Commands,
    axioms: Res<AxiomLibrary>,
    spell_stack: Res<SpellStack>,
) {
    // Get the most recently added spell, if it exists.
    if let Some(synapse_data) = spell_stack.spells.last() {
        // Get its first axiom.
        let axiom = synapse_data.axioms.get(synapse_data.step).unwrap();
        // Launch the axiom, which will send out some Events (if it's a Function,
        // which affect the game world) or add some target tiles (if it's a Form, which
        // decides where the Functions will take place.)
        commands.run_system(*axioms.library.get(&discriminant(axiom)).unwrap());
        // Clean up afterwards, continuing the spell execution.
        commands.run_system(spell_stack.cleanup_id);
    }
}

/// Target the caster's tile.
fn axiom_form_ego(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
) {
    // Get the currently executed spell.
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    // Get the caster's position.
    let caster_position = *position.get(synapse_data.caster).unwrap();
    // Place the visual effect.
    magic_vfx.send(PlaceMagicVfx {
        targets: vec![caster_position],
        sequence: EffectSequence::Sequential { duration: 0.04 },
        effect: EffectType::RedBlast,
        decay: 0.5,
        appear: 0.,
    });
    // Add that caster's position to the targets.
    synapse_data.targets.push(caster_position);
}

/// Target the player's tile.
fn axiom_form_player(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position, With<Player>>,
) {
    // Get the currently executed spell.
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    // Get the caster's position.
    let player_position = *position.get_single().unwrap();
    // Place the visual effect.
    magic_vfx.send(PlaceMagicVfx {
        targets: vec![player_position],
        sequence: EffectSequence::Sequential { duration: 0.04 },
        effect: EffectType::RedBlast,
        decay: 0.5,
        appear: 0.,
    });
    // Add that caster's position to the targets.
    synapse_data.targets.push(player_position);
}

/// Target all orthogonally adjacent tiles to the caster.
fn axiom_form_plus(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let caster_position = *position.get(synapse_data.caster).unwrap();
    let adjacent = [OrdDir::Up, OrdDir::Right, OrdDir::Down, OrdDir::Left];
    let mut output = Vec::new();
    for direction in adjacent {
        let mut new_pos = caster_position;
        let offset = direction.as_offset();
        new_pos.shift(offset.0, offset.1);
        output.push(new_pos);
    }
    magic_vfx.send(PlaceMagicVfx {
        targets: output.clone(),
        sequence: EffectSequence::Sequential { duration: 0.04 },
        effect: EffectType::RedBlast,
        decay: 0.5,
        appear: 0.,
    });
    synapse_data.targets.append(&mut output);
}

/// The targeted creatures dash in the direction of the caster's last move.
fn axiom_function_dash(
    mut teleport: EventWriter<TeleportEntity>,
    map: Res<Map>,
    spell_stack: Res<SpellStack>,
    momentum: Query<&OrdDir>,
    is_spellproof: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.last().unwrap();
    let caster_momentum = momentum.get(synapse_data.caster).unwrap();
    if let Axiom::Dash { max_distance } = synapse_data.axioms[synapse_data.step] {
        // For each (Entity, Position) on a targeted tile with a creature on it...
        for (dasher, dasher_pos) in synapse_data.get_all_targeted_entity_pos_pairs(&map) {
            // Spellproof entities cannot be affected.
            if is_spellproof.get(dasher).unwrap() {
                continue;
            }
            // The dashing creature starts where it currently is standing.
            let mut final_dash_destination = dasher_pos;
            // It will travel in the direction of the caster's last move.
            let (off_x, off_y) = caster_momentum.as_offset();
            // The dash has a maximum travel distance of `max_distance`.
            let mut distance_travelled = 0;
            while distance_travelled < max_distance {
                distance_travelled += 1;
                // Stop dashing if a solid Creature is hit (not implemented: "and the dasher is not intangible").
                if !map.is_passable(
                    final_dash_destination.x + off_x,
                    final_dash_destination.y + off_y,
                ) {
                    break;
                }
                // Otherwise, keep offsetting the dashing creature's position.
                final_dash_destination.shift(off_x, off_y);
            }

            // Once finished, release the Teleport event.
            teleport.send(TeleportEntity {
                destination: final_dash_destination,
                entity: dasher,
            });
        }
    } else {
        // This should NEVER trigger. This system was chosen to run because the
        // next axiom in the SpellStack explicitly requested it by being an Axiom::Dash.
        panic!()
    }
}

/// Fire a beam from the caster, towards the caster's last move. Target all travelled tiles,
/// including the first solid tile encountered, which stops the beam.
fn axiom_form_momentum_beam(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    map: Res<Map>,
    mut spell_stack: ResMut<SpellStack>,
    position_and_momentum: Query<(&Position, &OrdDir)>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let (caster_position, caster_momentum) =
        position_and_momentum.get(synapse_data.caster).unwrap();
    // Start the beam where the caster is standing.
    // The beam travels in the direction of the caster's last move.
    let (off_x, off_y) = caster_momentum.as_offset();
    let mut output = linear_beam(*caster_position, 10, off_x, off_y, &map);
    // Add some visual beam effects.
    magic_vfx.send(PlaceMagicVfx {
        targets: output.clone(),
        sequence: EffectSequence::Sequential { duration: 0.04 },
        effect: match caster_momentum {
            OrdDir::Up | OrdDir::Down => EffectType::VerticalBeam,
            OrdDir::Right | OrdDir::Left => EffectType::HorizontalBeam,
        },
        decay: 0.5,
        appear: 0.,
    });
    // Add these tiles to `targets`.
    synapse_data.targets.append(&mut output);
}

/// Target the tile adjacent to the caster, towards the caster's last move.
fn axiom_form_touch(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position_and_momentum: Query<(&Position, &OrdDir)>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let (caster_position, caster_momentum) =
        position_and_momentum.get(synapse_data.caster).unwrap();
    let (off_x, off_y) = caster_momentum.as_offset();
    let touch = Position::new(caster_position.x + off_x, caster_position.y + off_y);
    synapse_data.targets.push(touch);
    magic_vfx.send(PlaceMagicVfx {
        targets: vec![touch],
        sequence: EffectSequence::Sequential { duration: 0.04 },
        effect: EffectType::RedBlast,
        decay: 0.5,
        appear: 0.,
    });
}

/// Target a ring of `radius` around the caster.
fn axiom_form_halo(
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
    mut spell_stack: ResMut<SpellStack>,
    position: Query<&Position>,
) {
    let synapse_data = spell_stack.spells.last_mut().unwrap();
    let caster_position = position.get(synapse_data.caster).unwrap();
    if let Axiom::Halo { radius } = synapse_data.axioms[synapse_data.step] {
        let mut circle = circle_around(caster_position, radius);
        // Sort by clockwise rotation.
        circle.sort_by(|a, b| {
            let angle_a = angle_from_center(caster_position, a);
            let angle_b = angle_from_center(caster_position, b);
            angle_a.partial_cmp(&angle_b).unwrap()
        });
        // Add some visual halo effects.
        magic_vfx.send(PlaceMagicVfx {
            targets: circle.clone(),
            sequence: EffectSequence::Sequential { duration: 0.04 },
            effect: EffectType::GreenBlast,
            decay: 0.5,
            appear: 0.,
        });
        // Add these tiles to `targets`.
        synapse_data.targets.append(&mut circle);
    } else {
        panic!()
    }
}

/// The targeted passable tiles summon a new instance of species.
fn axiom_function_summon_creature(
    mut summon: EventWriter<SummonCreature>,
    spell_stack: Res<SpellStack>,
    position: Query<&Position>,
) {
    let synapse_data = spell_stack.spells.last().unwrap();
    let caster_position = position.get(synapse_data.caster).unwrap();
    if let Axiom::SummonCreature { species } = synapse_data.axioms[synapse_data.step] {
        for position in &synapse_data.targets {
            summon.send(SummonCreature {
                species,
                position: *position,
                momentum: OrdDir::Down,
                summon_tile: *caster_position,
                summoner: Some(synapse_data.caster),
            });
        }
    } else {
        panic!()
    }
}

/// Any targeted creature with the Wall component is removed.
/// Each removed wall heals the caster +1.
fn axiom_function_devour_wall(
    mut remove: EventWriter<RemoveCreature>,
    mut heal: EventWriter<DamageOrHealCreature>,
    spell_stack: Res<SpellStack>,
    map: Res<Map>,
    wall_check: Query<(Has<Wall>, Has<Spellproof>)>,
) {
    let synapse_data = spell_stack.spells.last().unwrap();
    let mut total_heal: isize = 0;
    for entity in synapse_data.get_all_targeted_entities(&map) {
        let (is_wall, is_spellproof) = wall_check.get(entity).unwrap();
        if is_wall && !is_spellproof {
            remove.send(RemoveCreature { entity });
            total_heal = total_heal.saturating_add(1);
        }
    }
    heal.send(DamageOrHealCreature {
        entity: synapse_data.caster,
        culprit: synapse_data.caster,
        hp_mod: total_heal,
    });
}

fn axiom_function_architect_cage(
    spell_stack: Res<SpellStack>,
    map: Res<Map>,
    wall_check: Query<Has<Wall>>,
    mut summon: EventWriter<SummonCreature>,
    position: Query<&Position>,
) {
    let synapse_data = spell_stack.spells.last().unwrap();
    let caster_position = position.get(synapse_data.caster).unwrap();
    // Get the caster's position.
    if let Some(cage_tile) = synapse_data.targets.last() {
        let mut possible_centers = Vec::new();
        for cage_offset_x in -3..=3 {
            for cage_offset_y in -3..=3 {
                possible_centers.push(Position::new(
                    cage_tile.x + cage_offset_x,
                    cage_tile.y + cage_offset_y,
                ));
            }
        }
        let mut rng = rand::thread_rng();
        possible_centers.shuffle(&mut rng);
        let mut chosen_center = None;
        let mut creatures_in_cage = Vec::new();
        for possible_center in possible_centers {
            let mut good_candidate = true;
            for cage_offset_x in -3..=3 {
                for cage_offset_y in -3..=3 {
                    if let Some(found_obstruction) = map.get_entity_at(
                        possible_center.x + cage_offset_x,
                        possible_center.y + cage_offset_y,
                    ) {
                        if wall_check.get(*found_obstruction).unwrap() {
                            good_candidate = false;
                        } else {
                            creatures_in_cage
                                .push(Position::new(4 + cage_offset_x, 4 + cage_offset_y));
                        }
                    }
                    if !good_candidate {
                        break;
                    }
                }
                if !good_candidate {
                    creatures_in_cage.clear();
                    break;
                }
            }
            if good_candidate {
                chosen_center = Some(possible_center);
                break;
            }
        }
        if let Some(chosen_center) = chosen_center {
            let cage = generate_room(creatures_in_cage);
            for (idx, tile_char) in cage.iter().enumerate() {
                let position = Position::new(
                    idx as i32 % 10 + chosen_center.x - 4,
                    idx as i32 / 10 + chosen_center.y - 4,
                );
                let species = match tile_char {
                    '#' => Species::Wall,
                    'H' => Species::Hunter,
                    'S' => Species::Spawner,
                    'T' => Species::Tinker,
                    '@' => Species::Player,
                    'W' => Species::WeakWall,
                    '2' => Species::Second,
                    'A' => Species::Apiarist,
                    'F' => Species::Shrike,
                    '^' | '>' | '<' | 'V' => Species::Airlock,
                    _ => continue,
                };
                let momentum = match tile_char {
                    '^' => OrdDir::Up,
                    '>' => OrdDir::Right,
                    '<' => OrdDir::Left,
                    'V' | _ => OrdDir::Down,
                };
                summon.send(SummonCreature {
                    species,
                    position,
                    momentum,
                    summon_tile: *caster_position,
                    summoner: Some(synapse_data.caster),
                });
            }
        }
    }
}

/// All creatures summoned by targeted creatures are removed.
fn axiom_function_abjuration(
    mut remove: EventWriter<RemoveCreature>,
    spell_stack: Res<SpellStack>,
    map: Res<Map>,
    summons: Query<(Entity, &Summoned)>,
    is_spellproof: Query<Has<Spellproof>>,
) {
    let synapse_data = spell_stack.spells.last().unwrap();
    for entity in synapse_data.get_all_targeted_entities(&map) {
        // Spellproof entities cannot be affected.
        if is_spellproof.get(entity).unwrap() {
            continue;
        }
        for (summoned_entity, summon) in summons.iter() {
            if summon.summoner == entity {
                remove.send(RemoveCreature {
                    entity: summoned_entity,
                });
            }
        }
    }
}

fn linear_beam(
    mut start: Position,
    max_distance: usize,
    off_x: i32,
    off_y: i32,
    map: &Map,
) -> Vec<Position> {
    let mut distance_travelled = 0;
    let mut output = Vec::new();
    // The beam has a maximum distance of max_distance.
    while distance_travelled < max_distance {
        distance_travelled += 1;
        start.shift(off_x, off_y);
        // The new tile is always added, even if it is impassable...
        output.push(start);
        // But if it is impassable, the beam stops.
        if !map.is_passable(start.x, start.y) {
            break;
        }
    }
    output
}

/// Generate the points across the outline of a circle.
fn circle_around(center: &Position, radius: i32) -> Vec<Position> {
    let mut circle = Vec::new();
    for r in 0..=(radius as f32 * (0.5f32).sqrt()).floor() as i32 {
        let d = (((radius * radius - r * r) as f32).sqrt()).floor() as i32;
        let adds = [
            Position::new(center.x - d, center.y + r),
            Position::new(center.x + d, center.y + r),
            Position::new(center.x - d, center.y - r),
            Position::new(center.x + d, center.y - r),
            Position::new(center.x + r, center.y - d),
            Position::new(center.x + r, center.y + d),
            Position::new(center.x - r, center.y - d),
            Position::new(center.x - r, center.y + d),
        ];
        for new_add in adds {
            if !circle.contains(&new_add) {
                circle.push(new_add);
            }
        }
    }
    circle
}

/// Find the angle of a point on a circle relative to its center.
fn angle_from_center(center: &Position, point: &Position) -> f64 {
    let delta_x = point.x - center.x;
    let delta_y = point.y - center.y;
    (delta_y as f64).atan2(delta_x as f64)
}

fn cleanup_last_axiom(mut spell_stack: ResMut<SpellStack>) {
    // Get the currently executed spell, removing it temporarily.
    let mut synapse_data = spell_stack.spells.pop().unwrap();
    // Step forwards in the axiom queue.
    synapse_data.step += 1;
    // If the spell is finished, do not push it back.
    if synapse_data.axioms.get(synapse_data.step).is_some() {
        spell_stack.spells.push(synapse_data);
    }
}

pub fn spell_stack_is_empty(spell_stack: Res<SpellStack>) -> bool {
    spell_stack.spells.is_empty()
}

use rand::{seq::SliceRandom, Rng};

const SIZE: usize = 9;

fn generate_room(creatures_in_cage: Vec<Position>) -> Vec<char> {
    let mut grid = vec![vec!['#'; SIZE]; SIZE];
    let mut rng = rand::thread_rng();

    // Set the X markers
    grid[0][4] = '^';
    grid[4][0] = '<';
    grid[4][8] = '>';
    grid[8][4] = 'V';

    // Create a variable central area
    create_variable_center(&mut grid, &mut rng);

    // Connect X markers to the center
    connect_to_center(&mut grid, 0, 4, 4, 4);
    connect_to_center(&mut grid, 4, 0, 4, 4);
    connect_to_center(&mut grid, 4, 8, 4, 4);
    connect_to_center(&mut grid, 8, 4, 4, 4);
    for creature_tile in creatures_in_cage {
        connect_to_center(
            &mut grid,
            creature_tile.x.try_into().unwrap(),
            creature_tile.y.try_into().unwrap(),
            4,
            4,
        );
    }

    // Add random paths
    for _ in 0..rng.gen_range(10..20) {
        let x = rng.gen_range(1..8);
        let y = rng.gen_range(1..8);
        if grid[x][y] == '#' {
            grid[x][y] = '.';
            // Expand the path
            for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx > 0 && nx < SIZE as i32 - 1 && ny > 0 && ny < SIZE as i32 - 1 {
                    if rng.gen_bool(0.5) {
                        grid[nx as usize][ny as usize] = '.';
                    }
                }
            }
        }
    }

    // Ensure all floor tiles are connected
    ensure_connectivity(&mut grid);

    // Replace inner walls with 'W'
    replace_inner_walls(&mut grid);

    // Place '@' and 'H' on random floor tiles
    place_special_tiles(&mut grid, &mut rng);

    grid.into_iter()
        .flat_map(|row| row.into_iter().chain(std::iter::once('\n')))
        .collect()
}

fn connect_to_center(
    grid: &mut Vec<Vec<char>>,
    start_x: usize,
    start_y: usize,
    end_x: usize,
    end_y: usize,
) {
    let mut x = start_x;
    let mut y = start_y;
    while x != end_x || y != end_y {
        grid[x][y] = '.';
        if x < end_x {
            x += 1;
        } else if x > end_x {
            x -= 1;
        } else if y < end_y {
            y += 1;
        } else if y > end_y {
            y -= 1;
        }
    }
}

fn ensure_connectivity(grid: &mut Vec<Vec<char>>) {
    let mut visited = vec![vec![false; SIZE]; SIZE];
    let mut stack = vec![];

    // Find the first floor tile
    'outer: for i in 0..SIZE {
        for j in 0..SIZE {
            if grid[i][j] == '.' {
                stack.push((i, j));
                visited[i][j] = true;
                break 'outer;
            }
        }
    }

    // DFS to mark all connected tiles
    while let Some((x, y)) = stack.pop() {
        for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < SIZE as i32 && ny >= 0 && ny < SIZE as i32 {
                let nx = nx as usize;
                let ny = ny as usize;
                if (grid[nx][ny] == '.') && !visited[nx][ny] {
                    visited[nx][ny] = true;
                    stack.push((nx, ny));
                }
            }
        }
    }

    // Connect any unvisited floor tiles
    for i in 0..SIZE {
        for j in 0..SIZE {
            if (grid[i][j] == '.') && !visited[i][j] {
                connect_to_nearest_visited(grid, &visited, i, j);
            }
        }
    }
}

fn connect_to_nearest_visited(
    grid: &mut Vec<Vec<char>>,
    visited: &Vec<Vec<bool>>,
    x: usize,
    y: usize,
) {
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((x, y, Vec::<(usize, usize)>::new()));
    let mut seen = vec![vec![false; SIZE]; SIZE];
    seen[x][y] = true;

    while let Some((cx, cy, path)) = queue.pop_front() {
        if visited[cx][cy] {
            for &(px, py) in &path {
                grid[px][py] = '.';
            }
            return;
        }

        for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx >= 0 && nx < SIZE as i32 && ny >= 0 && ny < SIZE as i32 {
                let nx = nx as usize;
                let ny = ny as usize;
                if !seen[nx][ny] {
                    seen[nx][ny] = true;
                    let mut new_path = path.clone();
                    new_path.push((nx, ny));
                    queue.push_back((nx, ny, new_path));
                }
            }
        }
    }
}

fn create_variable_center(grid: &mut Vec<Vec<char>>, rng: &mut impl Rng) {
    // Start with a clear 5x5 center
    for i in 4..5 {
        for j in 4..5 {
            grid[i][j] = '.';
        }
    }

    // Randomly add some walls in the center
    for _ in 0..rng.gen_range(1..9) {
        let x = rng.gen_range(2..7);
        let y = rng.gen_range(2..7);
        grid[x][y] = '#';
    }

    // Ensure the center remains connected
    let mut center_grid = grid[2..7]
        .iter()
        .map(|row| row[2..7].to_vec())
        .collect::<Vec<_>>();
    ensure_connectivity_subgrid(&mut center_grid);
    for i in 0..5 {
        for j in 0..5 {
            grid[i + 2][j + 2] = center_grid[i][j];
        }
    }
}

fn ensure_connectivity_subgrid(grid: &mut Vec<Vec<char>>) {
    let size = grid.len();
    let mut visited = vec![vec![false; size]; size];
    let mut stack = vec![];

    // Find the first floor tile
    'outer: for i in 0..size {
        for j in 0..size {
            if grid[i][j] == '.' {
                stack.push((i, j));
                visited[i][j] = true;
                break 'outer;
            }
        }
    }

    // DFS to mark all connected tiles
    while let Some((x, y)) = stack.pop() {
        for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < size as i32 && ny >= 0 && ny < size as i32 {
                let nx = nx as usize;
                let ny = ny as usize;
                if grid[nx][ny] == '.' && !visited[nx][ny] {
                    visited[nx][ny] = true;
                    stack.push((nx, ny));
                }
            }
        }
    }

    // Connect any unvisited floor tiles
    for i in 0..size {
        for j in 0..size {
            if grid[i][j] == '.' && !visited[i][j] {
                grid[i][j] = '#';
            }
        }
    }
}

fn replace_inner_walls(grid: &mut Vec<Vec<char>>) {
    for i in 1..SIZE - 1 {
        for j in 1..SIZE - 1 {
            if grid[i][j] == '#' {
                grid[i][j] = 'W';
            }
        }
    }
}

fn place_special_tiles(grid: &mut Vec<Vec<char>>, rng: &mut impl Rng) {
    let mut floor_tiles: Vec<(usize, usize)> = Vec::new();

    grid[0][4] = 'V';
    grid[4][0] = '<';
    grid[4][8] = '>';
    grid[8][4] = '^';

    for i in 0..SIZE {
        for j in 0..SIZE {
            if grid[i][j] == '.' {
                floor_tiles.push((i, j));
            }
        }
    }

    if floor_tiles.len() >= 2 {
        floor_tiles.shuffle(rng);
        let (x, y) = floor_tiles[0];
        grid[x][y] = *['A', 'T', 'F', '2', 'H'].choose(rng).unwrap();
        let (x, y) = floor_tiles[1];
        grid[x][y] = *['A', 'T', 'F', '2', 'H'].choose(rng).unwrap();
    }
}
