use crate::dominion::arena::effect::{CardActionQueue, Effect, EffectOutput};
use crate::dominion::types::{CardSpecifier, Error, Location, Result};
use crate::dominion::{Arena, CardKind};

pub(super) const EFFECT: &Effect =
    &Effect::Conditional(func, "You may play an Action card from your hand twice.");

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<EffectOutput> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    let card_index;

    if cards.is_empty() {
        card_index = None;
    } else if cards.len() == 1 {
        if !cards[0].is_action() {
            return error;
        }

        match arena
            .current_player()
            .hand
            .iter()
            .position(|&hand_card| hand_card == cards[0])
        {
            Some(i) => card_index = Some(CardSpecifier::Index(i)),
            None => return error,
        };
    } else {
        return error;
    }

    if let Some(card_index) = card_index {
        arena
            .move_card(
                Location::Hand { player_id },
                Location::Play { player_id },
                card_index,
            )
            .unwrap();

        let mut actions = CardActionQueue::from_card(cards[0]);
        actions.add_card(cards[0]);

        Ok(EffectOutput::Actions(actions))
    } else {
        Ok(EffectOutput::None)
    }
}

#[cfg(test)]
mod test {
    use super::super::test_util;
    use super::*;
    use crate::dominion::types::Error;
    use crate::dominion::CardKind;

    #[test]
    fn no_card_selected() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [];

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Ok(EffectOutput::None)
        );
    }

    #[test]
    fn card_not_in_hand() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Militia];

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
    }

    #[test]
    fn action_card_not_in_hand() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Militia];

        assert!(cards[0].is_action());

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
    }

    #[test]
    fn action_card_in_hand() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Militia];
        arena.current_player_mut().hand.push(cards[0]);

        assert!(cards[0].is_action());

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Ok(EffectOutput::Actions({
                let mut actions = CardActionQueue::new();

                actions.add_card(cards[0]);
                actions.add_card(cards[0]);

                actions
            }))
        );
    }

    #[test]
    fn non_action_card_in_hand() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Copper];
        arena.current_player_mut().hand.push(cards[0]);

        assert!(!cards[0].is_action());

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
    }
}
