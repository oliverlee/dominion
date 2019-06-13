#![feature(vec_remove_item)]

mod dominion;

use crate::dominion::turn_phase::TurnPhase;
use crate::dominion::types::CardVec;
use crate::dominion::{Arena, CardKind, KingdomSet, Result};
use std::io;

fn main() {
    let mut arena = Arena::new(KingdomSet::FirstGame, 2);
    println!(
        "Starting game with kingdom {:#?}\n",
        arena.supply().kingdom_cards
    );

    while !arena.supply().is_game_over() {
        println!("\n{:?}\n", arena.turn);

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        match process_command(command, &arena) {
            Some(command) => {
                let r = match command {
                    Command::ViewLocation(f) => {
                        let player_id = arena.turn.player_id;
                        f(&mut arena, player_id).map(|cards| {
                            println!("{:?}", cards);
                            ()
                        })
                    }
                    Command::EndPhase(f) => f(&mut arena),
                    Command::PlayOrBuyCard(f, card) => f(&mut arena, card),
                };

                if let Err(e) = r {
                    println!("Error: {:?}", e);
                    print_help();
                }
            }
            None => {
                println!("Not a valid command.");
                print_help();
            }
        }
    }
}

fn print_help() {
    println!(
        "Valid commands:\n\
         hand - view the player's hand\n\
         discard_pile - view the player's discard pile\n\
         play_zone - view the player's play zone\n\
         end - ends the current phase (action or buy)\n\
         play <card>\n\
         buy <card>"
    );
}

type LocFunc = fn(&Arena, usize) -> Result<&CardVec>;
type PhaseFunc = fn(&mut Arena) -> Result<()>;
type CardFunc = fn(&mut Arena, CardKind) -> Result<()>;

enum Command {
    ViewLocation(LocFunc),
    EndPhase(PhaseFunc),
    PlayOrBuyCard(CardFunc, CardKind),
}

fn process_command(command: String, arena: &Arena) -> Option<Command> {
    let mut iter = command.split_whitespace();

    match iter.next() {
        Some("hand") => Some(Command::ViewLocation(Arena::hand)),
        Some("discard_pile") => Some(Command::ViewLocation(Arena::discard_pile)),
        Some("play_zone") => Some(Command::ViewLocation(Arena::play_zone)),
        Some("end") => {
            let f = match arena.turn.phase {
                TurnPhase::Action(_) => Arena::end_action_phase,
                TurnPhase::Buy(_) => Arena::end_buy_phase,
            };
            Some(Command::EndPhase(f))
        }
        Some("play") => {
            let f = match arena.turn.phase {
                TurnPhase::Action(_) => Arena::play_action,
                TurnPhase::Buy(_) => Arena::play_treasure,
            };
            create_card_func(f, iter.next())
        }
        Some("buy") => create_card_func(Arena::buy_card, iter.next()),
        _ => None,
    }
}

fn parse_card(card: Option<&str>) -> serde_json::Result<CardKind> {
    serde_json::from_str(&format!("\"{}\"", card.unwrap_or("")))
}

fn create_card_func(func: CardFunc, card: Option<&str>) -> Option<Command> {
    match parse_card(card) {
        Ok(card) => Some(Command::PlayOrBuyCard(func, card)),
        Err(e) => {
            println!("Error: {}", e);
            None
        }
    }
}
