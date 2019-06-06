#![feature(vec_remove_item)]

mod dominion;

use dominion::arena::Location;
use dominion::{Arena, CardKind, KingdomSet};

fn main() {
    let mut arena = Arena::new(KingdomSet::FirstGame, 2);
    println!("supply {:#?}", arena.supply());

    println!("p1 hand: {:?}", arena.hand(0).unwrap());
    println!("p1 discard pile: {:?}", arena.discard_pile(0).unwrap());
    println!("");

    println!("p1 playing big money strat");
    println!("");

    let mut turn_number = 0;
    while !arena.supply().is_game_over() {
        turn_number += 1;

        big_money(&mut arena, 0);
        skip_turn(&mut arena, 1);

        println!("turn {}", turn_number);
        println!("p1 hand: {:?}", arena.hand(0).unwrap());
        println!("p1 discard pile: {:?}", arena.discard_pile(0).unwrap());
        println!("");
    }

    println!("arena: {:#?}", arena);
}

fn skip_turn(arena: &mut Arena, player_id: usize) {
    arena.end_action_phase(player_id).unwrap();
    arena.end_buy_phase(player_id).unwrap();
}

fn play_all_treasures(arena: &mut Arena, player_id: usize) {
    for &card in [CardKind::Gold, CardKind::Silver, CardKind::Copper].iter() {
        while arena
            .hand(player_id)
            .unwrap()
            .iter()
            .find(|&&x| x == card)
            .is_some()
        {
            arena.select_card(player_id, card, Location::Hand).unwrap();
        }
    }
}

fn big_money(arena: &mut Arena, player_id: usize) {
    arena.end_action_phase(player_id).unwrap();

    play_all_treasures(arena, player_id);

    let _ = arena.select_card(player_id, CardKind::Province, Location::Supply);
    let _ = arena.select_card(player_id, CardKind::Gold, Location::Supply);
    let _ = arena.select_card(player_id, CardKind::Silver, Location::Supply);

    arena.end_buy_phase(player_id).unwrap();
}
