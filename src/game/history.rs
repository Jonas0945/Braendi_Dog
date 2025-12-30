use crate::game::color::Color;
use crate::game::card::Card;
use super::action::*;

pub struct HistoryEntry {
    pub action: Action,

    pub beaten_piece_color: Option<Color>,
    pub interchanged_piece_color: Option<Color>,
    pub placed_piece_color: Option<Color>,

    pub split_rest_before: Option<u8>,
    pub trade_buffer_before: Vec<(Color, Card)>,
    pub left_start_before: bool,

    pub cards_dealt: Vec<(Color, Vec<Card>)>
}


