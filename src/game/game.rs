

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
swapping_phase: bool,
swap_buffer: Vec<(Player, Card)>
}

impl Game {
    pub fn player_mut_by_color(&mut self, color: Color) -> &mut Player {
        match color {
            Color::Red => &mut self.red,
            Color::Green => &mut self.green,
            Color::Blue => &mut self.blue,
            Color::Yellow => &mut self.yellow,
        }
    }
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
}

fn current_player(&self) -> &Player {
    match self.current_player_color {
        Color::Red => &self.red,
        Color::Green => &self.green,
        Color::Blue => &self.blue,
        Color::Yellow => &self.yellow,
    fn action(&mut self, _card: Card, _action: Action) -> Result<(), &'static str> {
        match _action.action {
            ActionKind::Place => {

                match _card {
                    Card::Ace | Card::King | Card::Joker => {},
                    _ => return Err("Cannot place piece with this card."),
                }
                
                let current_player_color = self.current_player_color;
                let start = Board::start_field(current_player_color) as usize;

                if self.current_player().pieces_to_place == 0 {
                    return Err("Cannot place piece: no pieces left to place.");
                }

                let mut beaten_piece_color = None;

                if let Some(piece) = self.board.tiles[start].take() {
                    if piece.color == current_player_color && !piece.left_start {
                        self.board.tiles[start] = Some(piece);
                        return Err("Cannot place piece: your protected piece is blocking.")
                    }
                    beaten_piece_color = Some(piece.color);
                    self.player_mut_by_color(piece.color).pieces_to_place += 1;
                }

                self.board.tiles[start] = Some (Piece::new(current_player_color));

                self.player_mut_by_color(current_player_color).remove_card(_card);
                self.discard.push(_card);

                self.history.push(HistoryEntry {
                    action: _action,
                    beaten_piece_color,
                    switched_piece_color: None,
                });

                self.player_mut_by_color(current_player_color).pieces_to_place -= 1;
                self.current_player_color = self.current_player_color.next();

                Ok(())
            }

            ActionKind::Move(_, _) => todo!(),
            ActionKind::Switch(from, to) => {

                match _card {
                    Card::Jack | Card::Joker => {},
                    _ => return Err("Cannot switch pieces with this card."),
                }

                let from_piece = match self.board.check_tile(from) {
                    Some(p) => p.clone(),
                    None => return Err("Cannot switch from an empty tile."),
                };

                let to_piece = match self.board.check_tile(to) {
                    Some(p) => p.clone(),
                    None => return Err("Cannot switch to an empty tile."),
                };

                if HOUSE_TILES.contains(&from) || HOUSE_TILES.contains(&to) {
                    return Err("Cannot switch pieces inside player's houses.");
                }

                let current_player_color = self.current_player_color;

                if from_piece.color != current_player_color {
                    return Err("First piece needs to be own piece.");
                }

                if !from_piece.left_start || !to_piece.left_start {
                    return Err("Cannot switch with protected piece.")
                }

                let from_index = from as usize;
                let to_index = to as usize;

                let switched_color = to_piece.color;

                self.board.tiles[from_index] = Some(to_piece);
                self.board.tiles[to_index] = Some(from_piece);

                self.player_mut_by_color(current_player_color).remove_card(_card);
                self.discard.push(_card);

                self.history.push(HistoryEntry {
                    action: _action,
                    beaten_piece_color: None,
                    switched_piece_color: Some(switched_color),
                });
                
                self.current_player_color = self.current_player_color.next();

                Ok(())
            },
        }
    }
}

fn board_state(&self) -> &[Option<Piece>; 80] {
    &self.board
}

fn action(&mut self, _card: Card, _action: Action) -> Result<(), &'static str> {
    match _action.action{
        ActionKind::Place => todo!(),
        ActionKind::Move(_, _) => todo!(),
        ActionKind::Switch(_, _) => todo!(),
        ActionKind::Swap(card_index) => {
            let playercolor = _action.player.color;
            let swapping_player;
            match playercolor {
                Color::Red => swapping_player = &self.red   ,
                Color::Green => swapping_player = &self.green,
                Color::Blue => swapping_player = &self.blue,
                Color::Yellow => swapping_player = &self.yellow,
            }
            //muss um 1 inkrementiert werde, da nach erstem mal karten austeilen round = 1 ist. 
            if swapping_player.swapped_cards_count+1 == self.round{
                if self.swapping_phase{
                    if self.swap_buffer.iter().any(|(p, _)| p.color == playercolor){
                        return Err("Es darf pro Spieler nur eine Karte getauscht werden")
                    }
                    if card_index >= swapping_player.cards.len() {
                        return Err("Ungültiger Kartenindex für den Tausch")
                    }

                    self.swap_buffer.push((swapping_player.clone(), swapping_player.cards.get(card_index).unwrap().clone()));
                    
                    match playercolor {
                        Color::Red => {self.red.cards.remove(card_index); self.red.swapped_cards_count +=1;},
                        Color::Green => {self.green.cards.remove(card_index); self.green.swapped_cards_count +=1;},
                        Color::Blue => {self.blue.cards.remove(card_index); self.blue.swapped_cards_count +=1;},
                        Color::Yellow => {self.yellow.cards.remove(card_index); self.yellow.swapped_cards_count +=1;},
}
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

    
    self.round += 1;
}

fn swap_cards(&mut self)-> &mut Self {
    todo!()
}

fn is_winner(&self) -> bool {
    todo!()
}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_place_on_empty_start() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Ace, Card::King, Card::Joker];

        let start = Board::start_field(Color::Red) as usize;
        let card = Card::Ace;
        let action = Action {
            player: Color::Red,
            action: ActionKind::Place,
            card: Card::Ace,
        };

        assert!(game.action(Card::Ace, action).is_ok());
        assert!(game.board.tiles[start].is_some());
        assert_eq!(game.player_mut_by_color(Color::Red).pieces_to_place, 3);
        assert!(!game.player_mut_by_color(Color::Red).cards.contains(&card));
        assert!(game.discard.contains(&card));
        assert_eq!(game.current_player_color, Color::Green);
    }

    #[test]
    fn test_invalid_card_cannot_place() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Ace, Card::King, Card::Joker];

        let invalid_card = Card::Two;
        let action = Action {
            player: Color::Red,
            action: ActionKind::Place,
            card: invalid_card
        };

        assert!(game.action(Card::Two, action).is_err());
    }

    #[test]
    fn test_cannot_place_on_own_protected_piece() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Ace, Card::King, Card::Joker];

        let start = Board::start_field(Color::Red) as usize;
        let card = Card::Ace;
        let action = Action {
            player: Color::Red,
            action: ActionKind::Place,
            card: Card::Ace,
        };

        game.board.tiles[start] = Some(Piece {
            color: Color::Red,
            left_start: false
        });

        assert!(game.action(card, action).is_err());
        assert_eq!(game.board.tiles[start].as_ref().unwrap().color, Color::Red);
    }

    #[test]
    fn test_place_and_beat_opponent() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Ace, Card::King, Card::Joker];

        let start = Board::start_field(Color::Red) as usize;
        let card = Card::Ace;
        let action = Action {
            player: Color::Red,
            action: ActionKind::Place,
            card: Card::Ace,
        };

        game.board.tiles[start] = Some(Piece {
            color: Color::Green,
            left_start: true
        });

        assert!(game.action(card, action).is_ok());
        assert_eq!(game.board.tiles[start].as_ref().unwrap().color, Color::Red);
    }

    #[test]
    fn test_switch_success() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Jack, Card::Joker];
        game.green.cards = vec![Card::Jack, Card::Joker];

        game.board.tiles[1] = Some(Piece {
            color: Color::Red,
            left_start: true,
        });

        game.board.tiles[2] = Some(Piece {
            color: Color::Green,
            left_start: true,
        });

        let action = Action { 
            player: Color::Red,
            action: ActionKind::Switch(1, 2),
            card: Card::Jack,
        };

        assert!(game.action(Card::Jack, action).is_ok());

        assert_eq!(game.board.tiles[1].as_ref().unwrap().color, Color::Green);
        assert_eq!(game.board.tiles[2].as_ref().unwrap().color, Color::Red);

        assert!(!game.player_mut_by_color(Color::Red).cards.contains(&Card::Jack));
        assert!(game.discard.contains(&Card::Jack));

        let entry = game.history.last().unwrap();
        assert_eq!(entry.switched_piece_color, Some(Color::Green));
        assert_eq!(entry.beaten_piece_color, None);
    }

    #[test]
    fn test_invalid_card() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Jack, Card::Joker];
        game.green.cards = vec![Card::Jack, Card::Joker];

        game.board.tiles[1] = Some(Piece {
            color: Color::Red,
            left_start: true,
        });

        game.board.tiles[2] = Some(Piece {
            color: Color::Green,
            left_start: true,
        });

        let action = Action { 
            player: Color::Red,
            action: ActionKind::Switch(1, 2),
            card: Card::Two,
        };

        assert!(game.action(Card::Two, action).is_err()); 
    }

    #[test]
    fn test_switch_empty_tile() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Jack, Card::Joker];
        game.green.cards = vec![Card::Jack, Card::Joker];

        game.board.tiles[1] = Some(Piece {
            color: Color::Red,
            left_start: true,
        });

        game.board.tiles[2] = Some(Piece {
            color: Color::Green,
            left_start: true,
        });

        let action = Action { 
            player: Color::Red,
            action: ActionKind::Switch(1, 3),
            card: Card::Jack,
        };

        assert!(game.action(Card::Jack, action).is_err());
    }

    #[test]
    fn test_switch_house_tile() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Jack, Card::Joker];
        game.green.cards = vec![Card::Jack, Card::Joker];

        game.board.tiles[64] = Some(Piece {
            color: Color::Red,
            left_start: true,
        });

        game.board.tiles[2] = Some(Piece {
            color: Color::Green,
            left_start: true,
        });

        let action = Action { 
            player: Color::Red,
            action: ActionKind::Switch(64, 2),
            card: Card::Jack,
        };

        assert!(game.action(Card::Jack, action).is_err());
    }

    #[test]
    fn test_switch_not_own_piece() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Jack, Card::Joker];
        game.green.cards = vec![Card::Jack, Card::Joker];

        game.board.tiles[1] = Some(Piece {
            color: Color::Red,
            left_start: true,
        });

        game.board.tiles[2] = Some(Piece {
            color: Color::Green,
            left_start: true,
        });

        let action = Action { 
            player: Color::Red,
            action: ActionKind::Switch(2, 1),
            card: Card::Jack,
        };

        assert!(game.action(Card::Jack, action).is_err());
    }

    #[test]
    fn test_switch_protected_piece() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Jack, Card::Joker];
        game.green.cards = vec![Card::Jack, Card::Joker];

        game.board.tiles[0] = Some(Piece {
            color: Color::Red,
            left_start: false,
        });

        game.board.tiles[2] = Some(Piece {
            color: Color::Green,
            left_start: true,
        });

        let action = Action { 
            player: Color::Red,
            action: ActionKind::Switch(0, 2),
            card: Card::Jack,
        };

        assert!(game.action(Card::Jack, action).is_err());
    }



    
}



/* let mut game =Game::new();
game.new_round();s
let a1=Action{player: game.red.clone(),card:Card::Ace, action: ActionKind::Swap(0)};
game.red.cards.push(Card::Ace);
game.action(a1);
game.swap_buffer.push((game.blue.clone(),Card::Five));
    game.swap_buffer.push((game.yellow.clone(),Card::Joker));

        game.swap_buffer.push((game.green.clone(), Card::Four));

assert!(game.blue.cards[0]==Card::Ace);*/
#[cfg(test)]
mod tests {
use crate::game::action;

use super::*;

#[test]
fn test_swap_successful() {
    let mut game = Game::new();
    
    // new_round() aufrufen, um Karten zu verteilen (5 pro Spieler in Runde 0)
    game.new_round();
    
    // Round zurücksetzen, damit Swapping in "Runde 0" funktioniert (swapped_cards_count == 0 == round)
    
    // Stelle sicher, dass Swapping-Phase aktiv ist (sollte von new() kommen)
    assert!(game.swapping_phase);
    
    // Swap für roten Spieler (verwende die erste Karte aus der Hand)
    let red_card = game.red.cards[0].clone();  // Echte Karte aus der Hand
    let a1 = Action {
        player: game.red.clone(),
        card: red_card,  // Verwende die echte Karte
        action: ActionKind::Swap(0),
    };
    game.action(Card::Ace, a1).unwrap();
    
    // Swap für grünen Spieler
    let green_card = game.green.cards[0].clone();
    let a2 = Action {
        player: game.green.clone(),
        card: green_card,
        action: ActionKind::Swap(0),
    };
    game.action(Card::Ace, a2).unwrap();
    
    // Swap für blauen Spieler
    let blue_card = game.blue.cards[0].clone();
    let a3 = Action {
        player: game.blue.clone(),
        card: blue_card,
        action: ActionKind::Swap(0),
    };
    game.action(Card::Ace, a3).unwrap();
    
    // Swap für gelben Spieler – triggert Verteilung
    let yellow_card = game.yellow.cards[0].clone();
    let a4 = Action {
        player: game.yellow.clone(),
        card: yellow_card,
        action: ActionKind::Swap(0),
    };
    game.action(Card::Ace, a4).unwrap();
    
    // Prüfe Verteilung an Teammates (verwende die echten Karten)
    assert!(game.blue.cards.contains(&red_card));     // Red -> Blue
    assert!(game.yellow.cards.contains(&green_card)); // Green -> Yellow
    assert!(game.red.cards.contains(&blue_card));     // Blue -> Red
    assert!(game.green.cards.contains(&yellow_card)); // Yellow -> Green
    
    // Buffer geleert, Phase beendet
    assert_eq!(game.swap_buffer.len(), 0);
    assert!(!game.swapping_phase);
}
#[test]
fn double_swap_by_same_player_through_index(){
    let mut game = Game::new();
    game.new_round();
    game.red.swapped_cards_count +=1;
    let a1=Action{player: game.red.clone(), card:Card::Eight, action: ActionKind::Swap(3),};
    
    assert_eq!(game.round, 1);
    // assert!(game.action(Card::Eight, a1).is_err());
assert_eq!(game.action(Card::Eight, a1).unwrap_err(), "Dieser Spieler darf keine Karte tauschen");
}
#[test]
fn swapping_in_not_swap_phase(){
    let mut game = Game::new();
    game.new_round();
    let a1=Action{player: game.red.clone(), card:Card::Eight, action: ActionKind::Swap(3),};
    game.swapping_phase = false;
    // assert!(game.action(Card::Eight, a1).is_err());
assert_eq!(game.action(Card::Eight, a1).unwrap_err(), "In dieser Phase des Spiels dürfen keine Karten getauscht werden");
}
#[test]
fn double_swap_by_same_player(){
    let mut game = Game::new();
    game.new_round();
    let a1=Action{player: game.red.clone(), card:Card::Eight, action: ActionKind::Swap(3),};
    game.action(Card::Seven, a1).expect("Es darf pro Spieler nur eine Karte getauscht werden");
    //nur zu test zwecken
    game.red.swapped_cards_count =0;
    let a2=Action{player: game.red.clone(), card:Card::Eight, action: ActionKind::Swap(4),};
    // assert!(game.action(Card::Eight, a1).is_err());
assert_eq!(game.action(Card::Eight, a2).unwrap_err(), "Es darf pro Spieler nur eine Karte getauscht werden");
}
#[test]
fn swapping_index_overflow(){
    let mut game = Game::new();
    game.new_round();
    let a1=Action{player: game.red.clone(), card:Card::Eight, action: ActionKind::Swap(5),};
assert_eq!(game.action(Card::Eight, a1).unwrap_err(), "Ungültiger Kartenindex für den Tausch");
}

}
