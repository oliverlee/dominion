use crate::dominion::turn::{self, Turn};
use crate::dominion::types::{CardSpecifier, CardVec, Error, Location, LocationView, Result};
use crate::dominion::{CardKind, KingdomSet};

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
    pub fn new(kingdom_set: KingdomSet, num_players: usize) -> Arena {
        let mut arena = Arena {
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

    pub fn view(&self, loc: Location) -> Result<LocationView> {
        match loc {
            Location::Draw { player_id } => self
                .player(player_id)
                .map(|player| LocationView::Ordered(&player.draw_pile)),
            Location::Discard { player_id } => self
                .player(player_id)
                .map(|player| LocationView::Ordered(&player.discard_pile)),
            Location::Hand { player_id } => self
                .player(player_id)
                .map(|player| LocationView::Ordered(&player.hand)),
            Location::Play { player_id } => self
                .player(player_id)
                .map(|player| LocationView::Ordered(&player.play_zone)),
            Location::Stage { player_id } => self
                .player(player_id)
                .map(|player| LocationView::Ordered(&player.stage)),
            Location::Supply => Ok(LocationView::Unordered(
                self.supply
                    .base_cards
                    .iter()
                    .chain(self.supply.kingdom_cards.iter()),
            )),
            Location::Trash => Ok(LocationView::Ordered(&self.trash)),
        }
    }

    pub fn kingdom(&self) -> impl std::iter::Iterator<Item = &'_ CardKind> {
        self.supply.kingdom_cards.keys()
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
        let player_id = self.current_player_id;

        if self.turn.as_action_phase_mut()?.remaining_actions == 0 {
            Err(Error::NoMoreActions)
        } else if card.is_action() {
            self.move_card(
                Location::Hand { player_id },
                Location::Play { player_id },
                CardSpecifier::Card(card),
            )?;
            self.turn.as_action_phase_mut().unwrap().remaining_actions -= 1;
            self.actions.as_mut().unwrap().add_card(card);
            self.try_resolve(player_id, None)
        } else {
            Err(Error::InvalidCardChoice)
        }
    }

    fn play_treasure(&mut self, card: CardKind) -> Result<()> {
        self.check_actions_resolved()?;
        let player_id = self.current_player_id;

        self.turn.as_buy_phase_mut()?;

        if card.is_treasure() {
            let additional_copper = card.resources().unwrap().copper;

            self.move_card(
                Location::Hand { player_id },
                Location::Play { player_id },
                CardSpecifier::Card(card),
            )?;

            self.turn.as_buy_phase_mut().unwrap().remaining_copper += additional_copper;

            Ok(())
        } else {
            Err(Error::InvalidCardChoice)
        }
    }

    pub fn buy_card(&mut self, card: CardKind) -> Result<()> {
        self.check_actions_resolved()?;
        let player_id = self.current_player_id;

        let &mut turn::BuyPhase {
            remaining_buys,
            remaining_copper,
        } = self.turn.as_buy_phase_mut()?;

        if remaining_buys == 0 {
            Err(Error::NoMoreBuys)
        } else if remaining_copper < card.cost() {
            Err(Error::NotEnoughCopper)
        } else {
            self.move_card(
                Location::Supply,
                Location::Discard { player_id },
                CardSpecifier::Card(card),
            )?;

            let buy_phase = self.turn.as_buy_phase_mut().unwrap();
            buy_phase.remaining_buys -= 1;
            buy_phase.remaining_copper -= card.cost();

            Ok(())
        }
    }

    // Select cards to resolve an action effect.
    pub fn select_cards(&mut self, player_id: usize, cards: &CardVec) -> Result<()> {
        if player_id >= self.players.len() {
            Err(Error::InvalidPlayerId)
        } else {
            self.try_resolve(player_id, Some(cards))
        }
    }

    fn move_card(
        &mut self,
        origin: Location,
        destination: Location,
        card: CardSpecifier,
    ) -> Result<()> {
        let card = match card {
            CardSpecifier::Top => match origin {
                Location::Supply => {
                    panic!("Cannot use CardSpecifier::Top with origin Location::Supply.")
                }
                _ => self.location(origin).pop().unwrap(),
            },
            CardSpecifier::Index(i) => match origin {
                Location::Supply => {
                    panic!("Cannot use CardSpecifier::Index with origin Location::Supply.")
                }
                _ => self.location(origin).remove(i),
            },
            CardSpecifier::Card(card) => match origin {
                Location::Supply => {
                    let card_supply = self
                        .supply
                        .get_mut(card)
                        .ok_or_else(|| Error::InvalidCardChoice)?;

                    if *card_supply == 0 {
                        Err(Error::NoMoreCards)?;
                    } else {
                        *card_supply -= 1;
                    }

                    card
                }
                _ => self
                    .location(origin)
                    .remove_item(&card)
                    .ok_or_else(|| Error::InvalidCardChoice)?,
            },
        };

        match destination {
            Location::Supply => panic!("Cannot move card to destination Location::Supply."),
            _ => self.location(destination).push(card),
        };

        Ok(())
    }

    fn try_resolve(&mut self, player_id: usize, selected_cards: Option<&CardVec>) -> Result<()> {
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

    fn location(&mut self, loc: Location) -> &mut CardVec {
        match loc {
            Location::Draw { player_id } => &mut self.player_mut(player_id).unwrap().draw_pile,
            Location::Discard { player_id } => {
                &mut self.player_mut(player_id).unwrap().discard_pile
            }
            Location::Hand { player_id } => &mut self.player_mut(player_id).unwrap().hand,
            Location::Play { player_id } => &mut self.player_mut(player_id).unwrap().play_zone,
            Location::Stage { player_id } => &mut self.player_mut(player_id).unwrap().stage,
            Location::Supply => panic!("Cannot return Location::Supply as a '&mut CardVec'"),
            Location::Trash => &mut self.trash,
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

        let copper_supply = arena.supply.get_mut(CardKind::Copper).unwrap().to_owned();

        let r = arena.buy_card(CardKind::Copper);

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

        let market_supply = arena.supply.get_mut(CardKind::Market).unwrap().to_owned();

        let r = arena.buy_card(CardKind::Market);

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
        assert_eq!(r.unwrap_err(), Error::InvalidCardChoice);
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
        assert_eq!(r.unwrap_err(), Error::InvalidCardChoice);
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
