use crate::dominion::CardKind;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidPlayerId,
    WrongTurnPhase,
    NoMoreActions,
    NoMoreBuys,
    NoMoreCards,
    NotEnoughCopper,
    InvalidCardChoice,
    UnresolvedActionEffect(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

pub type CardVec = Vec<CardKind>;
pub type CardPiles = HashMap<CardKind, usize>;

#[derive(Debug, PartialEq)]
pub enum Location {
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    Trash,
}

type CardVecView<'a> = &'a CardVec;
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

pub(crate) enum CardSpecifier {
    #[allow(dead_code)]
    Top,
    Index(usize),
    Card(CardKind),
}
