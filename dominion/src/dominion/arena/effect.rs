use crate::dominion::types::{Error, Result};
use crate::dominion::{Arena, CardKind};
use std::collections::VecDeque;

// Define the effect implementation prelude
mod prelude;

// Each card effect is defined in it's own file.
mod cellar;
mod chapel;
mod harbinger;
mod militia;
mod throne_room;
mod vassal;
mod workshop;

pub(self) enum Effect {
    Conditional(ConditionalFunction, &'static str),
    Unconditional(UnconditionalFunction),
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

impl PartialEq for Effect {
    fn eq(&self, other: &Self) -> bool {
        use Effect::*;
        // In the first two match arms, f1 and f2 are function pointer _references_.
        // e.g. &for<'r, 's> fn(
        //          &'r mut dominion::arena::Arena,
        //          usize,
        //          &'s [dominion::card::CardKind],
        //      ) -> std::result::Result<
        //          dominion::arena::effect::Outcome,
        //          dominion::types::Error,
        //      >;
        // To compare equality, we dereference and then cast the result to a regular pointer.
        match (self, other) {
            (Conditional(f1, s1), Conditional(f2, s2)) => {
                (*f1 as *const () == *f2 as *const ()) && (s1 == s2)
            }
            (Unconditional(f1), Unconditional(f2)) => *f1 as *const () == *f2 as *const (),
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(self) enum Outcome {
    Actions(CardActionQueue),
    Effect(&'static Effect),
    None,
}

type ConditionalFunction =
    fn(arena: &mut Arena, player_id: usize, cards: &[CardKind]) -> Result<Outcome>;
type UnconditionalFunction = fn(arena: &mut Arena, player_id: usize, card: CardKind) -> Outcome;

#[derive(Debug, PartialEq)]
struct CardAction {
    card: CardKind,
    effects: VecDeque<&'static Effect>,
}
impl CardAction {
    fn new(card: CardKind) -> Self {
        let mut effects = VecDeque::new();

        effects.push_back(ADD_RESOURCES_FUNC);
        match card {
            CardKind::Cellar => effects.push_back(cellar::EFFECT),
            CardKind::Chapel => effects.push_back(chapel::EFFECT),
            CardKind::Harbinger => effects.push_back(harbinger::EFFECT),
            CardKind::Vassal => effects.push_back(vassal::EFFECT),
            CardKind::Workshop => effects.push_back(workshop::EFFECT),
            //CardKind::Bureaucrat => unimplemeted!(),
            CardKind::Militia => effects.push_back(militia::EFFECT),
            //CardKind::Moneylender => unimplemeted!(),
            //CardKind::Poacher => unimplemeted!(),
            //CardKind::Remodel => unimplemeted!(),
            CardKind::ThroneRoom => effects.push_back(throne_room::EFFECT),
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

        Self { card, effects }
    }

    fn resolve(
        &mut self,
        arena: &mut Arena,
        player_id: usize,
        selected_cards: Option<&[CardKind]>,
    ) -> (Option<Error>, CardActionQueue) {
        let mut actions = CardActionQueue::new();

        while !self.effects.is_empty() {
            let result = match self.effects.front().unwrap() {
                Effect::Conditional(f, desc) => match selected_cards {
                    Some(cards) => f(arena, player_id, cards),
                    None => Err(Error::UnresolvedActionEffect(desc)),
                },
                Effect::Unconditional(f) => Ok(f(arena, player_id, self.card)),
            };

            match result {
                Ok(mut outcome) => {
                    match &mut outcome {
                        Outcome::Actions(a) => actions.append(a),
                        Outcome::Effect(e) => self.effects.push_back(e),
                        Outcome::None => (),
                    }
                    self.effects.pop_front();
                }
                Err(error) => return (Some(error), actions),
            }
        }

        (None, actions)
    }

    fn condition(&self) -> Option<&'static str> {
        self.effects.iter().find_map(|&x| {
            if let Effect::Conditional(_, desc) = *x {
                Some(desc)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub(super) struct CardActionQueue {
    actions: VecDeque<CardAction>,
}

impl CardActionQueue {
    pub(super) fn new() -> Self {
        Self {
            actions: VecDeque::new(),
        }
    }

    fn from_card(card: CardKind) -> Self {
        let mut actions = Self::new();

        actions.add_card(card);

        actions
    }

    pub(super) fn add_card(&mut self, card: CardKind) {
        self.actions.push_back(CardAction::new(card));
    }

    fn append(&mut self, other: &mut Self) {
        self.actions.append(&mut other.actions);
    }

    pub(super) fn is_resolved(&self) -> bool {
        self.actions.is_empty()
    }

    pub(super) fn resolve_condition(&self) -> Option<&'static str> {
        self.actions.iter().find_map(CardAction::condition)
    }

    pub(super) fn resolve(
        &mut self,
        arena: &mut Arena,
        player_id: usize,
        selected_cards: Option<&[CardKind]>,
    ) -> Result<()> {
        let mut player_id = player_id;
        let mut selected_cards = selected_cards;

        while !self.actions.is_empty() {
            let (error, mut spawned) =
                self.actions
                    .front_mut()
                    .unwrap()
                    .resolve(arena, player_id, selected_cards);

            self.append(&mut spawned);

            if let Some(error) = error {
                return Err(error);
            }

            // Don't use the same selected cards in subsequent spawned action card effects.
            player_id = arena.current_player_id;
            selected_cards = None;
            self.actions.pop_front();
        }

        Ok(())
    }
}

fn add_resources_func(arena: &mut Arena, _: usize, card: CardKind) -> Outcome {
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
    Outcome::None
}

const ADD_RESOURCES_FUNC: &Effect = &Effect::Unconditional(add_resources_func);

#[cfg(test)]
mod test_util;

#[cfg(test)]
mod test {
    use super::*;
    use crate::dominion::turn::{self, Turn};
    use crate::dominion::types::Location;

    #[test]
    fn empty_stack_is_resolved() {
        let stacks = CardActionQueue::new();
        assert!(stacks.is_resolved());
    }

    #[test]
    fn resolve_market_stack() {
        let (mut arena, mut actions) = test_util::setup_arena_actions();
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
        let (mut arena, mut actions) = test_util::setup_arena_actions();
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
        let (mut arena, mut actions) = test_util::setup_arena_actions();
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
        let (mut arena, mut actions) = test_util::setup_arena_actions();
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
        let (mut arena, mut actions) = test_util::setup_arena_actions();
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
