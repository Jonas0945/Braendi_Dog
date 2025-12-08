use super::color::Color;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece {
    color: Color,
    id: u8, // 0-3
    left_start: bool,
}

impl Piece {
    pub fn new(color: Color, id: u8) -> Self {
        Self {
            color,
            id,
            left_start: false,
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn id(&self) -> u8 {
        self.id
    }
}