use crate::dominion::types::CardPiles;
use crate::dominion::CardKind;

const BASE_CARDS: &'static [(CardKind, &'static dyn Fn(usize) -> usize)] = &[
    (CardKind::Copper, &|n| 60 - 7 * n),
    (CardKind::Silver, &|_| 40),
    (CardKind::Gold, &|_| 30),
    (CardKind::Estate, &|n| if n > 2 { 12 } else { 8 }),
    (CardKind::Duchy, &|n| if n > 2 { 12 } else { 8 }),
    (CardKind::Province, &|n| if n > 2 { 12 } else { 8 }),
    (CardKind::Curse, &|n| 10 * (n - 1)),
];

fn kingdom_card_size(card_id: CardKind, num_players: usize) -> usize {
    match card_id.victory_points() {
        Some(_) => {
            if num_players > 2 {
                12
            } else {
                8
            }
        }
        None => 10,
    }
}

#[derive(Debug)]
pub struct Supply {
    pub kingdom_cards: CardPiles,
    pub base_cards: CardPiles,
}

impl Supply {
    pub fn new(kingdom_card_ids: &'static [CardKind], num_players: usize) -> Supply {
        Supply {
            kingdom_cards: kingdom_card_ids
                .iter()
                .map(|&card_id| (card_id, kingdom_card_size(card_id, num_players)))
                .collect(),
            base_cards: BASE_CARDS
                .iter()
                .map(|(id, f)| (*id, f(num_players)))
                .collect(),
        }
    }

    pub fn get_mut(&mut self, c: CardKind) -> Option<&mut usize> {
        let r = self.base_cards.get_mut(&c);

        match r {
            Some(_) => r,
            None => self.kingdom_cards.get_mut(&c),
        }
    }

    pub fn is_game_over(&self) -> bool {
        if *self.base_cards.get(&CardKind::Province).unwrap() == 0 {
            true
        } else {
            self.base_cards
                .values()
                .chain(self.kingdom_cards.values())
                .filter(|&&x| x == 0)
                .nth(2)
                .is_some()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominion::KingdomSet;

    #[test]
    fn game_not_over_full_supply() {
        let s = Supply::new(KingdomSet::FirstGame.cards(), 2);

        assert!(!s.is_game_over());
    }

    #[test]
    fn game_over_empty_province_pile() {
        let mut s = Supply::new(KingdomSet::FirstGame.cards(), 2);
        *s.get_mut(CardKind::Province).unwrap() = 0;

        assert!(s.is_game_over());
    }

    #[test]
    fn game_not_over_2_empty_piles() {
        let mut s = Supply::new(KingdomSet::FirstGame.cards(), 2);
        *s.get_mut(CardKind::Copper).unwrap() = 0;
        *s.get_mut(CardKind::Cellar).unwrap() = 0;

        assert!(!s.is_game_over());
    }

    #[test]
    fn game_over_3_empty_piles() {
        let mut s = Supply::new(KingdomSet::FirstGame.cards(), 2);
        *s.get_mut(CardKind::Copper).unwrap() = 0;
        *s.get_mut(CardKind::Cellar).unwrap() = 0;
        *s.get_mut(CardKind::Militia).unwrap() = 0;

        assert!(s.is_game_over());
    }

    #[test]
    fn test_kingdom_card_size_regular_card() {
        let regular_card = CardKind::Cellar;
        assert!(regular_card.victory_points().is_none());

        for num_players in 2..5 {
            assert_eq!(kingdom_card_size(regular_card, num_players), 10);
        }
    }

    #[test]
    fn test_kingdom_card_size_victory_card() {
        let victory_card = CardKind::Estate;
        assert!(victory_card.victory_points().is_some());

        assert_eq!(kingdom_card_size(victory_card, 2), 8);
        assert_eq!(kingdom_card_size(victory_card, 3), 12);
        assert_eq!(kingdom_card_size(victory_card, 4), 12);
    }
}
