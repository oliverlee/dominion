use crate::dominion::types::{Error, Result};
use crate::dominion::CardKind;

pub trait Location {
    /// Returns the `CardKind` at position `index` or `None` if out of bounds.
    fn get(&self, index: usize) -> Option<CardKind>;

    /// Returns the position of the first instance of `CardKind` or `None` if not found.
    fn find(&self, card: CardKind) -> Option<usize>;

    fn remove_unchecked(&mut self, index: usize) -> CardKind;

    fn remove(&mut self, index: usize) -> Result<CardKind>;

    fn remove_card(&mut self, card: CardKind) -> Result<CardKind>;

    fn add_card(&mut self, card: CardKind) -> Result<CardKind>;

    /// Removes card at position `index` from `self` and adds it to `other`.
    ///
    /// # Panics
    ///
    /// Panics if `index` cannot be removed `self`, or if the resulting card cannot be
    /// added to `other`.
    fn move_unchecked(&mut self, other: &mut impl Location, index: usize) {
        let _ = other.add_card(self.remove_unchecked(index)).unwrap();
    }

    /// Removes card at position `index` from `self` and adds it to `other`.
    ///
    /// # Errors
    ///
    /// If `index` cannot be removed `self`, or if the resulting card cannot be
    /// added to `other`, then an error is returned.
    fn move_index(&mut self, other: &mut impl Location, index: usize) -> Result<CardKind> {
        self.remove(index).and_then(|card| other.add_card(card))
    }

    /// Removes card at position `index` from `self` and adds it to `other`.
    ///
    /// # Errors
    ///
    /// If `index` cannot be removed `self`, or if the resulting card cannot be
    /// added to `other`, then an error is returned.
    fn move_card(&mut self, other: &mut impl Location, card: CardKind) -> Result<CardKind> {
        self.remove_card(card).and_then(|card| other.add_card(card))
    }

    fn move_all(&mut self, other: &mut impl Location, indices: &[usize]) -> Result<()>;

    fn move_all_cards(&mut self, other: &mut impl Location, cards: &[CardKind]) -> Result<()>;
}

use shrinkwraprs::Shrinkwrap;

#[derive(Debug, Default, PartialEq, Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct CardVec(pub Vec<CardKind>);

#[cfg(test)]
macro_rules! cardvec {
    ($x:expr) => {
        CardVec(vec![$x])
    };
    ($elem:expr; $n:expr) => {
        CardVec(vec![$elem; $n])
    };
    ( ) => {
        CardVec(vec![])
    };
}

impl CardVec {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl Location for CardVec {
    fn get(&self, index: usize) -> Option<CardKind> {
        self.0.get(index).copied()
    }

    fn find(&self, card: CardKind) -> Option<usize> {
        self.0.iter().position(|&item| item == card)
    }

    /// Removes and returns the element at position `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    fn remove_unchecked(&mut self, index: usize) -> CardKind {
        self.0.remove(index)
    }

    /// Removes and returns the card at position `index`, shifting all cards
    /// after it to the left.
    ///
    /// # Errors
    ///
    /// If `index` is out of bounds, then an error is returned.
    fn remove(&mut self, index: usize) -> Result<CardKind> {
        if index < self.0.len() {
            Ok(self.remove_unchecked(index))
        } else {
            Err(Error::InvalidIndex)
        }
    }

    /// Removes and returns the first instance of `card`.
    ///
    /// # Errors
    ///
    /// If `card` is not contained in `self`, then an error is returned.
    fn remove_card(&mut self, card: CardKind) -> Result<CardKind> {
        match self.find(card) {
            Some(i) => Ok(self.remove_unchecked(i)),
            None => Err(Error::InvalidCard),
        }
    }

    /// Adds `card` to `self`.
    ///
    /// # Errors
    ///
    /// This implementation never returns an error.
    fn add_card(&mut self, card: CardKind) -> Result<CardKind> {
        self.0.push(card);
        Ok(card)
    }

    /// Moves all cards specified by `indices` from `self` to `other`. `indices`
    /// is sorted to ensure that index values remain consistent while moving cards.
    ///
    /// # Errors
    ///
    /// If `indices` is larger than `self`, or `indices` contains repeated
    /// values, or `indices` contains values that are out of bounds, then an
    /// error is returned.
    fn move_all(&mut self, other: &mut impl Location, indices: &[usize]) -> Result<()> {
        if indices.len() <= self.0.len() {
            let mut sorted = indices.to_vec();
            sorted.sort_unstable_by(|a, b| b.cmp(a));
            sorted.dedup();

            if sorted.len() == indices.len() {
                if sorted[0] < self.0.len() {
                    for i in sorted {
                        self.move_unchecked(other, i);
                    }
                    Ok(())
                } else {
                    // Index is out of bounds
                    Err(Error::InvalidIndex)
                }
            } else {
                // Can't use repeated indices to move cards out of a CardVec
                Err(Error::InvalidIndex)
            }
        } else {
            // Can't move more cards than location contains
            Err(Error::InvalidIndex)
        }
    }

    /// Moves all cards specified by `cards` from `self` to `other`.
    ///
    /// # Errors
    ///
    /// If `cards` is larger than `self`, or `cards` contains values not in
    /// `self`, then an error is returned.
    fn move_all_cards(&mut self, other: &mut impl Location, cards: &[CardKind]) -> Result<()> {
        if cards.len() <= self.0.len() {
            // Check if an index can be determined for each card in `cards`.
            let mut indices = Vec::new();
            let mut cards = cards.to_vec();

            for (i, card) in self.0.iter().enumerate() {
                if cards.remove_item(card).is_some() {
                    indices.push(i);
                    if cards.is_empty() {
                        break;
                    }
                }
            }

            // If `cards` is empty, then an index has been found each card.
            if cards.is_empty() {
                for &i in indices.iter().rev() {
                    self.move_unchecked(other, i);
                }
                Ok(())
            } else {
                // Can't move cards not contained in location
                Err(Error::InvalidCard)
            }
        } else {
            // Can't move more cards than location contains
            Err(Error::InvalidCard)
        }
    }
}
