use crate::dominion::arena::effect::CardActionQueue;
use crate::dominion::{Arena, KingdomSet};

#[cfg(test)]
pub(super) fn setup_arena_actions() -> (Arena, CardActionQueue) {
    let mut arena = Arena::new(KingdomSet::FirstGame, 2);
    let actions = arena.actions.take().unwrap();

    (arena, actions)
}

pub(super) fn setup_arena() -> Arena {
    Arena::new(KingdomSet::FirstGame, 2)
}
