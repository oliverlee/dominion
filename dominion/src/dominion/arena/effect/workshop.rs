use super::prelude::*;

pub(super) const EFFECT: &Effect = &Effect::Conditional(func, "Gain a card costing up to $4.");

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    if cards.is_empty() {
        if arena
            .supply
            .iter()
            .any(|(card, &count)| (card.cost() <= 4) && (count > 0))
        {
            // Player could have selected a card but didn't.
            error
        } else {
            Ok(Outcome::None)
        }
    } else if (cards.len() == 1) && (cards[0].cost() <= 4) {
        arena
            .supply
            .move_card(&mut current_player!(arena).discard_pile, cards[0])
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
    fn gain_valid_card() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Silver];

        assert!(cards[0].cost() <= 4);
        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(
            arena.current_player().discard_pile,
            cardvec![CardKind::Silver]
        );
    }

    #[test]
    fn gain_card_invalid_cost() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Gold];

        assert!(cards[0].cost() > 4);
        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
        assert_eq!(arena.current_player().discard_pile, cardvec![]);
    }

    #[test]
    fn gain_no_card_with_valid_candidates() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [];

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
        assert_eq!(arena.current_player().discard_pile, cardvec![]);
    }

    #[test]
    fn gain_no_card_without_valid_candidates() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [];

        arena.supply.iter_mut().for_each(|(card, count)| {
            if card.cost() <= 4 {
                *count = 0;
            }
        });

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().discard_pile, cardvec![]);
    }

    #[test]
    fn gain_multiple_cards() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Copper, CardKind::Copper];

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
        assert_eq!(arena.current_player().discard_pile, cardvec![]);
    }
}
