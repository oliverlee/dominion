use crate::dominion::types::{CardSpecifier, CardVec, Error, Location, Result};
use crate::dominion::Arena;
use crate::dominion::CardKind;

pub struct ActionEffectStack(Vec<ActionEffect>);

impl ActionEffectStack {
    pub fn new(card: CardKind) -> ActionEffectStack {
        let mut stack = Vec::new();

        match card {
            CardKind::Market => {
                stack.push(ActionEffect::Function(add_card_resources_func(
                    CardKind::Market,
                )));
            }
            CardKind::Militia => {
                stack.push(ActionEffect::Function(add_card_resources_func(
                    CardKind::Militia,
                )));
                stack.push(ActionEffect::Sink(action_effect_sink_militia));
                stack.push(ActionEffect::Source(&ACTION_EFFECT_SOURCE_MILITIA));
            }
            _ => (),
        }

        ActionEffectStack(stack)
    }

    pub fn is_resolved(&self) -> bool {
        self.0.is_empty()
    }

    pub fn resolve(
        &mut self,
        arena: &mut Arena,
        player_id: usize,
        selected_cards: Option<&CardVec>,
    ) -> Result<()> {
        loop {
            match self.0.last() {
                Some(item) => {
                    match item {
                        ActionEffect::Source(f) => {
                            if !selected_cards.map_or_else(
                                || false,
                                |cards| (f.condition)(arena, player_id, cards),
                            ) {
                                return Err(Error::UnresolvedActionStack(f.description));
                            }
                        }
                        ActionEffect::Sink(f) => f(arena, player_id, &selected_cards.unwrap()),
                        ActionEffect::Function(f) => f(arena, player_id),
                    };
                    self.0.pop();
                }
                None => break,
            };
        }

        Ok(())
    }
}

type SourceCondition = fn(arena: &Arena, player_id: usize, cards: &CardVec) -> bool;
type SinkFunction = fn(arena: &mut Arena, player_id: usize, cards: &CardVec);
type StackFunction = Box<dyn Fn(&mut Arena, usize)>;

struct ActionSource {
    condition: SourceCondition,
    description: &'static str,
}

enum ActionEffect {
    Source(&'static ActionSource),
    Sink(SinkFunction),
    Function(StackFunction),
}

fn add_card_resources_func(card: CardKind) -> StackFunction {
    Box::new(move |arena: &mut Arena, _: usize| {
        let resources = card.action().unwrap();
        let action_phase = arena.turn.phase.as_action_phase_mut().unwrap();

        action_phase.remaining_actions += resources.actions;
        action_phase.remaining_buys += resources.buys;
        action_phase.remaining_copper += resources.copper;

        let player = arena.current_player_mut();
        for _ in 0..resources.cards {
            player.draw_card();
        }
    })
}

const ACTION_EFFECT_SOURCE_MILITIA: &'static ActionSource = &ActionSource {
    condition: action_effect_source_cond_militia,
    description: "Each other player discards down to 3 cards in their hand.",
};

fn action_effect_source_cond_militia(arena: &Arena, player_id: usize, cards: &CardVec) -> bool {
    // TODO: Handle games with more than 2 players.
    if player_id == arena.next_player_id() {
        let hand = &arena.players[player_id].hand;

        if hand.len() == cards.len() + 3 {
            // TODO: Use something more efficient.
            let mut hand2 = hand.clone();
            cards.iter().all(|card| hand2.remove_item(card).is_some())
        } else {
            false
        }
    } else {
        false
    }
}

fn action_effect_sink_militia(arena: &mut Arena, player_id: usize, cards: &CardVec) {
    for &card in cards {
        arena.move_card(
            Location::Hand { player_id },
            Location::Discard { player_id },
            CardSpecifier::Card(card),
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dominion::turn_phase::{ActionPhase, TurnPhase};
    use crate::dominion::{Arena, KingdomSet};

    #[test]
    fn empty_stack_is_resolved() {
        let stack = ActionEffectStack(Vec::new());
        assert!(stack.is_resolved());
    }

    #[test]
    fn resolve_simple_stack() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        let mut stack = ActionEffectStack(vec![ActionEffect::Function(add_card_resources_func(
            CardKind::Market,
        ))]);

        assert!(!stack.is_resolved());
        assert_eq!(
            arena.turn_phase(),
            TurnPhase::Action(ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 0,
            })
        );
        assert_eq!(arena.current_player().hand.len(), 5);

        assert_eq!(stack.resolve(&mut arena, 0, None), Ok(()));
        assert!(stack.is_resolved());
        assert_eq!(
            arena.turn_phase(),
            TurnPhase::Action(ActionPhase {
                remaining_actions: 2,
                remaining_buys: 2,
                remaining_copper: 1,
            })
        );
        assert_eq!(arena.current_player().hand.len(), 6);
    }

    #[test]
    fn resolve_market_stack() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        let mut stack = ActionEffectStack::new(CardKind::Market);

        assert_eq!(stack.resolve(&mut arena, 0, None), Ok(()));
        assert!(stack.is_resolved());
        assert_eq!(
            arena.turn_phase(),
            TurnPhase::Action(ActionPhase {
                remaining_actions: 2,
                remaining_buys: 2,
                remaining_copper: 1,
            })
        );
    }

    #[test]
    fn resolve_militia_stack() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);

        let mut stack = ActionEffectStack::new(CardKind::Militia);

        assert_eq!(
            stack.resolve(&mut arena, 0, None),
            Err(Error::UnresolvedActionStack(
                "Each other player discards down to 3 cards in their hand."
            ))
        );
        assert!(!stack.is_resolved());

        println!("{:?}", arena.hand(1).unwrap());

        let discard_cards = vec![CardKind::Estate, CardKind::Copper];
        assert_eq!(
            stack.resolve(&mut arena, 0, Some(&discard_cards)),
            Err(Error::UnresolvedActionStack(
                "Each other player discards down to 3 cards in their hand."
            ))
        );
        assert!(!stack.is_resolved());

        assert_eq!(stack.resolve(&mut arena, 1, Some(&discard_cards)), Ok(()));
        assert!(stack.is_resolved());

        assert_eq!(arena.hand(1).unwrap().len(), 3);

        assert_eq!(
            arena.turn_phase(),
            TurnPhase::Action(ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 2,
            })
        );
    }
}
