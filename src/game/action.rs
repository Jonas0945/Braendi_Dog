use crate::player::Player;
use crate::card::Card;
use crate::board::Point;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Action {
    player: Player,
    card: Card,
    action: ActionKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionKind {
    Place,
    Move(Point, Point),
    Switch(Point, Point),
}