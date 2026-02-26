use super::color::Color;
use super::card::Card;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PlayerType{
    Human,
    RandomBot,
    EvalBot
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Player {
    pub player_type: PlayerType,
    pub color: Color,
    pub pieces_to_place: u8,
    pub pieces_in_house: u8,
    pub cards: Vec<Card>, 
}

impl Player {
    pub fn new(color: Color, player_type: PlayerType) -> Self {
        Self {
            player_type,
            color,
            pieces_to_place: 4,
            pieces_in_house: 0,
            cards: Vec::new(),
        }
    }

    pub fn remove_card(&mut self, card: Card) {
        if let Some(i) = self.cards.iter().position(|&c| c == card) {
            self.cards.remove(i);
        }
    }
}