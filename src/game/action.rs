use std::str::FromStr;

use crate::game::action;

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
    Place,
    Move(Point, Point),
    Switch(Point, Point),
}

impl FromStr for Action {
    type Err = &'static str;

    /// Example inputs:
    /// "G 0 P" - Green place piece with Joker (= 0)
    /// "Y 4 M 16 20" - Yellow moves from 16 to 20 with 4
    /// "B 11 S 40 45" - Blue switches between 40 and 45 with Jack (= 11)
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 4 {
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
            _ => return Err("Invalid action type"),
        };

        Ok(Action { player, card, action })
    }
}