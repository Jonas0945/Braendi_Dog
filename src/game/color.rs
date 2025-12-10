#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
}

impl Color {
    pub fn next(self) -> Color {
        match self {
            Color::Red => Color::Green,
            Color::Green => Color::Blue,
            Color::Blue => Color::Yellow,
            Color::Yellow => Color::Red,
        }
    }
}




