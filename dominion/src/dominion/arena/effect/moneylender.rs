use super::prelude::*;

pub(super) const EFFECT: &Effect = &Effect::Conditional(
    func,
    "You may trash a Copper from your hand. If you do, +$3.",
);

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    if cards.len() > 1 {
        error
    } else if cards.len() == 1 {
        if cards[0] == CardKind::Copper {
            arena
                .move_card(
                    Location::Hand { player_id },
                    Location::Trash,
                    CardSpecifier::Card(cards[0]),
                )
                .and_then(|_| {
                    arena.turn.as_action_phase_mut().unwrap().remaining_copper += 3;
                    Ok(Outcome::None)
                })
                .or(error)
        } else {
            error
        }
    } else {
        Ok(Outcome::None)
    }
}

#[cfg(test)]
mod test {
    use super::super::test_util;
    use super::*;
    use crate::dominion::types::Error;
    use crate::dominion::CardKind;

    #[test]
    fn trash_nothing() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [];

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.trash, vec![]);
    }

    #[test]
    fn trash_non_copper() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Silver];

        arena.current_player_mut().hand.push(cards[0]);

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
        assert_eq!(arena.trash, vec![]);
    }

    #[test]
    fn trash_copper_but_empty_hand() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Copper];

        arena.current_player_mut().hand.clear();

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
        assert_eq!(arena.trash, vec![]);
    }

    #[test]
    fn trash_copper() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Copper];

        arena.current_player_mut().hand.clear();
        arena.current_player_mut().hand.push(cards[0]);

        assert_eq!(func(&mut arena, player_id, &cards), Ok(Outcome::None));
        assert_eq!(arena.current_player().hand, vec![]);
        assert_eq!(arena.trash, vec![CardKind::Copper]);
    }

    #[test]
    fn trash_multiple_cards() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Copper, CardKind::Copper];

        assert_eq!(
            func(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT.description()))
        );
        assert_eq!(arena.trash, vec![]);
    }
}
