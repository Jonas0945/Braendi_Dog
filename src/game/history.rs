use crate::game::color::Color;

use super::action::*;

pub struct HistoryEntry {
    pub action: Action,
    pub beaten_piece_color: Option<Color>,
    pub switched_piece_color: Option<Color>,
}

