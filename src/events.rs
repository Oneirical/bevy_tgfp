use bevy::prelude::*;

use crate::{
    creature::{get_species_sprite, Creature, Hunt, Player, Species},
    graphics::{SlideAnimation, SpriteSheetAtlas},
    map::{Map, Position},
    spells::{Axiom, CastSpell, Spell},
    OrdDir,
};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SummonCreature>();
        app.add_event::<EndTurn>();
        app.add_event::<TeleportEntity>();
        app.init_resource::<Events<CreatureStep>>();
        app.insert_resource(TurnCount { turns: 0 });
    }
}

#[derive(Resource)]
pub struct TurnCount {
    turns: usize,
}

#[derive(Event)]
pub struct SummonCreature {
    pub position: Position,
    pub species: Species,
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
            sprite: Sprite {
                image: asset_server.load("spritesheet.png"),
                custom_size: Some(Vec2::new(64., 64.)),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.handle.clone(),
                    index: get_species_sprite(&event.species),
                }),
                ..default()
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
            _ => (),
        }
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

pub fn teleport_entity(
    mut events: EventReader<TeleportEntity>,
    mut creature: Query<&mut Position>,
    mut map: ResMut<Map>,
    mut commands: Commands,
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
            // ...and move that Entity to TeleportEntity's destination tile.
            creature_position.update(event.destination.x, event.destination.y);
            // Also, animate this creature, making its teleport action visible on the screen.
            commands.entity(event.entity).insert(SlideAnimation);
        } else {
            // Nothing here just yet, but this is where collisions between creatures
            // will be handled.
            continue;
        }
    }
}

#[derive(Event)]
pub struct EndTurn;

pub fn end_turn(
    mut events: EventReader<EndTurn>,
    mut step: EventWriter<CreatureStep>,
    mut spell: EventWriter<CastSpell>,
    mut turn_count: ResMut<TurnCount>,
    player: Query<&Position, With<Player>>,
    hunters: Query<(Entity, &Position), (With<Hunt>, Without<Player>)>,
    map: Res<Map>,
) {
    for _event in events.read() {
        turn_count.turns += 1;
        let player_pos = player.get_single().unwrap();
        for (hunter_entity, hunter_pos) in hunters.iter() {
            // Occasionally cast a spell.
            if turn_count.turns % 5 == 0 {
                spell.send(CastSpell {
                    caster: hunter_entity,
                    spell: Spell {
                        axioms: vec![Axiom::MomentumBeam, Axiom::Dash { max_distance: 5 }],
                    },
                });
            }
            // Try to find a tile that gets the hunter closer to the player.
            else if let Some(move_direction) = map.best_manhattan_move(*hunter_pos, *player_pos) {
                // If it is found, cause a CreatureStep event.

                step.send(CreatureStep {
                    direction: move_direction,
                    entity: hunter_entity,
                });
            }
        }
    }
}
