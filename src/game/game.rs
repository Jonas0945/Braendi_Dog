use core::error;

use crate::game::card;
use crate::game::player;

use super::piece::*;
use super::action::*;
use super::color::*;
use super::deck::*;
use super::card::*;
use super::player::*;

const CARDS_PER_ROUND: [u8;4] = [5,4,3,2];

pub struct Game {
    board: [Option<Piece>; 80],
    history: Vec<Action>,
    round: u8,

    deck: Deck,
    discard: Vec<Card>,

    red: Player,
    green: Player,
    blue: Player,
    yellow: Player,

    current_player_color: Color,
    swapping_phase: bool,
    swap_buffer: Vec<(Player, Card)>
}


pub trait DogGame {
    // Creates new instance with an empty board and initialized deck and players
    fn new() -> Self;

    // Returns the current state of the board
    fn board_state(&self) -> &[Option<Piece>; 80];

    // Returns the current player
    fn current_player(&self) -> &Player;

    // Matches and applies the action of playing the given card for the current player
    fn action(&mut self, action: Action) -> Result<(), &'static str>;

    // Undoes the last action
    fn undo(&mut self) -> Result<(), &'static str>;

    // Returns the current state of the board
    fn board(&self) -> &[Option<Piece>; 80];

    // Gives players new cards and lets theem swap one card
    fn new_round(&mut self);
    
    // Is called by new_round() and swaps two cards in between team members
    fn swap_cards(&mut self)-> &mut Self; 

    // Checks if there is yet a winning team
    fn is_winner(&self) -> bool;
}

impl DogGame for Game {
    fn new() -> Self {
        Self {
            board: [None; 80],
            history: Vec::new(),
            round: 0,

            deck: Deck::new(),
            discard: Vec::new(),

            red: Player::new(Color::Red),
            green: Player::new(Color::Green),
            blue: Player::new(Color::Blue),
            yellow: Player::new(Color::Yellow),

            current_player_color: Color::Red,
            swapping_phase: true,
            swap_buffer: Vec::new(),
        }
    }

    fn current_player(&self) -> &Player {
        match self.current_player_color {
            Color::Red => &self.red,
            Color::Green => &self.green,
            Color::Blue => &self.blue,
            Color::Yellow => &self.yellow,
        }
    }
    
    fn board_state(&self) -> &[Option<Piece>; 80] {
        &self.board
    }

    fn action(&mut self, mut action: Action) -> Result<(), &'static str> {
        match action.action{
            ActionKind::Place => todo!(),
            ActionKind::Move(_, _) => todo!(),
            ActionKind::Switch(_, _) => todo!(),
            ActionKind::Swap(card_index) => {
                if action.player.swapped_cards_count == self.round{
                    if self.swapping_phase{
                        if self.swap_buffer.iter().any(|(p, _)| *p == action.player){
                            return Err("Es darf pro Spieler nur eine Karte getauscht werden")
                        }
                        
                        self.swap_buffer.push((action.player, *action.player.cards.get(card_index).expect("Spieler hat weniger Karten als die angegeben anzahl")));
                        action.player.cards.remove(card_index);
                        if self.swap_buffer.len()==4 {
                                for (p, c) in self.swap_buffer.drain(..){
                                    match p.teammate() {
                                        Color::Red => self.red.cards.push(c),
                                        Color::Green => self.green.cards.push(c),
                                        Color::Blue => self.blue.cards.push(c),
                                        Color::Yellow => self.yellow.cards.push(c),
                                    }
                                }  
                            self.swapping_phase = false;
                            return Ok(())
                        }
                    }else {
                        return  Err("In dieser Phase des Spiels dürfen keine Karten getauscht werden");
                    }
                } else  {return  Err("Dieser Spieler darf keine Karte tauschen") };
            Ok(())},
        }
    }
    
    fn undo(&mut self) -> Result<(), &'static str> {
        todo!()
    }
    
    fn board(&self) -> &[Option<Piece>; 80] {
        todo!()
    }
    
    fn new_round(&mut self) {
        let current_round = (self.round % 4) as usize;
        let cards_to_deal = CARDS_PER_ROUND[current_round];
        if self.deck.len() <= (cards_to_deal as usize * 4 ){
            self.deck.replenish(&mut self.discard);
        }

        for _ in 0..cards_to_deal {
            self.red.cards.push(self.deck.draw().unwrap());
            self.green.cards.push(self.deck.draw().unwrap());
            self.blue.cards.push(self.deck.draw().unwrap());
            self.yellow.cards.push(self.deck.draw().unwrap());
        }

        self.swap_cards();
        
        self.round += 1;
    }
    
    fn swap_cards(&mut self)-> &mut Self {
        todo!()
    }
    
    fn is_winner(&self) -> bool {
        todo!()
    }
}

// Game Logik: 
// 4 Spieler je 5-2 Karten
// Tauschen 1 Karte 
// Spielt Karten aus
// Musst alle Karten ausspielen
// Wenn Legen nicht möglich, alle Karten ablegen

