mod dominion;

use dominion::{Arena, CardKind, KingdomSet};

fn main() {
    let mut arena = Arena::new(KingdomSet::FirstGame, 2);

    for p in arena.players() {
        p.cleanup();
    }
    println!("game state: {:#?}", arena.players());

    let p = &mut arena.players()[0];

    println!("buying copper on turn 1");
    p.start_turn().unwrap();
    p.start_buy_phase().unwrap();
    p.buy_card(&CardKind::Copper).unwrap();
    p.end_turn().unwrap();

    println!("game state: {:#?}", arena.players());
}
