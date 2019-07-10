use crate::dominion::CardKind;

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

#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Location {
    Draw { player_id: usize },
    Discard { player_id: usize },
    Hand { player_id: usize },
    Play { player_id: usize },
    Stage { player_id: usize },
    Supply,
    Trash,
}
