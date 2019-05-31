mod dominion;

use dominion::{Arena, KingdomSet};

use dominion::{CardKind};

fn main() {
    println!("Hello, world!");

    let arena = Arena::new(KingdomSet::FirstGame, 2);

    println!("game state: {:#?}", arena.supply());

    println!("market action: {:#?}", CardKind::Market.action());
    println!("milita action: {:#?}", CardKind::Militia.action());
    println!("curse action: {:#?}", CardKind::Curse.action());
}
