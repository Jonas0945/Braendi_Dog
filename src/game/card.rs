#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Card {
    Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten,
    Jack, Queen, King, 
    Joker,
}

impl Card {
    pub fn possible_distances(&self) -> Vec<u8> {
        match self {
            Card::Ace   => vec![1,11],
            Card::Two   => vec![2],
            Card::Three => vec![3],
            Card::Four  => vec![4],
            Card::Five  => vec![5],
            Card::Six   => vec![6],
            Card::Seven => vec![7],
            Card::Eight => vec![8],
            Card::Nine  => vec![9],
            Card::Ten   => vec![10],
            Card::Jack  => vec![],
            Card::Queen => vec![12],
            Card::King  => vec![13],
            Card::Joker => vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
        }
    }

    pub fn value(&self) -> u8 {
        match self {
            Card::Ace => 1,
            Card::Two => 2,
            Card::Three => 3,
            Card::Four => 4,
            Card::Five => 5,
            Card::Six => 6,
            Card::Seven => 7,
            Card::Eight => 8,
            Card::Nine => 9,
            Card::Ten => 10,
            Card::Jack => 11,
            Card::Queen => 12,
            Card::King => 13,
            Card::Joker => 0,
        }
    }

    pub fn is_place_card(&self) -> bool {
        matches!(self, Card::Ace | Card:: King | Card::Joker)
    }

    pub fn is_forward_move_card(&self) -> bool {
        !matches!(self, Card::Jack | Card::Seven)
    }

    pub fn is_backward_move_card(&self) -> bool {
        matches!(self, Card::Four | Card::Joker)
    }

    pub fn allows_backward_move(&self) -> bool {
        matches!(self, Card::Four | Card::Joker)
    }
}
