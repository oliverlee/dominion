use crate::dominion::player::Player;
use crate::dominion::turn_phase::{ActionPhase, BuyPhase, TurnPhase};
use crate::dominion::types::{CardSpecifier, CardVec, Error, Location, Result};
use crate::dominion::CardKind;
use crate::dominion::KingdomSet;
use crate::dominion::Supply;

const STARTING_TURNPHASE: TurnPhase = TurnPhase::Action(ActionPhase {
    remaining_actions: 1,
    remaining_buys: 1,
    remaining_copper: 0,
});

#[derive(Debug)]
pub(crate) struct Turn {
    pub(crate) player_id: usize,
    pub(crate) phase: TurnPhase,
}

#[derive(Debug)]
pub struct Arena {
    supply: Supply,
    pub(crate) players: Vec<Player>,
    pub(crate) turn: Turn,
    trash: CardVec,
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
            trash: CardVec::new(),
        };

        arena.start_game();

        arena
    }

    pub fn supply(&self) -> &Supply {
        &self.supply
    }

    pub fn turn_phase(&self) -> TurnPhase {
        self.turn.phase.clone()
    }

    pub fn end_action_phase(&mut self, player_id: usize) -> Result<()> {
        self.check_active(player_id)?;
        self.turn.phase = TurnPhase::Buy(self.turn.phase.as_action_phase_mut()?.as_buy_phase());
        Ok(())
    }

    pub fn end_buy_phase(&mut self, player_id: usize) -> Result<()> {
        self.check_active(player_id)?;

        self.players[player_id].cleanup();

        self.turn.player_id = self.next_player_id();
        self.turn.phase = self
            .turn
            .phase
            .as_buy_phase_mut()
            .map(|_| STARTING_TURNPHASE)?;

        Ok(())
    }

    pub fn play_action(&mut self, player_id: usize, card: CardKind) -> Result<()> {
        self.check_active(player_id)?;

        let action_phase = self.turn.phase.as_action_phase_mut()?;

        if action_phase.remaining_actions == 0 {
            Err(Error::NoMoreActions)
        } else {
            let card_index = self.players[player_id]
                .hand
                .iter()
                .position(|&x| x == card)
                .ok_or_else(|| Error::InvalidCardChoice)?;

            let resources = card.action().ok_or_else(|| Error::InvalidCardChoice)?;

            action_phase.remaining_actions -= 1;

            let player = &mut self.players[player_id];
            let card = player.hand.remove(card_index);
            player.play_zone.push(card);

            action_phase.remaining_actions += resources.actions;
            action_phase.remaining_buys += resources.buys;
            action_phase.remaining_copper += resources.copper;

            for _ in 0..resources.cards {
                self.players[player_id].draw_card();
            }

            Ok(())
        }
    }

    pub fn play_treasure(&mut self, player_id: usize, card: CardKind) -> Result<()> {
        self.check_active(player_id)?;

        let buy_phase = self.turn.phase.as_buy_phase_mut()?;

        let card_index = self.players[player_id]
            .hand
            .iter()
            .position(|&x| x == card)
            .ok_or_else(|| Error::InvalidCardChoice)?;

        let additional_copper = card.treasure().ok_or_else(|| Error::InvalidCardChoice)?;

        let player = &mut self.players[player_id];
        let card = player.hand.remove(card_index);
        player.play_zone.push(card);

        buy_phase.remaining_copper += additional_copper;

        Ok(())
    }

    pub fn buy_card(&mut self, player_id: usize, card: CardKind) -> Result<()> {
        self.check_active(player_id)?;

        let buy_phase = self.turn.phase.as_buy_phase_mut()?;

        let supply_count = self
            .supply
            .get_mut(card)
            .ok_or_else(|| Error::InvalidCardChoice)?;

        if *supply_count == 0 {
            Err(Error::NoMoreCards)
        } else if buy_phase.remaining_buys == 0 {
            Err(Error::NoMoreBuys)
        } else if buy_phase.remaining_copper < card.cost() {
            Err(Error::NotEnoughCopper)
        } else {
            buy_phase.remaining_buys -= 1;
            buy_phase.remaining_copper -= card.cost();
            *supply_count -= 1;

            self.players[player_id].discard_pile.push(card);

            Ok(())
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

    fn location(&mut self, loc: Location) -> &mut CardVec {
        match loc {
            Location::Draw { player_id } => &mut self.players[player_id].draw_pile,
            Location::Discard { player_id } => &mut self.players[player_id].discard_pile,
            Location::Hand { player_id } => &mut self.players[player_id].hand,
            Location::Play { player_id } => &mut self.players[player_id].play_zone,
            Location::Stage { player_id } => &mut self.players[player_id].stage,
            Location::Supply => panic!("Cannot return Location::Supply as a '&mut CardVec'"),
            Location::Trash => &mut self.trash,
        }
    }

    pub(crate) fn move_card(
        &mut self,
        origin: Location,
        destination: Location,
        card: CardSpecifier,
    ) {
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
            CardSpecifier::Card(c) => match origin {
                Location::Supply => {
                    let card_supply = self.supply.get_mut(c).unwrap();

                    if *card_supply == 0 {
                        panic!("Cannot move card from an empty supply pile.");
                    } else {
                        *card_supply -= 1;
                    }

                    c
                }
                _ => self.location(origin).remove_item(&c).unwrap(),
            },
        };

        match destination {
            Location::Supply => panic!("Cannot move card to destination Location::Supply."),
            _ => self.location(destination).push(card),
        };
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

    pub(crate) fn current_player(&self) -> &Player {
        &self.players[self.turn.player_id]
    }

    pub(crate) fn current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.turn.player_id]
    }

    pub(crate) fn next_player_id(&self) -> usize {
        (self.turn.player_id + 1) % self.players.len()
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
    fn end_action_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Action(ActionPhase {
            remaining_actions: 1,
            remaining_buys: 1,
            remaining_copper: 0,
        });

        let r = arena.end_action_phase(0);

        assert!(r.is_ok());
        assert_eq!(
            arena.turn.phase,
            TurnPhase::Buy(
                STARTING_TURNPHASE
                    .as_action_phase_mut()
                    .unwrap()
                    .as_buy_phase()
            )
        );
    }

    #[test]
    fn end_buy_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy(BuyPhase {
            remaining_buys: 1,
            remaining_copper: 0,
        });

        let r = arena.end_buy_phase(0);

        assert!(r.is_ok());
        assert_eq!(arena.turn.phase, STARTING_TURNPHASE);
        assert_eq!(arena.turn.player_id, 1);

        assert_eq!(arena.player(0).unwrap().hand.len(), 5);
        assert_eq!(arena.player(0).unwrap().discard_pile.len(), 5);
        assert_eq!(arena.player(0).unwrap().draw_pile.len(), 0);
    }

    #[test]
    fn buy_card_copper() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy(BuyPhase {
            remaining_buys: 1,
            remaining_copper: 0,
        });

        let copper_supply = arena.supply.get_mut(CardKind::Copper).unwrap().to_owned();

        let r = arena.buy_card(0, CardKind::Copper);

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
            TurnPhase::Buy(BuyPhase {
                remaining_buys: 0,
                remaining_copper: 0,
            })
        );
    }

    #[test]
    fn buy_card_market() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy(BuyPhase {
            remaining_buys: 1,
            remaining_copper: 5,
        });

        let market_supply = arena.supply.get_mut(CardKind::Market).unwrap().to_owned();

        let r = arena.buy_card(0, CardKind::Market);

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
            TurnPhase::Buy(BuyPhase {
                remaining_buys: 0,
                remaining_copper: 0,
            })
        );
    }

    #[test]
    fn buy_card_no_remaining_buys() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy(BuyPhase {
            remaining_buys: 0,
            remaining_copper: 100,
        });

        let r = arena.buy_card(0, CardKind::Gold);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::NoMoreBuys);
        assert!(arena.player(0).unwrap().discard_pile.is_empty());
    }

    #[test]
    fn buy_card_not_enough_wealth() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy(BuyPhase {
            remaining_buys: 100,
            remaining_copper: 0,
        });

        let r = arena.buy_card(0, CardKind::Gold);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::NotEnoughCopper);
        assert!(arena.player(0).unwrap().discard_pile.is_empty());
    }

    #[test]
    fn buy_card_not_in_kingdom() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy(BuyPhase {
            remaining_buys: 1,
            remaining_copper: 100,
        });

        let r = arena.buy_card(0, CardKind::Witch);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InvalidCardChoice);
        assert!(arena.player(0).unwrap().discard_pile.is_empty());
    }

    #[test]
    fn play_action_not_in_hand() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Action(ActionPhase {
            remaining_actions: 1,
            remaining_buys: 0,
            remaining_copper: 0,
        });

        let r = arena.play_action(0, CardKind::Market);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InvalidCardChoice);
    }

    #[test]
    fn play_action_out_of_turn() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy(BuyPhase {
            remaining_buys: 1,
            remaining_copper: 0,
        });

        assert_eq!(arena.turn.player_id, 0);

        arena.players[1].hand.clear();
        arena.players[1].hand.push(CardKind::Moat);
        let r = arena.play_action(1, CardKind::Moat);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::InactivePlayer);
        assert_eq!(arena.player(1).unwrap().hand.len(), 1);
    }

    #[test]
    fn play_action_smithy_during_action_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Action(ActionPhase {
            remaining_actions: 1,
            remaining_buys: 0,
            remaining_copper: 0,
        });

        arena.players[0].hand.clear();
        arena.players[0].hand.push(CardKind::Smithy);
        let r = arena.play_action(0, CardKind::Smithy);

        assert!(r.is_ok());
        assert_eq!(arena.player(0).unwrap().hand.len(), 3);
        assert_eq!(
            arena.turn.phase,
            TurnPhase::Action(ActionPhase {
                remaining_actions: 0,
                remaining_buys: 0,
                remaining_copper: 0
            })
        );
    }

    #[test]
    fn play_action_smithy_during_buy_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy(BuyPhase {
            remaining_buys: 1,
            remaining_copper: 0,
        });

        arena.players[0].hand.clear();
        arena.players[0].hand.push(CardKind::Smithy);
        let r = arena.play_action(0, CardKind::Smithy);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
        assert_eq!(arena.player(0).unwrap().hand.len(), 1);
    }

    #[test]
    fn play_treasure_gold_during_action_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Action(ActionPhase {
            remaining_actions: 1,
            remaining_buys: 0,
            remaining_copper: 0,
        });

        arena.players[0].hand.clear();
        arena.players[0].hand.push(CardKind::Gold);
        let r = arena.play_treasure(0, CardKind::Gold);

        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), Error::WrongTurnPhase);
        assert_eq!(arena.player(0).unwrap().hand.len(), 1);
    }

    #[test]
    fn play_treasuse_gold_during_buy_phase() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        arena.turn.phase = TurnPhase::Buy(BuyPhase {
            remaining_buys: 1,
            remaining_copper: 0,
        });

        arena.players[0].hand.clear();
        arena.players[0].hand.push(CardKind::Gold);
        let r = arena.play_treasure(0, CardKind::Gold);

        assert!(r.is_ok());
        assert_eq!(arena.player(0).unwrap().hand.len(), 0);
        assert_eq!(
            arena.turn.phase,
            TurnPhase::Buy(BuyPhase {
                remaining_buys: 1,
                remaining_copper: 3
            })
        );
    }
}
