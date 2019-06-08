use crate::dominion::CardKind;

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
    UnresolvedActionStack(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

pub type CardVec = Vec<CardKind>;

pub(crate) enum Location {
    Draw { player_id: usize },
    Discard { player_id: usize },
    Hand { player_id: usize },
    Play { player_id: usize },
    Stage { player_id: usize },
    Supply,
    Trash,
}

pub(crate) enum CardSpecifier {
    Top,
    Index(usize),
    Card(CardKind),
}
