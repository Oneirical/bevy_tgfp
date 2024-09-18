use bevy::{prelude::*, utils::HashMap};
use rand::seq::{IteratorRandom, SliceRandom};
use rand::Rng;

use crate::map::Position;

/// Marker for the player
#[derive(Component)]
pub struct Player;

#[derive(Component, Clone)]
pub struct Ipseity {
    active: HashMap<Soul, usize>,
    forefront: [Option<Soul>; 4],
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
            forefront: [None, None, None, None],
            repressed: HashMap::new(),
        }
    }

    /// Get `amount` souls from a creature. First, it goes to draw in `active`,
    /// if there is nothing left there, it draws from `forefront`, and if there
    /// is still nothing there, it returns something indicating that this creature
    /// can no longer take damage (basically 0 HP).
    pub fn harvest_random_souls(&mut self, mut amount: usize) -> DamageResult {
        let mut rng = rand::thread_rng();
        let available_souls_to_drain: Vec<Soul> = self
            .active
            .iter()
            .filter(|&(_, &value)| value > 0)
            .map(|(key, _)| *key)
            .choose_multiple(&mut rng, amount);

        for k in &available_souls_to_drain {
            let quantity = self.active.get_mut(k).unwrap();
            *quantity -= 1;
        }

        // If no active keys are available, use forefront
        while available_souls_to_drain.len() < amount {
            let forefront_soul_count = self.forefront.iter().filter_map(|x| *x).count();

            if forefront_soul_count == 0 {
                // This creature could not fully tank the damage.
                return DamageResult::Drained;
            }

            while forefront_soul_count > 0 {
                let selected = rng.gen_range(0..self.forefront.len());
                if let Some(soul) = self.forefront[selected] {
                    self.forefront[selected].take();
                    self.repress_soul(soul);
                    amount -= 1;
                    break;
                }
            }
        }

        DamageResult::Survived
    }

    fn repress_soul(&mut self, repressed_soul: Soul) {
        if let Some(repressed_quantity) = self.repressed.get_mut(&repressed_soul) {
            *repressed_quantity += 1;
        } else {
            self.repressed.insert(repressed_soul, 1);
        }
    }
}

pub enum DamageResult {
    Survived,
    Drained,
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
