use crate::dominion::arena::effect::{CardActionQueue, Effect, EffectOutput};
use crate::dominion::types::{Error, Location, Result};
use crate::dominion::{Arena, CardKind};

pub(super) const EFFECT: &Effect = &Effect::Unconditional(discard);

fn discard(arena: &mut Arena, _: usize, _: CardKind) -> EffectOutput {
    let player = arena.current_player_mut();
    match player.draw_card() {
        Some(card) => {
            let player_id = arena.current_player_id;

            arena
                .move_card(
                    Location::Hand { player_id },
                    Location::Discard { player_id },
                    card,
                )
                .unwrap();

            EffectOutput::Effect(PLAY_ACTION)
        }
        None => EffectOutput::None,
    }
}

#[allow(clippy::non_ascii_literal)]
const PLAY_ACTION: &Effect = &Effect::Conditional(
    select,
    "Discard the top card of your deck. If it’s an Action card, you may play it.",
);

fn select(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<EffectOutput> {
    let error = Err(Error::UnresolvedActionEffect(&EFFECT.description()));

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
        Ok(EffectOutput::Actions(CardActionQueue::from_card(cards[0])))
    } else {
        Ok(EffectOutput::None)
    }
}
