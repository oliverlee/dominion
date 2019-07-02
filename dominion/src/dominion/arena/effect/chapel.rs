use crate::dominion::arena::effect::{Effect, EffectResult};
use crate::dominion::types::Error;
use crate::dominion::{Arena, CardKind};

pub(super) const EFFECT: &Effect =
    &Effect::Conditional(func, "Trash up to 4 cards from your hand.");

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> EffectResult {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    let mut hand = &mut arena.current_player().hand.clone();

    if cards.len() <= hand.len() {
        if cards.len() > 4 {
            return error;
        }

        for card in cards {
            if hand.remove_item(card).is_none() {
                return error;
            }
        }
    } else {
        return error;
    }

    let player = arena.current_player_mut();
    std::mem::swap(&mut player.hand, &mut hand);
    for &card in cards {
        arena.trash.push(card);
    }

    Ok(None)
}
