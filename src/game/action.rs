use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

use super::board::Point;
use super::card::Card;
use super::color::Color;

/// Comments by Sebastian Servos
/// This module defines the Action struct and ActionKind enum, which represent the different types of actions that players can take in the game.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Action {
    pub player: Color,
    pub card: Option<Card>,
    pub action: ActionKind,
}

/// Represents the different types of actions that can be performed in the game, along with their specific parameters.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ActionKind {
    Place {
        target_player: usize,
    },
    Move {
        from: Point,
        to: Point,
    },
    Interchange {
        a: Point,
        b: Point,
    },
    Split {
        from: Point,
        to: Point,
    },
    Trade,
    Remove,
    Grab {
        target_card: usize,
        target_player: Color,
    },
    TradeGrab {
        target_card: usize,
    },
    Undo,
}

impl FromStr for Action {
    type Err = &'static str;

    /// Example inputs:
    /// "G 0 P " - Green places piece with Joker (= 0)
    /// "Y 4 M 16 20" - Yellow moves from 16 to 20 with 4
    /// "B 11 I 40 45" - Blue interchangees between 40 and 45 with Jack (= 11)
    /// "Y 0 T" - Yellow wants so trade his/her Joker with Green
    /// "R 7 R" - Red removes 7 from his hand
    /// "O 2 G 4 P" - Orange grabs the 4th card from opponent's hand (Purple)
    /// "B N G 3" - Blue grabs the 3rd card from right opponent during trading phase

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().eq_ignore_ascii_case("undo") {
            return Ok(Action {
                player: Color::Red, // Dummy, wird später überschrieben
                card: None,
                action: ActionKind::Undo,
            });
        }
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 3 {
            return Err("Invalid action format");
        }

        let player = match parts[0] {
            "R" => Color::Red,
            "G" => Color::Green,
            "B" => Color::Blue,
            "Y" => Color::Yellow,
            "P" => Color::Purple,
            "O" => Color::Orange,
            _ => return Err("Invalid player"),
        };

        let card = match parts[1] {
            "13" => Some(Card::King),
            "12" => Some(Card::Queen),
            "11" => Some(Card::Jack),
            "10" => Some(Card::Ten),
            "9" => Some(Card::Nine),
            "8" => Some(Card::Eight),
            "7" => Some(Card::Seven),
            "6" => Some(Card::Six),
            "5" => Some(Card::Five),
            "4" => Some(Card::Four),
            "3" => Some(Card::Three),
            "2" => Some(Card::Two),
            "1" => Some(Card::Ace),
            "0" => Some(Card::Joker),
            "N" => None,
            _ => return Err("Invalid card"),
        };

        let action = match parts[2] {
            "P" => {
                if parts.len() != 4 {
                    return Err("Invalid place format");
                }

                let target_player: Point = parts[3].parse().map_err(|_| "Invalid target player")?;

                ActionKind::Place { target_player }
            }
            "M" => {
                if parts.len() != 5 {
                    return Err("Invalid move format");
                }

                let from: Point = parts[3].parse().map_err(|_| "Invalid from point")?;
                let to: Point = parts[4].parse().map_err(|_| "Invalid to point")?;

                ActionKind::Move { from, to }
            }
            "I" => {
                if parts.len() != 5 {
                    return Err("Invalid interchange format");
                }

                let a: Point = parts[3].parse().map_err(|_| "Invalid a point")?;
                let b: Point = parts[4].parse().map_err(|_| "Invalid b point")?;

                ActionKind::Interchange { a, b }
            }

            "T" => {
                if parts.len() != 3 {
                    return Err("Invalid trade format");
                }

                ActionKind::Trade
            }

            "S" => {
                if parts.len() != 5 {
                    return Err("Invalid split format");
                }

                let from: Point = parts[3].parse().map_err(|_| "Invalid from point")?;
                let to: Point = parts[4].parse().map_err(|_| "Invalid to point")?;

                ActionKind::Split { from, to }
            }

            "R" => {
                if parts.len() != 3 {
                    return Err("Invalid remove format");
                }

                ActionKind::Remove
            }

            "G" => match card {
                None => {
                    if parts.len() != 4 {
                        return Err("Invalid grab format.");
                    }

                    let target_card = parts[3].parse().map_err(|_| "Invalid target card")?;

                    ActionKind::TradeGrab { target_card }
                }

                Some(_) => {
                    if parts.len() != 5 {
                        return Err("Invalid grab format.");
                    }

                    let target_card = parts[3].parse().map_err(|_| "Invalid target card")?;

                    let target_player = match parts[4] {
                        "R" => Color::Red,
                        "G" => Color::Green,
                        "B" => Color::Blue,
                        "Y" => Color::Yellow,
                        "P" => Color::Purple,
                        "O" => Color::Orange,
                        _ => return Err("Invalid player"),
                    };

                    ActionKind::Grab {
                        target_card,
                        target_player,
                    }
                }
            },
            _ => return Err("Invalid action type"),
        };

        Ok(Action {
            player,
            card,
            action,
        })
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let player_str = match self.player {
            Color::Red => "R",
            Color::Green => "G",
            Color::Blue => "B",
            Color::Yellow => "Y",
            Color::Purple => "P",
            Color::Orange => "O",
        };

        let card_str = match self.card {
            Some(Card::King) => "13",
            Some(Card::Queen) => "12",
            Some(Card::Jack) => "11",
            Some(Card::Ten) => "10",
            Some(Card::Nine) => "9",
            Some(Card::Eight) => "8",
            Some(Card::Seven) => "7",
            Some(Card::Six) => "6",
            Some(Card::Five) => "5",
            Some(Card::Four) => "4",
            Some(Card::Three) => "3",
            Some(Card::Two) => "2",
            Some(Card::Ace) => "1",
            Some(Card::Joker) => "0",
            None => "N",
        };

        let action_str = match self.action {
            ActionKind::Place { target_player } => format!("P {target_player}"),
            ActionKind::Move { from, to } => format!("M {from} {to}"),
            ActionKind::Interchange { a, b } => format!("I {a} {b}"),
            ActionKind::Trade => format!("T"),
            ActionKind::Split { from, to } => format!("S {from} {to}"),
            ActionKind::Remove => format!("R"),
            ActionKind::Grab {
                target_card,
                target_player,
            } => format!("G {target_card} {target_player}"),
            ActionKind::TradeGrab { target_card } => format!("G {target_card}"),
            ActionKind::Undo => format!("Undo"),
        };

        write!(f, "{player_str} {card_str} {action_str}")
    }
}
