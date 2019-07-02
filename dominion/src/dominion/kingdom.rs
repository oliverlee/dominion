use crate::dominion::CardKind;

pub type Kingdom = &'static [CardKind; 10];

#[allow(clippy::module_name_repetitions)]
#[derive(Copy, Clone, Debug)]
pub enum KingdomSet {
    FirstGame,
}

impl KingdomSet {
    pub fn cards(self) -> Kingdom {
        match self {
            KingdomSet::FirstGame => &[
                CardKind::Cellar,
                CardKind::Moat,
                CardKind::Village,
                CardKind::Merchant,
                CardKind::Workshop,
                CardKind::Smithy,
                CardKind::Remodel,
                CardKind::Militia,
                CardKind::Market,
                CardKind::Mine,
            ],
        }
    }
}
