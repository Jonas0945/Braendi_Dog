use super::color::Color;
use super::card::Card;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Player {
    pub color: Color,
    pub available_ids: Vec<u8>,
    pub pieces_in_house: u8,
    pub cards: Vec<Card>, 
    pub swapped_cards_count: u8,
}

impl Player {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            available_ids: vec![0,1,2,3],            
            pieces_in_house: 0,
            cards: Vec::new(),
            swapped_cards_count: 0,
        }
    }

    pub fn pieces_to_place(&self) -> u8 {
        self.available_ids.len() as u8
    }

    pub fn take_next_piece_id(&mut self) -> Option<u8> {
        self.available_ids.sort();
        if self.available_ids.is_empty(){
            None
        }else {
            Some(self.available_ids.remove(0))
        }
    }

    pub fn return_piece_id(&mut self, id: u8){
        self.available_ids.push(id);
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
