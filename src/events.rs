use bevy::prelude::*;

use crate::{
    creature::{get_species_sprite, Creature, Hunt, Immutable, Intangible, Player, Species},
    graphics::{SlideAnimation, SpriteSheetAtlas},
    map::{register_creatures, Map, Position},
    spells::{dispatch_events, Axiom, CastSpell, Spell},
    OrdDir,
};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreatureStep>();
        app.add_event::<TeleportEntity>();
        app.add_event::<AlterMomentum>();
        app.add_event::<SummonCreature>();
        app.init_resource::<Events<EndTurn>>();
        app.add_event::<CreatureCollision>();
        app.add_systems(FixedUpdate, creature_step.before(alter_momentum));
        app.add_systems(FixedUpdate, alter_momentum.after(creature_step));
        // Spell effects go next.
        app.add_systems(FixedUpdate, creature_collision.after(dispatch_events));
        app.add_systems(FixedUpdate, summon_creature.after(creature_collision));
        // Newly summoned creatures are registered by `register_creatures`.
        app.add_systems(FixedUpdate, teleport_entity.after(register_creatures));
        app.add_systems(FixedUpdate, end_turn.after(teleport_entity));
    }
}

#[derive(Event)]
pub struct CreatureStep {
    pub entity: Entity,
    pub direction: OrdDir,
}

fn creature_step(
    mut events: EventReader<CreatureStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut momentum: EventWriter<AlterMomentum>,
    mut turn_end: EventWriter<EndTurn>,
    creature: Query<(&Position, Has<Player>)>,
) {
    for event in events.read() {
        let (creature_pos, is_player) = creature.get(event.entity).unwrap();
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportEntity::new(
            event.entity,
            creature_pos.x + off_x,
            creature_pos.y + off_y,
        ));

        momentum.send(AlterMomentum {
            entity: event.entity,
            direction: event.direction,
        });
        if is_player {
            turn_end.send(EndTurn);
        }
    }
}

#[derive(Event)]
pub struct EndTurn;

fn end_turn(
    mut events: EventReader<EndTurn>,
    mut step: EventWriter<CreatureStep>,
    mut spell: EventWriter<CastSpell>,
    npcs: Query<(Entity, &Position, &Species), (Without<Player>, Without<Intangible>)>,
    player: Query<&Position, With<Player>>,
    map: Res<Map>,
    animation_timer: Res<SlideAnimation>,
    mut momentum: EventWriter<AlterMomentum>,
) {
    if !animation_timer.elapsed.finished() {
        return;
    }
    for _event in events.read() {
        let player_pos = player.get_single().unwrap();
        for (creature_entity, creature_position, creature_species) in npcs.iter() {
            match creature_species {
                Species::Hunter => {
                    // Try to find a tile that gets the hunter closer to the player.
                    if let Some(move_target) =
                        map.best_manhattan_move(*creature_position, *player_pos)
                    {
                        // If it is found, the hunter approaches the player by stepping.
                        step.send(CreatureStep {
                            direction: OrdDir::as_variant(
                                move_target.x - creature_position.x,
                                move_target.y - creature_position.y,
                            ),
                            entity: creature_entity,
                        });
                    }
                }
                Species::Spawner => {
                    // Find an empty tile around the spawner to place a Hunter in.
                    if let Some(empty_tile) =
                        map.best_manhattan_move(*creature_position, *player_pos)
                    {
                        let direction = OrdDir::as_variant(
                            empty_tile.x - creature_position.x,
                            empty_tile.y - creature_position.y,
                        );
                        // Make the spawner "face" that tile.
                        momentum.send(AlterMomentum {
                            entity: creature_entity,
                            direction,
                        });
                        spell.send(CastSpell {
                            caster: creature_entity,
                            spell: Spell {
                                axioms: vec![
                                    Axiom::Smooch,
                                    Axiom::SummonCreature {
                                        species: Species::Hunter,
                                    },
                                ],
                            },
                        });
                    }
                }
                _ => (),
            }
        }
    }
}

#[derive(Event)]
pub struct AlterMomentum {
    pub entity: Entity,
    pub direction: OrdDir,
}

pub fn alter_momentum(mut events: EventReader<AlterMomentum>, mut creature: Query<&mut OrdDir>) {
    for momentum_alteration in events.read() {
        *creature.get_mut(momentum_alteration.entity).unwrap() = momentum_alteration.direction;
    }
}

#[derive(Event)]
pub struct TeleportEntity {
    pub destination: Position,
    pub entity: Entity,
}

impl TeleportEntity {
    pub fn new(entity: Entity, x: i32, y: i32) -> Self {
        Self {
            destination: Position::new(x, y),
            entity,
        }
    }
}

fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut creature: Query<(&mut Position, Has<Intangible>, Has<Immutable>)>,
    mut map: ResMut<Map>,
    mut animation_timer: ResMut<SlideAnimation>,
) {
    for event in events.read() {
        let (mut creature_position, is_intangible, is_immutable) = creature
            // Get the Position of the Entity targeted by TeleportEntity.
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        if is_immutable {
            continue;
        }
        // If motion is possible... (destination tile is empty, or creature is intangible)
        if map.is_passable(event.destination.x, event.destination.y) || is_intangible {
            // ...update the Map to reflect this...
            map.move_creature(*creature_position, event.destination);
            // ...begin the sliding animation...
            animation_timer.elapsed.reset();
            // ...and move that Entity to TeleportEntity's destination tile.
            creature_position.update(event.destination.x, event.destination.y);
        } else {
            // Nothing here just yet, but this is where collisions between creatures
            // will be handled.
            continue;
        }
    }
}

#[derive(Event)]
pub struct SummonCreature {
    pub species: Species,
    pub position: Position,
}

pub fn summon_creature(
    mut commands: Commands,
    mut events: EventReader<SummonCreature>,
    asset_server: Res<AssetServer>,
    atlas_layout: Res<SpriteSheetAtlas>,
    map: Res<Map>,
) {
    for event in events.read() {
        // Avoid summoning if the tile is already occupied.
        if !map.is_passable(event.position.x, event.position.y) {
            continue;
        }
        let mut new_creature = commands.spawn(Creature {
            position: event.position,
            species: event.species,
            sprite: SpriteBundle {
                texture: asset_server.load("spritesheet.png"),
                transform: Transform::from_scale(Vec3::new(4., 4., 0.)),
                visibility: Visibility::Hidden,
                ..default()
            },
            atlas: TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index: get_species_sprite(&event.species),
            },
            momentum: OrdDir::Up,
        });
        // Add any species-specific components.
        match &event.species {
            Species::Player => {
                new_creature.insert(Player);
            }
            Species::Hunter => {
                new_creature.insert(Hunt);
            }
            Species::Wall => {
                new_creature.insert(Immutable);
            }
            _ => (),
        }
    }
}

#[derive(Event)]
pub struct CreatureCollision {
    pub attacker: Entity,
    pub defender: Entity,
    pub speed: usize,
}

fn creature_collision(
    mut events: EventReader<CreatureCollision>,
    mut removed_creature: Query<(Entity, &mut Sprite)>,
    mut map: ResMut<Map>,
    mut commands: Commands,
) {
    for event in events.read() {
        // The creature needs to have travelled at least one empty tile to actually be struck.
        if event.speed <= 1 {
            continue;
        }
        let (creature_entity, mut creature_sprite) =
            removed_creature.get_mut(event.defender).unwrap();
        // The creature becomes intangible.
        map.make_intangible(creature_entity);
        commands.entity(event.defender).insert(Intangible);
        creature_sprite.color.set_alpha(0.1);
    }
}
