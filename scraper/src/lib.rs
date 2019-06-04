use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Scrape {
    pub sets: Vec<Set>,
    pub types: Vec<String>,
    pub cards: Vec<Card>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Set {
    pub name: String,
    pub card_indices: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Cost {
    None,
    Copper(u8),
    Potion(u8),
    Special(u8),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
    pub name: String,
    pub type_indices: Vec<usize>,
    pub cost: Cost,
    pub description: String,
}
