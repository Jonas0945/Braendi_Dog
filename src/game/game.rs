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
    split_rest: Option<u8>,
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

    pub fn can_card_move(&self, _card: Card, forward: Option<u8>, backward: Option<u8>) -> bool {
        let distances = match _card.possible_distances() {
            Some(d) => d,
            None => return false,
        };

        let forward_ok = forward.map_or(false, |f| distances.contains(&f));
        let backward_ok = backward.map_or(false, |b| distances.contains(&b));

        forward_ok || backward_ok
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
            split_rest: None,
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
                    interchanged_piece_color: None,
                });

                self.player_mut_by_color(current_player_color).pieces_to_place -= 1;
                self.current_player_color = self.current_player_color.next();

                Ok(())
            }

            ActionKind::Move(from, to) => {
                match _card {
                    Card::Jack => return Err("Cannot move piece with Jack."),
                    Card::Seven => return Err("Cannot move with Seven (-> Split)"),
                    _ => {},
                }

                let current_player_color = self.current_player_color;
                
                let moving_piece = match self.board.check_tile(from) {
                    Some(p) => p,
                    None => return Err("Invalid move: no piece found."),
                };

                if moving_piece.color != current_player_color {
                    return Err("You can only move your own piece.");
                }

                // Calculate distances and check if card allows the move
                let forward_distance = self.board.distance_between(from, to, current_player_color);
                let backward_distance = self.board.distance_between(to, from, current_player_color);

                if !self.can_card_move(_card, forward_distance, backward_distance) {
                    return Err("Move not allowed with this card")
                };

                // Calculate path + direction and check for blocking pieces
                let is_backward = matches!(_card, Card::Four | Card::Joker)
                    && backward_distance == Some(4);

                let path = match self.board.passed_tiles(from, to, self.current_player_color, is_backward) {
                    Some(p) => p,
                    None => return Err("Invalid move: path cannot be calculated.")
                };

                for &tile in &path {
                    if let Some(piece) = self.board.tiles[tile as usize] {
                        if tile >= 64 {
                            return Err("Cannot move past piece inside the house");
                        } else if !piece.left_start {
                            return Err("Cannot move past protected piece.");
                        }
                    }
                }

                // Move execution
                let moving_piece = self.board.tiles[from as usize].take().unwrap();

                // Remove piece from destination tile if opponent piece is there
                let mut beaten_piece_color = None;

                if let Some(beaten_piece) = self.board.tiles[to as usize].take() {
                    beaten_piece_color = Some(beaten_piece.color);
                    self.player_mut_by_color(beaten_piece.color).pieces_to_place += 1;
                }

                // Piece placement and history update
                self.board.tiles[to as usize] = Some(moving_piece);

                self.player_mut_by_color(current_player_color).remove_card(_card);
                self.discard.push(_card);

                self.history.push(HistoryEntry { 
                    action: _action, 
                    beaten_piece_color, 
                    interchanged_piece_color: None 
                });

                self.current_player_color = self.current_player_color.next();        

                Ok(())
            },

            ActionKind::Interchange(from, to) => {

                match _card {
                    Card::Jack | Card::Joker => {},
                    _ => return Err("Cannot interchange pieces with this card."),
                }

                let from_piece = match self.board.check_tile(from) {
                    Some(p) => p.clone(),
                    None => return Err("Cannot interchange from an empty tile."),
                };

                let to_piece = match self.board.check_tile(to) {
                    Some(p) => p.clone(),
                    None => return Err("Cannot interchange to an empty tile."),
                };

                if HOUSE_TILES.contains(&from) || HOUSE_TILES.contains(&to) {
                    return Err("Cannot interchange pieces inside player's houses.");
                }

                let current_player_color = self.current_player_color;

                if from_piece.color != current_player_color {
                    return Err("First piece needs to be own piece.");
                }

                if !from_piece.left_start || !to_piece.left_start {
                    return Err("Cannot Interchange with protected piece.")
                }

                let from_index = from as usize;
                let to_index = to as usize;

                let interchanged_color = to_piece.color;

                self.board.tiles[from_index] = Some(to_piece);
                self.board.tiles[to_index] = Some(from_piece);

                self.player_mut_by_color(current_player_color).remove_card(_card);
                self.discard.push(_card);

                self.history.push(HistoryEntry {
                    action: _action,
                    beaten_piece_color: None,
                    interchanged_piece_color: Some(interchanged_color),
                });
                
                self.current_player_color = self.current_player_color.next();

                Ok(())
            },

            ActionKind::Trade => todo!(),
            ActionKind::Split(from, to) => {
                match _card {
                    Card::Seven | Card::Joker => {},
                    _ => return Err("Cannot split move with this card.")
                }

                let current_player_color = self.current_player_color;

                let distance = self.board.distance_between(from, to, current_player_color)
                    .ok_or("Invalid action.")?;

                if distance > 7 || distance == 0 {
                    return Err("Split move must have 1..7 steps.");
                }

                // Check split_rest
                let max_steps = self.split_rest.unwrap_or(7);
                if distance > max_steps {
                    return Err("Cannot move more steps than remaining split.");
                }

                let moving_piece = match self.board.check_tile(from) {
                    Some(p) => p,
                    None => return Err("Invalid move: no piece found."),
                };

                if moving_piece.color != current_player_color {
                    return Err("You can only move your own piece.");
                }

                // Calculate path and check for blocking pieces
                let path = match self.board.passed_tiles(from, to, self.current_player_color, false) {
                    Some(p) => p,
                    None => return Err("Invalid move: path cannot be calculated.")
                };

                for &tile in &path {
                    if let Some(piece) = self.board.tiles[tile as usize] {
                        if tile >= 64 {
                            return Err("Cannot move past piece inside the house");
                        } else if !piece.left_start {
                            return Err("Cannot move past protected piece.");
                        }
                    }
                }

                // Move execution along path
                let mut steps_taken = 0;
                let mut current_position = from;

                for &tile in &path {
                    steps_taken += 1;


                    // Create "mini"- history if piece is beaten
                    if let Some(opponent_piece) = self.board.tiles[tile as usize].take() {
                        
                        self.player_mut_by_color(opponent_piece.color).pieces_to_place += 1;

                        // Mini history
                        self.history.push( HistoryEntry { 
                            action: Action { 
                                player: current_player_color, 
                                card: _card, 
                                action: ActionKind::Split(current_position, tile) 
                            }, 
                            beaten_piece_color: Some(opponent_piece.color), 
                            interchanged_piece_color: None, 
                        });

                        current_position = tile;
                    }
                }

                // Piece placement and history update
                let moving_piece = self.board.tiles[from as usize].take().unwrap();
                self.board.tiles[to as usize] = Some(moving_piece);

                if current_position != to {

                    self.history.push(HistoryEntry {
                        action: Action { 
                            player: current_player_color, 
                            card: _card, 
                            action: ActionKind::Split(current_position, to) 
                        }, 
                        beaten_piece_color: None, 
                        interchanged_piece_color: None, 
                    });
                }

                // Update split_rest
                let remaining_steps = max_steps - steps_taken;

                if remaining_steps == 0 {
                    self.split_rest = None;

                    // Change player
                    self.player_mut_by_color(current_player_color).remove_card(_card);
                    self.discard.push(_card);

                    self.current_player_color = self.current_player_color.next();

                } else {
                    self.split_rest = Some(remaining_steps);
                }

                Ok(())
            },
        }
    }
    
    fn undo(&mut self) -> Result<(), &'static str> {
        let entry= self.history.pop().ok_or("No action to undo")?;

        match entry.action.action {
            ActionKind::Place => {
                let player = entry.action.player;
                let start = Board::start_field(player) as usize;

                self.board.tiles[start].take();
                self.player_mut_by_color(player).pieces_to_place += 1;

                if let Some(beaten_color) = entry.beaten_piece_color {
                    self.board.tiles[start] = Some(Piece {
                        color: beaten_color,
                        left_start: true,
                    });

                    self.player_mut_by_color(beaten_color).pieces_to_place -= 1;
                }

                let card = entry.action.card;
                self.discard.pop();
                self.player_mut_by_color(player).cards.push(card);

                self.current_player_color = player;
            },

            ActionKind::Interchange(from, to) => {
                let player = entry.action.player;

                let from_index = from as usize;
                let to_index = to as usize;

                let a = self.board.tiles[from_index].take();
                let b = self.board.tiles[to_index].take();

                self.board.tiles[from_index] = b;
                self.board.tiles[to_index] = a;

                let card = entry.action.card;
                self.discard.pop();
                self.player_mut_by_color(player).cards.push(card);

                self.current_player_color = player;
            }
            ActionKind::Move(_, _) => todo!(),
            ActionKind::Trade => todo!(),
            ActionKind::Split(_, _) => todo!()
        }

        Ok(())
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

    mod place_tests {
        use super::*;

        #[test]
        fn on_empty_start() {
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
        fn invalid_card_cannot_place() {
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
        fn cannot_place_on_own_protected_piece() {
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
        fn beat_opponent() {
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
    }
    
    mod interchange_tests {
        use super::*;

        #[test]
        fn interchange_success() {
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
                action: ActionKind::Interchange(1, 2),
                card: Card::Jack,
            };

            assert!(game.action(Card::Jack, action).is_ok());

            assert_eq!(game.board.tiles[1].as_ref().unwrap().color, Color::Green);
            assert_eq!(game.board.tiles[2].as_ref().unwrap().color, Color::Red);

            assert!(!game.player_mut_by_color(Color::Red).cards.contains(&Card::Jack));
            assert!(game.discard.contains(&Card::Jack));

            let entry = game.history.last().unwrap();
            assert_eq!(entry.interchanged_piece_color, Some(Color::Green));
            assert_eq!(entry.beaten_piece_color, None);
        }

        #[test]
        fn invalid_card() {
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
                action: ActionKind::Interchange(1, 2),
                card: Card::Two,
            };

            assert!(game.action(Card::Two, action).is_err()); 
        }

        #[test]
        fn empty_tile() {
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
                action: ActionKind::Interchange(1, 3),
                card: Card::Jack,
            };

            assert!(game.action(Card::Jack, action).is_err());
        }

        #[test]
        fn house_tile() {
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
                action: ActionKind::Interchange(64, 2),
                card: Card::Jack,
            };

            assert!(game.action(Card::Jack, action).is_err());
        }

        #[test]
        fn not_own_piece() {
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
                action: ActionKind::Interchange(2, 1),
                card: Card::Jack,
            };

            assert!(game.action(Card::Jack, action).is_err());
        }

        #[test]
        fn protected_piece() {
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
                action: ActionKind::Interchange(0, 2),
                card: Card::Jack,
            };

            assert!(game.action(Card::Jack, action).is_err());
        }
    }
 
    mod move_tests {
        use super::*;

        #[test]
        fn valid_move_forward() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Five, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Move(0, 5),
                card: Card::Five,
            };

            assert!(game.action(Card::Five, action).is_ok());
            assert!(game.board.tiles[0].is_none());
            assert_eq!(game.board.tiles[5].as_ref().unwrap().color, Color::Red);
        }

        #[test]
        fn valid_move_backward() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Four, Card::Ten];

            game.board.tiles[10] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Move(10, 6),
                card: Card::Four,
            };

            assert!(game.action(Card::Four, action).is_ok());
            assert!(game.board.tiles[10].is_none());
            assert_eq!(game.board.tiles[6].as_ref().unwrap().color, Color::Red);
        }

        #[test]
        fn valid_move_backward_with_joker() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Joker, Card::Ten];

            game.board.tiles[10] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Move(10, 6),
                card: Card::Joker,
            };

            assert!(game.action(Card::Joker, action).is_ok());
            assert!(game.board.tiles[10].is_none());
            assert_eq!(game.board.tiles[6].as_ref().unwrap().color, Color::Red);
        }

        #[test]
        fn invalid_move_with_jack() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Jack, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Move(0, 5),
                card: Card::Jack,
            };

            assert!(game.action(Card::Jack, action).is_err());
            assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
            assert!(game.board.tiles[5].is_none());
        }

        #[test]
        fn invalid_move_past_protected_piece() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Five, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            game.board.tiles[3] = Some(Piece {
                color: Color::Green,
                left_start: false,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Move(0, 5),
                card: Card::Five,
            };

            assert!(game.action(Card::Five, action).is_err());
            assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
            assert_eq!(game.board.tiles[3].as_ref().unwrap().color, Color::Green);
            assert!(game.board.tiles[5].is_none());
        }

        #[test]
        fn invalid_move_past_house_piece() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Five, Card::Ten];

            game.board.tiles[60] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            game.board.tiles[64] = Some(Piece {
                color: Color::Green,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Move(60, 65),
                card: Card::Five,
            };

            assert!(game.action(Card::Five, action).is_err());
            assert_eq!(game.board.tiles[60].as_ref().unwrap().color, Color::Red);
            assert_eq!(game.board.tiles[64].as_ref().unwrap().color, Color::Green);
            assert!(game.board.tiles[65].is_none());
        }

        #[test]
        fn invalid_move_not_own_piece() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Five, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Green,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Move(0, 5),
                card: Card::Five,
            };

            assert!(game.action(Card::Five, action).is_err());
            assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Green);
            assert!(game.board.tiles[5].is_none());
        }

        #[test]
        fn invalid_move_not_allowed_by_card() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Three, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Move(0, 5),
                card: Card::Three,
            };

            assert!(game.action(Card::Three, action).is_err());
            assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
            assert!(game.board.tiles[5].is_none());
        }

        #[test]
        fn invalid_move_empty_from_tile() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Five, Card::Ten];

            let action = Action {
                player: Color::Red,
                action: ActionKind::Move(0, 5),
                card: Card::Five,
            };

            assert!(game.action(Card::Five, action).is_err());
            assert!(game.board.tiles[0].is_none());
            assert!(game.board.tiles[5].is_none());
        }

        #[test]
        fn invalid_move_path_cannot_be_calculated() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Five, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            game.board.tiles[1] = Some(Piece {
                color: Color::Green,
                left_start: false,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Move(0, 5),
                card: Card::Five,
            };

            assert!(game.action(Card::Five, action).is_err());
            assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
            assert_eq!(game.board.tiles[1].as_ref().unwrap().color, Color::Green);
        }

        #[test]
        fn beat_opponent_piece() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Five, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            game.board.tiles[5] = Some(Piece {
                color: Color::Green,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Move(0, 5),
                card: Card::Five,
            };

            assert!(game.action(Card::Five, action).is_ok());
            assert!(game.board.tiles[0].is_none());
            assert_eq!(game.board.tiles[5].as_ref().unwrap().color, Color::Red);
            assert_eq!(game.player_mut_by_color(Color::Green).pieces_to_place, 5);
        }
    }

    mod split_tests {
        use super::*;

        #[test]
        fn split_within_limits() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Seven, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Split(0, 5),
                card: Card::Seven,
            };

            assert!(game.action(Card::Seven, action).is_ok());
            assert!(game.board.tiles[0].is_none());
            assert_eq!(game.board.tiles[5].as_ref().unwrap().color, Color::Red);
            assert_eq!(game.split_rest, Some(2));
        }

        #[test]
        fn split_outside_limits() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Seven, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Split(0, 10),
                card: Card::Seven,
            };

            assert!(game.action(Card::Seven, action).is_err());
            assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
        }

        #[test]
        fn split_with_joker() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Joker, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Split(0, 5),
                card: Card::Joker,
            };

            assert!(game.action(Card::Joker, action).is_ok());
            assert!(game.board.tiles[0].is_none());
            assert_eq!(game.board.tiles[5].as_ref().unwrap().color, Color::Red);
            assert_eq!(game.split_rest, Some(2));
        }

        #[test]
        fn split_beaten_piece_correct_history() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Seven, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            game.board.tiles[3] = Some(Piece {
                color: Color::Green,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Split(0, 5),
                card: Card::Seven,
            };

            assert!(game.action(Card::Seven, action).is_ok());
            assert!(game.board.tiles[0].is_none());
            assert_eq!(game.board.tiles[5].as_ref().unwrap().color, Color::Red);
            assert_eq!(game.player_mut_by_color(Color::Green).pieces_to_place, 5);

            let first_entry = &game.history[game.history.len() - 2];
            assert_eq!(first_entry.action.action, ActionKind::Split(0, 3));
            assert_eq!(first_entry.beaten_piece_color, Some(Color::Green));

            let second_entry = &game.history[game.history.len() - 1];
            assert_eq!(second_entry.action.action, ActionKind::Split(3, 5));
            assert_eq!(second_entry.beaten_piece_color, None);
        }

        #[test]
        fn split_complete_turn() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Seven, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Split(0, 7),
                card: Card::Seven,
            };

            assert!(game.action(Card::Seven, action).is_ok());
            assert!(game.board.tiles[0].is_none());
            assert_eq!(game.board.tiles[7].as_ref().unwrap().color, Color::Red);
            assert_eq!(game.split_rest, None);
            assert_eq!(game.current_player_color, Color::Green);
        }

        #[test]
        fn split_invalid_card() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Five, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Split(0, 5),
                card: Card::Five,
            };

            assert!(game.action(Card::Five, action).is_err());
            assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
        }

        #[test]
        fn split_not_own_piece() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Seven, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Green,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Split(0, 5),
                card: Card::Seven,
            };

            assert!(game.action(Card::Seven, action).is_err());
            assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Green);
        }

        #[test]
        fn split_path_blocked_by_protected_piece() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Seven, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            game.board.tiles[3] = Some(Piece {
                color: Color::Green,
                left_start: false,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Split(0, 5),
                card: Card::Seven,
            };

            assert!(game.action(Card::Seven, action).is_err());
            assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
        }

        #[test]
        fn split_path_blocked_by_house_piece() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Seven, Card::Ten];

            game.board.tiles[60] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            game.board.tiles[64] = Some(Piece {
                color: Color::Green,
                left_start: true,
            });

            let action = Action {
                player: Color::Red,
                action: ActionKind::Split(60, 65),
                card: Card::Seven,
            };

            assert!(game.action(Card::Seven, action).is_err());
            assert_eq!(game.board.tiles[60].as_ref().unwrap().color, Color::Red);
        }

        #[test]
        fn split_empty_tile() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Seven, Card::Ten];

            let action = Action {
                player: Color::Red,
                action: ActionKind::Split(0, 5),
                card: Card::Seven,
            };

            assert!(game.action(Card::Seven, action).is_err());
            assert!(game.board.tiles[0].is_none());
        }

        #[test]
        fn split_multiple_times_within_limits() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Seven, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            let first_action = Action {
                player: Color::Red,
                action: ActionKind::Split(0, 4),
                card: Card::Seven,
            };

            assert!(game.action(Card::Seven, first_action).is_ok());
            assert_eq!(game.split_rest, Some(3));

            let second_action = Action {
                player: Color::Red,
                action: ActionKind::Split(4, 7),
                card: Card::Seven,
            };

            assert!(game.action(Card::Seven, second_action).is_ok());
            assert_eq!(game.split_rest, None);
            assert_eq!(game.current_player_color, Color::Green);
        }

        #[test]
        fn split_multiple_times_correct_history() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Seven, Card::Ten];

            game.board.tiles[0] = Some(Piece {
                color: Color::Red,
                left_start: true,
            });

            let first_action = Action {
                player: Color::Red,
                action: ActionKind::Split(0, 4),
                card: Card::Seven,
            };

            assert!(game.action(Card::Seven, first_action).is_ok());
            assert_eq!(game.split_rest, Some(3));

            let second_action = Action {
                player: Color::Red,
                action: ActionKind::Split(4, 7),
                card: Card::Seven,
            };

            assert!(game.action(Card::Seven, second_action).is_ok());
            assert_eq!(game.split_rest, None);
            assert_eq!(game.current_player_color, Color::Green);

            let first_entry = &game.history[game.history.len() - 2];
            assert_eq!(first_entry.action.action, ActionKind::Split(0, 4));

            let second_entry = &game.history[game.history.len() - 1];
            assert_eq!(second_entry.action.action, ActionKind::Split(4, 7));
        }
        
    }
}