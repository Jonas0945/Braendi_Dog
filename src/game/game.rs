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

            ActionKind::Split(moves) => {
    if !matches!(_card, Card::Seven | Card::Joker) {
        return Err("Split needs 7 or Joker");
    }

    if moves.iter().map(|(_, s)| s).sum::<u8>() != 7 {
        return Err("Split sum != 7");
    }

    let me = self.current_player_color;
    let mut sandbox = self.board.clone();
    let mut kills = Vec::with_capacity(moves.len());

    for (pos, steps) in moves {
        let p = sandbox.tiles[*pos as usize]
            .take()
            .ok_or("Source tile empty")?;

        if p.color != me {
            return Err("Not my piece");
        }

        let target = sandbox.calculate_target(*pos, *steps as i8, me)
            .ok_or("Target oob")?;

        if let Some(occ) = &sandbox.tiles[target as usize] {
            if occ.color == me {
                return Err("Self block");
            }
        }

        if let Some(victim) = sandbox.tiles[target as usize].replace(p) {
            kills.push((victim.color, victim.id));
        }
    }

    self.board = sandbox;

    for (c, id) in &kills {
        self.player_mut_by_color(*c).return_piece_id(*id);
    }

    self.player_mut_by_color(me).remove_card(_card);
    self.discard.push(_card);

    self.history.push(HistoryEntry {
        action: _action,
        beaten_piece_color: kills.last().map(|(c, _)| *c),
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

            ActionKind::Exchange => todo!(),
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

            ActionKind::Switch(from, to) => {
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
            ActionKind::Exchange => todo!(),
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

//hilfsfunktion um zu gucken ob der Spieler alle Pieces in seinem Haus hat 

        let player_finished = |color: Color| -> bool {

            let house_indices = match color {

                Color::Red    => [64, 65, 66, 67],

                Color::Green  => [68, 69, 70, 71],

                Color::Blue   => [72, 73, 74, 75],

                Color::Yellow => [76, 77, 78, 79],

            };



            // Prüfen Sind ALLE diese Felder mit einer Figur der RICHTIGEN Farbe belegt?

            house_indices.iter().all(|&idx| {

                self.board.tiles[idx].is_some_and(|p| p.color == color)

            })

        };



        // Ein Team gewinnt, wenn BEIDE Partner fertig sind.

        // Team 1: Rot & Blau

        // Team 2: Grün & Gelb

        let team_rb_wins = player_finished(Color::Red) && player_finished(Color::Blue);

        let team_gy_wins = player_finished(Color::Green) && player_finished(Color::Yellow);



        team_rb_wins || team_gy_wins

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

        game.board.tiles[1] = Some(Piece {
            color: Color::Red,
            id: 0,
            left_start: true,
        });

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
    
   
    
}
