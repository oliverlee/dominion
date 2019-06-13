use crate::dominion::{Error, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TurnPhase {
    Action(ActionPhase),
    Buy(BuyPhase),
}

impl TurnPhase {
    pub fn as_action_phase_mut(&mut self) -> Result<&mut ActionPhase> {
        match self {
            TurnPhase::Action(ref mut action_phase) => Ok(action_phase),
            _ => Err(Error::WrongTurnPhase),
        }
    }

    pub fn as_buy_phase_mut(&mut self) -> Result<&mut BuyPhase> {
        match self {
            TurnPhase::Buy(ref mut buy_phase) => Ok(buy_phase),
            _ => Err(Error::WrongTurnPhase),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionPhase {
    pub remaining_actions: i32,
    pub remaining_buys: i32,
    pub remaining_copper: i32,
}

impl ActionPhase {
    pub fn as_buy_phase(&self) -> BuyPhase {
        BuyPhase {
            remaining_buys: self.remaining_buys,
            remaining_copper: self.remaining_copper,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuyPhase {
    pub remaining_buys: i32,
    pub remaining_copper: i32,
}
