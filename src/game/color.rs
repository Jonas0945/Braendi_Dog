use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
    Orange,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color_str = match self {
            Color::Red => "R",
            Color::Green => "G",
            Color::Blue => "B",
            Color::Yellow => "Y",
            Color::Purple => "P",
            Color::Orange => "O",
        };
        write!(f, "{}", color_str)
    }
}
