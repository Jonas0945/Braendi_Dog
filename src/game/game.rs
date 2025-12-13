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
    game_seven_rest: u8,
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
            game_seven_rest: 0,
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

            ActionKind::Move(from, to) => {
                match _card {
                    Card::Jack => {},
                    _ => return Err("Cannot move piece with this card."),
                }

                let current_player_color = self.current_player_color;

                let from_piece = match self.board.check_tile(from) {
                    Some(p) => p,
                    None => return Err("No piece on the from tile."),
                };

                if from_piece.color != current_player_color {
                    return Err("You can only move your own pieces.");
                }

                if from > 63 && to < from {
                    return Err("Cannot go backwards in the house.");
                }
                if from > 63 && to <= 63 {
                    return Err("Cannot leave the house once entered.");
                }

                let mut into_house = false;
                if to > 63 {
                    let house = PLAYER_HOUSE
                        .iter()
                        .find(|(c, _)| *c == current_player_color)
                        .unwrap()
                        .1;

                    if !house.contains(&to) {
                        return Err("Cannot move into another player's house.");
                    }
                    into_house = true;
                }

                // simulate backwards if card is Four
                let mut allow_backwards = false;
                if let Card::Four = _card {
                    if !into_house {
                        let mut nfrom = from;
                        allow_backwards = true;

                        for _ in 0..4 { 
                            if nfrom == 0 {
                                nfrom = 63;
                            } else {
                                nfrom -= 1;
                            }

                            if let Some(p) = self.board.check_tile(nfrom) {
                                if !p.left_start {
                                    allow_backwards = false;
                                    break;
                                }
                            }
                        }

                        if allow_backwards && nfrom == to {
                            let moving_piece = self.board.tiles[from as usize].take().unwrap();
                            let beaten_piece_color = if let Some(beaten) = self.board.tiles[to as usize].take() {
                                self.player_mut_by_color(beaten.color).pieces_to_place += 1;
                                Some(beaten.color)
                            } else {
                                None
                            };

                            self.board.tiles[to as usize] = Some(moving_piece);
                            self.player_mut_by_color(current_player_color).remove_card(_card);
                            self.discard.push(_card);

                            self.history.push(HistoryEntry {
                                action: _action,
                                beaten_piece_color,
                                switched_piece_color: None,
                            });

                            self.current_player_color = self.current_player_color.next();
                            return Ok(());
                        }
                    }
                }

                // simulate forward move
                let mut nfrom = from;
                let mut in_house = into_house;
                let mut actual_steps = 0;
                let max_steps = 13;
                let mut passed: Vec<Point> = Vec::new();

                for _ in 0..max_steps {
                    if nfrom == to {
                        break;
                    }

                    actual_steps += 1;

                    if !in_house && nfrom == Board::house_entry_for(current_player_color) && into_house {
                        nfrom = PLAYER_HOUSE
                            .iter()
                            .find(|(c, _)| *c == current_player_color)
                            .unwrap()
                            .1[0]; //
                        in_house = true;
                    } else if in_house {
                        nfrom += 1;
                    } else {
                        nfrom = (nfrom + 1) % 64;
                    }

                    if let Some(p) = self.board.check_tile(nfrom) {
                        if in_house {
                            return Err("Cannot pass pieces in the house.");
                        } else {
                            passed.push(nfrom);
                        }
                    }
                }

                if nfrom != to {
                    return Err("Move not reachable.");
                }
                match _card {
                    Card::Ace => {
                        if actual_steps != 1 && actual_steps != 11 {
                            return Err("Ace can only be used as 1 or 11 steps.")
                        }

                    }

                    Card::Seven => {
                        if actual_steps > 7 {
                            return Err("More than 7 steps used for 7.");
                        }
                        if self.game_seven_rest==0{
                           self.game_seven_rest = 7 - actual_steps;
                        }
                        else {
                            if actual_steps>self.game_seven_rest{
                                return Err("Seven split in more than 7 moves.")
                            }
                            self.game_seven_rest=self.game_seven_rest-actual_steps;
                        }
                        // Beat all pieces along the passed tiles
                        for &tile in &passed {
                            if let Some(p) = self.board.tiles[tile as usize].take() {
                                self.player_mut_by_color(p.color).pieces_to_place += 1;
                            }
                        }

                    }


                    _ => {
                        if _card.value() != actual_steps{
                            return Err("Value of card is not the same as steps.")
                        }
                    }
                }
                
                let moving_piece = self.board.tiles[from as usize].take().unwrap();
                let beaten_piece_color = if let Some(beaten) = self.board.tiles[to as usize].take() {
                    self.player_mut_by_color(beaten.color).pieces_to_place += 1;
                    Some(beaten.color)
                } else {
                    None
                };

                self.board.tiles[to as usize] = Some(moving_piece);
                if self.game_seven_rest==0{
                self.player_mut_by_color(current_player_color).remove_card(_card);
                self.discard.push(_card);
                self.current_player_color = self.current_player_color.next();
                } 
                self.history.push(HistoryEntry {
                    action: _action,
                    beaten_piece_color,
                    switched_piece_color: None,
                });


                Ok(())
            },
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


