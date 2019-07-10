use super::prelude::*;

// TODO chain unconditional -> conditional? see vassal
pub(super) const EFFECT: &Effect =
    &Effect::Conditional(func, "Discard a card per empty Supply pile.");

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    let empty_count = arena.supply.iter().filter(|(_, &count)| count == 0).count();

    // Player cannot discard more cards than in hand.
    let empty_count = std::cmp::min(arena.current_player().hand.len(), empty_count);

    if cards.len() == empty_count {
        let player = arena.current_player_mut();
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
    fn discard_nothing_no_empty_piles() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
    }

    #[test]
    fn discard_nothing_1_empty_pile() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [];

        *arena.supply.get_mut(CardKind::Duchy).unwrap() = 0;

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
    }

    #[test]
    fn discard_1_card_no_empty_piles() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Silver];

        arena.current_player_mut().hand.push(cards[0]);

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
    }

    #[test]
    fn discard_1_card_not_in_hand_1_empty_pile() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Silver];

        *arena.supply.get_mut(CardKind::Duchy).unwrap() = 0;
        arena.current_player_mut().hand.clear();

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
    }

    #[test]
    fn discard_1_card_1_empty_pile() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Silver];

        *arena.supply.get_mut(CardKind::Duchy).unwrap() = 0;
        arena.current_player_mut().hand.clear();
        arena.current_player_mut().hand.push(cards[0]);

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().hand, vec![]);
        assert_eq!(arena.current_player().discard_pile, vec![CardKind::Silver]);
    }

    #[test]
    fn discard_nothing_empty_hand_1_empty_pile() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [];

        *arena.supply.get_mut(CardKind::Duchy).unwrap() = 0;
        arena.current_player_mut().hand.clear();

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
    }
}
