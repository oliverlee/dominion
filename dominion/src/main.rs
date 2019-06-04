// Let pub silence unused warnings.
pub mod dominion;

use dominion::*;

fn generate_first_game_piles(player_count: usize) -> Vec<Pile> {
    [
        CardKind::Copper,
        CardKind::Silver,
        CardKind::Gold,
        CardKind::Estate,
        CardKind::Duchy,
        CardKind::Province,
        CardKind::Cellar,
        CardKind::Market,
        CardKind::Merchant,
        CardKind::Militia,
        CardKind::Mine,
        CardKind::Moat,
        CardKind::Remodel,
        CardKind::Smithy,
        CardKind::Village,
        CardKind::Workshop,
    ]
    .iter()
    .map(|&card| Pile {
        card,
        count: card.initial_count(player_count),
    })
    .collect()
}

fn generate_deck() -> Vec<CardKind> {
    std::iter::repeat(CardKind::Copper)
        .take(7)
        .chain(std::iter::repeat(CardKind::Estate).take(3))
        .collect()
}

fn main() {
    let mut rng = rand::thread_rng();

    let players = vec![
        Player::new(&mut rng, generate_deck()),
        Player::new(&mut rng, generate_deck()),
    ];

    let mut game = Game::new(rng, generate_first_game_piles(players.len()), players);

    game.process_event(Event::EndPhase);
    assert_eq!(1, game.turn);
    assert_eq!(0, game.current_player);

    game.process_event(Event::EndPhase);
    assert_eq!(2, game.turn);
    assert_eq!(1, game.current_player);
}
