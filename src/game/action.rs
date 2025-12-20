use std::{fmt::{Display, Pointer}, str::FromStr, vec};

use super::card::Card;
use super::board::Point;
use super::color::Color;

#[derive(Clone,  PartialEq, Eq, Debug)]
pub struct Action {
    pub player: Color,
    pub card: Card,
    pub action: ActionKind,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ActionKind {
    Place,
    Move(Point, Point),
    Switch(Point, Point),
    Split(Vec<(Point, u8)>), 

    Exchange(),
}

impl FromStr for Action {
    type Err = &'static str;

    /// Example inputs:
    /// "G 0 P" - Green places piece with Joker (= 0)
    /// "Y 4 M 16 20" - Yellow moves from 16 to 20 with 4
    /// "B 11 S 40 45" - Blue switches between 40 and 45 with Jack (= 11)
    /// "Y 0 E" - Yellow wants so exchange his/her Joker with Green
    
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
                if parts.len() != 3 {
                    return Err("Invalid place format");
                }
                
                ActionKind::Place
            }
            "M" => {
                if parts.len() != 5 {
                    return Err("Invalid move format");
                }

                let from: Point = parts[3].parse().map_err(|_| "Invalid from point")?;
                let to: Point = parts[4].parse().map_err(|_| "Invalid to point")?;
                
                ActionKind::Move(from, to)
            }
            "S" => {
                if parts.len() != 5 {
                    return Err("Invalid move format");
                }

                let from: Point = parts[3].parse().map_err(|_| "Invalid from point")?;
                let to: Point = parts[4].parse().map_err(|_| "Invalid to point")?;

                ActionKind::Switch(from, to)
            }

            "E" => {
                if parts.len() != 3 {
                    return Err("Invalid exchange format");

                }
                                    ActionKind::Exchange()


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

        let action_str = match &self.action {
            ActionKind::Place => format!("P"),
            ActionKind::Move(from, to) => format!("M {from} {to}"),
            ActionKind::Switch(from, to) => format!("S {from} {to}"),
            ActionKind::Split(moves) => {
                let details: Vec<String> = moves.iter()
                    .map(|(f,s)|format!("{}:{}", f, s))
                    .collect();
                format!("SPLIT {}", details.join(""))
            }
            ActionKind::Exchange() => format!("E"),
        };

        write!(f, "{player_str} {card_str} {action_str}")
    }
}
