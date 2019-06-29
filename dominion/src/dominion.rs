pub mod arena;
pub use self::arena::Arena;

pub mod card {
    include!(concat!(env!("OUT_DIR"), "/card.rs"));
}
pub use self::card::CardKind;

pub mod kingdom;
pub use self::kingdom::KingdomSet;

pub(crate) mod player;

pub mod supply;
pub use self::supply::Supply;

pub mod turn_phase;

pub mod types;
pub use self::types::{Error, Result};

pub(crate) mod effect;

pub mod command;
