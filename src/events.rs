use bevy::prelude::*;

use crate::{
    creature::{
        get_species_sprite, Creature, HealthBar, HealthPoint, Hunt, Intangible, Player, Species,
    },
    graphics::{AttackAnimation, HealthIndicator, SlideAnimation, SpriteSheetAtlas},
    map::{are_orthogonally_adjacent, Map, Position},
    spells::{Axiom, CastSpell, Spell},
    OrdDir,
};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreatureStep>();
        app.add_event::<TeleportEntity>();
        app.add_event::<SummonCreature>();
        app.add_event::<RepressionDamage>();
        app.add_event::<BecomeIntangible>();
        app.init_resource::<Events<EndTurn>>();
    }
}

#[derive(Event)]
pub struct CreatureStep {
    pub entity: Entity,
    pub direction: OrdDir,
}

pub fn creature_step(
    mut events: EventReader<CreatureStep>,
    mut teleporter: EventWriter<TeleportEntity>,
    mut turn_end: EventWriter<EndTurn>,
    mut creature: Query<(&Position, Has<Player>, &mut OrdDir)>,
) {
    for event in events.read() {
        let (creature_pos, is_player, mut creature_momentum) =
            creature.get_mut(event.entity).unwrap();
        let (off_x, off_y) = event.direction.as_offset();
        teleporter.send(TeleportEntity::new(
            event.entity,
            creature_pos.x + off_x,
            creature_pos.y + off_y,
        ));
        // Update the direction towards which this creature is facing.
        *creature_momentum = event.direction;
        // If this creature was the player, this will end the turn.
        if is_player {
            turn_end.send(EndTurn);
        }
    }
}

#[derive(Event)]
pub struct TeleportEntity {
    destination: Position,
    entity: Entity,
}

impl TeleportEntity {
    pub fn new(entity: Entity, x: i32, y: i32) -> Self {
        Self {
            destination: Position::new(x, y),
            entity,
        }
    }
}

pub fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut damage: EventWriter<RepressionDamage>,
    mut creature: Query<(&mut Position, Has<Intangible>)>,
    mut map: ResMut<Map>,
    mut commands: Commands,
) {
    for event in events.read() {
        let (mut creature_position, is_intangible) = creature
            // Get the Position of the Entity targeted by TeleportEntity.
            .get_mut(event.entity)
            .expect("A TeleportEntity was given an invalid entity");
        // If motion is possible...
        if map.is_passable(event.destination.x, event.destination.y) || is_intangible {
            // ...update the Map to reflect this...
            map.move_creature(event.entity, *creature_position, event.destination);
            // ...begin the sliding animation...
            commands.entity(event.entity).insert(SlideAnimation {
                elapsed: Timer::from_seconds(0.2, TimerMode::Once),
            });
            // ...and move that Entity to TeleportEntity's destination tile.
            creature_position.update(event.destination.x, event.destination.y);
        } else {
            // A collision between two creatures occurs.
            if are_orthogonally_adjacent(*creature_position, event.destination) {
                damage.send(RepressionDamage {
                    entity: *map
                        .get_tangible_entity_at(event.destination.x, event.destination.y)
                        .unwrap(),
                    damage: 1,
                });
                commands.entity(event.entity).insert(AttackAnimation {
                    elapsed: Timer::from_seconds(0.2, TimerMode::Once),
                    direction: OrdDir::direction_towards_adjacent_tile(
                        *creature_position,
                        event.destination,
                    ),
                });
            }
            continue;
        }
    }
}

#[derive(Event)]
pub struct RepressionDamage {
    pub entity: Entity,
    pub damage: i32,
}

pub fn repression_damage(
    mut events: EventReader<RepressionDamage>,
    mut damaged_creature: Query<(&mut HealthBar, &Children)>,
    mut hp_bar: Query<(&mut Visibility, &mut TextureAtlas)>,
    mut intangible: EventWriter<BecomeIntangible>,
) {
    for event in events.read() {
        let (mut hp, children) = damaged_creature.get_mut(event.entity).unwrap();
        let is_fully_repressed = hp.repress(event.damage);
        if is_fully_repressed {
            intangible.send(BecomeIntangible {
                entity: event.entity,
            });
        }
        for child in children.iter() {
            let (mut hp_vis, mut hp_bar) = hp_bar.get_mut(*child).unwrap();
            let max_hp = hp.deck.len() + hp.repressed.len();
            let current_hp = hp.deck.len();
            if max_hp == current_hp || current_hp == 0 {
                *hp_vis = Visibility::Hidden;
            } else {
                *hp_vis = Visibility::Visible;
                match current_hp as f32 / max_hp as f32 {
                    0.85..1.00 => hp_bar.index = 168,
                    0.70..0.85 => hp_bar.index = 169,
                    0.55..0.70 => hp_bar.index = 170,
                    0.40..0.55 => hp_bar.index = 171,
                    0.25..0.40 => hp_bar.index = 172,
                    0.10..0.25 => hp_bar.index = 173,
                    0.00..0.10 => hp_bar.index = 174,
                    _ => panic!("That is not a possible HP %!"),
                }
            }
        }
    }
}

#[derive(Event)]
pub struct BecomeIntangible {
    pub entity: Entity,
}

// TODO: This should be a permanent status effect instead.
pub fn become_intangible(
    mut events: EventReader<BecomeIntangible>,
    mut creature: Query<&mut Sprite>,
    mut commands: Commands,
) {
    for event in events.read() {
        let mut sprite = creature.get_mut(event.entity).unwrap();
        sprite.color.set_alpha(0.1);
        commands.entity(event.entity).insert(Intangible);
    }
}

#[derive(Event)]
pub struct EndTurn;

pub fn end_turn(
    mut events: EventReader<EndTurn>,
    mut step: EventWriter<CreatureStep>,
    mut spell: EventWriter<CastSpell>,
    npcs: Query<(Entity, &Position, &Species), Without<Player>>,
    player: Query<&Position, With<Player>>,
    map: Res<Map>,
) {
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
                    // Cast a spell which tries to summon Hunters on all orthogonally
                    // adjacent tiles.
                    spell.send(CastSpell {
                        caster: creature_entity,
                        spell: Spell {
                            axioms: vec![
                                Axiom::Plus,
                                Axiom::SummonCreature {
                                    species: Species::Hunter,
                                },
                            ],
                        },
                    });
                }
                _ => (),
            }
        }
    }
}

#[derive(Event)]
pub struct SummonCreature {
    pub species: Species,
    pub position: Position,
}

/// Place a new Creature on the map of Species and at Position.
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
                ..default()
            },
            atlas: TextureAtlas {
                layout: atlas_layout.handle.clone(),
                index: get_species_sprite(&event.species),
            },
            momentum: OrdDir::Up,
            health: HealthBar::new(2),
        });
        // Add any species-specific components.
        match &event.species {
            Species::Wall => {
                new_creature.insert(HealthBar::new(200));
            }
            Species::Player => {
                new_creature.insert(Player);
                new_creature.insert(HealthBar::new(6));
                // new_creature.insert(Intangible);
                // Lower the Z value, so it appears underneath other creatures.
                let mut transform = Transform::from_scale(Vec3::new(4., 4., 0.));
                transform.translation.z = -1.;
                new_creature.insert(transform);
            }
            Species::Hunter => {
                new_creature.insert(Hunt);
            }
            Species::Spawner => {
                new_creature.insert(Intangible);
                // Lower the Z value, so it appears underneath other creatures.
                let mut transform = Transform::from_scale(Vec3::new(4., 4., 0.));
                transform.translation.z = -1.;
                new_creature.insert(transform);
            }
            _ => (),
        }
        // Free the borrow on Commands.
        let new_creature_entity = new_creature.id();
        let hp_bar = commands
            .spawn(HealthIndicator {
                sprite: SpriteBundle {
                    texture: asset_server.load("spritesheet.png"),
                    // It already inherits the increased scale from the parent.
                    transform: Transform::from_scale(Vec3::new(1., 1., 0.)),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                atlas: TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index: 168,
                },
            })
            .id();
        commands.entity(new_creature_entity).add_child(hp_bar);
    }
}
