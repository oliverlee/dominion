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
    pub played: CardVec,
    pub discard_pile: CardVec,
}

impl Player {
    pub fn new() -> Player {
        let mut deck_pile = vec![CardKind::Copper; 7];
        deck_pile.append(&mut vec![CardKind::Estate; 3]);

        let mut p = Player {
            deck_pile,
            hand: CardVec::new(),
            played: CardVec::new(),
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
        self.discard_pile.append(&mut self.played);
        self.discard_pile.append(&mut self.hand);

        for _ in 0..5 {
            self.draw_card();
        }
    }

    pub fn in_deck(&self, card: CardKind) -> bool {
        self.deck_pile
            .iter()
            .chain(self.hand.iter())
            .chain(self.played.iter())
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
        p.played.push(CardKind::Silver);
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

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::dominion::KingdomSet;
//
//    fn create_player() -> Player {
//        let supply = Rc::new(RefCell::new(Supply::new(KingdomSet::FirstGame.cards(), 2)));
//
//        Player::new(supply)
//    }
//
//    #[test]
//    fn buy_card_copper() {
//        let mut p = create_player();
//
//        p.phase = Some(TurnPhase::Buy {
//            remaining_buys: 1,
//            total_wealth: 0,
//        });
//
//        let copper_supply = p
//            .supply
//            .borrow_mut()
//            .get_mut(&CardKind::Copper)
//            .unwrap()
//            .to_owned();
//
//        let r = p.buy_card(&CardKind::Copper);
//
//        assert!(r.is_ok());
//        assert_eq!(p.discard_pile[0], &CardKind::Copper);
//        assert_eq!(
//            p.supply
//                .borrow_mut()
//                .get_mut(&CardKind::Copper)
//                .unwrap()
//                .to_owned(),
//            copper_supply - 1
//        );
//    }
//
//    #[test]
//    fn buy_card_market() {
//        let mut p = create_player();
//
//        p.phase = Some(TurnPhase::Buy {
//            remaining_buys: 1,
//            total_wealth: 5,
//        });
//
//        let r = p.buy_card(&CardKind::Market);
//
//        assert!(r.is_ok());
//        assert_eq!(p.discard_pile[0], &CardKind::Market);
//        assert_eq!(
//            p.phase.unwrap(),
//            TurnPhase::Buy {
//                remaining_buys: 0,
//                total_wealth: 0,
//            }
//        );
//    }
//
//    #[test]
//    fn buy_card_no_remaining_buys() {
//        let mut p = create_player();
//
//        let c = &CardKind::Gold;
//
//        p.phase = Some(TurnPhase::Buy {
//            remaining_buys: 0,
//            total_wealth: c.cost(),
//        });
//
//        let r = p.buy_card(c);
//
//        assert!(r.is_err());
//        assert_eq!(r.unwrap_err(), Error::NoMoreBuys);
//        assert!(p.discard_pile.is_empty());
//    }
//
//    #[test]
//    fn buy_card_not_enough_wealth() {
//        let mut p = create_player();
//
//        p.phase = Some(TurnPhase::Buy {
//            remaining_buys: 1,
//            total_wealth: 0,
//        });
//
//        let r = p.buy_card(&CardKind::Gold);
//
//        assert!(r.is_err());
//        assert_eq!(r.unwrap_err(), Error::NotEnoughWealth);
//        assert!(p.discard_pile.is_empty());
//    }
//
//    #[test]
//    fn play_invalid_card() {
//        let mut p = create_player();
//
//        p.phase = Some(TurnPhase::Action {
//            remaining_actions: 1,
//            remaining_buys: 0,
//            total_wealth: 0,
//        });
//
//        let r = p.play_card(&CardKind::Copper);
//
//        assert!(r.is_err());
//        assert_eq!(r.unwrap_err(), Error::InvalidCardChoice);
//    }
//
//    #[test]
//    fn play_card_out_of_turn() {
//        let mut p = create_player();
//
//        p.hand.push(&CardKind::Gold);
//        let r = p.play_card(p.hand[0]);
//
//        assert!(r.is_err());
//        assert_eq!(p.hand.len(), 1);
//        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
//    }
//
//    #[test]
//    fn play_card_smithy_during_action_phase() {
//        let mut p = create_player();
//
//        p.phase = Some(TurnPhase::Action {
//            remaining_actions: 1,
//            remaining_buys: 0,
//            total_wealth: 0,
//        });
//
//        // Set hand to have a single card
//        p.hand.push(&CardKind::Smithy);
//        let r = p.play_card(p.hand[0]);
//
//        assert!(r.is_ok());
//        assert_eq!(p.hand.len(), 3);
//        assert_eq!(
//            p.phase.unwrap(),
//            TurnPhase::Action {
//                remaining_actions: 0,
//                remaining_buys: 0,
//                total_wealth: 0
//            }
//        );
//    }
//
//    #[test]
//    fn play_card_smithy_during_buy_phase() {
//        let mut p = create_player();
//
//        p.phase = Some(TurnPhase::Buy {
//            remaining_buys: 1,
//            total_wealth: 0,
//        });
//
//        // Set hand to have a single card
//        p.hand.push(&CardKind::Smithy);
//        let r = p.play_card(p.hand[0]);
//
//        assert!(r.is_err());
//        assert_eq!(p.hand.len(), 1);
//        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
//    }
//
//    #[test]
//    fn play_card_gold_during_action_phase() {
//        let mut p = create_player();
//
//        p.phase = Some(TurnPhase::Action {
//            remaining_actions: 1,
//            remaining_buys: 0,
//            total_wealth: 0,
//        });
//
//        // Set hand to have a single card
//        p.hand.push(&CardKind::Gold);
//        let r = p.play_card(p.hand[0]);
//
//        assert!(r.is_err());
//        assert_eq!(p.hand.len(), 1);
//        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
//    }
//
//    #[test]
//    fn play_card_gold_during_buy_phase() {
//        let mut p = create_player();
//
//        p.phase = Some(TurnPhase::Buy {
//            remaining_buys: 1,
//            total_wealth: 0,
//        });
//
//        // Set hand to have a single card
//        p.hand.push(&CardKind::Gold);
//        let r = p.play_card(p.hand[0]);
//
//        assert!(r.is_ok());
//        assert_eq!(p.hand.len(), 0);
//        assert_eq!(
//            p.phase.unwrap(),
//            TurnPhase::Buy {
//                remaining_buys: 1,
//                total_wealth: 3
//            }
//        );
//    }
//}
