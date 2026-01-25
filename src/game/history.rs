use crate::game::card::Card;
use super::action::*;

pub struct HistoryEntry {
    pub action: Action,
    pub played_card_index: Option<usize>,

    pub beaten_piece_owner: Option<usize>,
    pub interchanged_piece_owner: Option<(usize, usize)>,
    pub placed_piece_owner: Option<usize>,

    pub split_rest_before: Option<u8>,
    pub trade_buffer_before: Vec<(usize, usize, Card)>,
    pub left_start_before: bool,

    pub cards_dealt: Vec<(usize, Vec<Card>)>,

    pub grabbed_from_player: Option<usize>,
    pub grabbed_card: Option<Card>,
    pub grabbed_card_index: Option<usize>,
}


