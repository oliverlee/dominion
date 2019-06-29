pub mod arena;
pub mod card {
    include!(concat!(env!("OUT_DIR"), "/card.rs"));
}
pub mod kingdom;
pub mod turn;
pub mod types;

pub(crate) mod command;

// Re-export commonly used structs.
pub use self::arena::Arena;
pub use self::card::CardKind;
pub use self::kingdom::KingdomSet;
pub use self::types::{Error, Result};
