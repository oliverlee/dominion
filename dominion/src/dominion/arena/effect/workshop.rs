use super::prelude::*;

pub(super) const EFFECT: &Effect = &Effect::Conditional(func, "Gain a card costing up to $4.");

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    if cards.len() > 1 {
        error
    } else if cards.len() == 1 {
        if cards[0].cost() > 4 {
            error
        } else {
            arena
                .move_card(
                    Location::Supply,
                    Location::Discard { player_id },
                    CardSpecifier::Card(cards[0]),
                )
                .and(Ok(Outcome::None))
                .or(error)
        }
    } else {
        // No card selected
        if arena
            .view(Location::Supply)
            .unwrap()
            .unwrap_unordered()
            .any(|(&card, &count)| (card.cost() <= 4) && (count > 0))
        {
            // Card candidates are available but player didn'tselect one.
            error
        } else {
            // Card candidates are not available
            Ok(Outcome::None)
        }
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
        assert_eq!(arena.current_player().discard_pile, vec![CardKind::Silver]);
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
        assert_eq!(arena.current_player().discard_pile, vec![]);
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
        assert_eq!(arena.current_player().discard_pile, vec![]);
    }

    #[test]
    fn gain_no_card_without_valid_candidates() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [];

        {
            let candidates: Vec<_> = arena
                .view(Location::Supply)
                .unwrap()
                .unwrap_unordered()
                .map(|(card, _)| *card)
                .collect();

            // Set count to zero for all valid choices in the supply
            for card in candidates {
                if card.cost() <= 4 {
                    *arena.supply.get_mut(card).unwrap() = 0;
                }
            }
        }

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().discard_pile, vec![]);
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
        assert_eq!(arena.current_player().discard_pile, vec![]);
    }
}
