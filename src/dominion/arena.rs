use crate::dominion::KingdomSet;
use crate::dominion::Player;
use crate::dominion::Supply;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

#[derive(Debug)]
pub struct Arena {
    supply: Rc<RefCell<Supply>>,
    players: Vec<Player>,
}

impl Arena {
    pub fn new(kingdom_set: KingdomSet, num_players: usize) -> Arena {
        let supply = Rc::new(RefCell::new(Supply::new(kingdom_set.cards(), num_players)));
        let players = (0..num_players)
            .map(|_| Player::new(supply.clone()))
            .collect();

        let mut arena = Arena { supply, players };

        arena.start_game();

        arena
    }

    pub fn supply(&self) -> Ref<Supply> {
        self.supply.borrow()
    }

    pub fn players(&mut self) -> &mut Vec<Player> {
        &mut self.players
    }

    fn start_game(&mut self) {
        for p in self.players() {
            p.cleanup();
        }
    }
}
