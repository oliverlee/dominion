use crate::dominion::arena::effect::{Effect, EffectOutput};
use crate::dominion::types::{Error, Result};
use crate::dominion::{Arena, CardKind};

pub(super) const EFFECT: &Effect = &Effect::Conditional(
    func,
    "Each other player discards down to 3 cards in their hand.",
);

fn func(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<EffectOutput> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

    // TODO: Handle games with more than 2 players.
    if player_id == arena.current_player_id {
        return error;
    }

    let mut hand = &mut arena.player(player_id).unwrap().hand.clone();

    if hand.len() <= 3 {
        if !cards.is_empty() {
            return error;
        }
    } else if hand.len() == cards.len() + 3 {
        for card in cards {
            if hand.remove_item(card).is_none() {
                return error;
            }
        }
    } else {
        return error;
    }

    let player = arena.player_mut(player_id).unwrap();
    std::mem::swap(&mut player.hand, &mut hand);
    for &card in cards {
        player.discard_pile.push(card);
    }

    Ok(EffectOutput::None)
}
