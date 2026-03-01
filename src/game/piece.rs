use serde::{Deserialize, Serialize};

/// Comments by Sebastian Servos
/// This module defines the Piece struct, which represents a piece on the game board. 
/// Each piece has an owner (indicated by the player's index) and a boolean flag indicating whether it left the starting area.


#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Piece {
    pub owner: usize,
    pub left_start: bool,
}

impl Piece {
    pub fn new(owner: usize) -> Self {
        Self {
            owner,
            left_start: false,
        }
    }
}
