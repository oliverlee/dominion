use super::prelude::*;

pub(super) const EFFECT: &Effect =
    &Effect::Conditional(func, "You may play an Action card from your hand twice.");

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    if cards.is_empty() {
        Ok(Outcome::None)
    } else if (cards.len() == 1) && cards[0].is_action() {
        let player = arena.current_player_mut();
        player
            .hand
            .move_card(&mut player.play_zone, cards[0])
            .map(|card| {
                let mut actions = CardActionQueue::from_card(card);
                actions.add_card(card);
                Outcome::Actions(actions)
            })
            .or(error)
    } else {
        error
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

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
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
            Ok(Outcome::Actions({
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
