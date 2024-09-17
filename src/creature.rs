use bevy::{prelude::*, utils::HashMap};
// use rand::seq::IteratorRandom;

use crate::map::Position;

/// Marker for the player
#[derive(Component)]
pub struct Player;

#[derive(Component, Clone)]
pub struct Ipseity {
    active: HashMap<Soul, usize>,
    forefront: (Option<Soul>, Option<Soul>, Option<Soul>, Option<Soul>),
    repressed: HashMap<Soul, usize>,
}

impl Ipseity {
    pub fn new(starter: &[(Soul, usize)]) -> Self {
        let mut active = HashMap::new();
        for (soul_type, amount) in starter.iter() {
            active.insert(*soul_type, *amount);
        }
        Self {
            active,
            forefront: (None, None, None, None),
            repressed: HashMap::new(),
        }
    }

    pub fn gather_active_keys(&self) -> Vec<(&Soul, &usize)> {
        let keys = self.active.keys();
        let mut output = Vec::new();
        for key in keys {
            output.push(self.active.get_key_value(key).unwrap());
        }
        output
    }

    // TODO: Pick X random souls from active (or forefront if it fails),
    // restricting to the ones with above 0 quantity.

    // pub fn get_random_active_souls(&mut self, mut amount: usize) {
    //     let mut output = Vec::new();
    //     while amount > 0 {
    //         let active_keys = self.gather_active_keys();

    //         let chosen_soul: Option<&(&Soul, &usize)> = active_keys
    //             .iter()
    //             .filter(|(_soul, &quantity)| quantity > 0)
    //             .choose(&mut rand::thread_rng());
    //         if let Some(chosen_soul_type) = chosen_soul {
    //             let (chosen_soul_type, _) = *chosen_soul_type;
    //             output.push(chosen_soul_type);
    //             amount -= 1;
    //             *self.active.get_mut(chosen_soul_type).unwrap() -= 1;
    //         }
    //     }
    // }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum Soul {
    Saintly,
    Ordered,
    Artistic,
    Unhinged,
    Feral,
    Vile,
    Immutable,
}

#[derive(Component)]
pub struct Hunt;

#[derive(Bundle)]
pub struct Creature {
    pub position: Position,
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
    pub ipseity: Ipseity,
}
