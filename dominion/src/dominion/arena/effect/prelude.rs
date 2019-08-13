//! The Effect Implementation Prelude
//!
//! The purpose of this module is to alleviate imports of many common effect traits
//! by adding a glob import to the card effect implementation modules:
//!
//! ```ignore
//! # #![allow(unused_imports)]
//! use super::prelude::*;
//! ```
pub(super) use crate::dominion::arena::effect::{CardActionQueue, Effect, Outcome};
pub(super) use crate::dominion::location::Location;
pub(super) use crate::dominion::types::{Error, Result};
pub(super) use crate::dominion::{Arena, CardKind};

#[cfg(test)]
pub(super) use crate::dominion::location::CardVec;
