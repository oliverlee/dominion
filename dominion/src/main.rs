#![feature(vec_remove_item)]
#![feature(result_map_or_else)]

mod dominion;

use crate::dominion::command::{self, Command, ParseCommandError};
use crate::dominion::{Arena, KingdomSet, Result};

fn main() {
    let mut arena = Arena::new(KingdomSet::FirstGame, 2);

    {
        print!("Starting game with ");
        let mut iter = arena.kingdom();
        print!("{:?}", iter.next().unwrap());
        for card in iter {
            print!(", {:?}", card);
        }
        println!();
    }

    while !arena.is_game_over() {
        println!("\n{:?}\n", arena.turn());

        let mut command = String::new();
        std::io::stdin().read_line(&mut command).unwrap();

        let result: Result<()> = command.parse().map_or_else(
            |e| {
                if let ParseCommandError::InvalidCommand = e {
                    println!("{}", command::help());
                } else {
                    println!("Error {}", e);
                }

                Ok(())
            },
            |command| {
                // TODO: allow non-current player to select cards
                let player_id = arena.current_player_id();

                match command {
                    Command::View(location) => {
                        println!("{:?}", arena.view(location)?);
                        println!()
                    }
                    Command::EndPhase => {
                        arena.end_turn_phase()?;
                        println!("Starting {:?}", arena.turn());
                    }
                    Command::PlayCard(card) => {
                        arena.play_card(card)?;
                        println!("Player {} played {:?}", player_id, card);
                    }
                    Command::BuyCard(card) => {
                        arena.buy_card(card)?;
                        println!("Player {} bought {:?}", player_id, card);
                    }
                    Command::SelectCards(cards) => {
                        arena.select_cards(player_id, &cards)?;
                        println!("Player {} selected {:?}", player_id, &cards);
                    }
                };

                Ok(())
            },
        );

        if let Err(e) = result {
            println!("Error: {:?}", e);
        }
    }
}
