use super::color::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub color: Color,      
    pub id: u8,           
    pub left_start: bool, 
}

impl Piece {
    pub fn new(color: Color, id: u8) -> Self {
        Self { 
            color, 
            id, 
            left_start: false 
        }
    }
    
    pub fn new_test(color: Color, id: u8, left_start: bool) -> Self {
         Self { color, id, left_start }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn id(&self) -> u8 {
        self.id
    }
}
