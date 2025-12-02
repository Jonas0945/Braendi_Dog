use super::player::Player;
use super::card::Card;
use super::board::Point;

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