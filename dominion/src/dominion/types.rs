use crate::dominion::CardKind;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidPlayerId,
    InactivePlayer,
    WrongTurnPhase,
    InvalidCardLocation,
    NotYetImplemented,
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
    Draw { player_id: usize },
    Discard { player_id: usize },
    Hand { player_id: usize },
    Play { player_id: usize },
    Stage { player_id: usize },
    Supply,
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

impl<'a> LocationView<'a> {
    pub fn unwrap_ordered(self) -> CardVecView<'a> {
        match self {
            LocationView::Ordered(cards) => cards,
            _ => panic!("cannot unwrap LocationView::Unordered as LocationView::Ordered"),
        }
    }

    pub fn unwrap_unordered(self) -> CardPilesView<'a> {
        match self {
            LocationView::Unordered(cards) => cards,
            _ => panic!("cannot unwrap LocationView::Ordered as LocationView::Unordered"),
        }
    }
}

pub(crate) enum CardSpecifier {
    Top,
    Index(usize),
    Card(CardKind),
}
