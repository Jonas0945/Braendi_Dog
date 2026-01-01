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
    
    trading_phase: bool,
    trade_buffer: Vec<(Color,Card)>,

    deck: Deck,
    discard: Vec<Card>,

    red: Player,
    green: Player,
    blue: Player,
    yellow: Player,

    current_player_color: Color,
    pub split_rest: Option<u8>,
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

    pub fn player_by_color(&self, color: Color) -> &Player {
        match color {
            Color::Red => &self.red,
            Color::Green => &self.green,
            Color::Blue => &self.blue,
            Color::Yellow => &self.yellow,
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

    pub fn play(&mut self, s: &str) -> Result<(), &'static str> {
        let action: Action = s.parse()?;
        let card = action.card;
        self.action(card, action)
    }

    fn check_if_any_move_possible(&self) -> bool{
        let player = self.player_by_color(self.current_player_color);

        for card in &player.cards {
            if self.is_card_playable(*card) {
                return true;
            }
        }
        false
    }
    

    fn is_card_playable(&self, card: Card) -> bool {
        let color = self.current_player_color;

        if matches!(card, Card::Ace | Card::King | Card::Joker) {
            let start_field = Board::start_field(self.teammate_or_self(color)) as usize;
            if self.player_by_color(self.teammate_or_self(color)).pieces_to_place > 0 {
                if let Some(piece) = &self.board.tiles[start_field] {
                    if piece.color != self.teammate_or_self(color) || piece.left_start {
                        return true; 
                    }
                } else {
                    return true; 
                }
            }
        }

        if matches!(card, Card::Jack | Card::Joker) {
            let my_pieces = self.find_movable_pieces(color);
            let all_pieces = self.count_interchangeable_pieces();
            
            if !my_pieces.is_empty() && all_pieces >= 2 {
                return true;
            }
        }

        let pieces = self.find_movable_pieces(color);
        
       
        let distances = if card == Card::Seven {
            vec![1, 2, 3, 4, 5, 6, 7] 
        } else {
            card.possible_distances().unwrap_or_default()
        };

        for &pos in &pieces {
            for &dist in &distances {
                let try_backward = (matches!(card, Card::Four | Card::Joker) && dist == 4) 
                                    || (matches!(card, Card::Seven) && false); // 7 geht nur vorwärts

                if self.can_piece_move_distance(pos, dist, try_backward) {
                    return true;
                }
            }
        }

        false
    }

    fn find_movable_pieces(&self, color: Color) -> Vec<usize> {
        let mut positions = Vec::new();
        let target_color = self.teammate_or_self(color); // Partner-Logik

        for (idx, tile) in self.board.tiles.iter().enumerate() {
            if let Some(piece) = tile {
                if piece.color == target_color || (piece.color == color) {
                    if self.player_by_color(color).can_control_piece(*piece) {
                        positions.push(idx);
                    }
                }
            }
        }
        positions
    }

    fn count_interchangeable_pieces(&self) -> usize {
        self.board.tiles.iter().enumerate().filter(|(idx, tile)| {
            if let Some(piece) = tile {
                if *idx < 64 && piece.left_start {
                    return true;
                }
            }
            false
        }).count()
    }
    
    fn can_piece_move_distance(&self, from: usize, dist: u8, backward: bool) -> bool {
        let piece = self.board.tiles[from].as_ref().unwrap();
        
        for to in 0..80 {
            if !backward {
                if self.board.distance_between(from as u8, to as u8, piece.color) == Some(dist) {
                    if let Some(path) = self.board.passed_tiles(from as u8, to as u8, piece.color, false) {
                        if self.is_path_free(&path) { return true; }
                    }
                }
            } 
            else {
                if self.board.distance_between(to as u8, from as u8, piece.color) == Some(dist) {
                    if let Some(path) = self.board.passed_tiles(from as u8, to as u8, piece.color, true) {
                        if self.is_path_free(&path) { return true; }
                    }
                }
            }
        }
        false
    }

    fn is_path_free(&self, path: &[u8]) -> bool {
        for &tile in path {
            if let Some(p) = &self.board.tiles[tile as usize] {
                if tile >= 64 { return false; } 
                if !p.left_start { return false; }
            }
        }
        true
    }

    fn teammate_or_self(&self, color: Color) -> Color {
        if self.player_by_color(color).pieces_in_house == 4 {
            color.teammate()
        } else {
            color
        }
    }

    fn all_players_out_of_cards(&self) -> bool {
        self.red.cards.is_empty()
            && self.green.cards.is_empty()
            && self.blue.cards.is_empty()
            && self.yellow.cards.is_empty()
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
    fn undo_action(&mut self) -> Result<(), &'static str>;

    // // Undoes the last complete turn, including all actions that belong to it
    fn undo_turn(&mut self) -> Result<(), &'static str>;

    // Undoes multiple turns in sequence
    fn undo_sequence(&mut self, turns: usize) -> Result<(), &'static str>;

    // Gives players new cards after previous round is finished
    fn new_round(&mut self);

    // Checks if there is yet a winning team
    fn is_winner(&self) -> bool;
}

impl DogGame for Game {
    fn new() -> Self {
        Self {
            board: Board::new(),
            history: Vec::new(),
            round: 1,

            trading_phase: true,
            trade_buffer: Vec::new(),

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
        
        if self.split_rest.is_some(){
            if !matches!(_action.action, ActionKind::Split(_, _)) {
                return Err("Cannot perform actions other than Split during splitting phase.");
            }
        }


        if !self.player_mut_by_color(self.current_player_color).cards.contains(&_card) {
            return Err("Card not in player's hand.");
        }

        if self.trading_phase && !matches!(_action.action, ActionKind::Trade) {
            return Err("Cannot perform actions other than Trade during swapping phase.");
        }

        if self.current_player_color != _action.player {
            return Err("It's not this player's turn.");
        }

        match _action.action {
            ActionKind::Place => {

                match _card {
                    Card::Ace | Card::King | Card::Joker => {},
                    _ => return Err("Cannot place piece with this card."),
                }
                
                // Check if player can interact with teammate pieces
                let current_player_color = self.current_player_color;

                let place_color = if self.player_by_color(current_player_color).pieces_in_house == 4 {
                    current_player_color.teammate()
                } else {
                    current_player_color
                };

                let start = Board::start_field(place_color) as usize;

                if self.player_by_color(place_color).pieces_to_place == 0 {
                    return Err("Cannot place piece: no pieces left to place.");
                }

                let mut beaten_piece_color = None;

                if let Some(piece) = self.board.tiles[start].take() {
                    if piece.color == place_color && !piece.left_start {
                        self.board.tiles[start] = Some(piece);
                        return Err("Cannot place piece: your protected piece is blocking.")
                    }
                    beaten_piece_color = Some(piece.color);
                    self.player_mut_by_color(piece.color).pieces_to_place += 1;
                }
                
                // Piece placement and history update
                self.board.tiles[start] = Some (Piece::new(place_color));

                self.player_mut_by_color(current_player_color).remove_card(_card);
                self.discard.push(_card);

                self.history.push(HistoryEntry {
                    action: _action,
                    beaten_piece_color,
                    interchanged_piece_color: None,
                    placed_piece_color: Some(place_color),

                    split_rest_before: None,
                    trade_buffer_before: Vec::new(),
                    left_start_before: false,

                    cards_dealt: Vec::new(),
                });

                self.player_mut_by_color(place_color).pieces_to_place -= 1;
                self.current_player_color = self.current_player_color.next();
            }

            ActionKind::Move(from, to) => {
                match _card {
                    Card::Jack => return Err("Cannot move piece with Jack."),
                    Card::Seven => return Err("Cannot move with Seven (-> Split)"),
                    _ => {},
                }
                
                let moving_piece = match self.board.check_tile(from) {
                    Some(p) => p,
                    None => return Err("Invalid move: no piece found."),
                };

                let left_start_before = moving_piece.left_start;

                let current_player_color = self.current_player_color;
                let current_player = self.player_by_color(current_player_color);

                if !current_player.can_control_piece(moving_piece) {
                    return Err("You cannot move this piece.");
                }

                // Calculate distances and check if card allows the move
                let forward_distance = self.board.distance_between(from, to, moving_piece.color);
                let backward_distance = self.board.distance_between(to, from, moving_piece.color);

                if !self.can_card_move(_card, forward_distance, backward_distance) {
                    return Err("Move not allowed with this card")
                };

                // Calculate path + direction and check for blocking pieces
                let is_backward = matches!(_card, Card::Four | Card::Joker)
                    && backward_distance == Some(4);

                let path = match self.board.passed_tiles(from, to, moving_piece.color, is_backward) {
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
                self.board.tiles[from as usize].take();

                // Remove piece from destination tile if opponent piece is there
                let mut beaten_piece_color = None;

                if let Some(beaten_piece) = self.board.tiles[to as usize].take() {
                    beaten_piece_color = Some(beaten_piece.color);
                    self.player_mut_by_color(beaten_piece.color).pieces_to_place += 1;
                }

                // Piece placement and history update
                self.board.tiles[to as usize] = Some(Piece {
                    color: moving_piece.color, 
                    left_start: true 
                });

                // Piece moves into house
                if from < 64 && to >= 64 { 
                    self.player_mut_by_color(moving_piece.color).pieces_in_house += 1;
                }

                self.player_mut_by_color(current_player_color).remove_card(_card);
                self.discard.push(_card);

                self.history.push(HistoryEntry { 
                    action: _action,

                    beaten_piece_color, 
                    interchanged_piece_color: None,
                    placed_piece_color: None,

                    split_rest_before: None,
                    trade_buffer_before: Vec::new(),
                    left_start_before,

                    cards_dealt: Vec::new(),
                });

                self.current_player_color = self.current_player_color.next();        
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

                let current_player = self.player_by_color(self.current_player_color);

                if !current_player.can_control_piece(from_piece) {
                    return Err("Cannot interchange from a piece you don't control")
                }

                if !from_piece.left_start || !to_piece.left_start {
                    return Err("Cannot interchange with protected piece.")
                }

                let from_index = from as usize;
                let to_index = to as usize;

                let interchanged_color = to_piece.color;

                self.board.tiles[from_index] = Some(to_piece);
                self.board.tiles[to_index] = Some(from_piece);

                self.player_mut_by_color(self.current_player_color).remove_card(_card);
                self.discard.push(_card);

                self.history.push(HistoryEntry {
                    action: _action,

                    beaten_piece_color: None,
                    interchanged_piece_color: Some(interchanged_color),
                    placed_piece_color: None,

                    split_rest_before: None,
                    trade_buffer_before: Vec::new(),
                    left_start_before: true,

                    cards_dealt: Vec::new(),
                });
                
                self.current_player_color = self.current_player_color.next();
            },

            ActionKind::Trade => {
                if !self.trading_phase {
                    return Err("Cannot trade cards outside trading phase.");
                }

                let current_player_color = self.current_player_color;
                let trade_buffer_before = self.trade_buffer.clone();

                if self.trade_buffer.len() >= 4 {
                    return Err("Cannot trade more than one card per player.");
                }

                let card_index = self.player_mut_by_color(current_player_color).cards.iter().position(|&c| c == _card)
                    .ok_or("Cannot trade: card not found in player's hand.")?;

                let removed_card = self.player_mut_by_color(current_player_color).cards.remove(card_index);

                self.trade_buffer.push((current_player_color, removed_card));

                if self.trade_buffer.len() == 4 {
                    let trades: Vec<_> = self.trade_buffer.drain(..).collect();

                    for (col, c) in trades {
                        let teammate_color = col.teammate();
                        self.player_mut_by_color(teammate_color).cards.push(c);
                    }
                  self.trading_phase = false;  
                }

                self.history.push(HistoryEntry {
                    action: _action,

                    beaten_piece_color: None,
                    interchanged_piece_color: None,
                    placed_piece_color: None,

                    split_rest_before: None,
                    trade_buffer_before,
                    left_start_before: false,

                    cards_dealt: Vec::new(),
                });

                self.current_player_color = current_player_color.next();           
            },

            ActionKind::Split(from, to) => {
                match _card {
                    Card::Seven | Card::Joker => {},
                    _ => return Err("Cannot split move with this card.")
                }

                let current_player_color = self.current_player_color;
                let teammate_color = current_player_color.teammate();
                let mut split_rest_before = self.split_rest;

                let moving_piece = match self.board.check_tile(from) {
                    Some(p) => p,
                    None => return Err("Invalid move: no piece found."),
                };

                let mut left_start_before = moving_piece.left_start;

                if moving_piece.color != current_player_color
                    && moving_piece.color != teammate_color {
                    return Err("Cannot split-move opponent's piece.");
                }

                let total_distance = self.board
                    .distance_between(from, to, moving_piece.color)
                    .ok_or("Invalid action.")?;

                if total_distance == 0 || total_distance > 7 {
                    return Err("Split move must have 1..7 steps.");
                }

                // Check split_rest
                let mut remaining_steps = self.split_rest.unwrap_or(7);
                if total_distance > remaining_steps {
                    return Err("Cannot move more steps than remaining split.");
                }

                // Calculate path and check for blocking pieces
                let path = match self.board.passed_tiles(from, to, moving_piece.color, false) {
                    Some(p) => p,
                    None => return Err("Invalid split move: path cannot be calculated.")
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
                let mut current_position = from;

                for &tile in &path {

                    // Create "mini"- history if piece is beaten
                    if let Some(beaten_piece) = self.board.tiles[tile as usize].take() {
                        
                        self.player_mut_by_color(beaten_piece.color).pieces_to_place += 1;
                        
                        let distance = self.board
                            .distance_between(current_position, tile, moving_piece.color)
                            .expect("Distance must exist");
                        
                        // Mini history
                        self.history.push( HistoryEntry { 
                            action: Action { 
                                player: current_player_color, 
                                card: _card, 
                                action: ActionKind::Split(current_position, tile) 
                            },

                            beaten_piece_color: Some(beaten_piece.color), 
                            interchanged_piece_color: None,
                            placed_piece_color: None,

                            split_rest_before,
                            trade_buffer_before: Vec::new(),
                            left_start_before,

                            cards_dealt: Vec::new(),
                        });

                        // Update step-mechanism
                        remaining_steps -= distance;
                        split_rest_before = Some(remaining_steps);

                        current_position = tile;
                        left_start_before = true;
                    }
                }

                // Piece placement and history update
                self.board.tiles[from as usize].take();
                self.board.tiles[to as usize] = Some(Piece { 
                    color: moving_piece.color, 
                    left_start: true 
                });

                if from < 64 && to >= 64 {
                    self.player_mut_by_color(moving_piece.color).pieces_in_house += 1;
                }

                // History update if last step doesn't beat piece
                if current_position != to {

                    let distance = self.board
                            .distance_between(current_position, to, moving_piece.color)
                            .expect("Distance must exist");

                    if current_position != from {
                        left_start_before = true;
                    }
                    
                    self.history.push(HistoryEntry {
                        action: Action { 
                            player: current_player_color, 
                            card: _card, 
                            action: ActionKind::Split(current_position, to) 
                        }, 
                        beaten_piece_color: None, 
                        interchanged_piece_color: None,
                        placed_piece_color: None,

                        split_rest_before,
                        trade_buffer_before: Vec::new(),
                        left_start_before,

                        cards_dealt: Vec::new(),
                    });

                    remaining_steps -= distance;
                }

                // Update split_rest
                if remaining_steps == 0 {
                    self.split_rest = None;

                    // Change player
                    self.player_mut_by_color(current_player_color).remove_card(_card);
                    self.discard.push(_card);

                    self.current_player_color = self.current_player_color.next();

                } else {
                    self.split_rest = Some(remaining_steps);
                }
            },

            ActionKind::Remove => {
                if self.check_if_any_move_possible(){
                    return Err("Zugzwang. Du darfst nicht abwerfen!");
                }

                let current_player_color = self.current_player_color;

                let card_index = self.player_mut_by_color(current_player_color).cards.iter().position(|&c| c == _card)
                    .ok_or("Cannot remove: card not found in player's hand.")?;

                self.player_mut_by_color(current_player_color).cards.remove(card_index);
                self.discard.push(_card);

                self.history.push(HistoryEntry {
                    action: _action,

                    beaten_piece_color: None,
                    interchanged_piece_color: None,
                    placed_piece_color: None,

                    split_rest_before: None,
                    trade_buffer_before: Vec::new(),
                    left_start_before: false,

                    cards_dealt: Vec::new(),
                });

                self.current_player_color = self.current_player_color.next();
            },
        }
    
        if self.all_players_out_of_cards() {
            self.new_round();
        };

        Ok(())
    }

    fn undo_action(&mut self) -> Result<(), &'static str> {
        let entry= self.history.pop().ok_or("No action to undo")?;

        if !entry.cards_dealt.is_empty() {
            for (color, _) in &entry.cards_dealt {
                let player = self.player_mut_by_color(*color);
                player.cards.clear();
            }
            self.trading_phase = false;
            self.round -= 1;
        }

        let entry_player_color = entry.action.player;
        let played_card = entry.action.card;

        match entry.action.action {
            ActionKind::Place => {
                let placed_piece_color = entry.placed_piece_color.unwrap();
                let start = Board::start_field(placed_piece_color) as usize;

                self.board.tiles[start].take();
                self.player_mut_by_color(placed_piece_color).pieces_to_place += 1;

                if let Some(beaten_piece_color) = entry.beaten_piece_color {
                    self.board.tiles[start] = Some(Piece {
                        color: beaten_piece_color,
                        left_start: true,
                    });

                    self.player_mut_by_color(beaten_piece_color).pieces_to_place -= 1;
                }

                self.discard.pop();
                self.player_mut_by_color(entry_player_color).cards.push(played_card);

                self.current_player_color = entry_player_color;
            },

            ActionKind::Interchange(from, to) => {
                let from_index = from as usize;
                let to_index = to as usize;

                let from_piece = self.board.tiles[from_index].take();
                let to_piece = self.board.tiles[to_index].take();

                self.board.tiles[from_index] = to_piece;
                self.board.tiles[to_index] = from_piece;

                self.discard.pop();
                self.player_mut_by_color(entry_player_color).cards.push(played_card);

                self.current_player_color = entry_player_color;
            },

            ActionKind::Move(from, to) => {
                let from_index = from as usize;
                let to_index = to as usize;

                let moved_piece = self.board.tiles[to_index].take();
                let moved_piece_color = moved_piece.unwrap().color;
                self.board.tiles[from_index] = Some(Piece { 
                    color: moved_piece_color, 
                    left_start: entry.left_start_before 
                });

                if from_index < 64 && to_index >= 64 {
                    self.player_mut_by_color(moved_piece_color).pieces_in_house -= 1;
                }

                if let Some(beaten_piece_color) = entry.beaten_piece_color {
                    self.board.tiles[to_index] = Some(Piece {
                        color: beaten_piece_color,
                        left_start: true,
                    });

                    self.player_mut_by_color(beaten_piece_color).pieces_to_place -= 1;
                }

                self.discard.pop();
                self.player_mut_by_color(entry_player_color).cards.push(played_card);

                self.current_player_color = entry_player_color;
            },

            ActionKind::Trade => {

                // Check if trade phase just ended
                if entry.trade_buffer_before.len() == 3 {
                    
                    let mut trades: Vec<_> = entry.trade_buffer_before.clone();
                    trades.push((entry_player_color, played_card));

                    for (player_color, card) in trades {
                        let teammate_color = player_color.teammate();

                        self.player_mut_by_color(teammate_color).remove_card(card);
                    }

                    self.trading_phase = true;

                }

                self.player_mut_by_color(entry_player_color).cards.push(played_card);
                self.trade_buffer = entry.trade_buffer_before;
                self.current_player_color = entry_player_color;
            },

            ActionKind::Split(from, to) => {
                let from_index = from as usize;
                let to_index = to as usize;

                let moved_piece = self.board.tiles[to_index].take();
                let moved_piece_color = moved_piece.unwrap().color;
                self.board.tiles[from_index] = moved_piece;

                if from_index < 64 && to_index >= 64 {
                    self.player_mut_by_color(moved_piece_color).pieces_in_house -= 1;
                }

                if let Some(beaten_piece_color) = entry.beaten_piece_color {
                    self.board.tiles[to_index] = Some(Piece {
                        color: beaten_piece_color,
                        left_start: true,
                    });

                    self.player_mut_by_color(beaten_piece_color).pieces_to_place -= 1;
                }

                // Return card if split just began
                if entry.split_rest_before.is_none() {
                    self.discard.pop();
                    self.player_mut_by_color(entry_player_color).cards.push(played_card);
                }

                self.split_rest = entry.split_rest_before;
                self.current_player_color = entry_player_color;                
            },

            ActionKind::Remove => {
                self.discard.pop();
                self.player_mut_by_color(entry_player_color).cards.push(played_card);

                self.current_player_color = entry_player_color;
            },
        }

        Ok(())
    }
    
    fn undo_turn(&mut self) -> Result<(), &'static str> {
        if self.history.is_empty() {
            return Err("Nothing to undo");
        }

        // Loop through multi-step split or whole trading phase
        loop {
            let (action_kind, split_rest_before, trade_buffer_before) = {
                let entry = self.history.last().unwrap();
                (
                    entry.action.action.clone(),
                    entry.split_rest_before,
                    entry.trade_buffer_before.clone(),
                )
            };

            self.undo_action()?;

            match action_kind {
                ActionKind::Split(_, _) => {
                    if split_rest_before.is_none() {
                        break;
                    }
                },
            
                ActionKind::Trade => {
                    if trade_buffer_before.is_empty() {
                        break
                    }
                },

                _ => break,
            }
        }

        Ok(())
    }

    fn undo_sequence(&mut self, turns: usize) -> Result<(), &'static str> {
        for _ in 0..turns {
            self.undo_turn()?;
        }
        Ok(())
    }
        
    fn new_round(&mut self) {

        self.deck = Deck::new();
        self.deck.shuffle();
        self.discard.clear();

        let current_round = (self.round % 4) as usize;
        let cards_to_deal = CARDS_PER_ROUND[current_round - 1];

        for _ in 0..cards_to_deal {
            self.red.cards.push(self.deck.draw().unwrap());
            self.green.cards.push(self.deck.draw().unwrap());
            self.blue.cards.push(self.deck.draw().unwrap());
            self.yellow.cards.push(self.deck.draw().unwrap());
        }

        self.trading_phase = true;

        self.current_player_color = match self.round % 4 {
            0 => Color::Yellow, 
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Blue,
            _ => unreachable!(),
        };
        
        self.round += 1;

        if let Some(entry) = self.history.last_mut() {
            entry.cards_dealt = vec![
                (Color::Red, self.red.cards.clone()),
                (Color::Green, self.green.cards.clone()),
                (Color::Blue, self.blue.cards.clone()),
                (Color::Yellow, self.yellow.cards.clone()),
            ];
        }
    }
    
    fn is_winner(&self) -> bool {
        let current_player = self.player_by_color(self.current_player_color);
        let teammate = self.player_by_color(self.current_player_color.teammate());

        current_player.pieces_in_house == 4 && teammate.pieces_in_house == 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod helper_tests {
        use super::*;

        mod play_tests {
            use super::*;

            fn setup_game() -> Game {
                let mut game = Game::new();
                game.red.cards = vec![Card::Ace, Card::Two, Card::Three];
                game.green.cards = vec![Card::Four, Card::Five, Card::Six];
                game.blue.cards = vec![Card::Seven, Card::Eight, Card::Nine];
                game.yellow.cards = vec![Card::Ten, Card::Jack, Card::Queen];
                game
            }

            #[test]
            fn play_valid_place() {
                let mut game = setup_game();
                game.trading_phase = false;

                let input = "R 1 P"; // Red places with Ace

                assert!(game.play(input).is_ok());

                assert!(!game.red.cards.contains(&Card::Ace));
                assert_eq!(game.current_player_color, Color::Green);
            }

            #[test]
            fn play_invalid_card_string() {
                let mut game = setup_game();
                let input = "R X P";

                assert!(game.play(input).is_err());
                assert_eq!(game.current_player_color, Color::Red);
            }

            #[test]
            fn play_invalid_player() {
                let mut game = setup_game();
                let input = "S 5 T";

                assert!(game.play(input).is_err());
                assert_eq!(game.current_player_color, Color::Red);
            }

            #[test]
            fn play_invalid_action() {
                let mut game = setup_game();
                let input = "R 5 Z";

                assert!(game.play(input).is_err());
                assert_eq!(game.current_player_color, Color::Red);
            }

            #[test]
            fn play_trade_sequence() {
                let mut game = setup_game();

                let inputs = ["R 2 T", "G 5 T", "B 7 T", "Y 10 T"];
                for input in inputs.iter() {
                    assert!(game.play(input).is_ok());
                }

                assert!(!game.trading_phase);
                assert_eq!(game.red.cards.len(), 3);
                assert_eq!(game.green.cards.len(), 3);
            }

            #[test]
            fn play_place_action() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Joker];

                assert!(game.play("R 0 P").is_ok());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
                assert_eq!(game.red.pieces_to_place, 3)
            }

            #[test]
            fn play_move_action() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Four];
                game.board.tiles[60] = Some(Piece { 
                    color: Color::Red, 
                    left_start: true 
                });

                assert!(game.play("R 4 M 60 56").is_ok());
                assert!(game.board.tiles[60].is_none());
                assert_eq!(game.board.tiles[56].as_ref().unwrap().color, Color::Red);
            }

            #[test]
            fn play_remove_action() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Joker];

                assert!(game.play("R 0 R").is_ok());
                assert!(game.red.cards.is_empty());
            }

            #[test]
            fn play_interchange_action() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Joker];

                game.board.tiles[60] = Some(Piece { 
                    color: Color::Red, 
                    left_start: true 
                });

                game.board.tiles[56] = Some(Piece { 
                    color: Color::Green, 
                    left_start: true 
                });

                assert!(game.play("R 0 I 60 56").is_ok());
                assert_eq!(game.board.tiles[56].as_ref().unwrap().color, Color::Red);
                assert_eq!(game.board.tiles[60].as_ref().unwrap().color, Color::Green);
            }

            #[test]
            fn play_split_action() {
                let mut game = Game::new();
                game.trading_phase = false;
                game.red.cards = vec![Card::Seven];

                // Setup pieces for split
                game.board.tiles[0] = Some(Piece { color: Color::Red, left_start: true });
                game.board.tiles[3] = Some(Piece { color: Color::Green, left_start: true });

                let input1 = "R 7 S 0 5";
                let input2 = "R 7 S 5 7";

                assert!(game.play(input1).is_ok());
                assert_eq!(game.split_rest, Some(2));

                assert!(game.play(input2).is_ok());
                assert_eq!(game.split_rest, None);
                assert_eq!(game.current_player_color, Color::Green);
            }

            #[test]
            fn play_invalid_split() {
                let mut game = setup_game();
                game.red.cards = vec![Card::Five];

                let input = "R 5 S 0 10";

                assert!(game.play(input).is_err());
                assert_eq!(game.split_rest, None);
            }
        }
    }
    mod action_tests {
        use super::*;

        #[test]
        fn wrong_player_cannot_act() {
            let mut game = Game::new();
            game.trading_phase = false;
            game.current_player_color = Color::Red;

            let action = Action {
                player: Color::Green,
                action: ActionKind::Place,
                card: Card::Ace,
            };
      
            assert!(game.action(Card::Ace, action).is_err());
            assert_eq!(game.current_player_color, Color::Red);
        }

        #[test]
        fn invalid_action_does_not_change_state() {
            let mut game = Game::new();
            game.trading_phase = false;

            game.red.cards = vec![Card::Ace];
            game.current_player_color = Color::Red;

            let board_before = game.board.tiles.clone();
            let discard_before = game.discard.clone();
            let history_len_before = game.history.len();
            let red_cards_before = game.red.cards.clone();

            let action = Action {
                player: Color::Red,
                action: ActionKind::Place,
                card: Card::Two,
            };

            assert!(game.action(Card::Two, action).is_err());

            // 
            assert_eq!(game.board.tiles, board_before);
            assert_eq!(game.discard, discard_before);
            assert_eq!(game.history.len(), history_len_before);
            assert_eq!(game.red.cards, red_cards_before);
            assert_eq!(game.current_player_color, Color::Red);
        }

         #[test]
        fn cannot_act_during_other_phase() {
            let mut game = Game::new();

            game.red.cards = vec![Card::Ace];

            let action = Action {
                player: Color::Red,
                action: ActionKind::Remove,
                card: Card::Ace,
            };

            assert!(game.action(Card::Ace, action).is_err());
            assert_eq!(game.current_player_color, Color::Red);
            assert!(game.red.cards.contains(&Card::Ace));
        }

        mod action_place_tests {
            use super::*;

            #[test]
            fn place_on_empty_start() {
                let mut game = Game::new();
                game.trading_phase = false;

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
            fn cannot_place_without_pieces_to_place() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Ace];
                game.red.pieces_to_place = 0;

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Place,
                    card: Card::Ace,
                };

                assert!(game.action(Card::Ace, action).is_err());
            }

            #[test]
            fn invalid_card_cannot_place() {
                let mut game = Game::new();
                game.trading_phase = false;

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
                game.trading_phase = false;

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
            fn cannot_place_on_partner_protected_piece() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.pieces_in_house = 4;
                game.red.cards = vec![Card::Ace];

                let start = Board::start_field(Color::Blue) as usize;

                game.board.tiles[start] = Some(Piece {
                    color: Color::Blue,
                    left_start: false,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Place,
                    card: Card::Ace,
                };

                assert!(game.action(Card::Ace, action).is_err());
            }

            #[test]
            fn place_beat_opponent() {
                let mut game = Game::new();
                game.trading_phase = false;

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
            fn place_partner_piece() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.pieces_in_house = 4;
                game.red.cards = vec![Card::Ace];
                game.blue.pieces_to_place = 4;

                let start = Board::start_field(Color::Blue) as usize;
                let card = Card::Ace;

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Place,
                    card,
                };

                assert!(game.action(card, action).is_ok());
                assert_eq!(game.board.tiles[start].as_ref().unwrap().color, Color::Blue);
                assert_eq!(game.player_mut_by_color(Color::Blue).pieces_to_place, 3);
                assert!(!game.player_mut_by_color(Color::Red).cards.contains(&card));
            }

            #[test]
            fn invalid_place_does_not_change_state() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Ace];

                let start = Board::start_field(Color::Red) as usize;

                // block start with own protected piece
                game.board.tiles[start] = Some(Piece {
                    color: Color::Red,
                    left_start: false,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Place,
                    card: Card::Ace,
                };

                let current_player = game.current_player_color;
                let cards_before = game.red.cards.clone();

                assert!(game.action(Card::Ace, action).is_err());

                assert_eq!(game.current_player_color, current_player);
                assert_eq!(game.red.cards, cards_before);
                assert!(game.discard.is_empty());
            }
        }
        
        mod action_interchange_tests {
            use super::*;

            #[test]
            fn interchange_success() {
                let mut game = Game::new();
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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

            #[test]
            fn can_interchange_partner_piece() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.pieces_in_house = 4;

                game.red.cards = vec![Card::Jack, Card::Joker];
                game.blue.cards = vec![Card::Jack, Card::Joker];

                game.board.tiles[1] = Some(Piece { color: Color::Blue, left_start: true });
                game.board.tiles[2] = Some(Piece { color: Color::Red, left_start: true });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Interchange(1, 2),
                    card: Card::Jack,
                };

                assert!(game.action(Card::Jack, action).is_ok());

                assert_eq!(game.board.tiles[1].as_ref().unwrap().color, Color::Red);
                assert_eq!(game.board.tiles[2].as_ref().unwrap().color, Color::Blue);

                assert!(!game.player_mut_by_color(Color::Red).cards.contains(&Card::Jack));
                assert!(game.discard.contains(&Card::Jack));

                let entry = game.history.last().unwrap();
                assert_eq!(entry.interchanged_piece_color, Some(Color::Red));
                assert_eq!(entry.beaten_piece_color, None);
            }

            #[test]
            fn cannot_interchange_partner_if_less_than_4_in_house() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.pieces_in_house = 3;

                game.red.cards = vec![Card::Jack];
                game.blue.cards = vec![Card::Jack];

                game.board.tiles[1] = Some(Piece { color: Color::Blue, left_start: true });
                game.board.tiles[2] = Some(Piece { color: Color::Red, left_start: true });

                let action1 = Action {
                    player: Color::Red,
                    action: ActionKind::Interchange(1, 2),
                    card: Card::Jack,
                };

                let action2 = Action {
                    player: Color::Red,
                    action: ActionKind::Interchange(2, 1),
                    card: Card::Jack,
                };

                assert!(game.action(Card::Jack, action1).is_err());
                assert!(game.action(Card::Jack, action2).is_ok());
            }

            #[test]
            fn partner_piece_protected_cannot_interchange() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.pieces_in_house = 4;

                game.red.cards = vec![Card::Jack];
                game.blue.cards = vec![Card::Jack];

                game.board.tiles[1] = Some(Piece { color: Color::Blue, left_start: false });
                game.board.tiles[2] = Some(Piece { color: Color::Red, left_start: true });
                game.board.tiles[3] = Some(Piece { color: Color::Blue, left_start: true });

                let action1 = Action {
                    player: Color::Red,
                    action: ActionKind::Interchange(1, 2),
                    card: Card::Jack,
                };

                let action2 = Action {
                    player: Color::Red,
                    action: ActionKind::Interchange(3, 2),
                    card: Card::Jack,
                };

                assert!(game.action(Card::Jack, action1).is_err());
                assert!(game.action(Card::Jack, action2).is_ok());
            }

        }
    
        mod action_move_tests {
            use super::*;

            #[test]
            fn valid_move_forward() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Five, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    color: Color::Red,
                    left_start: false,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move(0, 5),
                    card: Card::Five,
                };

                assert!(game.action(Card::Five, action).is_ok());
                assert!(game.board.tiles[0].is_none());
                assert_eq!(game.board.tiles[5].as_ref().unwrap().color, Color::Red);
                assert_eq!(game.board.tiles[5].as_ref().unwrap().left_start, true);
            }

            #[test]
            fn valid_move_into_house() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Five, Card::Ten];

                game.board.tiles[60] = Some(Piece {
                    color: Color::Red,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move(60, 64),
                    card: Card::Five,
                };

                assert!(game.action(Card::Five, action).is_ok());
                assert!(game.board.tiles[60].is_none());
                assert_eq!(game.board.tiles[64].as_ref().unwrap().color, Color::Red);
                assert_eq!(game.red.pieces_in_house, 1);
            }

            #[test]
            fn valid_move_in_house() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Ace, Card::Ten];
                game.red.pieces_in_house = 1;

                game.board.tiles[64] = Some(Piece {
                    color: Color::Red,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move(64, 65),
                    card: Card::Ace,
                };

                assert!(game.action(Card::Ace, action).is_ok());
                assert!(game.board.tiles[64].is_none());
                assert_eq!(game.board.tiles[65].as_ref().unwrap().color, Color::Red);
                assert_eq!(game.red.pieces_in_house, 1);
            }

            #[test]
            fn valid_move_backward() {
                let mut game = Game::new();
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
            fn invalid_move_into_house() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Two];

                // Piece hasn't left start yet
                game.board.tiles[0] = Some(Piece { 
                    color: Color::Red, 
                    left_start: false 
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move(0, 64),
                    card: Card::Two,
                };

                assert!(game.action(Card::Two, action).is_err());
            }

            #[test]
            fn invalid_move_past_protected_piece() {
                let mut game = Game::new();
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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

            #[test]
            fn move_partner_piece_forward() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.pieces_in_house = 4;
                game.red.cards = vec![Card::Five];
                game.blue.cards = vec![Card::Five];

                game.board.tiles[0] = Some(Piece {
                    color: Color::Blue,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move(0, 5),
                    card: Card::Five,
                };

                assert!(game.action(Card::Five, action).is_ok());
                assert!(game.board.tiles[0].is_none());
                assert_eq!(game.board.tiles[5].as_ref().unwrap().color, Color::Blue);
                assert!(!game.player_mut_by_color(Color::Red).cards.contains(&Card::Five));
            }

            #[test]
            fn move_partner_piece_into_house() {
                let mut game = Game::new();
                game.trading_phase = false;
                game.current_player_color = Color::Blue;

                game.blue.pieces_in_house = 4;
                game.red.cards = vec![Card::Two];
                game.blue.cards = vec![Card::Two];

                // Partner-Figur kurz vor Haus
                game.board.tiles[63] = Some(Piece {
                    color: Color::Red,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Blue,
                    action: ActionKind::Move(63, 64),
                    card: Card::Two,
                };

                assert!(game.action(Card::Two, action).is_ok());
                assert!(game.board.tiles[63].is_none());
                assert_eq!(game.board.tiles[64].as_ref().unwrap().color, Color::Red);
                assert_eq!(game.player_by_color(Color::Red).pieces_in_house, 1);
            }

            #[test]
            fn cannot_move_partner_piece_if_not_in_house() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.pieces_in_house = 3;
                game.red.cards = vec![Card::Five];
                game.blue.cards = vec![Card::Five];

                game.board.tiles[0] = Some(Piece {
                    color: Color::Blue,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move(0, 5),
                    card: Card::Five,
                };

                assert!(game.action(Card::Five, action).is_err());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Blue);
                assert!(game.board.tiles[5].is_none());
            }

            #[test]
            fn cannot_move_partner_piece_past_protected_piece() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.pieces_in_house = 4;
                game.red.cards = vec![Card::Five];
                game.blue.cards = vec![Card::Five];

                game.board.tiles[0] = Some(Piece {
                    color: Color::Blue,
                    left_start: true,
                });

                game.board.tiles[3] = Some(Piece {
                    color: Color::Red,
                    left_start: false,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move(0, 5),
                    card: Card::Five,
                };

                assert!(game.action(Card::Five, action).is_err());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Blue);
                assert_eq!(game.board.tiles[3].as_ref().unwrap().color, Color::Red);
                assert!(game.board.tiles[5].is_none());
            }
        }

        mod action_split_tests {
            use super::*;

            #[test]
            fn split_within_limits() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Seven, Card::Ten];

                game.board.tiles[63] = Some(Piece {
                    color: Color::Red,
                    left_start: true,
                });

                game.board.tiles[4] = Some(Piece {
                    color: Color::Blue,
                    left_start: false,
                });

                let action1 = Action {
                    player: Color::Red,
                    action: ActionKind::Split(63, 67),
                    card: Card::Seven,
                };

                let action2 = Action {
                    player: Color::Red,
                    action: ActionKind::Split(4, 6),
                    card: Card::Seven
                };

                assert!(game.action(Card::Seven, action1).is_ok());
                assert!(game.board.tiles[63].is_none());
                assert_eq!(game.board.tiles[67].as_ref().unwrap().color, Color::Red);
                assert_eq!(game.board.tiles[67].as_ref().unwrap().left_start, true);
                assert_eq!(game.red.pieces_in_house, 1);
                assert_eq!(game.split_rest, Some(2));

                let _ = game.action(Card::Seven, action1);

                assert!(game.action(Card::Seven, action2).is_ok());
                assert!(game.board.tiles[4].is_none());
                assert_eq!(game.board.tiles[6].as_ref().unwrap().color, Color::Blue);
                assert_eq!(game.board.tiles[6].as_ref().unwrap().left_start, true);
                assert_eq!(game.red.pieces_in_house, 1);
                assert_eq!(game.split_rest, None);
                assert_eq!(game.current_player_color, Color::Green);
            }

            #[test]
            fn split_outside_limits() {
                let mut game = Game::new();
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

                game.green.pieces_to_place = 3;

                game.red.cards = vec![Card::Seven, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    color: Color::Red,
                    left_start: false,
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
                assert_eq!(game.player_mut_by_color(Color::Green).pieces_to_place, 4);

                let first_entry = &game.history[game.history.len() - 2];
                assert_eq!(first_entry.action.action, ActionKind::Split(0, 3));
                assert_eq!(first_entry.beaten_piece_color, Some(Color::Green));
                assert_eq!(first_entry.left_start_before, false);

                let second_entry = &game.history[game.history.len() - 1];
                assert_eq!(second_entry.action.action, ActionKind::Split(3, 5));
                assert_eq!(second_entry.beaten_piece_color, None);
                assert_eq!(second_entry.left_start_before, true);
            }

            #[test]
            fn split_complete_turn() {
                let mut game = Game::new();
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
                game.trading_phase = false;

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
            
            #[test]
            fn split_can_move_partner_piece_without_pieces_in_house() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Seven];
                game.red.pieces_in_house = 0;

                game.board.tiles[0] = Some(Piece {
                    color: Color::Blue,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split(0, 3),
                    card: Card::Seven,
                };

                assert!(game.action(Card::Seven, action).is_ok());
                assert!(game.board.tiles[0].is_none());
                assert_eq!(game.board.tiles[3].as_ref().unwrap().color, Color::Blue);
            }

            #[test]
            fn split_can_beat_partner_piece() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Seven];
                game.blue.pieces_to_place = 3;

                game.board.tiles[0] = Some(Piece { color: Color::Red, left_start: true });
                game.board.tiles[3] = Some(Piece { color: Color::Blue, left_start: true });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split(0, 5),
                    card: Card::Seven,
                };

                assert!(game.action(Card::Seven, action).is_ok());
                assert_eq!(game.blue.pieces_to_place, 4);
            }

            #[test]
            fn split_enter_house_only_counts_once() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Seven];

                game.board.tiles[63] = Some(Piece {
                    color: Color::Red,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split(63, 66),
                    card: Card::Seven,
                };

                assert!(game.action(Card::Seven, action).is_ok());
                assert_eq!(game.red.pieces_in_house, 1);
            }

            #[test]
            fn split_cannot_enter_wrong_house() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Seven];

                game.board.tiles[15] = Some(Piece {
                    color: Color::Red,
                    left_start: true,
                });

                
                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split(15, 68),
                    card: Card::Seven,
                };

                assert!(game.action(Card::Seven, action).is_err());
            }
        }

        mod action_trade_tests {
            use std::vec;

            use super::*;

            #[test]
            fn trade_succeeds() {
                let mut game = Game::new();

                game.red.cards = vec![Card::Five, Card::Ten];
                game.green.cards = vec![Card::Two, Card::Three];
                game.blue.cards = vec![Card::Seven, Card::Eight];
                game.yellow.cards = vec![Card::Nine, Card::Ten];

                let action_red = Action {
                    player: Color::Red,
                    action: ActionKind::Trade,
                    card: Card::Five,
                };

                assert!(game.action(Card::Five, action_red).is_ok());
                assert_eq!(game.red.cards.len(), 1);
                assert_eq!(game.red.cards[0], Card::Ten);
                assert_eq!(game.trade_buffer.len(), 1);

                let action_green = Action {
                    player: Color::Green,
                    action: ActionKind::Trade,
                    card: Card::Two,
                };

                assert!(game.action(Card::Two, action_green).is_ok());
                assert_eq!(game.green.cards.len(), 1);
                assert_eq!(game.green.cards[0], Card::Three);
                assert_eq!(game.trade_buffer.len(), 2);

                let action_blue = Action {
                    player: Color::Blue,
                    action: ActionKind::Trade,
                    card: Card::Seven,
                };

                assert!(game.action(Card::Seven, action_blue).is_ok());
                assert_eq!(game.blue.cards.len(), 1);
                assert_eq!(game.blue.cards[0], Card::Eight);
                assert_eq!(game.trade_buffer.len(), 3);

                let action_yellow = Action {
                    player: Color::Yellow,
                    action: ActionKind::Trade,
                    card: Card::Nine,
                };

                assert!(game.action(Card::Nine, action_yellow).is_ok());

                // Swap buffer is emptied and players get cards
                assert!(game.trade_buffer.is_empty());
                assert!(!game.trading_phase);

                assert_eq!(game.red.cards.len(), 2);
                assert!(game.red.cards.contains(&Card::Seven));

                assert_eq!(game.green.cards.len(), 2);
                assert!(game.green.cards.contains(&Card::Nine));

                assert_eq!(game.blue.cards.len(), 2);
                assert!(game.blue.cards.contains(&Card::Five));

                assert_eq!(game.yellow.cards.len(), 2);
                assert!(game.yellow.cards.contains(&Card::Two));
            }

            #[test]
            fn trade_outside_trading_phase_fails() {
                let mut game = Game::new();
                game.trading_phase = false;
                game.current_player_color = Color::Red;

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Trade,
                    card: Card::Five,
                };

                assert!(game.action(Card::Five, action).is_err());
            }

            #[test]
            fn trade_duplicate_card_fails() {
                let mut game = Game::new();
                
                game.red.cards = vec![Card::Five, Card::Ten];

                let action1 = Action {
                    player: Color::Red,
                    action: ActionKind::Trade,
                    card: Card::Five,
                };
                let action2 = Action {
                    player: Color::Red,
                    action: ActionKind::Trade,
                    card: Card::Ten,
                };

                assert!(game.action(Card::Five, action1).is_ok());
                assert!(game.action(Card::Ten, action2).is_err());
            }
        }
    
        mod action_remove_tests {
            use super::*;

            #[test]
            fn remove_card_success() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Two, Card::Five];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Remove,
                    card: Card::Two,
                };

                assert!(game.action(Card::Two, action).is_ok());

                assert!(!game.red.cards.contains(&Card::Two));

                assert!(game.red.cards.contains(&Card::Five));
            
                assert!(game.discard.contains(&Card::Two));

                assert_eq!(game.current_player_color, Color::Green);
            }

            #[test]
            fn remove_card_not_in_hand() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Seven];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Remove,
                    card: Card::Two,
                };

                assert!(game.action(Card::Two, action).is_err());
            }

            #[test]
            fn cannot_remove_during_trading_phase() {
                let mut game = Game::new();
                game.trading_phase = true;

                game.red.cards = vec![Card::Ace];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Remove,
                    card: Card::Ace,
                };

                assert!(game.action(Card::Ace, action).is_err());
            }

            #[test]
            fn invalid_remove_creates_no_history_entry() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Five];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Remove,
                    card: Card::Ace,
                };

                let history_len = game.history.len();

                assert!(game.action(Card::Ace, action).is_err());
                assert_eq!(game.history.len(), history_len);
            }
        }
    
    }    

    mod undo_tests {
        use super::*;

        mod undo_action_tests {
            use super::*;

            #[test]
            fn undo_move_with_empty_history_fails() {
                let mut game = Game::new();
                game.trading_phase = false;

                assert!(game.undo_action().is_err());
            }

            mod undo_place_tests {
                use super::*;

                #[test]
                fn undo_place_piece() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Ace, Card::Five];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Place,
                        card: Card::Ace,
                    };

                    assert!(game.action(Card::Ace, action).is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
                    assert_eq!(game.red.pieces_to_place, 3);
                    assert!(!game.red.cards.contains(&Card::Ace));

                    assert!(game.undo_action().is_ok());
                    assert!(game.board.tiles[0].is_none());
                    assert_eq!(game.red.pieces_to_place, 4);
                    assert!(game.red.cards.contains(&Card::Ace));
                }

                #[test]
                fn undo_place_restores_current_player() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Ace];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Place,
                        card: Card::Ace,
                    };

                    assert!(game.action(Card::Ace, action).is_ok());
                    assert_ne!(game.current_player_color, Color::Red);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.current_player_color, Color::Red);
                }

                #[test]
                fn invalid_place_creates_no_history_entry() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    // Card not in player's hand
                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Place,
                        card: Card::Ace,
                    };

                    let history_len = game.history.len();

                    assert!(game.action(Card::Ace, action).is_err());
                    assert_eq!(game.history.len(), history_len);
                }

                #[test]
                fn undo_place_teammate_piece() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.pieces_in_house = 4;
                    game.red.cards = vec![Card::Ace];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Place,
                        card: Card::Ace,
                    };

                    assert!(game.action(Card::Ace, action).is_ok());
                    assert_eq!(game.board.tiles[32].as_ref().unwrap().color, Color::Blue);
                    assert!(game.board.tiles[0].is_none());
                    assert_eq!(game.blue.pieces_to_place, 3);
                    assert!(!game.red.cards.contains(&Card::Ace));

                    assert!(game.undo_action().is_ok());
                    assert!(game.board.tiles[32].is_none());
                    assert_eq!(game.blue.pieces_to_place, 4);
                    assert!(game.red.cards.contains(&Card::Ace));

                }

                #[test]
                fn undo_place_with_beaten_piece() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Ace, Card::Five];

                    game.board.tiles[0] = Some(Piece { 
                        color: Color::Green, 
                        left_start: true,
                    });

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Place,
                        card: Card::Ace,
                    };

                    assert!(game.action(Card::Ace, action).is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
                    assert_eq!(game.red.pieces_to_place, 3);
                    assert!(!game.red.cards.contains(&Card::Ace));

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Green);
                    assert_eq!(game.red.pieces_to_place, 4);
                    assert!(game.red.cards.contains(&Card::Ace));
                }
            }
        
            mod undo_move_tests {
                use super::*;

                #[test]
                fn undo_move_piece() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Five];
                    game.board.tiles[0] = Some(Piece { 
                        color: Color::Red, 
                        left_start: false, 
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Card::Five,
                        action: ActionKind::Move(0, 5),
                    };

                    assert!(game.action(Card::Five, action).is_ok());
                    assert_eq!(game.board.tiles[5].as_ref().unwrap().color, Color::Red);
                    assert_eq!(game.board.tiles[5].as_ref().unwrap().left_start, true);
                    assert!(!game.red.cards.contains(&Card::Five));

                    assert!(game.undo_action().is_ok());
                    assert!(game.board.tiles[5].is_none());
                    assert!(game.red.cards.contains(&Card::Five));
                    assert_eq!(game.board.check_tile(0).unwrap().left_start, false);
                }

                #[test]
                fn double_undo_move_into_and_in_house() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Two, Card::Ace];
                    game.board.tiles[0] = Some(Piece { 
                        color: Color::Red, 
                        left_start: true 
                    });

                    let action1 = Action {
                        player: Color::Red,
                        card: Card::Two,
                        action: ActionKind::Move(0, 65),
                    };

                    let action2 = Action {
                        player: Color::Red,
                        card: Card::Ace,
                        action: ActionKind::Move(65, 66)
                    };

                    assert!(game.action(Card::Two, action1).is_ok());
                    assert_eq!(game.red.pieces_in_house, 1);

                    game.current_player_color = Color::Red;
                    assert!(game.action(Card::Ace, action2).is_ok());

                    // First undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.board.tiles[65].as_ref().unwrap().color, Color::Red);
                    assert!(game.board.tiles[66].is_none());
                    assert_eq!(game.red.pieces_in_house, 1);

                    // Second undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
                    assert!(game.board.tiles[65].is_none());
                    assert_eq!(game.red.pieces_in_house, 0);
                }

                #[test]
                fn undo_move_into_house_restores_card() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Two];

                    game.board.tiles[0] = Some(Piece {
                        color: Color::Red,
                        left_start: true,
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Card::Two,
                        action: ActionKind::Move(0, 65),
                    };

                    assert!(game.action(Card::Two, action).is_ok());
                    assert!(!game.red.cards.contains(&Card::Two));

                    assert!(game.undo_action().is_ok());
                    assert!(game.red.cards.contains(&Card::Two));
                    assert_eq!(game.red.pieces_in_house, 0);
                }

                #[test]
                fn undo_move_beating_opponent() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Two];

                    game.board.tiles[0] = Some(Piece { 
                        color: Color::Red, 
                        left_start: false 
                    });

                    game.board.tiles[2] = Some(Piece { 
                        color: Color::Green, 
                        left_start: true 
                    });
                    
                    let _action = Action {
                        player: Color::Red,
                        card: Card::Two,
                        action: ActionKind::Move(0, 2),
                    };

                    assert!(game.action(Card::Two, _action).is_ok());

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.board.tiles[2].as_ref().unwrap().color, Color::Green);
                    assert_eq!(game.board.tiles[2].as_ref().unwrap().left_start, true);

                    assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().left_start, false);
                }

                #[test]
                fn undo_move_teammate_piece_into_house() {
                    let mut game = Game::new();
                    game.trading_phase = false;
                    game.current_player_color = Color::Blue;

                    game.blue.cards = vec![Card::Two, Card::Ace];
                    game.blue.pieces_in_house = 4;


                    game.board.tiles[0] = Some(Piece { 
                        color: Color::Red, 
                        left_start: true 
                    });

                    let action = Action {
                        player: Color::Blue,
                        card: Card::Two,
                        action: ActionKind::Move(0, 65),
                    };

                    assert!(game.action(Card::Two, action).is_ok());

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.red.pieces_in_house, 0);
                    assert!(game.blue.cards.contains(&Card::Two));
                }

                #[test]
                fn undo_move_restores_current_player() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Five];

                    game.board.tiles[0] = Some(Piece {
                        color: Color::Red,
                        left_start: false,
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Card::Five,
                        action: ActionKind::Move(0, 5),
                    };

                    assert!(game.action(Card::Five, action).is_ok());
                    assert_ne!(game.current_player_color, Color::Red);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.current_player_color, Color::Red);
                }

                #[test]
                fn invalid_move_creates_no_history_entry() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Five];

                    // Kein Piece auf Feld 0
                    let action = Action {
                        player: Color::Red,
                        card: Card::Five,
                        action: ActionKind::Move(0, 5),
                    };

                    let history_len = game.history.len();

                    assert!(game.action(Card::Five, action).is_err());
                    assert_eq!(game.history.len(), history_len);
                }
            }

            mod undo_split_tests {
                use super::*;

                #[test]
                fn undo_split_piece() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Seven];

                    game.board.tiles[0] = Some(Piece { 
                        color: Color::Red, 
                        left_start: false, 
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Card::Seven,
                        action: ActionKind::Split(0, 5),
                    };

                    assert!(game.action(Card::Seven, action).is_ok());
                    assert_eq!(game.split_rest, Some(2));
                    assert_eq!(game.current_player_color, Color::Red);
                    assert_eq!(game.history.last().unwrap().split_rest_before, None);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.split_rest, None);
                    assert!(game.board.tiles[5].is_none());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
                }

                #[test]
                fn undo_split_into_house() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Seven];
                    game.red.pieces_in_house = 0;

                    game.board.tiles[63] = Some(Piece {
                        color: Color::Red,
                        left_start: true,
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Card::Seven,
                        action: ActionKind::Split(63, 65),
                    };

                    assert!(game.action(Card::Seven, action).is_ok());
                    assert_eq!(game.red.pieces_in_house, 1);
                    assert_eq!(game.split_rest, Some(4));

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.red.pieces_in_house, 0);
                    assert_eq!(game.board.tiles[63].as_ref().unwrap().color, Color::Red);
                    assert_eq!(game.split_rest, None);
                }

                #[test]
                fn undo_split_beating_opponent() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Seven];

                    game.board.tiles[0] = Some(Piece { 
                        color: Color::Red, 
                        left_start: false, 
                    });

                    game.green.pieces_to_place = 3;
                    game.board.tiles[3] = Some(Piece { 
                        color: Color::Green, 
                        left_start: true, 
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Card::Seven,
                        action: ActionKind::Split(0, 5),
                    };

                    assert!(game.action(Card::Seven, action).is_ok());
                    assert_eq!(game.split_rest, Some(2));
                    assert_eq!(game.green.pieces_to_place, 4);

                    // First undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.split_rest, Some(4));
                    assert!(game.board.tiles[5].is_none());
                    assert_eq!(game.board.tiles[3].as_ref().unwrap().color, Color::Red);
                    assert!(game.board.tiles[0].is_none());

                    // Second undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.split_rest, None);
                    assert_eq!(game.board.tiles[3].as_ref().unwrap().color, Color::Green);
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
                }

                #[test]
                fn undo_split_teammate() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Seven];

                    game.board.tiles[0] = Some(Piece { 
                        color: Color::Red, 
                        left_start: false, 
                    });

                    game.board.tiles[7] = Some(Piece { 
                        color: Color::Blue, 
                        left_start: true 
                    });

                    let action1 = Action {
                        player: Color::Red,
                        card: Card::Seven,
                        action: ActionKind::Split(0, 5),
                    };

                    let action2 = Action {
                        player: Color::Red,
                        card: Card::Seven,
                        action: ActionKind::Split(7, 9),
                    };

                    assert!(game.action(Card::Seven, action1).is_ok());
                    assert_eq!(game.split_rest, Some(2));

                    assert!(game.action(Card::Seven, action2).is_ok());
                    assert_eq!(game.split_rest, None);
                    assert_eq!(game.current_player_color, Color::Green);

                    // First undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.split_rest, Some(2));
                    assert!(game.board.tiles[9].is_none());
                    assert_eq!(game.board.tiles[7].as_ref().unwrap().color, Color::Blue);

                    // Second undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.split_rest, None);
                    assert!(game.board.tiles[5].is_none());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
                }

                #[test]
                fn undo_split_restores_joker_card(){
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Joker];

                    game.board.tiles[0] = Some(Piece {
                        color: Color::Red,
                        left_start: false,
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Card::Joker,
                        action: ActionKind::Split(0, 7),
                    };

                    assert!(game.action(Card::Joker, action).is_ok());
                    assert!(!game.red.cards.contains(&Card::Joker));
                    assert_eq!(game.split_rest, None);

                    assert!(game.undo_action().is_ok());
                    assert!(game.red.cards.contains(&Card::Joker));
                    assert_eq!(game.split_rest, None);
                }

                #[test]
                fn undo_split_does_not_change_player() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Seven];

                    game.board.tiles[0] = Some(Piece {
                        color: Color::Red,
                        left_start: false,
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Card::Seven,
                        action: ActionKind::Split(0, 5),
                    };

                    assert!(game.action(Card::Seven, action).is_ok());
                    assert_eq!(game.current_player_color, Color::Red);
                    assert_eq!(game.split_rest, Some(2));

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.current_player_color, Color::Red);
                    assert_eq!(game.split_rest, None);
                }

                #[test]
                fn invalid_split_creates_no_history_entry() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Seven];

                    let action = Action {
                        player: Color::Red,
                        card: Card::Seven,
                        action: ActionKind::Split(0, 5),
                    };

                    let history_len = game.history.len();

                    assert!(game.action(Card::Seven, action).is_err());
                    assert_eq!(game.history.len(), history_len);
                }

            }
        
            mod undo_remove_tests {
                use super::*;

                #[test]
                fn undo_remove_card() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Ace, Card::Five];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Remove,
                        card: Card::Ace,
                    };

                    assert!(game.action(Card::Ace, action).is_ok());
                    assert!(!game.red.cards.contains(&Card::Ace));
                    assert!(game.discard.contains(&Card::Ace));

                    assert!(game.undo_action().is_ok());
                    assert!(game.red.cards.contains(&Card::Ace));
                    assert!(!game.discard.contains(&Card::Ace));
                }

                #[test]
                fn undo_remove_restores_current_player() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Ace];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Remove,
                        card: Card::Ace,
                    };

                    let current_player = game.current_player_color;

                    assert!(game.action(Card::Ace, action).is_ok());
                    assert_ne!(game.current_player_color, current_player);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.current_player_color, current_player);
                }
            }
        
            mod undo_interchange_tests {
                use super::*;

                #[test]
                fn undo_interchange_successful() {
                    let mut game = Game::new();
                    game.trading_phase = false;

                    game.red.cards = vec![Card::Jack];

                    game.board.tiles[0] = Some(Piece { 
                        color: Color::Red, 
                        left_start: true 
                    });

                    game.board.tiles[1] = Some(Piece { 
                        color: Color::Green, 
                        left_start: true 
                    });

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Interchange(0, 1),
                        card: Card::Jack,
                    };

                    assert!(game.action(Card::Jack, action).is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Green);
                    assert_eq!(game.board.tiles[1].as_ref().unwrap().color, Color::Red);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
                    assert_eq!(game.board.tiles[1].as_ref().unwrap().color, Color::Green);
                    assert!(game.red.cards.contains(&Card::Jack));
                    assert!(!game.discard.contains(&Card::Jack));
                    assert_eq!(game.current_player_color, Color::Red);
                }
            }

            mod undo_trade_tests {
                use super::*;
                #[test]
                fn undo_trade_basic() {
                    let mut game = Game::new();
                    game.trading_phase = true;

                    game.red.cards = vec![Card::Five, Card::Ten];
                    game.green.cards = vec![Card::Two, Card::Three];
                    game.blue.cards = vec![Card::Seven, Card::Eight];
                    game.yellow.cards = vec![Card::Nine, Card::Ten];

                    let action_red = Action {
                        player: Color::Red,
                        action: ActionKind::Trade,
                        card: Card::Five,
                    };

                    assert!(game.action(Card::Five, action_red).is_ok());
                    assert_eq!(game.red.cards.len(), 1);
                    assert_eq!(game.red.cards[0], Card::Ten);
                    assert_eq!(game.trade_buffer.len(), 1);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.red.cards.len(), 2);
                    assert!(game.red.cards.contains(&Card::Five));
                    assert!(game.red.cards.contains(&Card::Ten));
                    assert_eq!(game.trade_buffer.len(), 0);
                }

                #[test]
                fn undo_trade_full_trade_phase() {
                    let mut game = Game::new();
                    game.trading_phase = true;

                    game.red.cards = vec![Card::Five, Card::Ten];
                    game.green.cards = vec![Card::Two, Card::Three];
                    game.blue.cards = vec![Card::Seven, Card::Eight];
                    game.yellow.cards = vec![Card::Nine, Card::Ten];

                    let action_red = Action {
                        player: Color::Red,
                        action: ActionKind::Trade,
                        card: Card::Five,
                    };

                    assert!(game.action(Card::Five, action_red).is_ok());
                    assert_eq!(game.red.cards.len(), 1);
                    assert_eq!(game.red.cards[0], Card::Ten);
                    assert_eq!(game.trade_buffer.len(), 1);

                    let action_green = Action {
                        player: Color::Green,
                        action: ActionKind::Trade,
                        card: Card::Two,
                    };

                    assert!(game.action(Card::Two, action_green).is_ok());
                    assert_eq!(game.green.cards.len(), 1);
                    assert_eq!(game.green.cards[0], Card::Three);
                    assert_eq!(game.trade_buffer.len(), 2);

                    let action_blue = Action {
                        player: Color::Blue,
                        action: ActionKind::Trade,
                        card: Card::Seven,
                    };

                    assert!(game.action(Card::Seven, action_blue).is_ok());
                    assert_eq!(game.blue.cards.len(), 1);
                    assert_eq!(game.blue.cards[0], Card::Eight);
                    assert_eq!(game.trade_buffer.len(), 3);

                    let action_yellow = Action {
                        player: Color::Yellow,
                        action: ActionKind::Trade,
                        card: Card::Nine,
                    };

                    assert!(game.action(Card::Nine, action_yellow).is_ok());
                    assert_eq!(game.yellow.cards.len(), 2);
                    assert_eq!(game.yellow.cards[0], Card::Ten);

                    // Swap buffer is emptied and players get cards
                    assert!(game.trade_buffer.is_empty());
                    assert!(!game.trading_phase);
                    assert_eq!(game.red.cards.len(), 2);
                    assert_eq!(game.green.cards.len(), 2);
                    assert_eq!(game.blue.cards.len(), 2);
                    assert_eq!(game.yellow.cards.len(), 2);
                    assert!(game.red.cards.contains(&Card::Seven));
                    assert!(game.green.cards.contains(&Card::Nine));
                    assert!(game.blue.cards.contains(&Card::Five));
                    assert!(game.yellow.cards.contains(&Card::Two));

                    // Undo last trade (yellow)
                    assert!(game.undo_action().is_ok());
                    assert!(game.trading_phase);
                    assert_eq!(game.trade_buffer.len(), 3);
                    assert_eq!(game.yellow.cards.len(), 2);
                    assert!(game.yellow.cards.contains(&Card::Nine));

                    // Undo third trade (blue)
                    assert!(game.undo_action().is_ok());
                    assert!(game.trading_phase);
                    assert_eq!(game.trade_buffer.len(), 2);
                    assert_eq!(game.blue.cards.len(), 2);
                    assert!(game.blue.cards.contains(&Card::Seven));

                    // Undo second trade (green)
                    assert!(game.undo_action().is_ok());
                    assert!(game.trading_phase);
                    assert_eq!(game.trade_buffer.len(), 1);
                    assert_eq!(game.green.cards.len(), 2);
                    assert!(game.green.cards.contains(&Card::Two));

                    // Undo first trade (red)
                    assert!(game.undo_action().is_ok());
                    assert!(game.trading_phase);
                    assert_eq!(game.trade_buffer.len(), 0);
                    assert_eq!(game.red.cards.len(), 2);
                    assert!(game.red.cards.contains(&Card::Five));               
                }
            }
        }

        mod undo_turn_tests {
            use super::*;

            #[test]
            fn undo_turn_single_action() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Ace];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Place,
                    card: Card::Ace,
                };

                assert!(game.action(Card::Ace, action).is_ok());

                // Sanity check
                let start = Board::start_field(Color::Red) as usize;
                assert!(game.board.tiles[start].is_some());
                assert_eq!(game.current_player_color, Color::Green);

                // Undo full turn
                assert!(game.undo_turn().is_ok());

                assert!(game.board.tiles[start].is_none());
                assert!(game.red.cards.contains(&Card::Ace));
                assert_eq!(game.current_player_color, Color::Red);
            }

            #[test]
            fn undo_turn_split() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Seven];

                game.board.tiles[0] = Some(Piece {
                    color: Color::Red,
                    left_start: false,
                });

                game.board.tiles[3] = Some(Piece { 
                    color: Color::Green, 
                    left_start: true 
                });

                let action1 = Action {
                    player: Color::Red,
                    action: ActionKind::Split(0, 5),
                    card: Card::Seven,
                };

                let action2 = Action {
                    player: Color::Red,
                    action: ActionKind::Split(5, 7),
                    card: Card::Seven,
                };

                // Simulate turns
                assert!(game.action(Card::Seven, action1).is_ok());
                assert_eq!(game.split_rest, Some(2));
                assert!(game.action(Card::Seven, action2).is_ok());
                assert_eq!(game.split_rest, None);
                assert_eq!(game.current_player_color, Color::Green);
                
                assert!(game.undo_turn().is_ok());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().color, Color::Red);
                assert_eq!(game.board.tiles[3].as_ref().unwrap().color, Color::Green);
                assert!(game.board.tiles[7].is_none());
                assert_eq!(game.current_player_color, Color::Red);
                assert_eq!(game.split_rest, None);
            }

            #[test]
            fn undo_turn_full_trade() {
                let mut game = Game::new();
                game.trading_phase = true;

                game.red.cards = vec![Card::Ace];
                game.green.cards = vec![Card::Two];
                game.blue.cards = vec![Card::Three];
                game.yellow.cards = vec![Card::Four];

                let action1 = Action {
                    player: Color::Red,
                    action: ActionKind::Trade,
                    card: Card::Ace,
                };

                let action2 = Action {
                    player: Color::Green,
                    action: ActionKind::Trade,
                    card: Card::Two,
                };

                let action3 = Action {
                    player: Color::Blue,
                    action: ActionKind::Trade,
                    card: Card::Three,
                };

                let action4 = Action {
                    player: Color::Yellow,
                    action: ActionKind::Trade,
                    card: Card::Four,
                };

                assert!(game.action(Card::Ace, action1).is_ok());
                assert!(game.action(Card::Two, action2).is_ok());
                assert!(game.action(Card::Three, action3).is_ok());
                assert!(game.action(Card::Four, action4).is_ok());
                assert!(!game.trading_phase);

                assert!(game.undo_turn().is_ok());
                assert!(game.trading_phase);
                assert_eq!(game.current_player_color, Color::Red);
                assert!(game.trade_buffer.is_empty());
                
            }
        }

        mod undo_sequence_tests {
            use super::*;

            #[test]
            fn undo_sequence_multiple_turns() {
                let mut game = Game::new();
                game.trading_phase = false;

                game.red.cards = vec![Card::Ace];
                game.green.cards = vec![Card::Ace];

                let action_red = Action {
                    player: Color::Red,
                    action: ActionKind::Place,
                    card: Card::Ace,
                };

                let action_green = Action {
                    player: Color::Green,
                    action: ActionKind::Place,
                    card: Card::Ace,
                };

                assert!(game.action(Card::Ace, action_red).is_ok());
                assert!(game.action(Card::Ace, action_green).is_ok());

                // Undo both turns
                assert!(game.undo_sequence(2).is_ok());

                let red_start = Board::start_field(Color::Red) as usize;
                let green_start = Board::start_field(Color::Green) as usize;

                assert!(game.board.tiles[red_start].is_none());
                assert!(game.board.tiles[green_start].is_none());

                assert!(game.red.cards.contains(&Card::Ace));
                assert!(game.green.cards.contains(&Card::Ace));
                assert_eq!(game.current_player_color, Color::Red);
            }

            #[test]
            fn undo_sequence_trade_and_place() {
                let mut game = Game::new();
                game.trading_phase = true;

                game.red.cards = vec![Card::Five];
                game.green.cards = vec![Card::Two];
                game.blue.cards = vec![Card::Ace];
                game.yellow.cards = vec![Card::Nine];

                // Full trade
                for (color, card) in [
                    (Color::Red, Card::Five),
                    (Color::Green, Card::Two),
                    (Color::Blue, Card::Ace),
                    (Color::Yellow, Card::Nine),
                ] {
                    let action = Action {
                        player: color,
                        action: ActionKind::Trade,
                        card,
                    };
                    assert!(game.action(card, action).is_ok());
                }

                let place_action = Action {
                    player: Color::Red,
                    card: Card::Ace,
                    action: ActionKind::Place,
                };

                assert!(game.action(Card::Ace, place_action).is_ok());

                // Undo entire trade
                assert!(game.undo_sequence(2).is_ok());

                assert!(game.trading_phase);
                assert_eq!(game.trade_buffer.len(), 0);

                assert!(game.red.cards.contains(&Card::Five));
                assert!(game.green.cards.contains(&Card::Two));
                assert!(game.blue.cards.contains(&Card::Ace));
                assert!(game.yellow.cards.contains(&Card::Nine));
            }
        }
    }
}
