extern crate rand;

use crate::dominion::CardKind;
use crate::dominion::Supply;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::cell::RefCell;
use std::rc::Rc;

static mut RNG: Option<StdRng> = None;

unsafe fn rng() -> &'static mut StdRng {
    if RNG.is_none() {
        RNG = Some(StdRng::seed_from_u64(1));
    }

    RNG.as_mut().unwrap()
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    NoMoreActions,
    NoMoreBuys,
    NoMoreCards,
    NotEnoughWealth,
    InvalidCardChoice,
    WrongTurnPhase,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Eq, PartialEq)]
enum TurnPhase {
    Action {
        remaining_actions: i32,
        remaining_buys: i32,
        total_wealth: i32,
    },
    Buy {
        remaining_buys: i32,
        total_wealth: i32,
    },
}

pub type CardVec = Vec<&'static CardKind>;

#[derive(Debug)]
pub struct Player {
    pub deck_pile: CardVec,
    pub hand: CardVec,
    pub in_play: CardVec,
    pub discard_pile: CardVec,
    phase: Option<TurnPhase>,
    supply: Rc<RefCell<Supply>>,
}

impl Player {
    pub fn new(supply: Rc<RefCell<Supply>>) -> Player {
        let mut deck_pile = vec![&CardKind::Copper; 7];
        deck_pile.append(&mut vec![&CardKind::Estate; 3]);

        let mut p = Player {
            deck_pile,
            hand: CardVec::new(),
            in_play: CardVec::new(),
            discard_pile: CardVec::new(),
            phase: None,
            supply,
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

        // TODO handle empty deck
        self.hand.push(self.deck_pile.remove(0));
    }

    pub fn cleanup(&mut self) {
        self.discard_pile.append(&mut self.in_play);
        self.discard_pile.append(&mut self.hand);

        for _ in 0..5 {
            self.draw_card();
        }
    }

    pub fn start_turn(&mut self) -> Result<()> {
        if let None = self.phase {
            self.phase = Some(TurnPhase::Action {
                remaining_actions: 1,
                remaining_buys: 1,
                total_wealth: 0,
            });

            Ok(())
        } else {
            Err(Error::WrongTurnPhase)
        }
    }

    pub fn start_buy_phase(&mut self) -> Result<()> {
        if let Some(TurnPhase::Action {
            remaining_actions: _,
            remaining_buys,
            total_wealth,
        }) = self.phase
        {
            self.phase = Some(TurnPhase::Buy {
                remaining_buys,
                total_wealth,
            });

            Ok(())
        } else {
            Err(Error::WrongTurnPhase)
        }
    }

    pub fn end_turn(&mut self) -> Result<()> {
        if let Some(TurnPhase::Buy { .. }) = self.phase {
            self.phase = None;
            self.cleanup();

            Ok(())
        } else {
            Err(Error::WrongTurnPhase)
        }
    }

    pub fn buy_card(&mut self, card: &'static CardKind) -> Result<()> {
        if let Some(phase) = &mut self.phase {
            if let TurnPhase::Buy {
                remaining_buys,
                total_wealth,
            } = phase
            {
                let r = match (*self.supply.borrow_mut()).get_mut(card) {
                    Some(supply_count) => {
                        if *supply_count == 0 {
                            Err(Error::NoMoreCards)
                        } else if *remaining_buys == 0 {
                            Err(Error::NoMoreBuys)
                        } else if card.cost() > *total_wealth {
                            Err(Error::NotEnoughWealth)
                        } else {
                            *remaining_buys -= 1;
                            *total_wealth -= card.cost();

                            *supply_count -= 1;

                            self.discard_pile.push(card);

                            Ok(())
                        }
                    }
                    _ => Err(Error::InvalidCardChoice),
                };

                return r;
            }
        }

        Err(Error::WrongTurnPhase)
    }

    pub fn play_card(&mut self, card: &'static CardKind) -> Result<()> {
        // TODO handle non-standard card actions
        let card = self.hand.remove_item(&card);

        if let Some(card) = card {
            if let Some(TurnPhase::Action {
                remaining_actions,
                remaining_buys,
                total_wealth,
            }) = &mut self.phase
            {
                if *remaining_actions == 0 {
                    self.hand.push(card);
                    Err(Error::NoMoreActions)
                } else if let Some(e) = card.action() {
                    *remaining_actions -= 1;

                    *remaining_actions += e.action;
                    *remaining_buys += e.buy;
                    *total_wealth += e.worth;

                    for _ in 0..e.card {
                        self.draw_card()
                    }

                    self.in_play.push(card);
                    Ok(())
                } else {
                    self.hand.push(card);
                    Err(Error::WrongTurnPhase)
                }
            } else if let Some(TurnPhase::Buy {
                remaining_buys: _,
                total_wealth,
            }) = &mut self.phase
            {
                if let Some(i) = card.treasure() {
                    *total_wealth += i;

                    self.in_play.push(card);
                    Ok(())
                } else {
                    self.hand.push(card);
                    Err(Error::WrongTurnPhase)
                }
            } else {
                self.hand.push(card);
                Err(Error::WrongTurnPhase)
            }
        } else {
            Err(Error::InvalidCardChoice)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominion::KingdomSet;

    fn create_player() -> Player {
        let supply = Rc::new(RefCell::new(Supply::new(KingdomSet::FirstGame.cards(), 2)));

        Player::new(supply)
    }

    #[test]
    fn buy_card_copper() {
        let mut p = create_player();

        p.phase = Some(TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 0,
        });

        let copper_supply = p
            .supply
            .borrow_mut()
            .get_mut(&CardKind::Copper)
            .unwrap()
            .to_owned();

        let r = p.buy_card(&CardKind::Copper);

        assert!(r.is_ok());
        assert_eq!(p.discard_pile[0], &CardKind::Copper);
        assert_eq!(
            p.supply
                .borrow_mut()
                .get_mut(&CardKind::Copper)
                .unwrap()
                .to_owned(),
            copper_supply - 1
        );
    }

    #[test]
    fn buy_card_market() {
        let mut p = create_player();

        p.phase = Some(TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 5,
        });

        let r = p.buy_card(&CardKind::Market);

        assert!(r.is_ok());
        assert_eq!(p.discard_pile[0], &CardKind::Market);
        assert_eq!(
            p.phase.unwrap(),
            TurnPhase::Buy {
                remaining_buys: 0,
                total_wealth: 0,
            }
        );
    }

    #[test]
    fn buy_card_no_remaining_buys() {
        let mut p = create_player();

        let c = &CardKind::Gold;

        p.phase = Some(TurnPhase::Buy {
            remaining_buys: 0,
            total_wealth: c.cost(),
        });

        let r = p.buy_card(c);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::NoMoreBuys);
        assert!(p.discard_pile.is_empty());
    }

    #[test]
    fn buy_card_not_enough_wealth() {
        let mut p = create_player();

        p.phase = Some(TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 0,
        });

        let r = p.buy_card(&CardKind::Gold);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::NotEnoughWealth);
        assert!(p.discard_pile.is_empty());
    }

    #[test]
    fn play_invalid_card() {
        let mut p = create_player();

        p.phase = Some(TurnPhase::Action {
            remaining_actions: 1,
            remaining_buys: 0,
            total_wealth: 0,
        });

        let r = p.play_card(&CardKind::Copper);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InvalidCardChoice);
    }

    #[test]
    fn play_card_out_of_turn() {
        let mut p = create_player();

        p.hand.push(&CardKind::Gold);
        let r = p.play_card(p.hand[0]);

        assert!(r.is_err());
        assert_eq!(p.hand.len(), 1);
        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
    }

    #[test]
    fn play_card_smithy_during_action_phase() {
        let mut p = create_player();

        p.phase = Some(TurnPhase::Action {
            remaining_actions: 1,
            remaining_buys: 0,
            total_wealth: 0,
        });

        // Set hand to have a single card
        p.hand.push(&CardKind::Smithy);
        let r = p.play_card(p.hand[0]);

        assert!(r.is_ok());
        assert_eq!(p.hand.len(), 3);
        assert_eq!(
            p.phase.unwrap(),
            TurnPhase::Action {
                remaining_actions: 0,
                remaining_buys: 0,
                total_wealth: 0
            }
        );
    }

    #[test]
    fn play_card_smithy_during_buy_phase() {
        let mut p = create_player();

        p.phase = Some(TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 0,
        });

        // Set hand to have a single card
        p.hand.push(&CardKind::Smithy);
        let r = p.play_card(p.hand[0]);

        assert!(r.is_err());
        assert_eq!(p.hand.len(), 1);
        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
    }

    #[test]
    fn play_card_gold_during_action_phase() {
        let mut p = create_player();

        p.phase = Some(TurnPhase::Action {
            remaining_actions: 1,
            remaining_buys: 0,
            total_wealth: 0,
        });

        // Set hand to have a single card
        p.hand.push(&CardKind::Gold);
        let r = p.play_card(p.hand[0]);

        assert!(r.is_err());
        assert_eq!(p.hand.len(), 1);
        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
    }

    #[test]
    fn play_card_gold_during_buy_phase() {
        let mut p = create_player();

        p.phase = Some(TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 0,
        });

        // Set hand to have a single card
        p.hand.push(&CardKind::Gold);
        let r = p.play_card(p.hand[0]);

        assert!(r.is_ok());
        assert_eq!(p.hand.len(), 0);
        assert_eq!(
            p.phase.unwrap(),
            TurnPhase::Buy {
                remaining_buys: 1,
                total_wealth: 3
            }
        );
    }
}
