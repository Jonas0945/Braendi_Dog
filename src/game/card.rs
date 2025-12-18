//#[derive(Clone, Copy, PartialEq, Eq, Debug)]
//pub enum Card {
  //  Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten,
   // Jack, Queen, King, 
  //  Joker,
//}

//impl Card {
   // pub fn value(&self) -> u8 {
      //  match self {
       //     Card::Ace   => 1,
       //     Card::Two   => 2,
       //     Card::Three => 3,
       //     Card::Four  => 4,
       //     Card::Five  => 5,
       //     Card::Six   => 6,
         //   Card::Seven => 7,
        //    Card::Eight => 8,
        //    Card::Nine  => 9,
          //  Card::Ten   => 10,
          //  Card::Jack  => 11,
     //       Card::Queen => 12,
       //     Card::King  => 13,

         use std::vec;

        //   Card::Joker => 0,
      //  }
  //  }
//}
use crate::game::action::ActionKind;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Card {
    Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Joker,
}
impl Card {
    pub fn value(&self) -> u8 {
        match self {
            Card::Ace => 1, // Oder 11
            Card::Two => 2,
            Card::Three => 3,
            Card::Four => 4,
            Card::Five => 5,
            Card::Six => 6,
            Card::Seven => 7,
            Card::Eight => 8,
            Card::Nine => 9,
            Card::Ten => 10,
            Card::Jack => 11, // Dummy Wert
            Card::Queen => 12,
            Card::King => 13,
            Card::Joker => 0,
        }
    }
pub fn possible_actions(&self) -> Vec<ActionKind> {
        match self {
            Card::Ace => vec![
                ActionKind::Place, 
                ActionKind::Move(0, 1),   
                ActionKind::Move(0, 11)   
            ],
            Card::King => vec![
                ActionKind::Place, 
                ActionKind::Move(0, 13)   
            ], 
            Card::Jack => vec![
                ActionKind::Switch(0, 0), 
                ActionKind::Move(0, 11)  
            ],
            Card::Four => vec![
                ActionKind::Move(0, 0)      
            ],
            Card::Seven => vec![
                ActionKind::Split(vec![])           
            ],
            Card::Joker => vec![
                ActionKind::Place, 
                ActionKind::Switch(0,0),
                ActionKind::Move(0,0),
                ActionKind::Split(vec![]) 
            ],
            
            _ => vec![
                ActionKind::Move(0, self.value() as u8) 
            ],
        }
    }
}
