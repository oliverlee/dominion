use crate::dominion::arena::effect::{Effect, EffectResult};
use crate::dominion::types::Error;
use crate::dominion::{Arena, CardKind};

pub(super) const EFFECT: &Effect =
    &Effect::Conditional(func, "Discard any number of cards, then draw that many.");

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> EffectResult {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    if player_id != arena.current_player_id {
        return error;
    }

    let mut hand = &mut arena.current_player().hand.clone();

    if cards.len() <= hand.len() {
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
        player.discard_pile.push(card);
    }
    for _ in cards {
        player.draw_card();
    }

    Ok(None)
}
