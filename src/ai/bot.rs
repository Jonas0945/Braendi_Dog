use crate::game::{Game};
use crate::Action;

pub trait Bot {
    fn decide_action(&self, game: &Game) -> Action;
}