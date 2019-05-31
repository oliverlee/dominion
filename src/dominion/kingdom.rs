pub type CardId = &'static str;
pub type Kingdom = [CardId; 5];

#[derive(Debug)]
pub enum KingdomSet {
    FirstGame,
}

impl KingdomSet {
    pub fn cards(&self) -> &Kingdom {
        match self {
            KingdomSet::FirstGame => &[
                "1",
                "2",
                "3",
                "4",
                "5"
            ],
        }
    }
}
