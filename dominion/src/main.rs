#![feature(vec_remove_item)]

mod dominion;

use crate::dominion::turn_phase::TurnPhase;
use crate::dominion::{Arena, CardKind, KingdomSet};

fn main() {
    let mut arena = Arena::new(KingdomSet::FirstGame, 2);
    println!("supply {:#?}", arena.supply());
}
