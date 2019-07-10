use super::prelude::*;

pub(super) const EFFECT: &Effect = &Effect::Conditional(
    func,
    "Look through your discard pile. You may put a card from it onto your deck.",
);

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    if cards.is_empty() {
        Ok(Outcome::None)
    } else if cards.len() == 1 {
        let player = arena.current_player_mut();
        player
            .discard_pile
            .move_card(&mut player.draw_pile, cards[0])
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
    fn empty_discard_pile() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [];

        assert!(arena.current_player().discard_pile.is_empty());
        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
    }

    #[test]
    fn choose_not_to_move_card() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [];

        arena.current_player_mut().discard_pile.push(CardKind::Gold);

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
    }

    #[test]
    fn choose_to_move_card() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Gold];

        arena.current_player_mut().discard_pile.push(cards[0]);

        assert_ne!(arena.current_player().draw_pile.last().unwrap(), &cards[0]);
        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().draw_pile.last().unwrap(), &cards[0]);
        assert!(arena.current_player().discard_pile.is_empty());
    }

    #[test]
    fn choose_card_not_in_discard_pile() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Silver];

        arena.current_player_mut().discard_pile.push(CardKind::Gold);

        assert!(!arena.current_player().discard_pile.contains(&cards[0]));
        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
        assert_eq!(arena.current_player().discard_pile.len(), 1);
    }
}
