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
    
    pub fn next_color(&self) -> Color {
        match self.color {
            Color::Red => Color::Green,
            Color::Green => Color::Blue,
            Color::Blue => Color::Yellow,
            Color::Yellow => Color::Red,
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

}