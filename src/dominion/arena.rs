use crate::dominion::kingdom::KingdomSet;
use crate::dominion::supply::Supply;

#[derive(Debug)]
pub struct Arena {
    players: Vec<usize>,
    supply: Supply,
}

impl Arena {
    pub fn new(kingdom_set: KingdomSet, num_players: usize) -> Arena {
        Arena {
            players: vec![0; num_players],
            supply: Supply::new(kingdom_set.cards(), num_players),
        }
    }

    pub fn supply(&self) -> &Supply {
        &self.supply
    }
}


