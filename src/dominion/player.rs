extern crate rand;

use crate::dominion::CardKind;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

static mut RNG: Option<StdRng> = None;

unsafe fn rng() -> &'static mut StdRng {
    if RNG.is_none() {
        RNG = Some(StdRng::seed_from_u64(1));
    }

    RNG.as_mut().unwrap()
}

type CardVec = Vec<&'static CardKind>;

#[derive(Debug)]
pub struct Player {
    deck_pile: CardVec,
    hand: CardVec,
    in_play: CardVec,
    discard_pile: CardVec,
}

impl Player {
    pub fn new() -> Player {
        let mut deck_pile = vec![&CardKind::Copper; 7];
        deck_pile.append(&mut vec![&CardKind::Estate; 3]);

        let mut p = Player {
            deck_pile,
            hand: CardVec::new(),
            in_play: CardVec::new(),
            discard_pile: CardVec::new(),
        };

        p.shuffle_deck();

        p
    }

    fn shuffle_deck(&mut self) {
        unsafe {
            self.deck_pile.shuffle(rng());
        }
    }

    fn draw_card(&mut self) {
        if self.deck_pile.is_empty() {
            self.deck_pile.append(&mut self.discard_pile);
            self.shuffle_deck();
        }

        self.hand.push(self.deck_pile.remove(0));
    }

    pub fn cleanup(&mut self) {
        self.discard_pile.append(&mut self.hand);

        for _ in 0..5 {
            self.draw_card();
        }
    }
}
