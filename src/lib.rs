pub mod game;
pub mod ui;

pub use game::game::{Game, DogGame};
pub use game::color::Color;
pub use game::piece::Piece;
pub use game::action::{Action, ActionKind};

pub use ui::render;