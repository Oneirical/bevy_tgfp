+++
title = "Bevy Traditional Roguelike Quick-Start - 7. Peace Was Never An Option"
date = 2024-12-11
authors = ["Julien Robert"]
[taxonomies]
tags = ["rust", "bevy", "tutorial"]
+++



```rust
#[derive(Event)]
pub struct RemoveCreature {
    entity: Entity,
}

pub fn remove_creature(
    mut events: EventReader<RemoveCreature>,
    mut commands: Commands,
    mut map: ResMut<Map>,
    creature: Query<(&Position, Has<Player>)>,
    mut spell_stack: ResMut<SpellStack>,
    mut magic_vfx: EventWriter<PlaceMagicVfx>,
) {
    for event in events.read() {
        let (position, is_player) = creature.get(event.entity).unwrap();
        magic_vfx.send(PlaceMagicVfx {
            targets: vec![*position],
            sequence: EffectSequence::Simultaneous,
            effect: EffectType::XCross,
            decay: 0.5,
            appear: 0.,
        });
        if !is_player {
            map.creatures.remove(position);
            commands.entity(event.entity).despawn();
            spell_stack
                .spells
                .retain(|spell| spell.caster != event.entity);
        }
    }
}
```

```rust
#[derive(Event)]
pub struct HarmCreature {
    entity: Entity,
    culprit: Entity,
    damage: usize,
}

pub fn harm_creature(
    mut events: EventReader<HarmCreature>,
    mut remove: EventWriter<RemoveCreature>,
    mut creature: Query<&mut Health>,
) {
    for event in events.read() {
        let mut health = creature.get_mut(event.entity).unwrap();
        health.hp = health.hp.saturating_sub(event.damage);
        if health.hp == 0 {
            remove.send(RemoveCreature {
                entity: event.entity,
            });
        }
    }
}
```

```rust
            commands.entity(event.entity).despawn_recursive();
```
