use std::{fmt::Display, str::FromStr};

use super::card::Card;
use super::board::Point;
use super::color::Color;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Action {
    pub player: Color,
    pub card: Card,
    pub action: ActionKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionKind {
    Place { target_player: usize },
    Move { from: Point, to: Point },
    Interchange { a: Point, b: Point },
    Split { from: Point, to: Point },
    Trade,
    Remove,
}

impl FromStr for Action {
    type Err = &'static str;

    /// Example inputs:
    /// "G 0 P" - Green places piece with Joker (= 0)
    /// "Y 4 M 16 20" - Yellow moves from 16 to 20 with 4
    /// "B 11 I 40 45" - Blue interchangees between 40 and 45 with Jack (= 11)
    /// "Y 0 T" - Yellow wants so trade his/her Joker with Green
    /// "R 7 R" - Red removes 7 from his hand
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
            "13" => Card::King,
            "12" => Card::Queen,
            "11" => Card::Jack,
            "10" => Card::Ten,
            "9" => Card::Nine,
            "8" => Card::Eight,
            "7" => Card::Seven,
            "6" => Card::Six,
            "5" => Card::Five,
            "4" => Card::Four,
            "3" => Card::Three,
            "2" => Card::Two,
            "1" => Card::Ace,
            "0" => Card::Joker,
            _ => return Err("Invalid card"),
        };

        let action = match parts[2] {
            "P" => {
                if parts.len() != 4 {
                    return Err("Invalid place format");
                }

                let target_player: Point = parts[3].parse().map_err(|_| "Invalid to point")?; 
                
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

                let a: Point = parts[3].parse().map_err(|_| "Invalid from point")?;
                let b: Point = parts[4].parse().map_err(|_| "Invalid to point")?;

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
            _ => return Err("Invalid action type"),
        };

        Ok(Action { player, card, action })
    }
}

impl Display for Action {
    fn fmt (&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let player_str = match self.player {
            Color::Red => "R",
            Color::Green => "G",
            Color::Blue => "B",
            Color::Yellow => "Y",
            Color::Purple => "P",
            Color::Orange => "O",
        };

        let card_str = match self.card {
            Card::King => "King",
            Card::Queen => "Queen",
            Card::Jack => "Jack",
            Card::Ten => "10",
            Card::Nine => "9",
            Card::Eight => "8",
            Card::Seven => "7",
            Card::Six => "6",
            Card::Five => "5",
            Card::Four => "4",
            Card::Three => "3",
            Card::Two => "2",
            Card::Ace => "Ace",
            Card::Joker => "Joker",
        };

        let action_str = match self.action {
            ActionKind::Place { target_player } => format!("P {target_player}"),
            ActionKind::Move { from, to } => format!("M {from} {to}"),
            ActionKind::Interchange { a, b } => format!("I {a} {b}"),
            ActionKind::Trade => format!("T"),
            ActionKind::Split { from, to } => format!("S {from} {to}"),
            ActionKind::Remove => format!("R"),
        };

        write!(f, "{player_str} {card_str} {action_str}")
    }
}