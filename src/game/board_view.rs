use crate::game::{Game};

#[derive(Clone, Copy, Debug)]
pub struct BoardPieceInfo {
    pub position: usize,
    pub owner: usize,
    pub left_start: bool,
}

pub fn collect_board_pieces(game: &Game) -> Vec<BoardPieceInfo> {
    game.board.tiles
        .iter()
        .enumerate()
        .filter_map(|(position, tile)| {
            tile.as_ref().map(|p| BoardPieceInfo {
                position,
                owner: p.owner,
                left_start: p.left_start,
            })
        })
        .collect()
}