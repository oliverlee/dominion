use super::prelude::*;

// "Gain a silver card; put it on top of your deck."
pub(super) const EFFECT_A: &Effect = &Effect::Unconditional(gain_silver);

fn gain_silver(arena: &mut Arena, _: usize, _: CardKind) -> Outcome {
    // This can fail if the supply count for Silver is empty but it doesn't matter.
    let _ = arena
        .supply
        .move_card(&mut current_player!(arena).draw_pile, CardKind::Silver);

    Outcome::None
}

pub(super) const EFFECT_B: &Effect = &Effect::Conditional (
    reveal_victory_card,
    "Each other player reveals a Victory card from his hand and puts it on his deck (or reveals a hand with no Victory cards)."
);

fn reveal_victory_card(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT_B.description()));

    if player_id == arena.current_player_id {
        return error;
    }

    if cards.len() > 1 {
        error
    } else if cards.len() == 1 {
        if cards[0].is_victory() {
            // TODO: Reveal card to other players
            let player = arena.player_mut(player_id).unwrap();
            let _ = player
                .hand
                .move_card(&mut player.draw_pile, cards[0])
                .unwrap();
            Ok(Outcome::None)
        } else {
            error
        }
    } else {
        // No card selected
        if arena
            .player(player_id)
            .unwrap()
            .hand
            .iter()
            .any(|&card| card.is_victory())
        {
            // Player can reveal a victory card but didn't.
            error
        } else {
            // TODO: Reveal hand to other players
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
    fn reveal_multiple_cards() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        let cards = [CardKind::Copper, CardKind::Copper];

        assert_eq!(
            reveal_victory_card(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT_B.description()))
        );
    }

    #[test]
    fn reveal_nothing_but_could_have() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        let cards = [];

        arena
            .player_mut(player_id)
            .unwrap()
            .hand
            .push(CardKind::Estate);

        assert!(CardKind::Estate.is_victory());
        assert_eq!(
            reveal_victory_card(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT_B.description()))
        );
    }

    #[test]
    fn reveal_nothing_and_could_not_have() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();
        let cards = [];

        arena.player_mut(player_id).unwrap().hand.clear();

        assert_eq!(
            reveal_victory_card(&mut arena, player_id, &cards),
            Ok(Outcome::None)
        );
    }

    #[test]
    fn reveal_non_victory_card() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        let cards = [CardKind::Silver];

        arena.player_mut(player_id).unwrap().hand.push(cards[0]);

        assert!(!cards[0].is_victory());
        assert_eq!(
            reveal_victory_card(&mut arena, player_id, &cards),
            Err(Error::UnresolvedActionEffect(EFFECT_B.description()))
        );
    }

    #[test]
    fn reveal_valid_card() {
        let mut arena = test_util::setup_arena();
        let player_id = arena.next_player_id();

        let cards = [CardKind::Duchy];

        arena.player_mut(player_id).unwrap().hand.push(cards[0]);

        println!("{:#?}", arena.player(player_id).unwrap());
        assert!(cards[0].is_victory());
        assert_ne!(
            arena.player(player_id).unwrap().draw_pile.last().unwrap(),
            &cards[0]
        );
        assert_eq!(
            reveal_victory_card(&mut arena, player_id, &cards),
            Ok(Outcome::None)
        );
        println!("{:#?}", arena.player(player_id).unwrap());
        assert_eq!(
            arena.player(player_id).unwrap().draw_pile.last().unwrap(),
            &cards[0]
        );
    }
}
