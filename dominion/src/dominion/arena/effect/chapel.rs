use super::prelude::*;

pub(super) const EFFECT: &Effect =
    &Effect::Conditional(func, "Trash up to 4 cards from your hand.");

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    let mut hand = &mut arena.current_player().hand.clone();

    if cards.len() <= hand.len() {
        if cards.len() > 4 {
            return error;
        }

        for card in cards {
            if hand.remove_item(card).is_none() {
                return error;
            }
        }
    } else {
        return error;
    }

    std::mem::swap(&mut arena.current_player_mut().hand, &mut hand);
    for &card in cards {
        arena.trash.push(card);
    }

    Ok(Outcome::None)
}

#[cfg(test)]
mod test {
    use super::super::test_util;
    use super::*;
    use crate::dominion::types::Error;
    use crate::dominion::CardKind;

    #[test]
    fn trash_0_cards() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let hand_size = arena.current_player().hand.len();
        let cards = [];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().hand.len(), hand_size);
        assert_eq!(arena.trash.len(), cards.len());
    }

    #[test]
    fn trash_1_card() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let hand = &arena.current_player().hand;
        let hand_size = hand.len();
        let cards = [hand[0]];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().hand.len(), hand_size - cards.len());
        assert_eq!(arena.trash.len(), cards.len());
    }

    #[test]
    fn trash_card_not_in_hand() {
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
        assert_eq!(arena.trash.len(), 0);
    }

    #[test]
    fn trash_4_cards() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let hand = &arena.current_player().hand;
        let hand_size = hand.len();
        let cards = [hand[0], hand[1], hand[2], hand[3]];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().hand.len(), hand_size - cards.len());
        assert_eq!(arena.trash.len(), cards.len());
    }

    #[test]
    fn trash_5_cards() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let hand = &arena.current_player().hand;
        let hand_size = hand.len();
        let cards = [hand[0], hand[1], hand[2], hand[3], hand[4]];

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
        assert_eq!(arena.current_player().hand.len(), hand_size);
        assert_eq!(arena.trash.len(), 0);
    }
}
