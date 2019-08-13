use crate::dominion::location::Location;
use crate::dominion::types::{Error, Result};
use crate::dominion::CardKind;
use std::collections::HashMap;
use std::convert::TryFrom;

const BASE_CARDS: &[(CardKind, &'static dyn Fn(usize) -> usize)] = &[
    (CardKind::Copper, &|n| 60 - 7 * n),
    (CardKind::Silver, &|_| 40),
    (CardKind::Gold, &|_| 30),
    (CardKind::Estate, &|n| if n > 2 { 12 } else { 8 }),
    (CardKind::Duchy, &|n| if n > 2 { 12 } else { 8 }),
    (CardKind::Province, &|n| if n > 2 { 12 } else { 8 }),
    (CardKind::Curse, &|n| 10 * (n - 1)),
];

fn kingdom_card_size(card_id: CardKind, num_players: usize) -> usize {
    if card_id.is_victory() {
        if num_players > 2 {
            12
        } else {
            8
        }
    } else {
        10
    }
}

type Entry = (CardKind, usize);
type EntryRef<'a> = (CardKind, &'a usize);
type EntryMut<'a> = (CardKind, &'a mut usize);

#[derive(Debug)]
pub struct Supply {
    pub kingdom_cards: Vec<Entry>,
    pub base_cards: Vec<Entry>,
}

impl Supply {
    pub fn new(kingdom_card_ids: &'static [CardKind], num_players: usize) -> Self {
        Self {
            kingdom_cards: kingdom_card_ids
                .iter()
                .map(|&card_id| (card_id, kingdom_card_size(card_id, num_players)))
                .collect(),
            base_cards: BASE_CARDS
                .iter()
                .map(|&(id, f)| (id, f(num_players)))
                .collect(),
        }
    }

    pub fn get_entry(&self, index: usize) -> Option<EntryRef> {
        let n = self.kingdom_cards.len();

        let entry = if index < n {
            Some(&self.kingdom_cards[index])
        } else if (index - n) < self.base_cards.len() {
            Some(&self.base_cards[index - n])
        } else {
            None
        };

        entry.map(|&(k, ref v)| (k, v))
    }

    pub fn get_entry_mut(&mut self, index: usize) -> Option<EntryMut> {
        let n = self.kingdom_cards.len();

        let entry = if index < n {
            Some(&mut self.kingdom_cards[index])
        } else if (index - n) < self.base_cards.len() {
            Some(&mut self.base_cards[index - n])
        } else {
            None
        };

        entry.map(|&mut (k, ref mut v)| (k, v))
    }

    pub fn is_game_over(&self) -> bool {
        const PROVINCE_INDEX: usize = 5;

        if self.base_cards[PROVINCE_INDEX].1 == 0 {
            true
        } else {
            self.iter()
                .filter(|(_, &count)| count == 0)
                .nth(2)
                .is_some()
        }
    }

    pub fn iter(&self) -> Iter {
        Iter {
            supply: self,
            cur: 0,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            supply: self,
            cur: 0,
        }
    }

    /// Increments supply count for card as position `index` by `count` and
    /// returns the card.
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds, or supply count would be less than 0, then
    /// an error is returned.
    fn add_count(&mut self, index: usize, count: i32) -> Result<CardKind> {
        let (card, supply_count) = self.get_entry_mut(index).ok_or(Error::InvalidIndex)?;
        let sum = i32::try_from(*supply_count).unwrap() + count;
        if sum >= 0 {
            *supply_count = usize::try_from(sum).unwrap();
            Ok(card)
        } else {
            Err(Error::NoMoreCards)
        }
    }
}

pub struct Iter<'a> {
    supply: &'a Supply,
    cur: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = EntryRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.supply.get_entry(self.cur);
        self.cur += 1;
        entry
    }
}

pub struct IterMut<'a> {
    supply: &'a mut Supply,
    cur: usize,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = EntryMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.supply.get_entry_mut(self.cur);
        self.cur += 1;
        entry.map(|(card, count)| unsafe { (card, &mut *(count as *mut _)) })
    }
}

impl<'a> IntoIterator for &'a Supply {
    type Item = EntryRef<'a>;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Supply {
    type Item = EntryMut<'a>;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl Location for Supply {
    fn get(&self, index: usize) -> Option<CardKind> {
        self.get_entry(index).map(|(k, _)| k)
    }

    fn find(&self, card: CardKind) -> Option<usize> {
        self.iter().position(|(k, _)| k == card)
    }

    /// Reduces the count and returns the card at position `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds, or if count is 0.
    fn remove_unchecked(&mut self, index: usize) -> CardKind {
        let (card, count) = self.get_entry_mut(index).expect("index is out of bounds");

        if *count == 0 {
            panic!("can't reduce count past 0")
        } else {
            *count -= 1;
            card
        }
    }

    /// Reduces the count and returns the card at position `index`.
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds, or if count is 0, then an error is returned.
    fn remove(&mut self, index: usize) -> Result<CardKind> {
        self.add_count(index, -1)
    }

    /// Reduces the count for `card` and returns `card`.
    ///
    /// # Errors
    ///
    /// If `card` is not contained in `self`, or if count is 0, then an error is returned.
    fn remove_card(&mut self, card: CardKind) -> Result<CardKind> {
        self.find(card)
            .ok_or(Error::InvalidCard)
            .and_then(|i| self.add_count(i, -1))
    }

    /// Increments count for `card` and returns `card`.
    ///
    /// # Errors
    ///
    /// If `card` if not in `self`, then an error is returned.
    fn add_card(&mut self, card: CardKind) -> Result<CardKind> {
        self.find(card)
            .ok_or(Error::InvalidCard)
            .and_then(|i| self.add_count(i, 1))
    }

    /// Moves all cards specified by `indices` from `self` to `other`.
    ///
    /// # Errors
    ///
    /// If `indices` contains values that are out of bounds, or if the supply
    /// count would be reduced past 0 for the card specified by an element in
    /// `indices`, then an error is returned.
    fn move_all(&mut self, other: &mut impl Location, indices: &[usize]) -> Result<()> {
        // Find any runs in `indices`
        let mut runs = HashMap::new();
        for i in indices {
            *runs.entry(i).or_insert(0) += 1;
        }

        let valid_indices = runs.iter().all(|(&&index, &count)| {
            self.get_entry(index)
                .map_or(false, |(_, &supply_count)| supply_count >= count)
        });

        if valid_indices {
            for (&index, count) in runs {
                self.add_count(index, -(i32::try_from(count).unwrap()))
                    .unwrap();

                let card = self.get(index).unwrap();
                for _ in 0..count {
                    // `other` should not be a supply so `add_card` should not fail.
                    other.add_card(card).unwrap();
                }
            }

            Ok(())
        } else {
            Err(Error::InvalidIndex)
        }
    }

    /// Moves all cards specified by `cards` from `self` to `other`.
    ///
    /// # Errors
    ///
    /// If `cards` contains values that are not in `self`, or if the supply
    /// count would be reduced past 0 for the card specified by an element in
    /// `cards`, then an error is returned.
    fn move_all_cards(&mut self, other: &mut impl Location, cards: &[CardKind]) -> Result<()> {
        let indices = cards
            .iter()
            .map(|&card| self.find(card).ok_or(Error::InvalidCard))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        self.move_all(other, &indices).or(Err(Error::InvalidCard))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominion::KingdomSet;

    impl Supply {
        fn set_count(&mut self, card: CardKind, count: usize) {
            let (_, supply_count) = self.find(card).and_then(|i| self.get_entry_mut(i)).unwrap();

            *supply_count = count;
        }
    }

    #[test]
    fn game_not_over_full_supply() {
        let s = Supply::new(KingdomSet::FirstGame.cards(), 2);

        assert!(!s.is_game_over());
    }

    #[test]
    fn game_over_empty_province_pile() {
        let mut s = Supply::new(KingdomSet::FirstGame.cards(), 2);
        s.set_count(CardKind::Province, 0);
        assert!(s.is_game_over());
    }

    #[test]
    fn game_not_over_2_empty_piles() {
        let mut s = Supply::new(KingdomSet::FirstGame.cards(), 2);
        s.set_count(CardKind::Copper, 0);
        s.set_count(CardKind::Cellar, 0);
        assert!(!s.is_game_over());
    }

    #[test]
    fn game_over_3_empty_piles() {
        let mut s = Supply::new(KingdomSet::FirstGame.cards(), 2);
        s.set_count(CardKind::Copper, 0);
        s.set_count(CardKind::Cellar, 0);
        s.set_count(CardKind::Militia, 0);
        assert!(s.is_game_over());
    }

    #[test]
    fn test_kingdom_card_size_regular_card() {
        let regular_card = CardKind::Cellar;
        assert!(!regular_card.is_victory());

        for num_players in 2..5 {
            assert_eq!(kingdom_card_size(regular_card, num_players), 10);
        }
    }

    #[test]
    fn test_kingdom_card_size_victory_card() {
        let victory_card = CardKind::Estate;
        assert!(victory_card.is_victory());

        assert_eq!(kingdom_card_size(victory_card, 2), 8);
        assert_eq!(kingdom_card_size(victory_card, 3), 12);
        assert_eq!(kingdom_card_size(victory_card, 4), 12);
    }
}
