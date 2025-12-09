use super::card::Card;
use super::board::Point;
use super::color::Color;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Action {
    pub player: Color,
    pub card: Card,
    pub action: ActionKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionKind {
    Place,
    Move(Point, Point),
    Switch(Point, Point),
}