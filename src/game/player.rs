use crate::game::piece::Piece;

use super::color::Color;
use super::card::Card;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Player {
    pub color: Color,
    pub pieces_to_place: u8,
    pub pieces_in_house: u8,
    pub cards: Vec<Card>, 
}

impl Player {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            pieces_to_place: 4,
            pieces_in_house: 0,
            cards: Vec::new(),
        }
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

    pub fn can_control_piece(&self, piece: Piece) -> bool {
        if piece.color == self.color {
            return true;
        }

        if piece.color == self.teammate() && self.pieces_in_house == 4 {
            return true;
        }

        false
    }

}