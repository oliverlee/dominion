use crate::dominion::{Error, Result};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Turn {
    Action(ActionPhase),
    Buy(BuyPhase),
}

impl Turn {
    pub fn new() -> Turn {
        Turn::Action(ActionPhase {
            remaining_actions: 1,
            remaining_buys: 1,
            remaining_copper: 0,
        })
    }

    pub fn as_action_phase_mut(&mut self) -> Result<&mut ActionPhase> {
        match self {
            Turn::Action(ref mut action_phase) => Ok(action_phase),
            _ => Err(Error::WrongTurnPhase),
        }
    }

    pub fn as_buy_phase_mut(&mut self) -> Result<&mut BuyPhase> {
        match self {
            Turn::Buy(ref mut buy_phase) => Ok(buy_phase),
            _ => Err(Error::WrongTurnPhase),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ActionPhase {
    pub remaining_actions: u8,
    pub remaining_buys: u8,
    pub remaining_copper: u8,
}

impl ActionPhase {
    pub fn to_buy_phase(&self) -> BuyPhase {
        BuyPhase {
            remaining_buys: self.remaining_buys,
            remaining_copper: self.remaining_copper,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct BuyPhase {
    pub remaining_buys: u8,
    pub remaining_copper: u8,
}
