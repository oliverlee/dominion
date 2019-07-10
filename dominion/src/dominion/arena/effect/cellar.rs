use super::prelude::*;

pub(super) const EFFECT: &Effect =
    &Effect::Conditional(func, "Discard any number of cards, then draw that many.");

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    let player = arena.current_player_mut();
    player
        .hand
        .move_all_cards(&mut player.discard_pile, cards)
        .and_then(|_| {
            for _ in cards {
                player.draw_card();
            }
            Ok(Outcome::None)
        })
        .or(error)
}

#[cfg(test)]
mod test {
    use super::super::test_util;
    use super::*;
    use crate::dominion::types::Error;
    use crate::dominion::CardKind;

    #[test]
    fn discard_0_cards() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let hand_size = arena.current_player().hand.len();
        let cards = [];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().hand.len(), hand_size);
        assert_eq!(arena.current_player().discard_pile.len(), cards.len());
    }

    #[test]
    fn discard_1_card() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let hand = &arena.current_player().hand;
        let hand_size = hand.len();
        let cards = [hand[0]];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().hand.len(), hand_size);
        assert_eq!(arena.current_player().discard_pile.len(), cards.len());
    }

    #[test]
    fn discard_1_card_empty_deck_and_discard() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let hand = &arena.current_player().hand;
        let hand_size = hand.len();
        let cards = [hand[0]];

        let player = arena.current_player_mut();
        player.draw_pile.clear();
        player.discard_pile.clear();

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().hand.len(), hand_size);
        assert_eq!(arena.current_player().discard_pile.len(), 0);
    }

    #[test]
    fn discard_card_not_in_hand() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let hand = &arena.current_player().hand;
        let hand_size = hand.len();
        let cards = [CardKind::Gold];

        assert!(!hand.contains(&cards[0]));
        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
        assert_eq!(arena.current_player().hand.len(), hand_size);
        assert_eq!(arena.current_player().discard_pile.len(), 0);
    }

    #[test]
    fn discard_5_cards() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let hand = &arena.current_player().hand;
        let hand_size = hand.len();
        let cards = [hand[0], hand[1], hand[2], hand[3], hand[4]];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().hand.len(), hand_size);
        assert_eq!(arena.current_player().discard_pile.len(), cards.len());
    }

    #[test]
    fn discard_more_cards_than_in_hand() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let hand = &arena.current_player().hand;
        let hand_size = hand.len();
        let cards = [hand[0], hand[1], hand[2], hand[3], hand[4], hand[4]];

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
        assert_eq!(arena.current_player().hand.len(), hand_size);
        assert_eq!(arena.current_player().discard_pile.len(), 0);
    }
}
