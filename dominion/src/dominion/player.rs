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

pub type CardVec = Vec<CardKind>;

#[derive(Debug)]
pub struct Player {
    pub deck_pile: CardVec,
    pub hand: CardVec,
    pub play_zone: CardVec,
    pub discard_pile: CardVec,
}

impl Player {
    pub fn new() -> Player {
        let mut deck_pile = vec![CardKind::Copper; 7];
        deck_pile.append(&mut vec![CardKind::Estate; 3]);

        let mut p = Player {
            deck_pile,
            hand: CardVec::new(),
            play_zone: CardVec::new(),
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

    pub fn draw_card(&mut self) {
        if self.deck_pile.is_empty() {
            self.deck_pile.append(&mut self.discard_pile);
            self.shuffle_deck();
        }

        // TODO handle empty deck
        self.hand.push(self.deck_pile.remove(0));
    }

    pub fn cleanup(&mut self) {
        self.discard_pile.append(&mut self.play_zone);
        self.discard_pile.append(&mut self.hand);

        for _ in 0..5 {
            self.draw_card();
        }
    }

    pub fn in_deck(&self, card: CardKind) -> bool {
        self.deck_pile
            .iter()
            .chain(self.hand.iter())
            .chain(self.play_zone.iter())
            .chain(self.discard_pile.iter())
            .any(|&x| x == card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_card_no_shuffle() {
        let mut p = Player::new();

        p.deck_pile.clear();
        assert!(p.deck_pile.is_empty());

        p.deck_pile.push(CardKind::Copper);
        p.deck_pile.push(CardKind::Silver);

        p.draw_card();

        assert_eq!(p.deck_pile, vec![CardKind::Silver]);
        assert_eq!(p.hand, vec![CardKind::Copper]);
    }

    #[test]
    fn test_draw_card_shuffle() {
        let mut p = Player::new();

        p.deck_pile.clear();
        assert!(p.deck_pile.is_empty());

        for _ in 0..5 {
            p.discard_pile.push(CardKind::Copper);
        }

        p.draw_card();

        assert_eq!(p.deck_pile, vec![CardKind::Copper; 4]);
        assert_eq!(p.hand, vec![CardKind::Copper]);
    }

    #[test]
    fn test_cleanup() {
        let mut p = Player::new();

        p.deck_pile.clear();
        assert!(p.deck_pile.is_empty());

        for _ in 0..5 {
            p.deck_pile.push(CardKind::Copper);
        }
        p.play_zone.push(CardKind::Silver);
        p.hand.push(CardKind::Gold);

        p.cleanup();

        assert!(p
            .discard_pile
            .iter()
            .find(|&&x| x == CardKind::Silver)
            .is_some());
        assert!(p
            .discard_pile
            .iter()
            .find(|&&x| x == CardKind::Gold)
            .is_some());
        assert_eq!(p.discard_pile.len(), 2);
        assert_eq!(p.hand, vec![CardKind::Copper; 5]);
        assert!(p.deck_pile.is_empty());
    }

    #[test]
    fn test_card_in_deck() {
        let p = Player::new();

        assert!(p.in_deck(CardKind::Copper));
        assert!(!p.in_deck(CardKind::Gold));
    }
}
