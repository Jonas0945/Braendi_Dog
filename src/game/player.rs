use super::color::Color;
use super::card::Card;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Player {
    pub color: Color,
    pub avaiable_ids: Vec<u8>,
    pub pieces_in_house: u8,
    pub cards: Vec<Card>, 
}

impl Player {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            avaiable_ids: vec![0,1,2,3],            
            pieces_in_house: 0,
            cards: Vec::new(),
        }
    }

    pub fn pieces_to_place(&self) -> u8 {
        self.avaiable_ids.len() as u8
    }

    pub fn take_next_piece_id(&mut self) -> Option<u8> {
        self.avaiable_ids.sort();
        if self.avaiable_ids.is_empty(){
            None
        }else {
            Some(self.avaiable_ids.remove(0))
        }
    }

    pub fn return_piece_id(&mut self, id: u8){
        self.avaiable_ids.push(id);
    }

    pub fn teammate(&self) -> Color {
        match self.color {
            Color::Red => Color::Blue,
            Color::Blue => Color::Red,
            Color::Green => Color::Yellow,
            Color::Yellow => Color::Green
        }
    }

    pub fn remove_card(&mut self, card: Card) {
        if let Some(i) = self.cards.iter().position(|&c| c == card) {
            self.cards.remove(i);
        }
    }

}
