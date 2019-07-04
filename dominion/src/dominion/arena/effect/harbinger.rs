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
