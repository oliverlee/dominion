use crate::dominion::types::{CardSpecifier, CardVec, Error, Location, Result};
use crate::dominion::Arena;
use crate::dominion::CardKind;
use std::collections::VecDeque;

#[derive(Debug)]
struct ActionCardEffect(Vec<ActionEffectFunc>);

impl ActionCardEffect {
    fn new(card: CardKind) -> ActionCardEffect {
        let mut stack = Vec::new();

        match card {
            //CardKind::Cellar => unimplemeted!(),
            //CardKind::Chapel => unimplemeted!(),
            //CardKind::Harbinger => unimplemeted!(),
            //CardKind::Vassal => unimplemeted!(),
            //CardKind::Workshop => unimplemeted!(),
            //CardKind::Bureaucrat => unimplemeted!(),
            CardKind::Militia => {
                stack.push(ActionEffectFunc::Sink(action_effect_sink_militia));
                stack.push(ActionEffectFunc::Source(&ACTION_EFFECT_SOURCE_MILITIA));
            }
            //CardKind::Moneylender => unimplemeted!(),
            //CardKind::Poacher => unimplemeted!(),
            //CardKind::Remodel => unimplemeted!(),
            CardKind::ThroneRoom => {
                stack.push(ActionEffectFunc::Sink(action_effect_sink_throne_room));
                stack.push(ActionEffectFunc::Source(&ACTION_EFFECT_SOURCE_THRONE_ROOM));
            }
            //CardKind::Bandit => unimplemeted!(),
            //CardKind::CouncilRoom => unimplemeted!(),
            //CardKind::Festival => unimplemeted!(),
            //CardKind::Laboratory => unimplemeted!(),
            //CardKind::Library => unimplemeted!(),
            //CardKind::Mine => unimplemeted!(),
            //CardKind::Sentry => unimplemeted!(),
            //CardKind::Witch => unimplemeted!(),
            //CardKind::Artisan => unimplemeted!(),
            _ => (),
        }
        stack.push(ActionEffectFunc::Function(add_card_resources_func(card)));

        ActionCardEffect(stack)
    }

    fn resolve_next(
        &mut self,
        arena: &mut Arena,
        player_id: usize,
        selected_cards: Option<&CardVec>,
    ) -> Result<AdditionalCardEffects> {
        let r = match self.0.last().unwrap() {
            ActionEffectFunc::Source(f) => {
                if selected_cards
                    .map_or_else(|| false, |cards| (f.condition)(arena, player_id, cards))
                {
                    Ok(None)
                } else {
                    Err(Error::UnresolvedActionEffect(f.description))
                }
            }
            ActionEffectFunc::Sink(f) => Ok(f(arena, player_id, &selected_cards.unwrap())),
            ActionEffectFunc::Function(f) => Ok(f(arena, player_id)),
        };

        if r.is_ok() {
            self.0.pop();
        }

        r
    }
}

#[derive(Debug)]
pub(crate) struct ActionEffect(VecDeque<ActionCardEffect>);

impl ActionEffect {
    pub(crate) fn new() -> ActionEffect {
        ActionEffect(VecDeque::new())
    }

    pub(crate) fn queue_card_effect(&mut self, card: CardKind) {
        self.0.push_back(ActionCardEffect::new(card));
    }

    pub(crate) fn is_resolved(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn resolve_condition(&self) -> &'static str {
        match self.0.front().unwrap().0.last().unwrap() {
            ActionEffectFunc::Source(x) => x.description,
            _ => panic!("Expected enum type ActionEffectFunc::Source."),
        }
    }

    pub(crate) fn resolve(
        &mut self,
        arena: &mut Arena,
        player_id: usize,
        selected_cards: Option<&CardVec>,
    ) -> Result<()> {
        let mut player_id = player_id;
        let mut selected_cards = selected_cards;

        loop {
            for _ in 0..self.0.front().map_or(0, |v| v.0.len()) {
                let cards =
                    self.0
                        .front_mut()
                        .unwrap()
                        .resolve_next(arena, player_id, selected_cards)?;

                if let Some(cards) = cards {
                    for card in cards {
                        self.queue_card_effect(card);
                    }
                };
            }

            match self.0.pop_front() {
                Some(_) => {
                    // Don't use the same selected cards in subsequent spawned action card effects.
                    player_id = arena.turn.player_id;
                    selected_cards = None;
                }
                None => break,
            }
        }

        Ok(())
    }
}

type AdditionalCardEffects = Option<CardVec>;
type SourceCondition = fn(arena: &Arena, player_id: usize, cards: &CardVec) -> bool;
type SinkFunction =
    fn(arena: &mut Arena, player_id: usize, cards: &CardVec) -> AdditionalCardEffects;
type StackFunction = Box<dyn Fn(&mut Arena, usize) -> AdditionalCardEffects>;

struct ActionSource {
    condition: SourceCondition,
    description: &'static str,
}

enum ActionEffectFunc {
    Source(&'static ActionSource),
    Sink(SinkFunction),
    Function(StackFunction),
}

impl std::fmt::Debug for ActionEffectFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ActionEffectFunc::Source(x) => {
                write!(f, "ActionEffectFunc::Source({:?})", x.description)
            }
            ActionEffectFunc::Sink(_) => write!(f, "ActionEffectFunc::Sink"),
            ActionEffectFunc::Function(_) => write!(f, "ActionEffectFunc::Function"),
        }
    }
}

fn add_card_resources_func(card: CardKind) -> StackFunction {
    Box::new(
        move |arena: &mut Arena, _: usize| -> AdditionalCardEffects {
            let resources = card.action().unwrap();
            let action_phase = arena.turn.phase.as_action_phase_mut().unwrap();

            action_phase.remaining_actions += resources.actions;
            action_phase.remaining_buys += resources.buys;
            action_phase.remaining_copper += resources.copper;

            let player = arena.current_player_mut();
            for _ in 0..resources.cards {
                player.draw_card();
            }

            None
        },
    )
}

const ACTION_EFFECT_SOURCE_MILITIA: &'static ActionSource = &ActionSource {
    condition: action_effect_source_cond_militia,
    description: "Each other player discards down to 3 cards in their hand.",
};

fn action_effect_source_cond_militia(arena: &Arena, player_id: usize, cards: &CardVec) -> bool {
    // TODO: Handle games with more than 2 players.
    if player_id == arena.next_player_id() {
        let hand = &arena.players[player_id].hand;

        if std::cmp::max(hand.len(), 3) == cards.len() + 3 {
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

fn action_effect_sink_militia(
    arena: &mut Arena,
    player_id: usize,
    cards: &CardVec,
) -> AdditionalCardEffects {
    for &card in cards {
        arena
            .move_card(
                Location::Hand { player_id },
                Location::Discard { player_id },
                CardSpecifier::Card(card),
            )
            .unwrap();
    }

    None
}

const ACTION_EFFECT_SOURCE_THRONE_ROOM: &'static ActionSource = &ActionSource {
    condition: action_effect_source_cond_throne_room,
    description: "You may play an Action card from your hand twice.",
};

fn action_effect_source_cond_throne_room(arena: &Arena, _: usize, cards: &CardVec) -> bool {
    let hand = &arena.current_player().hand;

    if cards.len() == 0 {
        true
    } else if cards.len() == 1 {
        hand.iter()
            .find(|&&hand_card| hand_card == cards[0])
            .is_some()
    } else {
        false
    }
}

fn action_effect_sink_throne_room(
    arena: &mut Arena,
    player_id: usize,
    cards: &CardVec,
) -> AdditionalCardEffects {
    if cards.len() == 1 {
        arena
            .move_card(
                Location::Hand { player_id },
                Location::Play { player_id },
                CardSpecifier::Card(cards[0]),
            )
            .unwrap();

        Some(vec![cards[0]; 2])
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dominion::turn_phase::{ActionPhase, TurnPhase};
    use crate::dominion::{Arena, KingdomSet};

    #[test]
    fn empty_stack_is_resolved() {
        let stacks = ActionEffect(VecDeque::new());
        assert!(stacks.is_resolved());
    }

    #[test]
    fn resolve_market_stack() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);
        arena.action_effect.queue_card_effect(CardKind::Market);

        let r = arena.try_resolve(0, None);

        assert_eq!(r, Ok(()));
        assert!(arena.action_effect.is_resolved());

        // Market is never played so no resources are used.
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
        arena.action_effect.queue_card_effect(CardKind::Militia);

        let r = arena.try_resolve(0, None);

        assert_eq!(
            r,
            Err(Error::UnresolvedActionEffect(
                "Each other player discards down to 3 cards in their hand."
            ))
        );
        assert!(!arena.action_effect.is_resolved());

        let discard_cards = vec![CardKind::Estate, CardKind::Copper];
        let r = arena.select_cards(0, &discard_cards);

        assert_eq!(
            r,
            Err(Error::UnresolvedActionEffect(
                "Each other player discards down to 3 cards in their hand."
            ))
        );
        assert!(!arena.action_effect.is_resolved());

        let r = arena.select_cards(1, &discard_cards);

        assert_eq!(r, Ok(()));
        assert!(arena.action_effect.is_resolved());

        assert_eq!(arena.hand(1).unwrap().len(), 3);

        // Militia is never played so no resources are used.
        assert_eq!(
            arena.turn_phase(),
            TurnPhase::Action(ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 2,
            })
        );
    }

    #[test]
    fn resolve_throne_room_stack_no_action() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);
        arena.action_effect.queue_card_effect(CardKind::ThroneRoom);

        let r = arena.try_resolve(0, None);

        assert_eq!(
            r,
            Err(Error::UnresolvedActionEffect(
                "You may play an Action card from your hand twice."
            ))
        );
        assert!(!arena.action_effect.is_resolved());

        let throne_room_action = vec![];
        let r = arena.select_cards(0, &throne_room_action);

        assert_eq!(r, Ok(()));
        assert!(arena.action_effect.is_resolved());

        assert_eq!(arena.current_player().hand.len(), 5);
        assert_eq!(
            arena.turn_phase(),
            TurnPhase::Action(ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 0,
            })
        );
    }

    #[test]
    fn resolve_throne_room_stack_smithy() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);
        arena.current_player_mut().hand.push(CardKind::Smithy);

        assert_eq!(arena.current_player().hand.len(), 6);
        assert_eq!(
            arena.turn_phase(),
            TurnPhase::Action(ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 0,
            })
        );

        arena.action_effect.queue_card_effect(CardKind::ThroneRoom);
        let throne_room_action = vec![CardKind::Smithy];
        let r = arena.select_cards(0, &throne_room_action);

        assert_eq!(r, Ok(()));
        assert!(arena.action_effect.is_resolved());

        // There are only 5 cards that can be drawn + 5 in hand.
        assert_eq!(arena.current_player().hand.len(), 10);

        // Throne Room and Smithy are never played normally so no resources are used.
        assert_eq!(
            arena.turn_phase(),
            TurnPhase::Action(ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 0,
            })
        );
    }

    #[test]
    fn resolve_throne_room_stack_militia() {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);
        arena.current_player_mut().hand.push(CardKind::Militia);

        assert_eq!(arena.current_player().hand.len(), 6);
        assert_eq!(
            arena.turn_phase(),
            TurnPhase::Action(ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 0,
            })
        );

        // Ensure that the ActionEffect is not resolved as the other player must select cards to
        // discard.
        arena.action_effect.queue_card_effect(CardKind::ThroneRoom);
        let throne_room_action = vec![CardKind::Militia];
        let r = arena.select_cards(0, &throne_room_action);
        assert_eq!(
            r,
            Err(Error::UnresolvedActionEffect(
                "Each other player discards down to 3 cards in their hand."
            ))
        );
        assert!(!arena.action_effect.is_resolved());

        // Ensure that card selection only resolves first Militia action effect.
        let discard_cards = vec![CardKind::Copper; 2];
        let r = arena.select_cards(1, &discard_cards);
        assert_eq!(
            r,
            Err(Error::UnresolvedActionEffect(
                "Each other player discards down to 3 cards in their hand."
            ))
        );
        assert!(!arena.action_effect.is_resolved());

        // No need to discard cards on the second Militia action effect.
        let discard_cards = vec![];
        let r = arena.select_cards(1, &discard_cards);
        assert_eq!(r, Ok(()));
        assert!(arena.action_effect.is_resolved());

        // Throne Room and Militia are never played normally so no resources are used.
        assert_eq!(
            arena.turn_phase(),
            TurnPhase::Action(ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 4,
            })
        );
    }
}
