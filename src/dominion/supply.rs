use std::collections::HashMap;
use crate::dominion::kingdom::{CardId, Kingdom};

type CardPiles = HashMap<CardId, usize>;

#[derive(Debug)]
pub struct Supply {
    kingdom_cards: CardPiles,
    base_cards: CardPiles,
}

fn starting_size(card: CardId, num_players: usize) -> usize{
    // TODO different size depending on victory/num_players
    match card {
        "Copper" => 60 - 7*num_players,
        "Silver" => 40,
        "Gold" => 30,
        "Estate" => 8 + 4*((num_players > 2) as usize),
        "Duchy" => 8 + 4*((num_players > 2) as usize),
        "Province" => 8 + 4*((num_players > 2) as usize),
        "Curse" => 10 * (num_players - 1),
        _ => 10,
    }
}

impl Supply {
    pub fn new(kingdom: Kingdom, num_players: usize) -> Supply {
        let mut kingdom_cards = CardPiles::new();
        let mut base_cards = CardPiles::new();

        let insert_card = |card_pile: &mut CardPiles, card_id| {
            card_pile.insert(card_id, starting_size(card_id, num_players))
        };

        for k in ["Copper",
                  "Silver",
                  "Gold",
                  "Estate",
                  "Duchy",
                  "Province",
                  "Curse"].iter() {
            insert_card(&mut base_cards, k);
        }

        for k in kingdom.iter() {
            insert_card(&mut kingdom_cards, k);
        }

        Supply {
            kingdom_cards,
            base_cards,
        }
    }
}
