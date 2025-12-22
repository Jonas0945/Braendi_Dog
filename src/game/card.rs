#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Card {
    Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten,
    Jack, Queen, King, 
    Joker,
}

impl Card {
    pub fn possible_distances(&self) -> Option<Vec<u8>> {
        match self {
            Card::Ace   => Some(vec![1,11]),
            Card::Two   => Some(vec![2]),
            Card::Three => Some(vec![3]),
            Card::Four  => Some(vec![4]),
            Card::Five  => Some(vec![5]),
            Card::Six   => Some(vec![6]),
            Card::Seven => Some(vec![7]),
            Card::Eight => Some(vec![8]),
            Card::Nine  => Some(vec![9]),
            Card::Ten   => Some(vec![10]),
            Card::Jack  => None,
            Card::Queen => Some(vec![12]),
            Card::King  => Some(vec![13]),
            Card::Joker => Some(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],)
        }
    }
}
