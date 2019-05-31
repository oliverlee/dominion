mod dominion;

use dominion::{Arena, KingdomSet};

fn main() {
    println!("Hello, world!");

    let arena = Arena::new(KingdomSet::FirstGame, 2);

    //println!("Initial arena: {:#?}", arena);

    println!("game state: {:#?}", arena.supply());
}
