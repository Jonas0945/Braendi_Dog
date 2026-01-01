#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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