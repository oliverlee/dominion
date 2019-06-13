use serde::Deserialize;

#[derive(Copy, Clone, Deserialize, Debug, Eq, Hash, PartialEq)]
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
    Witch,
    ThroneRoom,
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
            CardKind::Witch => 5,
            CardKind::ThroneRoom => 4,
        }
    }

    pub fn victory_points(&self) -> Option<i32> {
        match self {
            CardKind::Estate => Some(1),
            CardKind::Duchy => Some(3),
            CardKind::Province => Some(6),
            CardKind::Curse => Some(-1),
            _ => None,
        }
    }

    pub fn treasure(&self) -> Option<i32> {
        match self {
            CardKind::Copper => Some(1),
            CardKind::Silver => Some(2),
            CardKind::Gold => Some(3),
            _ => None,
        }
    }

    // TODO: support non-standard actions
    pub fn action(&self) -> Option<&'static CardResources> {
        match self {
            CardKind::Cellar => Some(&CardResources {
                cards: 0, // FIXME
                actions: 1,
                buys: 0,
                copper: 0,
            }),
            CardKind::Moat => Some(&CardResources {
                cards: 2,
                actions: 0,
                buys: 0,
                copper: 0,
            }),
            CardKind::Village => Some(&CardResources {
                cards: 1,
                actions: 2,
                buys: 0,
                copper: 0,
            }),
            CardKind::Merchant => Some(&CardResources {
                cards: 1,
                actions: 1,
                buys: 0,
                copper: 0, // FIXME
            }),
            CardKind::Smithy => Some(&CardResources {
                cards: 3,
                actions: 0,
                buys: 0,
                copper: 0,
            }),
            CardKind::Militia => Some(&CardResources {
                cards: 0,
                actions: 0,
                buys: 0,
                copper: 2,
            }),
            CardKind::Market => Some(&CardResources {
                cards: 1,
                actions: 1,
                buys: 1,
                copper: 1,
            }),
            CardKind::Witch => Some(&CardResources {
                cards: 2,
                actions: 0,
                buys: 0,
                copper: 0,
            }),
            CardKind::ThroneRoom => Some(&CardResources {
                cards: 0,
                actions: 0,
                buys: 0,
                copper: 0,
            }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct CardResources {
    pub cards: i32,
    pub actions: i32,
    pub buys: i32,
    pub copper: i32,
}
