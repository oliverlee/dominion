use crate::dominion::arena::effect::{Effect, Outcome};
use crate::dominion::types::{CardSpecifier, Error, Location, Result};
use crate::dominion::{Arena, CardKind};

pub(super) const EFFECT: &Effect = &Effect::Conditional(
    func,
    "Look through your discard pile. You may put a card from it onto your deck.",
);

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    let card_index;

    if cards.is_empty() {
        card_index = None;
    } else if cards.len() == 1 {
        match arena
            .current_player()
            .discard_pile
            .iter()
            .position(|&card| card == cards[0])
        {
            Some(i) => card_index = Some(CardSpecifier::Index(i)),
            None => return error,
        }
    } else {
        return error;
    }

    if let Some(card_index) = card_index {
        arena
            .move_card(
                Location::Discard { player_id },
                Location::Draw { player_id },
                card_index,
            )
            .unwrap();
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
