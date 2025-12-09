use super::piece::*;
use super::action::*;
use super::color::*;
use super::deck::*;
use super::card::*;
use super::player::*;
use super::board::*;
use super::history::*;

const CARDS_PER_ROUND: [u8;4] = [5,4,3,2];

pub struct Game {
    board: Board,
    history: Vec<HistoryEntry>,
    round: u8,

    deck: Deck,
    discard: Vec<Card>,

    red: Player,
    green: Player,
    blue: Player,
    yellow: Player,

    current_player_color: Color,
}


pub trait DogGame {
    // Creates new instance with an empty board and initialized deck and players
    fn new() -> Self;

    // Returns the current state of the board
    fn board_state(&self) -> &[Option<Piece>; 80];

    // Returns the current player
    fn current_player(&self) -> &Player;

    // Matches and applies the action of playing the given card for the current player
    fn action(&mut self, card: Card, action: Action) -> Result<(), &'static str>;

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
            board: Board::new(),
            history: Vec::new(),
            round: 0,

            deck: Deck::new(),
            discard: Vec::new(),

            red: Player::new(Color::Red),
            green: Player::new(Color::Green),
            blue: Player::new(Color::Blue),
            yellow: Player::new(Color::Yellow),

            current_player_color: Color::Red,
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
        &self.board.tiles
    }

    fn action(&mut self, _card: Card, _action: Action) -> Result<(), &'static str> {
        match _action.action {
            ActionKind::Place => {

                match _card {
                    Card::Ace | Card::King | Card::Joker => {}
                    _ => return Err("Cannot place piece with this card."),
                }

                let color = self.current_player_color;
                let start = Board::start_field(color) as usize;

                let mut beaten_piece_color: Option<Color> = None;

                if let Some(piece) = self.board.tiles[start].take() {
                    if piece.color == color && !piece.left_start {
                        self.board.tiles[start] = Some(piece);
                        return Err("Cannot place piece: your protected piece is blocking.");
                    } else {
                        beaten_piece_color = Some(piece.color);
                        match piece.color {
                            Color::Red => self.red.pieces_to_place += 1,
                            Color::Green => self.green.pieces_to_place += 1,
                            Color::Blue => self.blue.pieces_to_place += 1,
                            Color::Yellow => self.yellow.pieces_to_place += 1,
                        }
                    }
                }

                self.board.tiles[start] = Some (Piece::new(color));

                 let player_cards = match color {
                    Color::Red => &mut self.red.cards,
                    Color::Green => &mut self.green.cards,
                    Color::Blue => &mut self.blue.cards,
                    Color::Yellow => &mut self.yellow.cards,
                };

                if let Some(index) = player_cards.iter().position(|&c| c == _card) {
                    player_cards.remove(index);
                }

                self.discard.push(_card);

                self.history.push(HistoryEntry {
                    action: _action,
                    beaten_piece_color,
                    switched_piece_color: None,
                });

                Ok(())
            }

            ActionKind::Move(_, _) => todo!(),
            ActionKind::Switch(_, _) => todo!(),
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

