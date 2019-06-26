#![feature(vec_remove_item)]
#![feature(result_map_or_else)]

mod dominion;

use crate::dominion::command;
use crate::dominion::command::{Command, ParseCommandError};
use crate::dominion::turn_phase::TurnPhase;
use crate::dominion::{Arena, CardKind, KingdomSet, Result};
use std::convert::TryInto;
use std::str::FromStr;

fn main() {
    let mut arena = Arena::new(KingdomSet::FirstGame, 2);

    {
        print!("Starting game with ");
        let mut iter = arena.kingdom();
        print!("{:?}", iter.next().unwrap());
        for card in iter {
            print!(", {:?}", card);
        }
        print!("\n");
    }

    while !arena.is_game_over() {
        println!("\n{:?}\n", arena.turn);

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
                let player_id = arena.turn.player_id;

                match command {
                    Command::View(location) => {
                        println!("{:?}", arena.view(location)?);
                        println!()
                    }
                    Command::EndPhase => {
                        let _ = arena.end_phase()?;
                        println!("Starting {:?}", arena.turn);
                    }
                    Command::PlayCard(card) => {
                        let _ = arena.play_card(card)?;
                        println!("Player {} played {:?}", player_id, card);
                    }
                    Command::BuyCard(card) => {
                        let _ = arena.buy_card(card)?;
                        println!("Player {} bought {:?}", player_id, card);
                    }
                    Command::SelectCards(cards) => {
                        let _ = arena.select_cards(player_id, &cards)?;
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
