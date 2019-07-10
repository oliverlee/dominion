use super::prelude::*;

pub(super) const EFFECT: &Effect = &Effect::Unconditional(discard);

fn discard(arena: &mut Arena, _: usize, _: CardKind) -> Outcome {
    let top_index: Option<usize> = match arena.current_player().draw_pile.len() {
        0 => None,
        x => Some(x),
    };

    top_index
        .map(|x| {
            let player = arena.current_player_mut();
            player
                .draw_pile
                .move_index(&mut player.discard_pile, x)
                .map(|card| {
                    if card.is_action() {
                        Outcome::Effect(SECONDARY_EFFECT)
                    } else {
                        Outcome::None
                    }
                })
                .unwrap_or(Outcome::None)
        })
        .unwrap_or(Outcome::None)
}

#[allow(clippy::non_ascii_literal)]
const SECONDARY_EFFECT: &Effect = &Effect::Conditional(
    select,
    "Discard the top card of your deck. If it’s an Action card, you may play it.",
);

fn select(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(
        &SECONDARY_EFFECT.description(),
    ));

    if player_id != arena.current_player_id {
        return error;
    }

    let play_card;

    // Player chooses to play the discarded card with cards: [discarded_card].
    // Player chooses not to play the discarded card with cards: [].
    // Other inputs are treated as errors.
    if cards.is_empty() {
        play_card = false;
    } else if cards.len() == 1 {
        if cards[0] == *arena.current_player().discard_pile.last().unwrap() {
            play_card = true;
        } else {
            return error;
        }
    } else {
        return error;
    }

    if play_card {
        Ok(Outcome::Actions(CardActionQueue::from_card(cards[0])))
    } else {
        Ok(Outcome::None)
    }
}

#[cfg(test)]
mod test {
    use super::super::test_util;
    use super::*;
    use crate::dominion::types::Error;
    use crate::dominion::{Arena, CardKind};

    #[test]
    fn discard_empty_draw_and_discard_pile() {
        let mut arena = test_util::setup_arena();
        let ignored_player_id = arena.current_player_id;
        let ignored_card = CardKind::Silver;

        let player = arena.current_player_mut();
        player.discard_pile.clear();
        player.draw_pile.clear();

        assert_eq!(
            discard(&mut arena, ignored_player_id, ignored_card),
            Outcome::None
        );
    }

    #[test]
    fn discard_non_empty_draw_discard_pile_non_action() {
        let mut arena = test_util::setup_arena();
        let ignored_player_id = arena.current_player_id;
        let ignored_card = CardKind::Silver;

        let player = arena.current_player_mut();
        player.discard_pile.clear();
        player.draw_pile.clear();
        player.draw_pile.push(CardKind::Province);

        assert!(!CardKind::Province.is_action());
        assert_eq!(
            discard(&mut arena, ignored_player_id, ignored_card),
            Outcome::None
        );
    }

    #[test]
    fn discard_non_empty_draw_discard_pile_action() {
        let mut arena = test_util::setup_arena();
        let ignored_player_id = arena.current_player_id;
        let ignored_card = CardKind::Silver;

        let player = arena.current_player_mut();
        player.discard_pile.clear();
        player.draw_pile.clear();
        player.draw_pile.push(CardKind::Smithy);

        assert!(CardKind::Smithy.is_action());
        assert_eq!(
            discard(&mut arena, ignored_player_id, ignored_card),
            Outcome::Effect(SECONDARY_EFFECT)
        );
    }

    fn setup_select() -> Arena {
        let mut arena = test_util::setup_arena();

        let player = arena.current_player_mut();
        player.draw_pile.clear();
        player.discard_pile.clear();
        player.discard_pile.push(CardKind::Smithy);

        arena
    }

    #[test]
    fn select_choose_to_play_card() {
        let mut arena = setup_select();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Smithy];

        assert_eq!(
            select(&mut arena, player_id, &cards),
            Ok(Outcome::Actions(CardActionQueue::from_card(cards[0])))
        );
    }

    #[test]
    fn select_choose_not_to_play_card() {
        let mut arena = setup_select();
        let player_id = arena.current_player_id;

        let cards = [];

        assert_eq!(select(&mut arena, player_id, &cards), Ok(Outcome::None));
    }

    #[test]
    fn select_choose_to_play_wrong_card() {
        let mut arena = setup_select();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Militia];

        assert_eq!(
            select(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(
                SECONDARY_EFFECT.description()
            ))
        );
    }

    #[test]
    fn select_choose_to_play_wrong_cards() {
        let mut arena = setup_select();
        let player_id = arena.current_player_id;

        let cards = [CardKind::Smithy, CardKind::Smithy];

        assert_eq!(
            select(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(
                SECONDARY_EFFECT.description()
            ))
        );
    }
}
