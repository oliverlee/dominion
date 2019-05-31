#[derive(Debug, Eq, Hash, PartialEq)]
pub enum CardKind {
    Copper,
    Silver,
    Gold,
    Estate,
    Duchy,
    Province,
    Curse,
    Cellar,
    Moat,
    Village,
    Merchant,
    Workshop,
    Smithy,
    Remodel,
    Militia,
    Market,
    Mine,
}


impl CardKind {
    pub fn cost(&self) -> i32 {
        match self {
            CardKind::Copper => 0,
            CardKind::Silver => 3,
            CardKind::Gold => 6,
            CardKind::Estate => 2,
            CardKind::Duchy => 5,
            CardKind::Province => 8,
            CardKind::Curse => 0,
            CardKind::Cellar => 2,
            CardKind::Moat => 2,
            CardKind::Village => 3,
            CardKind::Merchant => 3,
            CardKind::Workshop => 3,
            CardKind::Smithy => 4,
            CardKind::Remodel => 4,
            CardKind::Militia => 4,
            CardKind::Market => 5,
            CardKind::Mine => 5,
        }
    }
    pub fn is_victory(&self) -> Option<i32> {
        match self {
            CardKind::Estate => Some(1),
            CardKind::Duchy => Some(3),
            CardKind::Province => Some(6),
            CardKind::Curse => Some(-1),
            _ => None,
        }
    }
}
