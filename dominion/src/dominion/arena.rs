use crate::dominion::player::{CardVec, Player};
use crate::dominion::CardKind;
use crate::dominion::KingdomSet;
use crate::dominion::Supply;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidPlayerId,
    InactivePlayer,
    WrongTurnPhase,
    InvalidCardLocation,
    NotYetImplemented,
    NoMoreActions,
    NoMoreBuys,
    NoMoreCards,
    NotEnoughWealth,
    InvalidCardChoice,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Eq, PartialEq)]
pub enum TurnPhase {
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

pub enum Location {
    Discard,
    Hand,
    Stage,
    Supply,
    Trash,
}

const STARTING_TURNPHASE: TurnPhase = TurnPhase::Action {
    remaining_actions: 1,
    remaining_buys: 1,
    total_wealth: 1,
};

#[derive(Debug)]
struct Turn {
    player_id: usize,
    phase: TurnPhase,
}

#[derive(Debug)]
pub struct Arena {
    supply: Supply,
    players: Vec<Player>,
    turn: Turn,
}

impl Arena {
    pub fn new(kingdom_set: KingdomSet, num_players: usize) -> Arena {
        let mut arena = Arena {
            supply: Supply::new(kingdom_set.cards(), num_players),
            players: (0..num_players).map(|_| Player::new()).collect(),
            turn: Turn {
                player_id: 0,
                phase: STARTING_TURNPHASE,
            },
        };

        arena.start_game();

        arena
    }

    pub fn supply(&self) -> &Supply {
        &self.supply
    }

    pub fn end_action_phase(&mut self, player_id: usize) -> Result<()> {
        self.check_active(player_id)?;

        match self.turn.phase {
            TurnPhase::Action {
                remaining_actions: _,
                remaining_buys,
                total_wealth,
            } => {
                self.turn.phase = TurnPhase::Buy {
                    remaining_buys,
                    total_wealth,
                };
                Ok(())
            }
            _ => Err(Error::WrongTurnPhase),
        }
    }

    pub fn end_buy_phase(&mut self, player_id: usize) -> Result<()> {
        self.check_active(player_id)?;

        match &mut self.turn.phase {
            TurnPhase::Buy { .. } => {
                self.players[player_id].cleanup();
                let player_id = (player_id + 1) % self.players.len();
                self.turn = Turn {
                    player_id,
                    phase: STARTING_TURNPHASE,
                };
                Ok(())
            }
            _ => Err(Error::WrongTurnPhase),
        }
    }

    // This function is context dependent
    pub fn select_card(
        &mut self,
        player_id: usize,
        card: CardKind,
        location: Location,
    ) -> Result<()> {
        self.check_active(player_id)?;

        match &mut self.turn.phase {
            TurnPhase::Action {
                remaining_actions,
                remaining_buys,
                total_wealth,
            } => match location {
                Location::Hand => match self.players[player_id].hand.remove_item(&card) {
                    Some(card) => match card.action() {
                        Some(e) => {
                            if *remaining_actions == 0 {
                                self.players[player_id].hand.push(card);
                                Err(Error::NoMoreActions)
                            } else {
                                *remaining_actions -= 1;
                                *remaining_actions += e.action;
                                *remaining_buys += e.buy;
                                *total_wealth += e.worth;

                                for _ in 0..e.card {
                                    self.players[player_id].draw_card();
                                }

                                self.players[player_id].play_zone.push(card);
                                Ok(())
                            }
                        }
                        None => {
                            self.players[player_id].hand.push(card);
                            Err(Error::InvalidCardChoice)
                        }
                    },
                    None => Err(Error::InvalidCardChoice),
                },
                _ => Err(Error::InvalidCardLocation),
            },
            TurnPhase::Buy {
                remaining_buys,
                total_wealth,
            } => match location {
                Location::Hand => match self.players[player_id].hand.remove_item(&card) {
                    Some(card) => match card.treasure() {
                        Some(i) => {
                            *total_wealth += i;
                            self.players[player_id].play_zone.push(card);
                            Ok(())
                        }
                        None => {
                            self.players[player_id].hand.push(card);
                            Err(Error::InvalidCardChoice)
                        }
                    },
                    None => Err(Error::InvalidCardChoice),
                },
                Location::Supply => match self.supply.get_mut(card) {
                    Some(supply_count) => {
                        if *supply_count == 0 {
                            Err(Error::NoMoreCards)
                        } else if *remaining_buys == 0 {
                            Err(Error::NoMoreBuys)
                        } else if *total_wealth < card.cost() {
                            Err(Error::NotEnoughWealth)
                        } else {
                            *supply_count -= 1;
                            *remaining_buys -= 1;
                            *total_wealth -= card.cost();
                            self.players[player_id].discard_pile.push(card);
                            Ok(())
                        }
                    }
                    None => Err(Error::InvalidCardChoice),
                },
                _ => Err(Error::InvalidCardLocation),
            },
        }
    }

    pub(crate) fn hand(&self, player_id: usize) -> Result<&CardVec> {
        self.player(player_id).map(|player| &player.hand)
    }

    pub(crate) fn discard_pile(&self, player_id: usize) -> Result<&CardVec> {
        self.player(player_id).map(|player| &player.discard_pile)
    }

    pub(crate) fn play_zone(&self, player_id: usize) -> Result<&CardVec> {
        self.player(player_id).map(|player| &player.play_zone)
    }

    pub(crate) fn in_deck(&self, player_id: usize, card: CardKind) -> Result<bool> {
        self.player(player_id).map(|player| player.in_deck(card))
    }

    fn check_active(&mut self, player_id: usize) -> Result<()> {
        if player_id >= self.players.len() {
            Err(Error::InvalidPlayerId)
        } else if player_id != self.turn.player_id {
            Err(Error::InactivePlayer)
        } else {
            Ok(())
        }
    }

    fn start_game(&mut self) {
        for p in self.players.iter_mut() {
            p.cleanup();
        }
    }

    fn player(&self, player_id: usize) -> Result<&Player> {
        if player_id >= self.players.len() {
            Err(Error::InvalidPlayerId)
        } else {
            Ok(&self.players[player_id])
        }
    }

    fn current_player(&self) -> &Player {
        &self.players[self.turn.player_id]
    }

    fn current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.turn.player_id]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominion::{Arena, KingdomSet};

    #[test]
    fn test_check_active_player() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        let r = arena.check_active(0);
        assert!(r.is_ok());

        let r = arena.check_active(1);
        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InactivePlayer);
    }

    #[test]
    fn player_valid_index() {
        let arena = Arena::new(KingdomSet::FirstGame, 2);

        let r = arena.player(0);
        assert!(r.is_ok());
        assert_eq!(r.unwrap() as *const _, &arena.players[0] as *const _);

        let r = arena.player(1);
        assert!(r.is_ok());
        assert_eq!(r.unwrap() as *const _, &arena.players[1] as *const _);
    }

    #[test]
    fn player_invalid_index() {
        let arena = Arena::new(KingdomSet::FirstGame, 2);

        let r = arena.player(2);
        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InvalidPlayerId);
    }

    #[test]
    fn buy_card_copper() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 0,
        };

        let copper_supply = arena.supply.get_mut(CardKind::Copper).unwrap().to_owned();

        let r = arena.select_card(0, CardKind::Copper, Location::Supply);

        assert!(r.is_ok());
        assert_eq!(
            arena.player(0).unwrap().discard_pile,
            vec![CardKind::Copper]
        );
        assert_eq!(
            arena.supply.get_mut(CardKind::Copper).unwrap().to_owned(),
            copper_supply - 1
        );
        assert_eq!(
            arena.turn.phase,
            TurnPhase::Buy {
                remaining_buys: 0,
                total_wealth: 0,
            }
        );
    }

    #[test]
    fn buy_card_market() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 5,
        };

        let market_supply = arena.supply.get_mut(CardKind::Market).unwrap().to_owned();

        let r = arena.select_card(0, CardKind::Market, Location::Supply);

        assert!(r.is_ok());
        assert_eq!(
            arena.player(0).unwrap().discard_pile,
            vec![CardKind::Market]
        );
        assert_eq!(
            arena.supply.get_mut(CardKind::Market).unwrap().to_owned(),
            market_supply - 1
        );
        assert_eq!(
            arena.turn.phase,
            TurnPhase::Buy {
                remaining_buys: 0,
                total_wealth: 0,
            }
        );
    }

    #[test]
    fn buy_card_no_remaining_buys() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy {
            remaining_buys: 0,
            total_wealth: 100,
        };

        let r = arena.select_card(0, CardKind::Gold, Location::Supply);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::NoMoreBuys);
        assert!(arena.player(0).unwrap().discard_pile.is_empty());
    }

    #[test]
    fn buy_card_not_enough_wealth() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy {
            remaining_buys: 100,
            total_wealth: 0,
        };

        let r = arena.select_card(0, CardKind::Gold, Location::Supply);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::NotEnoughWealth);
        assert!(arena.player(0).unwrap().discard_pile.is_empty());
    }

    #[test]
    fn buy_card_not_in_kingdom() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 100,
        };

        let r = arena.select_card(0, CardKind::Witch, Location::Supply);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InvalidCardChoice);
        assert!(arena.player(0).unwrap().discard_pile.is_empty());
    }

    #[test]
    fn play_card_not_in_hand() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Action {
            remaining_actions: 1,
            remaining_buys: 0,
            total_wealth: 0,
        };

        let r = arena.select_card(0, CardKind::Copper, Location::Hand);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InvalidCardChoice);
    }

    #[test]
    fn play_card_out_of_turn() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 0,
        };

        assert_eq!(arena.turn.player_id, 0);

        arena.players[1].hand.clear();
        arena.players[1].hand.push(CardKind::Gold);
        let r = arena.select_card(1, CardKind::Gold, Location::Hand);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InactivePlayer);
        assert_eq!(arena.player(1).unwrap().hand.len(), 1);
    }

    #[test]
    fn play_card_smithy_during_action_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Action {
            remaining_actions: 1,
            remaining_buys: 0,
            total_wealth: 0,
        };

        arena.players[0].hand.clear();
        arena.players[0].hand.push(CardKind::Smithy);
        let r = arena.select_card(0, CardKind::Smithy, Location::Hand);

        assert!(r.is_ok());
        assert_eq!(arena.player(0).unwrap().hand.len(), 3);
        assert_eq!(
            arena.turn.phase,
            TurnPhase::Action {
                remaining_actions: 0,
                remaining_buys: 0,
                total_wealth: 0
            }
        );
    }

    #[test]
    fn play_card_smithy_during_buy_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 0,
        };

        arena.players[0].hand.clear();
        arena.players[0].hand.push(CardKind::Smithy);
        let r = arena.select_card(0, CardKind::Smithy, Location::Hand);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InvalidCardChoice);
        assert_eq!(arena.player(0).unwrap().hand.len(), 1);
    }

    #[test]
    fn play_card_gold_during_action_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Action {
            remaining_actions: 1,
            remaining_buys: 0,
            total_wealth: 0,
        };

        arena.players[0].hand.clear();
        arena.players[0].hand.push(CardKind::Gold);
        let r = arena.select_card(0, CardKind::Gold, Location::Hand);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InvalidCardChoice);
        assert_eq!(arena.player(0).unwrap().hand.len(), 1);
    }

    #[test]
    fn play_card_gold_during_buy_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy {
            remaining_buys: 1,
            total_wealth: 0,
        };

        arena.players[0].hand.clear();
        arena.players[0].hand.push(CardKind::Gold);
        let r = arena.select_card(0, CardKind::Gold, Location::Hand);

        assert!(r.is_ok());
        assert_eq!(arena.player(0).unwrap().hand.len(), 0);
        assert_eq!(
            arena.turn.phase,
            TurnPhase::Buy {
                remaining_buys: 1,
                total_wealth: 3
            }
        );
    }
}
