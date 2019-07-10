use super::prelude::*;

pub(super) const EFFECT: &Effect = &Effect::Conditional(
    func,
    "Each other player discards down to 3 cards in their hand.",
);

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id == arena.current_player_id {
        return error;
    }

    let hand = &arena.player(player_id).unwrap().hand;

    if (hand.len() <= 3) && cards.is_empty() {
        Ok(Outcome::None)
    } else if hand.len() == cards.len() + 3 {
        let player = arena.player_mut(player_id).unwrap();
        player
            .hand
            .move_all_cards(&mut player.discard_pile, cards)
            .and(Ok(Outcome::None))
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
    fn other_player_0_cards_in_hand_discard_0() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        arena.player_mut(player_id).unwrap().hand.clear();
        let cards = [];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
    }

    #[test]
    fn other_player_1_card_in_hand_discard_0() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        while arena.player(player_id).unwrap().hand.len() > 1 {
            arena.player_mut(player_id).unwrap().hand.pop();
        }
        let cards = [];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
    }

    #[test]
    fn other_player_2_cards_in_hand_discard_0() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        while arena.player(player_id).unwrap().hand.len() > 2 {
            arena.player_mut(player_id).unwrap().hand.pop();
        }
        let cards = [];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
    }

    #[test]
    fn other_player_3_cards_in_hand_discard_0() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        while arena.player(player_id).unwrap().hand.len() > 3 {
            arena.player_mut(player_id).unwrap().hand.pop();
        }
        let cards = [];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
    }

    #[test]
    fn other_player_4_cards_in_hand_discard_1() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        while arena.player(player_id).unwrap().hand.len() > 4 {
            arena.player_mut(player_id).unwrap().hand.pop();
        }
        let cards = [arena.player(player_id).unwrap().hand[0]];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
    }

    #[test]
    fn other_player_4_cards_in_hand_discard_2() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        while arena.player(player_id).unwrap().hand.len() > 4 {
            arena.player_mut(player_id).unwrap().hand.pop();
        }
        let cards = [
            arena.player(player_id).unwrap().hand[0],
            arena.player(player_id).unwrap().hand[1],
        ];

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
    }

    #[test]
    fn other_player_5_cards_in_hand_discard_2() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        let cards = [
            arena.player(player_id).unwrap().hand[0],
            arena.player(player_id).unwrap().hand[1],
        ];

        assert_eq!(arena.player(player_id).unwrap().hand.len(), 5);
        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
    }

    #[test]
    fn other_player_cards_not_in_hand() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        let hand = &mut arena.player_mut(player_id).unwrap().hand;
        hand.clear();
        hand.push(CardKind::Copper);
        hand.push(CardKind::Copper);
        hand.push(CardKind::Copper);
        hand.push(CardKind::Copper);
        hand.push(CardKind::Copper);

        let cards = [CardKind::Silver, CardKind::Silver];

        assert_eq!(arena.player(player_id).unwrap().hand.len(), 5);
        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
    }

    #[test]
    fn other_player_not_enough_copies_in_hand() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        let hand = &mut arena.player_mut(player_id).unwrap().hand;
        hand.clear();
        hand.push(CardKind::Silver);
        hand.push(CardKind::Copper);
        hand.push(CardKind::Copper);
        hand.push(CardKind::Copper);
        hand.push(CardKind::Copper);

        let cards = [CardKind::Silver, CardKind::Silver];

        assert_eq!(arena.player(player_id).unwrap().hand.len(), 5);
        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
    }
}
