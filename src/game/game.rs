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
    swapping_phase: true,
    swap_buffer: Vec::new(),
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

            ActionKind::Move(from, to) => {
                match _card {
                    Card::Jack => return Err("Cannot move piece with Jack(Bube) ."),
                    Card::Seven => return Err("Cannot move piece with Seven, need to go through split."),
                    _ => {}
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
                    if !from_piece.left_start {
                        return Err("Piece has not left start and cant enter house yet");
                    }
                    into_house = true;
                }

                // simulate backwards if card is Four
                if matches!(_card, Card::Four | Card::Joker) {
                    if !into_house {
                        let mut nfrom = from;

                        for _ in 0..4 { 
                            if nfrom == 0 {
                                nfrom = 63;
                            } else {
                                nfrom -= 1;
                            }

                            if let Some(p) = self.board.check_tile(nfrom) {
                                if !p.left_start {
                                    break;
                                }
                            }
                        }

                        if nfrom == to {
                            let mut moving_piece = self.board.tiles[from as usize].take().unwrap();
                            moving_piece.left_start = true;
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

                for _ in 0..max_steps {
                    if nfrom == to {
                        break;
                    }

                    actual_steps += 1;

                    if !in_house && nfrom == Board::start_field(current_player_color) && into_house {
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
                    _ => {
                        if !matches!(_card, Card::Joker) && _card.value() != actual_steps {
                            return Err("Value of card is not the same as steps.")
                        }
                    }
                }
                
                let mut moving_piece = self.board.tiles[from as usize].take().unwrap();
                moving_piece.left_start = true;
                let beaten_piece_color = if let Some(beaten) = self.board.tiles[to as usize].take() {
                    self.player_mut_by_color(beaten.color).pieces_to_place += 1;
                    Some(beaten.color)
                } else {
                    None
                };

                self.board.tiles[to as usize] = Some(moving_piece);
                self.player_mut_by_color(current_player_color).remove_card(_card);
                self.discard.push(_card);
                self.current_player_color = self.current_player_color.next();
                
                self.history.push(HistoryEntry {
                    action: _action,
                    beaten_piece_color,
                    switched_piece_color: None,
                });


                Ok(())
            },

            ActionKind::Split(ref moves) => {
                if !matches!(_card, Card::Seven | Card::Joker) {
                    return Err("Split needs 7 or Joker");
                }

                if moves.iter().map(|(_, s)| s).sum::<u8>() != 7 {
                    return Err("Split sum != 7");
                }

                let me = self.current_player_color;
                let mut sandbox = self.board.clone();
                let mut kills = Vec::with_capacity(moves.len());

                for &(pos, steps) in moves {
                    let p = sandbox.tiles[pos as usize]
                        .take()
                        .ok_or("Source tile empty")?;

                    if p.color != me {
                        return Err("Not my piece");
                    }

                    let target = sandbox.calculate_target(pos, steps as i8, me)
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

                for (c, id) in  &kills {
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

            ActionKind::Exchange(card_index) => {
            let playercolor = _action.player;
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
        let entry= self.history.pop().ok_or("No action to undo")?;

        match entry.action.action {
            ActionKind::Place => {
                let player = entry.action.player;
                let start = Board::start_field(player) as usize;
                if let Some(piece_on_start) = self.board.tiles[start].take() {
                    self.player_mut_by_color(player).return_piece_id(piece_on_start.id);
                } else {
                    return Err("No piece on start tile");
                    //Impossible if undoing Place
                }
                if let Some(beaten_color) = entry.beaten_piece_color {
                    let id_to_remove =self.player_mut_by_color(beaten_color).take_next_piece_id().unwrap();
                    self.board.tiles[start] = Some(Piece {
                        color: beaten_color,
                        id: self.player_mut_by_color(beaten_color).take_next_piece_id().unwrap(),
                        left_start: true,
                    });
                    self.player_mut_by_color(beaten_color).avaiable_ids.retain(|&x| x == id_to_remove);
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
            ActionKind::Split(_) => todo!(),
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
    fn test_steps_success() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Two];

        game.board.tiles[0] = Some(Piece {
            color: Color::Red,
            left_start: true,
        });

        let action = Action {
            player: Color::Red,
            action: ActionKind::Move(0, 2),
            card: Card::Two,
        };

        assert!(game.action(Card::Two, action).is_ok());
        assert!(game.board.tiles[0].is_none());
        assert_eq!(game.board.tiles[2].as_ref().unwrap().color, Color::Red);
        assert_eq!(game.current_player_color, Color::Green);
    }

    #[test]
    fn test_move_wrong_distance_fails() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Two];

        game.board.tiles[0] = Some(Piece {
            color: Color::Red,
            left_start: true,
        });

        let action = Action {
            player: Color::Red,
            action: ActionKind::Move(0, 3),
            card: Card::Two,
        };

        assert!(game.action(Card::Two, action).is_err());
        assert!(game.board.tiles[0].is_some());
    }

    #[test]
    fn test_move_opponent_piece_fails() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Two];

        game.board.tiles[0] = Some(Piece {
            color: Color::Green,
            left_start: true,
        });

        let action = Action {
            player: Color::Red,
            action: ActionKind::Move(0, 2),
            card: Card::Two,
        };

        assert!(game.action(Card::Two, action).is_err());
    }

    #[test]
    fn test_move_and_beat_opponent() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Two];

        game.board.tiles[0] = Some(Piece {
            color: Color::Red,
            left_start: true,
        });

        game.board.tiles[2] = Some(Piece {
            color: Color::Green,
            left_start: true,
        });

        let green_pieces_before =
            game.player_mut_by_color(Color::Green).pieces_to_place;

        let action = Action {
            player: Color::Red,
            action: ActionKind::Move(0, 2),
            card: Card::Two,
        };

        assert!(game.action(Card::Two, action).is_ok());
        assert_eq!(game.board.tiles[2].as_ref().unwrap().color, Color::Red);
        assert_eq!(
            game.player_mut_by_color(Color::Green).pieces_to_place,
            green_pieces_before + 1
        );
    }

    #[test]
    fn test_move_from_empty_tile_fails() {
        let mut game = Game::new();

        game.red.cards = vec![Card::Two];

        let action = Action {
            player: Color::Red,
            action: ActionKind::Move(0, 2),
            card: Card::Two,
        };

        assert!(game.action(Card::Two, action).is_err());
    }
    #[test]
    fn test_ace_move_one_step() {
        let mut game = Game::new();
        game.red.cards = vec![Card::Ace];

        game.board.tiles[0] = Some(Piece { color: Color::Red, left_start: true });

        let action = Action {
            player: Color::Red,
            action: ActionKind::Move(0, 1),
            card: Card::Ace,
        };

        assert!(game.action(Card::Ace, action).is_ok());
    }
    #[test]
    fn test_ace_move_eleven_steps() {
        let mut game = Game::new();
        game.red.cards = vec![Card::Ace];

        game.board.tiles[0] = Some(Piece { color: Color::Red, left_start: true });

        let action = Action {
            player: Color::Red,
            action: ActionKind::Move(0, 11),
            card: Card::Ace,
        };

        assert!(game.action(Card::Ace, action).is_ok());
    }



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
    
   #[test]
   fn test_undo_place() {
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
       assert!(game.undo().is_ok());
       assert!(game.board.tiles[start].is_none());
       assert_eq!(game.player_mut_by_color(Color::Red).pieces_to_place(), 4);
       assert!(game.player_mut_by_color(Color::Red).cards.contains(&card));
       assert!(!game.discard.contains(&card));
       assert_eq!(game.current_player_color, Color::Red);
   }

   #[test]
    fn test_undo_switch() {
       let mut game = Game::new();
       game.red.cards = vec![Card::Jack, Card::Joker];

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
       assert!(game.undo().is_ok());

       assert_eq!(game.board.tiles[1].as_ref().unwrap().color, Color::Red);
       assert_eq!(game.board.tiles[2].as_ref().unwrap().color, Color::Green);
    
    }
    
#[test]
fn test_undo_place_restores_state() {
    let mut g = Game::new();

    // give Red a card to place
    g.player_mut_by_color(Color::Red).cards.push(Card::Ace);

    // perform place
    let action = Action { player: Color::Red, card: Card::Ace, action: ActionKind::Place };
    assert!(g.action(Card::Ace, action.clone()).is_ok());

    // piece should now be on start
    let start = Board::start_field(Color::Red) as usize;
    assert!(g.board_state()[start].is_some());

    // undo
    assert!(g.undo().is_ok());

    // start should be empty again
    assert!(g.board_state()[start].is_none());

    // card returned to player's hand
    assert!(g.player_mut_by_color(Color::Red).cards.contains(&Card::Ace));

    // current player reset to Red
    assert_eq!(g.current_player().color, Color::Red);
}

    #[test]
    fn test_undo_switch_restores_tiles_and_card() {
        let mut g = Game::new();
    
        // prepare two pieces on board at non-house positions 0 and 1
        let from = 0u8;
        let to = 1u8;
    
        // Put a Red piece at 'from' and a Green piece at 'to'
        g.board.tiles[from as usize] = Some(Piece::new_test(Color::Red, 0, true));
        g.board.tiles[to as usize] = Some(Piece::new_test(Color::Green, 1, true));
    
        // give Red a Jack to perform switch
        g.player_mut_by_color(Color::Red).cards.push(Card::Jack);
    
        let action = Action { player: Color::Red, card: Card::Jack, action: ActionKind::Switch(from, to) };
        assert!(g.action(Card::Jack, action.clone()).is_ok());
    
        // tiles should be swapped
        let after_from = g.board.check_tile(from).unwrap();
        let after_to = g.board.check_tile(to).unwrap();
        assert_eq!(after_from.color, Color::Green);
        assert_eq!(after_to.color, Color::Red);
    
        // undo the switch
        assert!(g.undo().is_ok());
    
        // tiles back to original
        let back_from = g.board.check_tile(from).unwrap();
        let back_to = g.board.check_tile(to).unwrap();
        assert_eq!(back_from.color, Color::Red);
        assert_eq!(back_to.color, Color::Green);
    
        // card returned to Red
        assert!(g.player_mut_by_color(Color::Red).cards.contains(&Card::Jack));
    
        // current player reset to Red
        assert_eq!(g.current_player().color, Color::Red);
    }
       #[test]
    fn double_swap_by_same_player_through_index(){
        let mut game = Game::new();
        game.new_round();
        game.red.swapped_cards_count +=1;
        let a1=Action{player: game.red.color, card:Card::Eight, action: ActionKind::Swap(3),};
        assert_eq!(game.round, 1);
        // assert!(game.action(Card::Eight, a1).is_err());
    assert_eq!(game.action(Card::Eight, a1).unwrap_err(), "Dieser Spieler darf keine Karte tauschen");
    }
    #[test]
    fn swapping_in_not_swap_phase(){
        let mut game = Game::new();
        game.new_round();
        let a1=Action{player: game.red.color, card:Card::Eight, action: ActionKind::Swap(3),};
        game.swapping_phase = false;
        // assert!(game.action(Card::Eight, a1).is_err());
    assert_eq!(game.action(Card::Eight, a1).unwrap_err(), "In dieser Phase des Spiels dürfen keine Karten getauscht werden");
    }
    #[test]
    fn double_swap_by_same_player(){
        let mut game = Game::new();
        game.new_round();
        let a1=Action{player: game.red.color, card:Card::Eight, action: ActionKind::Swap(3),};
        game.action(Card::Seven, a1).expect("Es darf pro Spieler nur eine Karte getauscht werden");
        //nur zu test zwecken
        game.red.swapped_cards_count =0;
        let a2=Action{player: game.red.color, card:Card::Eight, action: ActionKind::Swap(4),};
        // assert!(game.action(Card::Eight, a1).is_err());
    assert_eq!(game.action(Card::Eight, a2).unwrap_err(), "Es darf pro Spieler nur eine Karte getauscht werden");
    }
    #[test]
    fn swapping_index_overflow(){
        let mut game = Game::new();
        game.new_round();
        let a1=Action{player: game.red.color, card:Card::Eight, action: ActionKind::Swap(5),};
    assert_eq!(game.action(Card::Eight, a1).unwrap_err(), "Ungültiger Kartenindex für den Tausch");
    }
    
    }
    }
