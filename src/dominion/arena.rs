use crate::dominion::KingdomSet;
use crate::dominion::Player;
use crate::dominion::Supply;

#[derive(Debug)]
pub struct Arena {
    players: Vec<Player>,
    supply: Supply,
}

impl Arena {
    pub fn new(kingdom_set: KingdomSet, num_players: usize) -> Arena {
        Arena {
            players: (0..num_players).map(|_| Player::new()).collect(),
            supply: Supply::new(kingdom_set.cards(), num_players),
        }
    }

    pub fn supply(&self) -> &Supply {
        &self.supply
    }

    pub fn players(&mut self) -> &mut Vec<Player> {
        &mut self.players
    }
}
