use crate::dominion::player::{CardVec, Player};
use crate::dominion::CardKind;
use crate::dominion::KingdomSet;
use crate::dominion::Supply;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

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
    supply: Rc<RefCell<Supply>>,
    players: Vec<Player>,
    turn: Turn,
}

impl Arena {
    pub fn new(kingdom_set: KingdomSet, num_players: usize) -> Arena {
        let supply = Rc::new(RefCell::new(Supply::new(kingdom_set.cards(), num_players)));
        let players: Vec<_> = (0..num_players)
            .map(|_| Player::new(supply.clone()))
            .collect();

        let mut arena = Arena {
            supply,
            players,
            turn: Turn {
                player_id: 0,
                phase: STARTING_TURNPHASE,
            },
        };

        arena.start_game();

        arena
    }

    pub fn supply(&self) -> Ref<Supply> {
        self.supply.borrow()
    }

    pub fn turn(&self) -> &Turn {
        &self.turn
    }

    fn check_active_player(&mut self, player_id: usize) -> Result<()> {
        if player_id >= self.players.len() {
            Err(Error::InvalidPlayerId)
        } else if player_id != self.turn.player_id {
            Err(Error::InactivePlayer)
        } else {
            Ok(())
        }
    }

    pub fn end_action_phase(&mut self, player_id: usize) -> Result<()> {
        self.check_active_player(player_id)?;

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
        self.check_active_player(player_id)?;

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
        card: &'static CardKind,
        location: Location,
    ) -> Result<()> {
        self.check_active_player(player_id)?;

        match &mut self.turn.phase {
            TurnPhase::Action {
                remaining_actions,
                remaining_buys,
                total_wealth,
            } => match location {
                Location::Hand => Err(Error::NotYetImplemented), // TODO
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
                            self.players[player_id].in_play.push(&card);
                            Ok(())
                        }
                        None => {
                            self.players[player_id].hand.push(&card);
                            Err(Error::InvalidCardChoice)
                        }
                    },
                    None => Err(Error::InvalidCardChoice),
                },
                Location::Supply => match (*self.supply.borrow_mut()).get_mut(card) {
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

    pub fn check_hand(&self, player_id: usize) -> Result<&CardVec> {
        Ok(&self.get_player(player_id)?.hand)
    }

    pub fn check_discard_pile(&self, player_id: usize) -> Result<&CardVec> {
        Ok(&self.get_player(player_id)?.discard_pile)
    }

    pub fn check_in_play(&self, player_id: usize) -> Result<&CardVec> {
        Ok(&self.get_player(player_id)?.in_play)
    }

    fn start_game(&mut self) {
        for p in self.players.iter_mut() {
            p.cleanup();
        }
    }

    fn get_player(&self, player_id: usize) -> Result<&Player> {
        if player_id >= self.players.len() {
            Err(Error::InvalidPlayerId)
        } else {
            Ok(&self.players[player_id])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominion::{Arena, KingdomSet};

    #[test]
    fn get_player_valid_index() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        let r = arena.get_player(0);
        assert!(r.is_ok());
        assert_eq!(r.unwrap() as *const _, &arena.players[0] as *const _);

        let r = arena.get_player(1);
        assert!(r.is_ok());
        assert_eq!(r.unwrap() as *const _, &arena.players[1] as *const _);
    }

    #[test]
    fn get_player_invalid_index() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        let r = arena.get_player(2);
        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InvalidPlayerId);
    }
}
