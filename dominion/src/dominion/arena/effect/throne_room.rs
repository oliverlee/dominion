use crate::dominion::arena::effect::{CardActionQueue, Effect, EffectResult};
use crate::dominion::types::{CardSpecifier, Error, Location};
use crate::dominion::{Arena, CardKind};

pub(super) const EFFECT: &Effect =
    &Effect::Conditional(func, "You may play an Action card from your hand twice.");

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> EffectResult {
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
            .hand
            .iter()
            .position(|&hand_card| hand_card == cards[0])
        {
            Some(i) => card_index = Some(CardSpecifier::Index(i)),
            None => return error,
        };
    } else {
        return error;
    }

    if let Some(card_index) = card_index {
        arena
            .move_card(
                Location::Hand { player_id },
                Location::Play { player_id },
                card_index,
            )
            .unwrap();

        let mut actions = CardActionQueue::from_card(cards[0]);
        actions.add_card(cards[0]);

        Ok(Some(actions))
    } else {
        Ok(None)
    }
}
