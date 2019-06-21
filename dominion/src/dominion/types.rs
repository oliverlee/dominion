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
pub(crate) enum Location {
    Draw { player_id: usize },
    Discard { player_id: usize },
    Hand { player_id: usize },
    Play { player_id: usize },
    Stage { player_id: usize },
    Supply,
    Trash,
}

#[derive(Debug)]
pub(crate) enum LocationContents<'a> {
    NonSupply(&'a CardVec),
    Supply(&'a CardPiles),
}

pub(crate) enum CardSpecifier {
    Top,
    Index(usize),
    Card(CardKind),
}
