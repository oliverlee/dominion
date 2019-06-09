extern crate rand;

use crate::dominion::types::CardVec;
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

#[derive(Debug)]
pub(crate) struct Player {
    pub(crate) draw_pile: CardVec,
    pub(crate) hand: CardVec,
    pub(crate) play_zone: CardVec,
    pub(crate) stage: CardVec,
    pub(crate) discard_pile: CardVec,
}

impl Player {
    pub(crate) fn new() -> Player {
        let mut draw_pile = vec![CardKind::Copper; 7];
        draw_pile.append(&mut vec![CardKind::Estate; 3]);

        let mut p = Player {
            draw_pile,
            hand: CardVec::new(),
            play_zone: CardVec::new(),
            stage: CardVec::new(),
            discard_pile: CardVec::new(),
        };

        p.shuffle_deck();

        p
    }

    pub(crate) fn draw_card(&mut self) {
        if self.draw_pile.is_empty() {
            std::mem::swap(&mut self.draw_pile, &mut self.discard_pile);
            self.shuffle_deck();
        }

        // We consider the top of the draw pile to be the end that is popped.
        self.draw_pile.pop().map(|x| self.hand.push(x));
    }

    pub(crate) fn cleanup(&mut self) {
        self.discard_pile.append(&mut self.play_zone);
        self.discard_pile.append(&mut self.hand);

        for _ in 0..5 {
            self.draw_card();
        }
    }

    pub(crate) fn in_deck(&self, card: CardKind) -> bool {
        self.draw_pile
            .iter()
            .chain(self.hand.iter())
            .chain(self.play_zone.iter())
            .chain(self.discard_pile.iter())
            .any(|&x| x == card)
    }

    fn shuffle_deck(&mut self) {
        unsafe {
            self.draw_pile.shuffle(rng());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_card_no_shuffle() {
        let mut p = Player::new();

        p.draw_pile.clear();
        assert!(p.draw_pile.is_empty());

        p.draw_pile.push(CardKind::Silver);
        p.draw_pile.push(CardKind::Copper);

        p.draw_card();

        assert_eq!(p.draw_pile, vec![CardKind::Silver]);
        assert_eq!(p.hand, vec![CardKind::Copper]);
    }

    #[test]
    fn test_draw_card_shuffle() {
        let mut p = Player::new();

        p.draw_pile.clear();
        assert!(p.draw_pile.is_empty());

        for _ in 0..5 {
            p.discard_pile.push(CardKind::Copper);
        }

        p.draw_card();

        assert_eq!(p.draw_pile, vec![CardKind::Copper; 4]);
        assert_eq!(p.hand, vec![CardKind::Copper]);
    }

    #[test]
    fn test_draw_card_empty_draw_and_discard() {
        let mut p = Player::new();

        p.draw_pile.clear();
        p.discard_pile.clear();

        p.hand.push(CardKind::Copper);
        p.hand.push(CardKind::Copper);

        p.draw_card();
        assert_eq!(p.draw_pile, vec![]);
        assert_eq!(p.discard_pile, vec![]);
        assert_eq!(p.hand, vec![CardKind::Copper; 2]);
    }

    #[test]
    fn test_cleanup() {
        let mut p = Player::new();

        p.draw_pile.clear();
        assert!(p.draw_pile.is_empty());

        for _ in 0..5 {
            p.draw_pile.push(CardKind::Copper);
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
        assert!(p.draw_pile.is_empty());
    }

    #[test]
    fn test_card_in_deck() {
        let p = Player::new();

        assert!(p.in_deck(CardKind::Copper));
        assert!(!p.in_deck(CardKind::Gold));
    }
}
