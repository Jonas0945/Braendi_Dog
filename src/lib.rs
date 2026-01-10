// src/lib.rs
pub mod game;
pub mod ui;

pub use game::game::{Game, DogGame};
pub use game::color::Color;
pub use game::piece::Piece;
pub use game::action::{Action, ActionKind};
pub use game::card::Card;          
pub use game::game::GameVariant;

pub use ui::render;
