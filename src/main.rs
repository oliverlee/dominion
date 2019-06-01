mod dominion;

use dominion::{Arena, KingdomSet};

fn main() {
    let mut arena = Arena::new(KingdomSet::FirstGame, 2);
    println!("game state: {:#?}\n", arena.supply());

    for p in arena.players() {
        p.cleanup();
    }
    println!("game players: {:#?}", arena.players());
}
