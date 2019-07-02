extern crate dominion;

use dominion::{Arena, CardKind, KingdomSet, Location, Turn};

fn skip_turn(arena: &mut Arena, _: usize) {
    arena.end_turn_phase().unwrap();
    arena.end_turn_phase().unwrap();
}

fn play_all_treasures(arena: &mut Arena, player_id: usize) {
    for &card in &[CardKind::Gold, CardKind::Silver, CardKind::Copper] {
        while arena
            .view(Location::Hand { player_id })
            .unwrap()
            .unwrap_ordered()
            .iter()
            .any(|&x| x == card)
        {
            arena.play_card(card).unwrap();
        }
    }
}

fn big_money(arena: &mut Arena, player_id: usize) {
    arena.end_turn_phase().unwrap();

    play_all_treasures(arena, player_id);
    println!(
        "p{} playing: {:?}",
        player_id,
        arena.view(Location::Play { player_id }).unwrap()
    );

    arena
        .buy_card(CardKind::Province)
        .map_err(|_| arena.buy_card(CardKind::Gold))
        .map_err(|_| arena.buy_card(CardKind::Silver))
        .or_else(|_| -> Result<(), ()> {
            match arena.turn() {
                Turn::Action(_) => panic!("expected Turn::Buy but got Turn::Action."),
                Turn::Buy(buy_phase) => {
                    assert!(buy_phase.remaining_copper < CardKind::Silver.cost());
                    Ok(())
                }
            }
        })
        .unwrap();

    arena.end_turn_phase().unwrap();
}

#[test]
fn play_big_money() {
    let mut arena = Arena::new(KingdomSet::FirstGame, 2);
    let mut turn_number = 0;

    while !arena.is_game_over() {
        turn_number += 1;

        big_money(&mut arena, 0);
        skip_turn(&mut arena, 1);

        let player_id = 0;
        println!("turn {}", turn_number);
        println!(
            "p1 discard pile: {:?}",
            arena.view(Location::Discard { player_id }).unwrap()
        );
        println!(
            "p1 hand: {:?}",
            arena.view(Location::Hand { player_id }).unwrap()
        );
        println!();

        assert!(turn_number <= 50);
    }
}
