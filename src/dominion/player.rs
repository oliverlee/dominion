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

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    NoMoreActions,
    NoMoreBuys,
    NoMoreCards,
    NotEnoughWorth,
    InvalidCardIndex,
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

type CardVec = Vec<&'static CardKind>;

#[derive(Debug)]
pub struct Player {
    deck_pile: CardVec,
    hand: CardVec,
    in_play: CardVec,
    discard_pile: CardVec,
    phase: Option<TurnPhase>,
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
            phase: None,
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
                remaining_buys: 0,
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
            mut remaining_buys,
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

            Ok(())
        } else {
            Err(Error::WrongTurnPhase)
        }
    }

    pub fn play_card(&mut self, card_index: usize) -> Result<()> {
        // TODO handle non-standard card actions
        if card_index >= self.hand.len() {
            Err(Error::InvalidCardIndex)
        } else if let Some(phase) = &mut self.phase {
            match phase {
                TurnPhase::Action {
                    remaining_actions,
                    remaining_buys,
                    total_wealth,
                } => {
                    if *remaining_actions > 0 {
                        if let Some(e) = self.hand[card_index].action() {
                            *remaining_actions -= 1;

                            *remaining_actions += e.action;
                            *remaining_buys += e.buy;
                            *total_wealth += e.worth;

                            for _ in 0..e.card {
                                self.draw_card();
                            }

                            self.in_play.push(self.hand.remove(card_index));
                            Ok(())
                        } else {
                            Err(Error::WrongTurnPhase)
                        }
                    } else {
                        Err(Error::NoMoreActions)
                    }
                }
                TurnPhase::Buy {
                    remaining_buys,
                    total_wealth,
                } => {
                    if let Some(i) = self.hand[card_index].treasure() {
                        *total_wealth += i;

                        self.in_play.push(self.hand.remove(card_index));
                        Ok(())
                    } else {
                        Err(Error::WrongTurnPhase)
                    }
                }
            }
        } else {
            Err(Error::WrongTurnPhase)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_invalid_card() {
        let mut p = Player::new();

        p.phase = Some(TurnPhase::Action {
            remaining_actions: 1,
            remaining_buys: 0,
            total_wealth: 0,
        });

        let r = p.play_card(0);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InvalidCardIndex);
    }

    #[test]
    fn play_card_out_of_turn() {
        let mut p = Player::new();

        // Set hand to have a single card
        p.hand.push(&CardKind::Gold);

        let r = p.play_card(0);

        assert!(r.is_err());
        assert_eq!(p.hand.len(), 1);
        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
    }

    #[test]
    fn play_card_smithy_during_action_phase() {
        let mut p = Player::new();

        p.phase = Some(TurnPhase::Action {
            remaining_actions: 1,
            remaining_buys: 0,
            total_wealth: 0,
        });

        // Set hand to have a single card
        p.hand.push(&CardKind::Smithy);

        let r = p.play_card(0);

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
        let mut p = Player::new();

        p.phase = Some(TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 0,
        });

        // Set hand to have a single card
        p.hand.push(&CardKind::Smithy);

        let r = p.play_card(0);

        assert!(r.is_err());
        assert_eq!(p.hand.len(), 1);
        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
    }

    #[test]
    fn play_card_gold_during_action_phase() {
        let mut p = Player::new();

        p.phase = Some(TurnPhase::Action {
            remaining_actions: 1,
            remaining_buys: 0,
            total_wealth: 0,
        });

        // Set hand to have a single card
        p.hand.push(&CardKind::Gold);

        let r = p.play_card(0);

        assert!(r.is_err());
        assert_eq!(p.hand.len(), 1);
        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
    }

    #[test]
    fn play_card_gold_during_buy_phase() {
        let mut p = Player::new();

        p.phase = Some(TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 0,
        });

        // Set hand to have a single card
        p.hand.push(&CardKind::Gold);

        let r = p.play_card(0);

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
