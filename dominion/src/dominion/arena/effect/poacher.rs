use super::prelude::*;

pub(super) const EFFECT: &Effect = &Effect::Unconditional(check_discard);

fn check_discard(arena: &mut Arena, _: usize, _: CardKind) -> Outcome {
    if empty_count(arena) > 0 {
        Outcome::Effect(SECONDARY_EFFECT)
    } else {
        Outcome::None
    }
}

pub(super) const SECONDARY_EFFECT: &Effect =
    &Effect::Conditional(discard, "Discard a card per empty Supply pile.");

fn discard(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    if cards.len() == empty_count(arena) {
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

fn empty_count(arena: &Arena) -> usize {
    let n = arena.supply.iter().filter(|(_, &count)| count == 0).count();

    // Player cannot discard more cards than in hand.
    std::cmp::min(arena.current_player().hand.len(), n)
}

#[cfg(test)]
mod test {
    use super::super::test_util;
    use super::*;
    use crate::dominion::arena::supply::Supply;
    use crate::dominion::types::Error;
    use crate::dominion::CardKind;

    impl Supply {
        fn empty(&mut self, card: CardKind) {
            let (_, supply_count) = self.find(card).and_then(|i| self.get_entry_mut(i)).unwrap();

            *supply_count = 0;
        }
    }

    #[test]
    fn check_discard_no_empty_piles() {
        let mut arena = test_util::setup_arena();
        let ignored_player_id = arena.current_player_id;
        let ignored_card = CardKind::Copper;

        assert_eq!(
            check_discard(&mut arena, ignored_player_id, ignored_card),
            Outcome::None
        );
    }

    #[test]
    fn check_discard_1_empty_pile() {
        let mut arena = test_util::setup_arena();
        let ignored_player_id = arena.current_player_id;
        let ignored_card = CardKind::Copper;

        arena.supply.empty(CardKind::Silver);

        assert_eq!(
            check_discard(&mut arena, ignored_player_id, ignored_card),
            Outcome::Effect(SECONDARY_EFFECT)
        );
    }

    #[test]
    fn discard_nothing_no_empty_piles() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;
        let cards = [];

        assert_eq!(discard(&mut arena, player_id, &cards), Ok(Outcome::None));
    }

    #[test]
    fn discard_nothing_1_empty_pile() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;
        let cards = [];

        arena.supply.empty(CardKind::Province);

        assert_eq!(
            discard(&mut arena, player_id, &cards),
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
            discard(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
    }

    #[test]
    fn discard_1_card_not_in_hand_1_empty_pile() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Silver];

        arena.supply.empty(CardKind::Duchy);
        arena.current_player_mut().hand.clear();

        assert_eq!(
            discard(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
    }

    #[test]
    fn discard_1_card_1_empty_pile() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Silver];

        arena.supply.empty(CardKind::Duchy);
        arena.current_player_mut().hand.clear();
        arena.current_player_mut().hand.push(cards[0]);

        assert_eq!(discard(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().hand, cardvec![]);
        assert_eq!(
            arena.current_player().discard_pile,
            cardvec![CardKind::Silver]
        );
    }

    #[test]
    fn discard_nothing_empty_hand_1_empty_pile() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [];

        arena.supply.empty(CardKind::Duchy);
        arena.current_player_mut().hand.clear();

        assert_eq!(discard(&mut arena, player_id, &cards), Ok(Outcome::None));
    }
}
