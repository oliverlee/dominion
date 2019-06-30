use crate::dominion::types::{CardSpecifier, Error, Location, Result};
use crate::dominion::{Arena, CardKind};
use std::collections::VecDeque;

enum Effect {
    Conditional(ConditionalEffectFunction, &'static str),
    Unconditional(UnconditionalEffectFunction),
}

impl Effect {
    fn description(&self) -> &'static str {
        match self {
            Effect::Conditional(_, desc) => desc,
            Effect::Unconditional(_) => "",
        }
    }
}

impl std::fmt::Debug for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Effect::Conditional(_, desc) => write!(f, "Effect::Conditional({:?})", desc),
            Effect::Unconditional(_) => write!(f, "Effect::Unconditional"),
        }
    }
}

type EffectResult = Result<Option<CardActionQueue>>;
type ConditionalEffectFunction =
    fn(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> EffectResult;
type UnconditionalEffectFunction =
    fn(arena: &mut Arena, player_id: usize, origin_card: CardKind) -> Option<CardActionQueue>;

#[derive(Debug)]
struct CardAction {
    card: CardKind,
    effects: Vec<&'static Effect>,
}

impl CardAction {
    fn new(card: CardKind) -> CardAction {
        let mut effects = Vec::new();

        match card {
            //CardKind::Cellar => unimplemeted!(),
            //CardKind::Chapel => unimplemeted!(),
            //CardKind::Harbinger => unimplemeted!(),
            //CardKind::Vassal => unimplemeted!(),
            //CardKind::Workshop => unimplemeted!(),
            //CardKind::Bureaucrat => unimplemeted!(),
            CardKind::Militia => {
                effects.push(MILITIA_EFFECT);
            }
            //CardKind::Moneylender => unimplemeted!(),
            //CardKind::Poacher => unimplemeted!(),
            //CardKind::Remodel => unimplemeted!(),
            CardKind::ThroneRoom => {
                effects.push(THRONE_ROOM_EFFECT);
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
        effects.push(ADD_RESOURCES_FUNC);

        CardAction { card, effects }
    }

    fn resolve(
        &mut self,
        arena: &mut Arena,
        player_id: usize,
        selected_cards: Option<&[CardKind]>,
    ) -> impl Iterator<Item = EffectResult> {
        let mut results = Vec::new();

        while !self.effects.is_empty() {
            let result = match self.effects.last().unwrap() {
                Effect::Conditional(f, desc) => match selected_cards {
                    Some(cards) => f(arena, player_id, cards),
                    None => Err(Error::UnresolvedActionEffect(desc)),
                },
                Effect::Unconditional(f) => Ok(f(arena, player_id, self.card)),
            };

            results.push(result);

            if results.last().unwrap().is_ok() {
                self.effects.pop();
            } else {
                break;
            }
        }

        results.into_iter()
    }

    fn condition(&self) -> Option<&'static str> {
        self.effects
            .iter()
            .filter_map(|&x| {
                if let Effect::Conditional(_, desc) = *x {
                    Some(desc)
                } else {
                    None
                }
            })
            .next()
    }
}

#[derive(Debug)]
pub(crate) struct CardActionQueue {
    actions: VecDeque<CardAction>,
}

impl CardActionQueue {
    pub(crate) fn new() -> CardActionQueue {
        CardActionQueue {
            actions: VecDeque::new(),
        }
    }

    fn from_card(card: CardKind) -> CardActionQueue {
        let mut actions = VecDeque::new();
        actions.push_back(CardAction::new(card));
        CardActionQueue { actions }
    }

    pub(crate) fn add_card(&mut self, card: CardKind) {
        self.actions.push_back(CardAction::new(card));
    }

    fn append(&mut self, other: &mut CardActionQueue) {
        self.actions.append(&mut other.actions);
    }

    pub(crate) fn is_resolved(&self) -> bool {
        self.actions.is_empty()
    }

    pub(crate) fn resolve_condition(&self) -> Option<&'static str> {
        self.actions.iter().filter_map(|x| x.condition()).next()
    }

    pub(crate) fn resolve(
        &mut self,
        arena: &mut Arena,
        player_id: usize,
        selected_cards: Option<&[CardKind]>,
    ) -> Result<()> {
        let mut player_id = player_id;
        let mut selected_cards = selected_cards;

        while !self.actions.is_empty() {
            let results =
                self.actions
                    .front_mut()
                    .unwrap()
                    .resolve(arena, player_id, selected_cards);

            for r in results.into_iter() {
                match r {
                    Ok(mut actions) => {
                        if let Some(ref mut actions) = actions {
                            self.append(actions);
                        }
                    }
                    Err(e) => return Err(e),
                }
            }

            // Don't use the same selected cards in subsequent spawned action card effects.
            player_id = arena.current_player_id;
            selected_cards = None;
            self.actions.pop_front();
        }

        Ok(())
    }
}

fn add_resources_func(arena: &mut Arena, _: usize, card: CardKind) -> Option<CardActionQueue> {
    if let Some(resources) = card.resources() {
        let action_phase = arena.turn.as_action_phase_mut().unwrap();

        action_phase.remaining_actions += resources.actions;
        action_phase.remaining_buys += resources.buys;
        action_phase.remaining_copper += resources.copper;

        let player = arena.current_player_mut();
        for _ in 0..resources.cards {
            player.draw_card();
        }
    }

    // Adding card resources never adds new effects to the queue.
    None
}

const ADD_RESOURCES_FUNC: &Effect = &Effect::Unconditional(add_resources_func);

const MILITIA_EFFECT: &Effect = &Effect::Conditional(
    militia_effect,
    "Each other player discards down to 3 cards in their hand.",
);

fn militia_effect(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> EffectResult {
    let error = Error::UnresolvedActionEffect(&MILITIA_EFFECT.description());

    // TODO: Handle games with more than 2 players.
    if player_id == arena.current_player_id {
        return Err(Error::UnresolvedActionEffect(&MILITIA_EFFECT.description()));
    }

    let hand = &arena.player(player_id).unwrap().hand;
    let mut hand2 = hand.clone();

    if hand.len() <= 3 {
        if !cards.is_empty() {
            return Err(error);
        }
    } else if hand.len() == cards.len() + 3 {
        // TODO: Use something more efficient.
        if !cards.iter().all(|card| hand2.remove_item(card).is_some()) {
            return Err(error);
        }
    } else {
        return Err(error);
    }

    let player = arena.player_mut(player_id).unwrap();
    std::mem::swap(&mut player.hand, &mut hand2);
    for &card in cards {
        player.discard_pile.push(card);
    }

    Ok(None)
}

const THRONE_ROOM_EFFECT: &Effect = &Effect::Conditional(
    throne_room_effect,
    "You may play an Action card from your hand twice.",
);

fn throne_room_effect(arena: &mut Arena, _: usize, cards: &[CardKind]) -> EffectResult {
    let error = Error::UnresolvedActionEffect(&THRONE_ROOM_EFFECT.description());
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
            None => return Err(error),
        };
    } else {
        return Err(error);
    }

    if let Some(card) = card_index {
        let player_id = arena.current_player_id;

        arena
            .move_card(
                Location::Hand { player_id },
                Location::Play { player_id },
                card,
            )
            .unwrap();

        let mut actions = CardActionQueue::from_card(cards[0]);
        actions.add_card(cards[0]);

        Ok(Some(actions))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dominion::turn::{self, Turn};
    use crate::dominion::{Arena, KingdomSet};

    #[test]
    fn empty_stack_is_resolved() {
        let stacks = CardActionQueue::new();
        assert!(stacks.is_resolved());
    }

    fn setup_arena_actions() -> (Arena, CardActionQueue) {
        let mut arena = Arena::new(KingdomSet::FirstGame, 2);
        let mut actions: Option<CardActionQueue> = None;

        std::mem::swap(&mut arena.actions, &mut actions);

        (arena, actions.unwrap())
    }

    #[test]
    fn resolve_market_stack() {
        let (mut arena, mut actions) = setup_arena_actions();
        actions.add_card(CardKind::Market);

        let r = actions.resolve(&mut arena, 0, None);

        assert_eq!(r, Ok(()));
        assert!(actions.is_resolved());

        // Market is never played so no resources are used.
        assert_eq!(
            arena.turn(),
            Turn::Action(turn::ActionPhase {
                remaining_actions: 2,
                remaining_buys: 2,
                remaining_copper: 1,
            })
        );
    }

    #[test]
    fn resolve_militia_stack() {
        let (mut arena, mut actions) = setup_arena_actions();
        actions.add_card(CardKind::Militia);

        let r = actions.resolve(&mut arena, 0, None);

        // Action effect must still be resolved after 'no-selection'.
        assert_eq!(
            r,
            Err(Error::UnresolvedActionEffect(
                "Each other player discards down to 3 cards in their hand."
            ))
        );
        assert!(!actions.is_resolved());

        let discard_cards: Vec<_> = arena
            .view(Location::Hand { player_id: 0 })
            .unwrap()
            .unwrap_ordered()
            .iter()
            .take(2)
            .cloned()
            .collect();
        let r = actions.resolve(&mut arena, 0, Some(&discard_cards));

        // Effect fails to resolve due to incorrect player selecting cards.
        assert_eq!(
            r,
            Err(Error::UnresolvedActionEffect(
                "Each other player discards down to 3 cards in their hand."
            ))
        );
        assert!(!actions.is_resolved());

        let discard_cards: Vec<_> = arena
            .view(Location::Hand { player_id: 1 })
            .unwrap()
            .unwrap_ordered()
            .iter()
            .take(2)
            .cloned()
            .collect();
        let r = actions.resolve(&mut arena, 1, Some(&discard_cards));

        // Effect successfully resolves.
        assert_eq!(r, Ok(()));
        assert!(actions.is_resolved());

        assert_eq!(arena.player(1).unwrap().hand.len(), 3);

        // Militia is never played so no resources are used.
        assert_eq!(
            arena.turn(),
            Turn::Action(turn::ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 2,
            })
        );
    }

    #[test]
    fn resolve_throne_room_stack_no_action() {
        let (mut arena, mut actions) = setup_arena_actions();
        actions.add_card(CardKind::ThroneRoom);

        let r = actions.resolve(&mut arena, 0, None);

        assert_eq!(
            r,
            Err(Error::UnresolvedActionEffect(
                "You may play an Action card from your hand twice."
            ))
        );
        assert!(!actions.is_resolved());

        let throne_room_action = vec![];
        let r = actions.resolve(&mut arena, 0, Some(&throne_room_action));

        assert_eq!(r, Ok(()));
        assert!(actions.is_resolved());

        assert_eq!(arena.current_player().hand.len(), 5);
        assert_eq!(
            arena.turn(),
            Turn::Action(turn::ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 0,
            })
        );
    }

    #[test]
    fn resolve_throne_room_stack_smithy() {
        let (mut arena, mut actions) = setup_arena_actions();
        arena.current_player_mut().hand.push(CardKind::Smithy);

        assert_eq!(arena.current_player().hand.len(), 6);
        assert_eq!(
            arena.turn(),
            Turn::Action(turn::ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 0,
            })
        );

        actions.add_card(CardKind::ThroneRoom);
        let throne_room_action = vec![CardKind::Smithy];
        let r = actions.resolve(&mut arena, 0, Some(&throne_room_action));

        assert_eq!(r, Ok(()));
        assert!(actions.is_resolved());

        // There are only 5 cards that can be drawn + 5 in hand.
        assert_eq!(arena.current_player().hand.len(), 10);

        // Throne Room and Smithy are never played normally so no resources are used.
        assert_eq!(
            arena.turn(),
            Turn::Action(turn::ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 0,
            })
        );
    }

    #[test]
    fn resolve_throne_room_stack_militia() {
        let (mut arena, mut actions) = setup_arena_actions();
        arena.current_player_mut().hand.push(CardKind::Militia);

        assert_eq!(arena.current_player().hand.len(), 6);
        assert_eq!(
            arena.turn(),
            Turn::Action(turn::ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 0,
            })
        );

        // Ensure that the CardActionQueue is not resolved as the other player must select cards to
        // discard.
        actions.add_card(CardKind::ThroneRoom);
        let throne_room_action = vec![CardKind::Militia];
        let r = actions.resolve(&mut arena, 0, Some(&throne_room_action));

        assert_eq!(
            r,
            Err(Error::UnresolvedActionEffect(
                "Each other player discards down to 3 cards in their hand."
            ))
        );
        assert!(!actions.is_resolved());

        // Ensure that card selection only resolves first Militia action effect.
        let discard_cards = vec![CardKind::Copper; 2];
        let r = actions.resolve(&mut arena, 1, Some(&discard_cards));

        assert_eq!(
            r,
            Err(Error::UnresolvedActionEffect(
                "Each other player discards down to 3 cards in their hand."
            ))
        );
        assert!(!actions.is_resolved());

        // No need to discard cards on the second Militia action effect.
        let discard_cards = vec![];
        let r = actions.resolve(&mut arena, 1, Some(&discard_cards));

        assert_eq!(r, Ok(()));
        assert!(actions.is_resolved());

        // Throne Room and Militia are never played normally so no resources are used.
        assert_eq!(
            arena.turn(),
            Turn::Action(turn::ActionPhase {
                remaining_actions: 1,
                remaining_buys: 1,
                remaining_copper: 4,
            })
        );
    }
}
