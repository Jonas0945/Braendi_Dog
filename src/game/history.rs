use serde::{Deserialize, Serialize};

use crate::game::card::Card;
use super::action::*;

/// Comments by Sebastian Servos
/// This module defines the HistoryEntry struct, which represents a single entry in the game's history log. 
/// Each entry contains detailed information about the action taken, the state of the game before the action, and any relevant changes to the game state (such as pieces beaten, cards played, etc.). 
/// This allows for undoing actions and turns by restoring the game state based on these history entries.

#[derive(Clone, Debug, Serialize, Deserialize)]
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
