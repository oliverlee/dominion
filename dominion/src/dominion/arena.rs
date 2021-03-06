use crate::dominion::location::{CardVec, Location};
use crate::dominion::turn::{self, Turn};
use crate::dominion::types::{Error, Result};
use crate::dominion::{CardKind, KingdomSet};

// These declarative macros are used to borrow a single player from the arena struct.
// This is useful when moving cards between the supply/trash and a player. Using
// `current_player_mut()` and similar functions will borrow the entire arena
// struct, preventing a second borrow of the arena.supply.
// These are defined before the effect module.
macro_rules! current_player {
    ($x:ident) => {
        $x.players[$x.current_player_id]
    };
}
//macro_rules! other_player {
//    ($x:ident) => { $x.players[$x.next_player_id()] }
//}

mod effect;
mod player;
mod supply;
use self::effect::CardActionQueue;
use self::player::Player;
use self::supply::Supply;

#[derive(Debug)]
pub struct Arena {
    supply: Supply,
    trash: CardVec,
    players: Vec<Player>,
    turn: Turn,
    current_player_id: usize,
    actions: Option<CardActionQueue>,
}

impl Arena {
    pub fn new(kingdom_set: KingdomSet, num_players: usize) -> Self {
        let mut arena = Self {
            supply: Supply::new(kingdom_set.cards(), num_players),
            trash: CardVec::new(),
            players: (0..num_players).map(|_| Player::new()).collect(),
            turn: Turn::new(),
            current_player_id: 0,
            actions: Some(CardActionQueue::new()),
        };

        arena.start_game();

        arena
    }

    pub fn kingdom(&self) -> impl std::iter::Iterator<Item = &'_ CardKind> {
        self.supply.kingdom_cards.iter().map(|(card, _)| card)
    }

    pub fn is_game_over(&self) -> bool {
        self.supply.is_game_over()
    }

    #[allow(dead_code)]
    pub fn in_deck(&self, player_id: usize, card: CardKind) -> Result<bool> {
        self.player(player_id).map(|player| player.in_deck(card))
    }

    pub fn turn(&self) -> Turn {
        self.turn
    }

    pub fn end_turn_phase(&mut self) -> Result<()> {
        match self.turn {
            Turn::Action(_) => self.end_action_phase(),
            Turn::Buy(_) => self.end_buy_phase(),
        }
    }

    fn end_action_phase(&mut self) -> Result<()> {
        self.check_actions_resolved()?;

        self.turn = Turn::Buy(self.turn.as_action_phase_mut()?.to_buy_phase());

        Ok(())
    }

    fn end_buy_phase(&mut self) -> Result<()> {
        self.check_actions_resolved()?;
        self.current_player_mut().cleanup();

        self.turn = Turn::new();
        self.current_player_id = self.next_player_id();

        Ok(())
    }

    pub fn play_card(&mut self, card: CardKind) -> Result<()> {
        match self.turn {
            Turn::Action(_) => self.play_action(card),
            Turn::Buy(_) => self.play_treasure(card),
        }
    }

    fn play_action(&mut self, card: CardKind) -> Result<()> {
        self.check_actions_resolved()?;

        if self.turn.as_action_phase_mut()?.remaining_actions == 0 {
            Err(Error::NoMoreActions)
        } else if card.is_action() {
            let player = self.current_player_mut();
            let _ = player.hand.move_card(&mut player.play_zone, card)?;

            self.turn.as_action_phase_mut().unwrap().remaining_actions -= 1;

            self.actions.as_mut().unwrap().add_card(card);
            self.try_resolve(self.current_player_id, None)
        } else {
            Err(Error::InvalidCard)
        }
    }

    fn play_treasure(&mut self, card: CardKind) -> Result<()> {
        self.check_actions_resolved()?;
        self.turn.as_buy_phase_mut()?;

        if card.is_treasure() {
            let additional_copper = card.resources().unwrap().copper;

            let player = self.current_player_mut();
            let _ = player.hand.move_card(&mut player.play_zone, card)?;

            self.turn.as_buy_phase_mut().unwrap().remaining_copper += additional_copper;

            Ok(())
        } else {
            Err(Error::InvalidCard)
        }
    }

    pub fn buy_card(&mut self, card: CardKind) -> Result<()> {
        self.check_actions_resolved()?;

        let &mut turn::BuyPhase {
            remaining_buys,
            remaining_copper,
        } = self.turn.as_buy_phase_mut()?;

        if remaining_buys == 0 {
            Err(Error::NoMoreBuys)
        } else if remaining_copper < card.cost() {
            Err(Error::NotEnoughCopper)
        } else {
            let _ = self
                .supply
                .move_card(&mut current_player!(self).discard_pile, card)?;

            let buy_phase = self.turn.as_buy_phase_mut().unwrap();
            buy_phase.remaining_buys -= 1;
            buy_phase.remaining_copper -= card.cost();

            Ok(())
        }
    }

    // Select cards to resolve an action effect.
    pub fn select_cards(&mut self, player_id: usize, cards: &[CardKind]) -> Result<()> {
        if player_id >= self.players.len() {
            Err(Error::InvalidPlayerId)
        } else {
            self.try_resolve(player_id, Some(cards))
        }
    }

    fn try_resolve(&mut self, player_id: usize, selected_cards: Option<&[CardKind]>) -> Result<()> {
        let mut temp: Option<CardActionQueue> = None;

        // The Arena contains the ActionEffect to track the state of resolving an action card.
        // However, the ActionEffect::resolve method requires a mutable reference to the
        // Arena as it will need to modify the game state. To prevent more than one mutable borrow,
        // we swap Some(CardActionQueue) with None.
        std::mem::swap(&mut temp, &mut self.actions);

        let result = temp
            .as_mut()
            .unwrap()
            .resolve(self, player_id, selected_cards);

        std::mem::swap(&mut temp, &mut self.actions);

        result
    }

    fn start_game(&mut self) {
        for p in &mut self.players {
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

    fn player_mut(&mut self, player_id: usize) -> Result<&mut Player> {
        if player_id >= self.players.len() {
            Err(Error::InvalidPlayerId)
        } else {
            Ok(&mut self.players[player_id])
        }
    }

    fn current_player(&self) -> &Player {
        &self.players[self.current_player_id]
    }

    fn current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.current_player_id]
    }

    fn next_player_id(&self) -> usize {
        (self.current_player_id + 1) % self.players.len()
    }

    fn check_actions_resolved(&mut self) -> Result<()> {
        if self.actions.as_ref().unwrap().is_resolved() {
            Ok(())
        } else {
            Err(Error::UnresolvedActionEffect(
                self.actions
                    .as_ref()
                    .unwrap()
                    .resolve_condition()
                    .unwrap_or(""),
            ))
        }
    }

    pub fn current_player_id(&self) -> usize {
        self.current_player_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominion::{Arena, KingdomSet};

    impl Supply {
        fn count(&self, card: CardKind) -> usize {
            self.find(card)
                .map(|i| {
                    let (_, count) = self.get_entry(i).unwrap();
                    *count
                })
                .unwrap()
        }
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
    fn end_action_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn = Turn::Action(turn::ActionPhase {
            remaining_actions: 1,
            remaining_buys: 1,
            remaining_copper: 0,
        });

        let r = arena.end_action_phase();

        assert!(r.is_ok());
        assert_eq!(
            arena.turn,
            Turn::Buy(Turn::new().as_action_phase_mut().unwrap().to_buy_phase())
        );
    }

    #[test]
    fn end_buy_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn = Turn::Buy(turn::BuyPhase {
            remaining_buys: 1,
            remaining_copper: 0,
        });

        let r = arena.end_buy_phase();

        assert!(r.is_ok());
        assert_eq!(arena.turn, Turn::new());
        assert_eq!(arena.current_player_id, 1);

        assert_eq!(arena.player(0).unwrap().hand.len(), 5);
        assert_eq!(arena.player(0).unwrap().discard_pile.len(), 5);
        assert_eq!(arena.player(0).unwrap().draw_pile.len(), 0);
    }

    #[test]
    fn buy_card_copper() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn = Turn::Buy(turn::BuyPhase {
            remaining_buys: 1,
            remaining_copper: 0,
        });

        let copper_count = arena.supply.count(CardKind::Copper);

        let r = arena.buy_card(CardKind::Copper);

        assert!(r.is_ok());
        assert_eq!(
            arena.player(0).unwrap().discard_pile,
            cardvec![CardKind::Copper]
        );
        assert_eq!(arena.supply.count(CardKind::Copper), copper_count - 1);
        assert_eq!(
            arena.turn,
            Turn::Buy(turn::BuyPhase {
                remaining_buys: 0,
                remaining_copper: 0,
            })
        );
    }

    #[test]
    fn buy_card_market() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn = Turn::Buy(turn::BuyPhase {
            remaining_buys: 1,
            remaining_copper: 5,
        });

        let market_count = arena.supply.count(CardKind::Market);

        let r = arena.buy_card(CardKind::Market);

        assert!(r.is_ok());
        assert_eq!(
            arena.player(0).unwrap().discard_pile,
            cardvec![CardKind::Market]
        );
        assert_eq!(arena.supply.count(CardKind::Market), market_count - 1);
        assert_eq!(
            arena.turn,
            Turn::Buy(turn::BuyPhase {
                remaining_buys: 0,
                remaining_copper: 0,
            })
        );
    }

    #[test]
    fn buy_card_no_remaining_buys() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn = Turn::Buy(turn::BuyPhase {
            remaining_buys: 0,
            remaining_copper: 100,
        });

        let r = arena.buy_card(CardKind::Gold);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::NoMoreBuys);
        assert!(arena.player(0).unwrap().discard_pile.is_empty());
    }

    #[test]
    fn buy_card_not_enough_copper() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn = Turn::Buy(turn::BuyPhase {
            remaining_buys: 100,
            remaining_copper: 0,
        });

        let r = arena.buy_card(CardKind::Gold);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::NotEnoughCopper);
        assert!(arena.player(0).unwrap().discard_pile.is_empty());
    }

    #[test]
    fn buy_card_not_in_kingdom() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn = Turn::Buy(turn::BuyPhase {
            remaining_buys: 1,
            remaining_copper: 100,
        });

        let r = arena.buy_card(CardKind::Witch);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InvalidCard);
        assert!(arena.player(0).unwrap().discard_pile.is_empty());
    }

    #[test]
    fn play_action_not_in_hand() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn = Turn::Action(turn::ActionPhase {
            remaining_actions: 1,
            remaining_buys: 0,
            remaining_copper: 0,
        });

        let r = arena.play_action(CardKind::Market);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InvalidCard);
    }

    #[test]
    fn play_action_smithy_during_action_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn = Turn::Action(turn::ActionPhase {
            remaining_actions: 1,
            remaining_buys: 0,
            remaining_copper: 0,
        });

        arena.players[0].hand.clear();
        arena.players[0].hand.push(CardKind::Smithy);
        let r = arena.play_action(CardKind::Smithy);

        assert!(r.is_ok());
        assert_eq!(arena.player(0).unwrap().hand.len(), 3);
        assert_eq!(
            arena.turn,
            Turn::Action(turn::ActionPhase {
                remaining_actions: 0,
                remaining_buys: 0,
                remaining_copper: 0
            })
        );
    }

    #[test]
    fn play_action_smithy_during_buy_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn = Turn::Buy(turn::BuyPhase {
            remaining_buys: 1,
            remaining_copper: 0,
        });

        arena.players[0].hand.clear();
        arena.players[0].hand.push(CardKind::Smithy);
        let r = arena.play_action(CardKind::Smithy);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
        assert_eq!(arena.player(0).unwrap().hand.len(), 1);
    }

    #[test]
    fn play_treasure_gold_during_action_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn = Turn::Action(turn::ActionPhase {
            remaining_actions: 1,
            remaining_buys: 0,
            remaining_copper: 0,
        });

        arena.players[0].hand.clear();
        arena.players[0].hand.push(CardKind::Gold);
        let r = arena.play_treasure(CardKind::Gold);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
        assert_eq!(arena.player(0).unwrap().hand.len(), 1);
    }

    #[test]
    fn play_treasuse_gold_during_buy_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn = Turn::Buy(turn::BuyPhase {
            remaining_buys: 1,
            remaining_copper: 0,
        });

        arena.players[0].hand.clear();
        arena.players[0].hand.push(CardKind::Gold);
        let r = arena.play_treasure(CardKind::Gold);

        assert!(r.is_ok());
        assert_eq!(arena.player(0).unwrap().hand.len(), 0);
        assert_eq!(
            arena.turn,
            Turn::Buy(turn::BuyPhase {
                remaining_buys: 1,
                remaining_copper: 3
            })
        );
    }
}
