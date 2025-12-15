use super::color::Color;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece {
    pub color: Color,
    pub left_start: bool,
}

impl Piece {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            left_start: false,
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }
}