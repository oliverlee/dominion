extern crate dominion;

use dominion::{Arena, CardKind, KingdomSet, TurnPhase};

fn skip_turn(arena: &mut Arena, _: usize) {
    arena.end_action_phase().unwrap();
    arena.end_buy_phase().unwrap();
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
            arena.play_treasure(card).unwrap();
        }
    }
}

fn big_money(arena: &mut Arena, player_id: usize) {
    arena.end_action_phase().unwrap();

    play_all_treasures(arena, player_id);
    println!(
        "p{} playing: {:?}",
        player_id,
        arena.play_zone(player_id).unwrap()
    );

    arena
        .buy_card(CardKind::Province)
        .map_err(|_| arena.buy_card(CardKind::Gold))
        .map_err(|_| arena.buy_card(CardKind::Silver))
        .or_else(|_| -> Result<(), ()> {
            match arena.turn_phase() {
                TurnPhase::Action(_) => {
                    panic!("Expected TurnPhase::Buy but got TurnPhase::Action.")
                }
                TurnPhase::Buy(buy_phase) => {
                    assert!(buy_phase.remaining_copper < CardKind::Silver.cost());
                    Ok(())
                }
            }
        })
        .unwrap();

    arena.end_buy_phase().unwrap();
}

#[test]
fn play_big_money() {
    let mut arena = Arena::new(KingdomSet::FirstGame, 2);
    let mut turn_number = 0;

    while !arena.supply().is_game_over() {
        turn_number += 1;

        big_money(&mut arena, 0);
        skip_turn(&mut arena, 1);

        println!("turn {}", turn_number);
        println!("p1 discard pile: {:?}", arena.discard_pile(0).unwrap());
        println!("p1 hand: {:?}", arena.hand(0).unwrap());
        println!("");

        assert!(turn_number <= 50);
    }
}
