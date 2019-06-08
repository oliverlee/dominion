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
}

pub type Result<T> = std::result::Result<T, Error>;
