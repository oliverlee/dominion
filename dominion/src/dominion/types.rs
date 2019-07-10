use crate::dominion::CardKind;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidPlayerId,
    InvalidCard,
    InvalidIndex,
    WrongTurnPhase,
    NoMoreActions,
    NoMoreBuys,
    NoMoreCards,
    NotEnoughCopper,
    UnresolvedActionEffect(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

pub type CardVec = Vec<CardKind>;
pub type CardPiles = HashMap<CardKind, usize>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Location {
    Draw {
        player_id: usize,
    },
    Discard {
        player_id: usize,
    },
    Hand {
        player_id: usize,
    },
    Play {
        player_id: usize,
    },
    #[allow(dead_code)]
    Stage {
        player_id: usize,
    },
    Supply,
    Trash,
}

type CardVecView<'a> = &'a [CardKind];
type CardPilesIter<'a> = std::collections::hash_map::Iter<'a, CardKind, usize>;
pub type CardPilesView<'a> = std::iter::Chain<CardPilesIter<'a>, CardPilesIter<'a>>;

#[derive(Debug)]
pub enum LocationView<'a> {
    Ordered(CardVecView<'a>),
    Unordered(CardPilesView<'a>),
}

// These methods are only used by tests.
impl<'a> LocationView<'a> {
    #[allow(dead_code)]
    pub fn unwrap_ordered(self) -> CardVecView<'a> {
        match self {
            LocationView::Ordered(cards) => cards,
            _ => panic!("cannot unwrap LocationView::Unordered as LocationView::Ordered"),
        }
    }

    #[allow(dead_code)]
    pub fn unwrap_unordered(self) -> CardPilesView<'a> {
        match self {
            LocationView::Unordered(cards) => cards,
            _ => panic!("cannot unwrap LocationView::Ordered as LocationView::Unordered"),
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum CardSpecifier {
    #[allow(dead_code)]
    Top,
    Index(usize),
    Card(CardKind),
}
