#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
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
    pub fn action(&self) -> Option<&'static CardEffect> {
        match self {
            CardKind::Cellar => Some(&CardEffect {
                card: 0, // FIXME
                action: 1,
                buy: 0,
                worth: 0,
            }),
            CardKind::Moat => Some(&CardEffect {
                card: 2,
                action: 0,
                buy: 0,
                worth: 0,
            }),
            CardKind::Village => Some(&CardEffect {
                card: 1,
                action: 2,
                buy: 0,
                worth: 0,
            }),
            CardKind::Merchant => Some(&CardEffect {
                card: 1,
                action: 1,
                buy: 0,
                worth: 0, // FIXME
            }),
            CardKind::Smithy => Some(&CardEffect {
                card: 3,
                action: 0,
                buy: 0,
                worth: 0,
            }),
            CardKind::Militia => Some(&CardEffect {
                card: 3,
                action: 0,
                buy: 0,
                worth: 0,
            }),
            CardKind::Market => Some(&CardEffect {
                card: 1,
                action: 1,
                buy: 1,
                worth: 1,
            }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct CardEffect {
    pub card: i32,
    pub action: i32,
    pub buy: i32,
    pub worth: i32,
}
