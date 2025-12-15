use super::piece::*;
use super::action::*;
use super::color::*;
use super::deck::*;
use super::card::*;
use super::player::*;
use super::board::*;
use super::history::*;

const CARDS_PER_ROUND: [u8;4] = [5,4,3,2];
const HOUSE_TILES: [u8; 16] = [
    64, 65, 66, 67, 
    68, 69, 70, 71, 
    72, 73, 74, 75, 
    76, 77, 78, 79
];
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

                // 1. Check: Haben wir überhaupt noch IDs im Haus?
                if self.current_player().pieces_to_place() == 0 {
                    return Err("Cannot place piece: no pieces left to place.");
                }

                let mut beaten_piece_color = None;

                // 2. Check: Was liegt auf dem Startfeld?
                if let Some(piece) = self.board.tiles[start].take() {
                    if piece.color == current_player_color && !piece.left_start {
                        // Eigene geschützte Figur blockiert -> Fehler, Figur zurücklegen
                        self.board.tiles[start] = Some(piece);
                        return Err("Cannot place piece: your protected piece is blocking.")
                    }
                    // Fremde Figur oder eigene ungeschützte -> Schlagen!
                    beaten_piece_color = Some(piece.color);
                    
                    // WICHTIG: ID an den Besitzer zurückgeben!
                    self.player_mut_by_color(piece.color).return_piece_id(piece.id);
                }

                // 3. Neue Figur erstellen: Wir holen uns eine echte ID vom Spieler!
                let new_id = self.player_mut_by_color(current_player_color)
                                 .take_next_piece_id()
                                 .expect("Should work because check #1 passed");

                self.board.tiles[start] = Some(Piece::new(current_player_color, new_id));

                self.player_mut_by_color(current_player_color).remove_card(_card);
                self.discard.push(_card);

                self.history.push(HistoryEntry {
                    action: _action,
                    beaten_piece_color,
                    switched_piece_color: None,
                });

                self.current_player_color = self.current_player_color.next();
                Ok(())
            }

            ActionKind::Move(_, _) => todo!(),

            ActionKind::Split(ref sub_moves) => {
    match _card {
        Card::Seven | Card::Joker => {
        }
        _ => return Err("Split is only allowed with 7 or Joker."),
    }

    let total_steps: u8 = sub_moves
        .iter()
        .map(|(_, steps)| steps)
        .sum();

    if total_steps != 7 {
        return Err("Total steps for split must be exactly 7.");
    }

    let active_color = self.current_player_color;

    let mut tmp_board = self.board.clone();

    let mut beaten: Vec<(Color, u8)> = Vec::new();

    for (from, steps) in sub_moves.iter().copied() {
        let piece = match tmp_board.check_tile(from) {
            Some(p) => p,
            None => return Err("Cannot move from empty tile in split."),
        };

        if piece.color != active_color {
            return Err("Can only move own pieces.");
        }

        let target = match tmp_board.calculate_target(from, steps as i8, active_color) {
            Some(t) => t,
            None => return Err("Move invalid: Target out of bounds."),
        };

        if let Some(other) = tmp_board.check_tile(target) {
            if other.color == active_color {
                return Err("Cannot land on own piece.");
            }
        }

        let moving_piece = tmp_board.tiles[from as usize]
            .take()
            .expect("Checked tile above, should exist here");

    
        if let Some(hit_piece) = tmp_board.tiles[target as usize].replace(moving_piece) {
            beaten.push((hit_piece.color, hit_piece.id));
        }

    }

    self.board = tmp_board;

    for (color, id) in &beaten {
        self.player_mut_by_color(*color).return_piece_id(*id);
    }

    self.player_mut_by_color(active_color).remove_card(_card);
    self.discard.push(_card);

    let last_beaten_color = beaten.last().map(|(c, _)| *c);

    self.history.push(HistoryEntry {
        action: _action,
        beaten_piece_color: last_beaten_color,
        switched_piece_color: None,
    });

    self.current_player_color = self.current_player_color.next();

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
        assert_eq!(game.player_mut_by_color(Color::Red).pieces_to_place(), 3);
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

        // KORREKTUR: id: 0 hinzugefügt
        game.board.tiles[start] = Some(Piece {
            color: Color::Red,
            id: 0, 
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

        // KORREKTUR: id: 0 hinzugefügt
        game.board.tiles[start] = Some(Piece {
            color: Color::Green,
            id: 0,
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

        // KORREKTUR: id: 0 hinzugefügt
        game.board.tiles[1] = Some(Piece {
            color: Color::Red,
            id: 0,
            left_start: true,
        });

        // KORREKTUR: id: 0 hinzugefügt
        game.board.tiles[2] = Some(Piece {
            color: Color::Green,
            id: 0,
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
    }
    
    // ... (für die restlichen Tests gilt das gleiche Prinzip: überall id: 0 einfügen)
    #[test]
    fn test_split_success() {
        let mut game = Game::new();

        // 1. Vorbereitung: Karte geben
        game.red.cards = vec![Card::Seven];

        // 2. Vorbereitung: Zwei Figuren aufs Feld stellen
        // Figur A auf Feld 0
        game.board.tiles[0] = Some(Piece {
            color: Color::Red,
            id: 0,
            left_start: true,
        });
        // Figur B auf Feld 10
        game.board.tiles[10] = Some(Piece {
            color: Color::Red,
            id: 0,
            left_start: true,
        });

        // 3. Die Aktion: 7 aufteilen in 3 und 4
        let split_moves = vec![(0, 3), (10, 4)];
        
        let action = Action {
            player: Color::Red,
            action: ActionKind::Split(split_moves),
            card: Card::Seven,
        };

        // 4. Ausführen und prüfen
        assert!(game.action(Card::Seven, action).is_ok());

        // Sind die alten Felder leer?
        assert!(game.board.tiles[0].is_none());
        assert!(game.board.tiles[10].is_none());

        // Sind die Figuren auf den neuen Feldern? (0 + 3 = 3) und (10 + 4 = 14)
        assert_eq!(game.board.tiles[3].as_ref().unwrap().color, Color::Red);
        assert_eq!(game.board.tiles[14].as_ref().unwrap().color, Color::Red);
        
        // Ist die Karte weg?
        assert!(game.player_mut_by_color(Color::Red).cards.is_empty());
    }
    #[test]
    fn test_split_atomic_fail() {
        let mut game = Game::new();
        game.red.cards = vec![Card::Seven];

        // Figur A auf 0
        game.board.tiles[0] = Some(Piece { color: Color::Red, id: 0, left_start: true });
        // Figur B auf 10
        game.board.tiles[10] = Some(Piece { color: Color::Red, id: 0, left_start: true });
        
        // BLOCKADE: Wir stellen eine eigene Figur auf Feld 14.
        // Wenn Figur B (von 10) 4 läuft, würde sie hier crashen.
        game.board.tiles[14] = Some(Piece { color: Color::Red, id: 0, left_start: true });

        // Aktion: 0->3 (wäre OK) und 10->14 (CRASH gegen eigene Figur)
        let split_moves = vec![(0, 3), (10, 4)];
        
        let action = Action {
            player: Color::Red,
            action: ActionKind::Split(split_moves),
            card: Card::Seven,
        };

        // Der Zug MUSS fehlschlagen
        assert!(game.action(Card::Seven, action).is_err());

        // WICHTIG: Das Board muss unverändert sein!
        // Figur A muss immer noch auf 0 stehen, obwohl ihr Teilzug okay gewesen wäre.
        assert!(game.board.tiles[0].is_some()); 
        assert!(game.board.tiles[10].is_some());
        
        // Feld 3 (wo A hinwollte) muss leer bleiben
        assert!(game.board.tiles[3].is_none());
    }
}
