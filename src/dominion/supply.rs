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
        let insert_card = |mut card_pile: CardPiles, (card_id, card_count)| {
            card_pile.insert(card_id, card_count);
            card_pile
        };

        Supply {
            kingdom_cards: kingdom_card_ids
                .iter()
                .map(|&card_id| (card_id, 10))
                .fold(CardPiles::new(), insert_card),
            base_cards: BASE_CARDS
                .iter()
                .map(|&(id, f)| (id, f(num_players)))
                .fold(CardPiles::new(), insert_card),
        }
    }
}
