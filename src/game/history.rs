use crate::game::card::Card;
use super::action::*;

pub struct HistoryEntry {
    pub action: Action,

    pub beaten_piece_owner: Option<usize>,
    pub interchanged_piece_owner: Option<(usize, usize)>,
    pub placed_piece_owner: Option<usize>,

    pub split_rest_before: Option<u8>,
    pub trade_buffer_before: Vec<(usize, Card)>,
    pub left_start_before: bool,

    pub cards_dealt: Vec<(usize, Vec<Card>)>
}


