use crate::dominion::kingdom::{CardId};
use std::collections::HashMap;

type CardPiles = HashMap<CardId, usize>;

#[derive(Debug)]
pub struct Supply {
    kingdom_cards: CardPiles,
    base_cards: CardPiles,
}

const BASE_CARDS: &'static [(&'static str, &'static dyn Fn(usize) -> usize)] = &[
    ("Copper", &|n| 60 - 7 * n),
    ("Silver", &|_| 40),
    ("Gold", &|_| 30),
    ("Estate", &|n| if n > 2 { 12 } else { 8 }),
    ("Duchy", &|n| if n > 2 { 12 } else { 8 }),
    ("Province", &|n| if n > 2 { 12 } else { 8 }),
    ("Curse", &|n| 10 * (n - 1)),
];

impl Supply {
    pub fn new(kingdom_card_ids: &[CardId], num_players: usize) -> Supply {
        Supply {
            kingdom_cards: kingdom_card_ids
                .iter()
                .map(|&card_id| (card_id, 10))
                .collect(),
            base_cards: BASE_CARDS
                .iter()
                .map(|&(id, f)| (id, f(num_players)))
                .collect(),
        }
    }
}
