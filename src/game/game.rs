use super::piece::*;
use super::action::*;
use super::color::*;
use super::deck::*;
use super::card::*;
use super::player::*;
use super::board::*;
use super::history::*;

const CARDS_PER_ROUND: [u8;5] = [6,5,4,3,2];

#[derive(Clone, Debug, PartialEq)]
pub enum GameVariant {
    TwoVsTwo,
    ThreeVsThree,
    TwoVsTwoVsTwo,
    FreeForAll(usize),
}


#[derive(Clone)]
pub struct Game {
    pub game_variant: GameVariant,
    pub board: Board,
    pub history: Vec<HistoryEntry>,
    pub round: usize,
    
    pub trading_phase: bool,
    pub trade_buffer: Vec<(usize, usize, Card)>,

    pub deck: Deck,
    pub discard: Vec<Card>,

    pub players: Vec<Player>,
    pub teams: Option<Vec<Vec<usize>>>,
    pub current_player_index: usize,

    pub split_rest: Option<u8>,
}

impl Game {
    pub fn new(variant: GameVariant) -> Self {
        match variant {
            GameVariant::TwoVsTwo => Self::new_2v2(),
            GameVariant::ThreeVsThree => Self::new_3v3(),
            GameVariant::TwoVsTwoVsTwo => Self::new_2v2v2(),
            GameVariant::FreeForAll(n) => Self::new_free_for_all(n),
        }
    }

    fn new_2v2() -> Self {
        let players = vec![
            Player::new(Color::Red),
            Player::new(Color::Green),
            Player::new(Color::Blue),
            Player::new(Color::Yellow),
        ];

        let teams = Some(vec![
            vec![0, 2], // Team 0: Red + Blue
            vec![1, 3], // Team 1: Green + Yellow
        ]);



        Self {
            game_variant: GameVariant::TwoVsTwo,
            board: Board::new(4),
            history: Vec::new(),
            round: 1,

            trading_phase: true,
            trade_buffer: Vec::new(),

            deck: Deck::new(),
            discard: Vec::new(),

            players,
            teams,
            current_player_index: 0,

            split_rest: None,
        }
    }

    fn new_3v3() -> Self {
        let players = vec![
            Player::new(Color::Red),
            Player::new(Color::Green),
            Player::new(Color::Purple),
            Player::new(Color::Blue),
            Player::new(Color::Yellow),
            Player::new(Color::Orange),
        ];

        let teams = Some(vec![
            vec![0, 2, 4], // Team 0: Red + Purple + Yellow
            vec![1, 3, 5], // Team 1: Green + Blue + Orange
        ]);

        Self {
            game_variant: GameVariant::ThreeVsThree,
            board: Board::new(6),
            history: Vec::new(),
            round: 1,

            trading_phase: true,
            trade_buffer: Vec::new(),

            deck: Deck::new(),
            discard: Vec::new(),

            players,
            teams,
            current_player_index: 0,

            split_rest: None,
        }
    }

    fn new_2v2v2() -> Self {
        let players = vec![
            Player::new(Color::Red),
            Player::new(Color::Green),
            Player::new(Color::Purple),
            Player::new(Color::Blue),
            Player::new(Color::Yellow),
            Player::new(Color::Orange),
        ];

        let teams = Some(vec![
            vec![0, 3], // Team 0: Red + Blue
            vec![1, 4], // Team 1: Green + Yellow
            vec![2, 5], // Team 2: Purple + Orange
        ]);

        Self {
            game_variant: GameVariant::TwoVsTwoVsTwo,
            board: Board::new(6),
            history: Vec::new(),
            round: 1,

            trading_phase: true,
            trade_buffer: Vec::new(),

            deck: Deck::new(),
            discard: Vec::new(),

            players,
            teams,
            current_player_index: 0,

            split_rest: None,
        }
    }

    fn new_free_for_all(n: usize) -> Self {
        assert!(n >=2 && n <= 6, "Free-for-All can only be played with 2 to 6 players.");

        let colors = [
            Color::Red, 
            Color::Green, 
            Color::Blue, 
            Color::Yellow, 
            Color::Purple, 
            Color::Orange
        ];

        let players = (0..n)
            .map(|i| Player {
                color: colors[i],
                pieces_to_place: 3,
                pieces_in_house: 0,
                cards: Vec::new(),
            })
            .collect();

        let mut board = Board::new(n);
        
        for i in 0..n {
            let start = board.start_field(i);
            board.tiles[start] = Some(Piece { 
                owner: i, 
                left_start: false 
            });
        }

        Self {
            game_variant: GameVariant::FreeForAll(n),
            board,
            history: Vec::new(),
            round: 1,

            trading_phase: true,
            trade_buffer: Vec::new(),

            deck: Deck::new(),
            discard: Vec::new(),

            players,
            teams: None,
            current_player_index: 0,

            split_rest: None,
        }
    }

    pub fn player_mut_by_color(&mut self, color: Color) -> &mut Player {
        self.players
            .iter_mut()
            .find(|p| p.color == color)
            .expect("Player color doesn't exist.")
    }

    pub fn player_by_color(&self, color: Color) -> &Player {
        self.players
            .iter()
            .find(|p| p.color == color)
            .expect("Player color doesn't exist.")
    }

    pub fn can_card_move(&self, _card: Card, forward: Option<u8>, backward: Option<u8>) -> bool {
        let distances = _card.possible_distances();

        let forward_ok = forward.map_or(false, |f| distances.contains(&f));
        let backward_ok = backward.map_or(false, |b| distances.contains(&b));

        forward_ok || backward_ok
    }

    pub fn play(&mut self, s: &str) -> Result<(), &'static str> {
        let action: Action = s.parse()?;
        let card = action.card;
        self.action(card, action)
    }

    fn all_players_out_of_cards(&self) -> bool {
        self.players
            .iter()
            .all(|p| p.cards.is_empty())
    }

    pub fn current_player(&self) -> &Player {
        &self.players[self.current_player_index]
    }

    pub fn current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.current_player_index]
    }

    pub fn player_by_index(&self, index: usize) -> &Player {
        &self.players[index]
    }

    pub fn player_mut_by_index(&mut self, index: usize) -> &mut Player {
        &mut self.players[index]
    }

    pub fn next_player(&mut self) {
        self.current_player_index = (self.current_player_index + 1) % self.players.len();
    }

    pub fn prev_player(&mut self) {
        if self.current_player_index == 0 {
            self.current_player_index = self.players.len() - 1;
        } else {
            self.current_player_index -= 1;
        }
    }

    pub fn index_of_color(&self, color: Color) -> usize {
        self.players
            .iter()
            .position(|p| p.color == color)
            .expect("Color not found in game")
    }

    pub fn teammate_indices(&self, player_index: usize) -> Vec<usize> {
        match &self.teams {
            Some(teams) => teams
                .iter()
                .find(|team| team.contains(&player_index))
                .map(|team| {
                    team.iter()
                        .copied()
                        .filter(|&i| i != player_index)
                        .collect()
                })
                .unwrap_or_default(),
            None => Vec::new(),
        }
    }

    fn controllable_player_indices(&self, player_index: usize) -> Vec<usize> {
        let mut indices = vec![player_index];
        indices.extend(self.teammate_indices(player_index));
        indices
    }

    // Return first teammate that is not the player himself
    pub fn teammate_index(&self, player_index: usize) -> Option<usize> {
        if let Some(teams) = &self.teams {
            for team in teams {
                if team.contains(&player_index) {
                    
                    return team.iter().find(|&&i| i != player_index).copied();
                }
            }
        }
        None
    }
    
    pub fn can_control_piece(&self, actor_index: usize, piece_owner_index: usize) -> bool {
        
        // Own piece
        if actor_index == piece_owner_index {
            return true;
        }

        // Team piece
        if self.players[actor_index].pieces_in_house == 4 {
            if let Some(teams) = &self.teams {
                return teams.iter().any(|team|
                    team.contains(&actor_index) && team.contains(&piece_owner_index)
                );
            }
        }

        false
    }

    fn can_place_for(&self, placer: usize, target: usize) -> bool {
        if placer == target {
            return true;
        }

        // FFA catch
        let Some(teams) = &self.teams else {
            return false;
        };

        // Check if placer has 4 pieces in house
        if self.players[placer].pieces_in_house != 4 {
            return false;
        }

        // Both players must be in the same team
        teams.iter().any(|team| {
            team.contains(&placer) && team.contains(&target)
        })
    }

    fn check_if_any_action_possible(&self) -> bool{
        let current_player = self.player_by_index(self.current_player_index);

        for card in &current_player.cards {
            if self.is_card_playable(*card) {
                return true;
            }
        }

        false
    }

    fn is_card_playable(&self, card: Card) -> bool {
        let current_player_index = self.current_player_index;
        let controllable_player_indices = self.controllable_player_indices(current_player_index);

        // Check if place is possible
        if matches!(card, Card::Ace | Card::King | Card::Joker) {
            for &player_index in &controllable_player_indices {
                if self.players[player_index].pieces_to_place == 0 {
                    continue;
                }

                let start = self.board.start_field(player_index);

                match &self.board.tiles[start] {
                    None => return true,
                    Some(piece) => {
                        if piece.left_start {
                            return true;
                        }
                    }
                }
            }
        }

        let team_pieces = self.find_team_pieces(current_player_index);

        if team_pieces.is_empty() {
            return false;
        }

        // Check if split is possible
        if matches!(card, Card::Seven | Card::Joker) {
            let mut total_steps: u8 = 0;

            for &from in &team_pieces {
                let steps = self.board.max_path_from(from, &controllable_player_indices);
                total_steps = total_steps.saturating_add(steps);

                if total_steps >= 7 {
                    return true;
                }
                
            }
        }

        let movable_pieces = self.find_movable_pieces(current_player_index);

        // Check if interchange is possible
        if matches!(card, Card::Jack | Card::Joker) {
            let interchangeable_pieces = self.count_interchangeable_pieces();

            if !movable_pieces.is_empty() && interchangeable_pieces >= 2 {
                return true;
            }
        }

        // Check if normal move is possible
        let distances = card.possible_distances();

        for &from in &movable_pieces {
            for &dist in &distances {
                let backward =
                    matches!(card, Card::Four | Card::Joker) && dist == 4;

                if self.can_piece_move_distance(from, dist, backward) {
                    return true;
                }
            }
        }

        false
    }

    fn find_team_pieces(&self, player_index: usize) -> Vec<usize> {
        let controllable_player_indices = self.controllable_player_indices(player_index);
        let mut positions = Vec::new();

        for (index, tile) in self.board.tiles.iter().enumerate() {
            if let Some(piece) = tile {
                if controllable_player_indices.contains(&piece.owner) {
                    positions.push(index);
                }
            }
        }

        positions
    }

    fn find_movable_pieces(&self, player_index: usize) -> Vec<usize> {
        let controllable_player_indices = self.controllable_player_indices(player_index);
        let mut positions = Vec::new();

        for (index, tile) in self.board.tiles.iter().enumerate() {
            if let Some(piece) = tile {
                if controllable_player_indices.contains(&piece.owner)
                    && self.can_control_piece(player_index, piece.owner)
                {
                    positions.push(index);
                }
            }
        }

        positions
    }

    fn count_interchangeable_pieces(&self) -> usize {
        self.board
            .tiles
            .iter()
            .enumerate()
            .filter(|(idx, tile)| {
                *idx < 64 &&
                tile.as_ref().map_or(false, |p| p.left_start)
            })
            .count()
    }

    pub fn can_piece_move_distance(&self, from: usize, dist: u8, backward: bool) -> bool {
        let piece = self.board.tiles[from].as_ref().unwrap();

        for to in 0..self.board.tiles.len() {
            let valid = if backward {
                self.board.distance_between(to, from, piece.owner) == Some(dist)
            } else {
                self.board.distance_between(from, to, piece.owner) == Some(dist)
            };

            if !valid {
                continue;
            }

            if let Some(path) =
                self.board.passed_tiles(from, to,  piece.owner, backward)
            {
                if self.board.is_path_free(&path) {
                    return true;
                }
            }
        }

        false
    }

    pub fn can_piece_move_from_to(&self, from: usize, to: usize, backward: bool) -> bool {
        let piece = self.board.tiles[from].as_ref().unwrap();

        let distance = if backward {
            self.board.distance_between(to, from, piece.owner)
        } else {
            self.board.distance_between(from, to, piece.owner)
        };

        let Some(_distance) = distance else {return false};

        if let Some(path) = self.board.passed_tiles(from, to, piece.owner, backward) {
            self.board.is_path_free(&path)
        } else {
            false
        }
    }

    fn action_place(&mut self, player_index: usize, _action:Action) -> Result<(), &'static str> {

        let ActionKind::Place { target_player } = _action.action else {
            return Err("Invalid action for place");
        };

        match _action.card {
            Some(Card::Ace) | Some(Card::King) | Some(Card::Joker) => {}
            _ => return Err("Cannot place piece with this card"),
        }

        if !self.can_place_for(player_index, target_player) {
            return Err("Not allowed to place a piece for this player");
        }

        let start = self.board.start_field(target_player);

        if self.players[target_player].pieces_to_place == 0 {
            return Err("No pieces left to place");
        }

        let mut beaten_piece_owner = None;
        if let Some(piece) = self.board.tiles[start].take() {
            if !piece.left_start {
                self.board.tiles[start] = Some(piece);
                return Err("Protected piece blocks the start field");
            }
            
            beaten_piece_owner = Some(piece.owner);
            self.player_mut_by_index(piece.owner).pieces_to_place += 1;
        }

        self.board.tiles[start] = Some(Piece::new(target_player));

        let played_card_index = Some(self.current_player()
            .cards.iter()
            .position(|&c| Some(c) == _action.card)
            .unwrap()
        );

        if let Some(card) = _action.card {
            self.player_mut_by_index(player_index).remove_card(card);
            self.discard.push(card);
        }

        self.players[target_player].pieces_to_place -= 1;

        self.history.push(HistoryEntry {
            action: _action,
            played_card_index,

            beaten_piece_owner,
            interchanged_piece_owner: None,
            placed_piece_owner: Some(target_player),

            split_rest_before: None,
            trade_buffer_before: Vec::new(),
            left_start_before: false,

            cards_dealt: Vec::new(),

            grabbed_from_player: None,
            grabbed_card: None,
            grabbed_card_index: None,
        });

        self.next_player();

        Ok(())
    }

    fn action_move(& mut self, player_index: usize, _action:Action) -> Result<(), &'static str> {

        let ActionKind::Move { from, to } = _action.action else {
            return Err("Invalid action for move");
        };

        match _action.card {
            Some(Card::Jack) => return Err("Cannot move piece with Jack"),
            Some(Card::Seven) => return Err("Cannot move with Seven (use Split)"),
            _ => {}   
        }

        let moving_piece = self.board.check_tile(from)
            .ok_or("Invalid move: no piece found")?;

        let left_start_before = moving_piece.left_start;

        if !self.can_control_piece(player_index, moving_piece.owner) {
            return Err("Cannot move this piece");
        }

        // Calculate distances and check if card allows the move
        let forward_distance = self.board.distance_between(from, to, moving_piece.owner);
        let backward_distance = self.board.distance_between(to, from, moving_piece.owner);

        if !_action.card.map_or(true, |c| self.can_card_move(c, forward_distance, backward_distance)) {
            return Err("Move not allowed with this card");
        }

        // Calculate path + direction
        let is_backward = 
            matches!(_action.card, Some(Card::Four | Card::Joker)) 
            && backward_distance == Some(4);

        let path = self.board
            .passed_tiles(from, to, moving_piece.owner, is_backward)
            .ok_or("Invalid move: path cannot be calculated")?;

        // Check for blocking pieces
        for &tile in &path {
            if let Some(piece) = self.board.tiles[tile] {
                if tile >= self.board.ring_size {
                    return Err("Cannot move past piece inside the house");
                } else if !piece.left_start {
                    return Err("Cannot move past protected piece");
                }
            }
        }

        // Move execution
        self.board.tiles[from] = None;

        // Remove piece from destination tile if opponent piece is there
        let mut beaten_piece_owner = None;

        if let Some(beaten_piece) = self.board.tiles[to].take() {
            beaten_piece_owner = Some(beaten_piece.owner);
            self.players[beaten_piece.owner].pieces_to_place += 1;
        }

        // Piece placement and history update
        self.board.tiles[to] = Some(Piece {
            owner: moving_piece.owner, 
            left_start: true 
            });

        // Piece moves into house
        if from < self.board.ring_size && to >= self.board.ring_size { 
            self.players[moving_piece.owner].pieces_in_house += 1;
        }

        let played_card_index = Some(self.current_player()
            .cards.iter()
            .position(|&c| Some(c) == _action.card)
            .unwrap()
        );

        if let Some(card) = _action.card {
            self.player_mut_by_index(player_index).remove_card(card);
            self.discard.push(card);
        }

        self.history.push(HistoryEntry { 
            action: _action,
            played_card_index,

            beaten_piece_owner, 
            interchanged_piece_owner: None,
            placed_piece_owner: None,

            split_rest_before: None,
            trade_buffer_before: Vec::new(),
            left_start_before,

            cards_dealt: Vec::new(),

            grabbed_from_player: None,
            grabbed_card: None,
            grabbed_card_index: None,
        });

        self.next_player();

        Ok(())
    }


    fn action_interchange(&mut self, player_index: usize, _action: Action) -> Result<(), &'static str> {
        
        let ActionKind::Interchange { a, b } = _action.action else {
            return Err("Invalid action for interchange");
        };

        match _action.card {
            Some(Card::Jack) | Some(Card::Joker) => {},
            _ => return Err("Cannot interchange pieces with this card"),
        }

        let a_piece = self.board.check_tile(a)
            .ok_or("Cannot interchange from an empty tile")?
            .clone();
        let b_piece = self.board.check_tile(b)
            .ok_or("Cannot interchange to an empty tile")?
            .clone();

        // Interchange only allowed on "ring-pieces"
        if a >= self.board.ring_size || b >= self.board.ring_size {
            return Err("Cannot interchange pieces inside player's houses");
        }

        if !self.can_control_piece(player_index, a_piece.owner) {
            return Err("Cannot interchange from a piece you don't control");
        }

        if !a_piece.left_start || !b_piece.left_start {
            return Err("Cannot interchange with protected piece");
        }

        // Interchange pieces
        self.board.tiles[a] = Some(b_piece);
        self.board.tiles[b] = Some(a_piece);

        let played_card_index = Some(self.current_player()
            .cards.iter()
            .position(|&c| Some(c) == _action.card)
            .unwrap()
        );

        self.player_mut_by_index(player_index).remove_card(_action.card.unwrap());
        self.discard.push(_action.card.unwrap());


        self.history.push(HistoryEntry {
            action: _action,
            played_card_index,

            beaten_piece_owner: None,
            interchanged_piece_owner: Some((a_piece.owner, b_piece.owner)),
            placed_piece_owner: None,

            split_rest_before: None,
            trade_buffer_before: Vec::new(),
            left_start_before: true,

            cards_dealt: Vec::new(),

            grabbed_from_player: None,
            grabbed_card: None,
            grabbed_card_index: None,
        });

        self.next_player();

        Ok(())
    }

    fn action_trade(&mut self, player_index: usize, _action: Action) -> Result<(), &'static str> {
        
        if !matches!(_action.action, ActionKind::Trade) {
            return Err("Invalid action for trade");
        }

        if !self.trading_phase {
            return Err("Cannot trade cards outside trading phase");
        }

        let trade_buffer_before = self.trade_buffer.clone();

        if self.trade_buffer.len() >= self.players.len() {
            return Err("Cannot trade more than one card per player");
        }

        let card = _action.card.ok_or("Trade requires a card")?;
        let played_card_index = Some(self.player_mut_by_index(player_index)
            .cards
            .iter()
            .position(|&c| c == card)
            .ok_or("Cannot trade: card not found in player's hand")?
        );
        let removed_card = self.player_mut_by_index(player_index).cards.remove(played_card_index.unwrap());

        let teammate_index = self.teammate_index(player_index)
            .ok_or("Trade: teammate not found")?;

        self.trade_buffer.push((player_index, teammate_index, removed_card));

        // Trade cards when every player has chosen a card
        if self.trade_buffer.len() == self.players.len() {
            let trades: Vec<_> = self.trade_buffer
                .drain(..)
                .collect();

            for (_from_index, to_index, card) in trades {
                self.player_mut_by_index(to_index).cards.push(card);
            }

            self.trading_phase = false;
        }

        // History update
        self.history.push(HistoryEntry {
            action: _action,
            played_card_index,

            beaten_piece_owner: None,
            interchanged_piece_owner: None,
            placed_piece_owner: None,

            split_rest_before: None,
            trade_buffer_before,
            left_start_before: false,

            cards_dealt: Vec::new(),

            grabbed_from_player: None,
            grabbed_card: None,
            grabbed_card_index: None,
        });

        self.next_player();

        Ok(())
    }

    fn action_trade_grab(&mut self, player_index: usize, _action: Action) -> Result<(), &'static str> {

        let ActionKind::TradeGrab { target_card } = _action.action else {
            return Err("Invalid action for trade grab");
        };

        if _action.card.is_some() {
            return Err("Invalid action: trade grab doesn't need card");
        }

        match self.game_variant {
            GameVariant::FreeForAll(_) => {}
            _ => return Err("Invalid action: cannot perform trade grab in team games."),
        };

        if !self.trading_phase {
            return Err("Cannot trade grab cards outside trading phase");
        }

        let previous_player = if player_index == 0 {
                self.players.len() - 1
            } else {
                player_index - 1
        };

        if target_card >= self.players[previous_player].cards.len() {
            return Err("Invalid action: cannot grab selected card");
        }

        let trade_buffer_before = self.trade_buffer.clone();

        let removed_card = self.player_mut_by_index(previous_player)
            .cards
            .remove(target_card);

        self.trade_buffer.push((previous_player, player_index, removed_card));

        if self.trade_buffer.len() == self.players.len() {
            let trades: Vec<_> = self.trade_buffer
                .drain(..)
                .collect();

            for (_from_index, to_index, card) in trades {
                self.player_mut_by_index(to_index).cards.push(card);
            }

            self.trading_phase = false;
        }

        // History update
        self.history.push(HistoryEntry { 
            action: _action,
            played_card_index: None,

            beaten_piece_owner: None, 
            interchanged_piece_owner: None, 
            placed_piece_owner: None, 
            
            split_rest_before: None, 
            trade_buffer_before, 
            left_start_before: false, 
            
            cards_dealt: Vec::new(), 
            
            grabbed_from_player: Some(previous_player), 
            grabbed_card: Some(removed_card), 
            grabbed_card_index: Some(target_card) 
        });

        self.next_player();

        Ok(())
    }

    fn action_split(&mut self, player_index: usize, _action: Action) -> Result<(), &'static str> {
        
        match _action.card {
            Some(Card::Seven) | Some(Card::Joker) => {},
            _ => return Err("Cannot split move with this card."),
        }

        let current_player_index = player_index;

        let (from, to) = match _action.action {
            ActionKind::Split { from, to } => (from, to),
            _ => return Err("Invalid action kind for split"),
        };

        let moving_piece = self.board.check_tile(from)
            .ok_or("Invalid move: no piece found.")?;

        let team_indices = match self.game_variant {
            GameVariant::FreeForAll(n) => (0..n).collect(), // FFA: 7 can split-move every piece
            _ => self.teammate_indices(current_player_index),
        };

        // Ony team pieces can be moved
        if moving_piece.owner != current_player_index && !team_indices.contains(&moving_piece.owner) {
            return Err("Cannot split-move a piece you do not control (own or teammate).");
        }

        let mut remaining_steps = self.split_rest.unwrap_or(7);
        let total_distance = self.board.distance_between(from, to, moving_piece.owner)
            .ok_or("Invalid action: cannot calculate distance")?;

        if total_distance == 0 || total_distance > 7 {
            return Err("Split move must be 1..7 steps.");
        }

        //Check split_rest
        if total_distance > remaining_steps {
            return Err("Cannot move more steps than remaining split.");
        }

        // Calculate path
        let path = self.board.passed_tiles(from, to, moving_piece.owner, false)
            .ok_or("Invalid split move: path cannot be calculated.")?;

        // Check for blocked tiles
        for &tile in &path {
            if let Some(piece) = self.board.tiles[tile] {
                if tile >= self.board.ring_size {
                    return Err("Cannot move past piece inside the house");
                } else if !piece.left_start {
                    return Err("Cannot move past protected piece.");
                }
            }
        }

        // Split execution along path
        let mut current_position = from;
        let mut split_rest_before = self.split_rest;
        let mut left_start_before = moving_piece.left_start;

        let played_card_index = Some(self.current_player()
            .cards.iter()
            .position(|&c| Some(c) == _action.card)
            .unwrap()
        );

        for &tile in &path {

            // Create "mini"- history if piece is beaten
            if let Some(beaten_piece) = self.board.tiles[tile].take() {
                self.player_mut_by_index(beaten_piece.owner).pieces_to_place += 1;

                let distance = self.board
                    .distance_between(current_position, tile, moving_piece.owner)
                    .expect("Distance must exist");

                // Mini history
                self.history.push(HistoryEntry {
                    action: Action {
                        player: _action.player,
                        card: _action.card,
                        action: ActionKind::Split { from: current_position, to: tile },
                    },
                    played_card_index,

                    beaten_piece_owner: Some(beaten_piece.owner),
                    interchanged_piece_owner: None,
                    placed_piece_owner: None,

                    split_rest_before,
                    trade_buffer_before: Vec::new(),
                    left_start_before,

                    cards_dealt: Vec::new(),

                    grabbed_from_player: None,
                    grabbed_card: None,
                    grabbed_card_index: None,
                });

                // Update step-mechanism
                remaining_steps -= distance;
                split_rest_before = Some(remaining_steps);

                current_position = tile;
                left_start_before = true;
            }
        }

        // Piece placement and history update
        self.board.tiles[from] = None;
        self.board.tiles[to] = Some(Piece {
            owner: moving_piece.owner,
            left_start: true,
        });

        if from < self.board.ring_size && to >= self.board.ring_size {
            self.player_mut_by_index(moving_piece.owner).pieces_in_house += 1;
        }

        // History update if last step doesn't beat piece
        if current_position != to {
            let distance = self.board
                .distance_between(current_position, to, moving_piece.owner)
                .expect("Distance must exist");

            if current_position != from {
                left_start_before = true;
            }

            self.history.push(HistoryEntry {
                action: Action {
                    player: _action.player,
                    card: _action.card,
                    action: ActionKind::Split { from: current_position, to },
                },
                played_card_index,

                beaten_piece_owner: None,
                interchanged_piece_owner: None,
                placed_piece_owner: None,

                split_rest_before,
                trade_buffer_before: Vec::new(),
                left_start_before,

                cards_dealt: Vec::new(),

                grabbed_from_player: None,
                grabbed_card: None,
                grabbed_card_index: None,
            });

            remaining_steps -= distance;
        }

        // Update split_rest & update current player
        if remaining_steps == 0 {
            self.split_rest = None;
            self.player_mut_by_index(current_player_index).remove_card(_action.card.unwrap());
            self.discard.push(_action.card.unwrap());
            self.next_player();
        } else {
            self.split_rest = Some(remaining_steps);
        }

        Ok(())
    }

    fn action_remove(&mut self, player_index: usize, _action: Action) -> Result<(), &'static str> {
        
        if self.check_if_any_action_possible() {
            return Err("Cannot remove: other action possible.");
        }

        let card = _action.card.ok_or("Remove action requires a card.")?;

        let played_card_index = Some(self.players[player_index]
            .cards
            .iter()
            .position(|&c| c == card)
            .ok_or("Cannot remove: card not found in player's hand.")?
        );

        self.players[player_index].remove_card(card);
        self.discard.push(card);

        // History update
        self.history.push(HistoryEntry {
            action: _action,
            played_card_index,

            beaten_piece_owner: None,
            interchanged_piece_owner: None,
            placed_piece_owner: None,

            split_rest_before: None,
            trade_buffer_before: Vec::new(),
            left_start_before: false,

            cards_dealt: Vec::new(),

            grabbed_from_player: None,
            grabbed_card: None,
            grabbed_card_index: None,
        });

        self.next_player();

        Ok(())
    }

    fn action_grab(&mut self, player_index: usize, _action: Action) -> Result<(), &'static str> {
        let card = _action.card.ok_or("Grab action requires a card.")?;

        match self.game_variant {
            GameVariant::FreeForAll(_) => {}
            _ => return Err("Invalid action: cannot perform grab in team games."),
        };

        match card {
            Card::Two => {}
            _ => return Err("Invalid action: cannot grab with this card."),
        };

        let ActionKind::Grab { target_card, target_player } = _action.action else {
            return Err("Invalid grab action.");
        };

        let target_player_index = self.index_of_color(target_player);

        if target_player_index == player_index {
            return Err("Cannot grab a card from yourself.");
        }

        if target_card >= self.players[target_player_index].cards.len() {
            return Err("Invalid action: cannot grab selected card.");
        }

        // Update player cards
        let grabbed_card = self.players[target_player_index]
            .cards
            .remove(target_card);

        self.players[player_index].cards.push(grabbed_card);

        let played_card_index = Some(self.current_player()
            .cards.iter()
            .position(|&c| Some(c) == _action.card)
            .unwrap()
        );

        self.players[player_index].remove_card(card);
        self.discard.push(card);

        // History update
        self.history.push(HistoryEntry {
            action: _action,
            played_card_index,

            beaten_piece_owner: None,
            interchanged_piece_owner: None,
            placed_piece_owner: None,

            split_rest_before: None,
            trade_buffer_before: Vec::new(),
            left_start_before: false,

            cards_dealt: Vec::new(),

            grabbed_from_player: Some(target_player_index),
            grabbed_card: Some(grabbed_card),
            grabbed_card_index: Some(target_card),
        });

        self.next_player();

        Ok(())
    }

}

pub trait DogGame {
    // Creates new instance with an empty board and initialized deck and players based on chosen variant
    fn new(variant: GameVariant) -> Self;

    // Returns the current state of the board
    fn board_state(&self) -> &[Option<Piece>];

    // Returns the current player
    fn current_player(&self) -> &Player;

    // Matches and applies the action of playing the given card for the current player
    fn action(&mut self, card: Option<Card>, action: Action) -> Result<(), &'static str>;

    // Undoes the last action
    fn undo_action(&mut self) -> Result<(), &'static str>;

    // // Undoes the last complete turn, including all actions that belong to it
    fn undo_turn(&mut self) -> Result<(), &'static str>;

    // Undoes multiple turns in sequence
    fn undo_sequence(&mut self, turns: usize) -> Result<(), &'static str>;

    // Gives players new cards after previous round is finished
    fn new_round(&mut self);

    // Checks if there is yet a winning team / player
    fn is_winner(&self) -> bool;
}

impl DogGame for Game {
    fn new(variant: GameVariant) -> Self {
        Game::new(variant)
    }

    fn current_player(&self) -> &Player {
        self.current_player()
    }
    
    fn board_state(&self) -> &[Option<Piece>] {
        &self.board.get_board()
    }

    fn action(&mut self, card: Option<Card>, _action: Action) -> Result<(), &'static str> {
        
        if self.current_player().color != _action.player {
            return Err("It's not his player's turn.");
        }

        // Split check
        if self.split_rest.is_some() && !matches!(_action.action, ActionKind::Split { .. }) {
                return Err("Cannot perform actions other than Split during splitting phase.");
        }

        // Trading phase check
        if self.trading_phase && !matches!(_action.action, ActionKind::Trade | ActionKind::TradeGrab { .. }) {
            return Err("Cannot perform actions other than Trade during swapping phase.");
        }

        // Card check
        match _action.action {
            ActionKind::TradeGrab { .. } => {
                if card.is_some() {
                    return Err("Invalid action: TradeGrab does not use a card.");
                }
            }
            _ => {
                let c = card.ok_or("Invalid action: this action requires a card")?;

                if !self.current_player().cards.contains(&c) {
                    return Err("Invalid action: Card not in player's hand.");
                }
            }

        }

        let current_player = self.current_player_index;

        match _action.action {

            // Place: Player can place on of his remaining pieces on the board
            ActionKind::Place { .. } => {
                self.action_place(current_player, _action)?;
            }

            // Move: Player can move piece for a given distance
            ActionKind::Move { .. } => {
                self.action_move(current_player, _action)?;      
            },

            // Interchange: Player can switch the position of two pieces on the ring
            ActionKind::Interchange  { .. } => {
                self.action_interchange(current_player, _action)?;
            },

            // Trade: Player trades on card to his team members at the beginning of each round
            ActionKind::Trade => {
                self.action_trade(current_player, _action)?;        
            },

            // TradeGrab: FFA variant of Trade at the beginning of each new round (player chooses card of right-sided neighbour)
            ActionKind::TradeGrab { .. } => {
                self.action_trade_grab(current_player, _action)?;
            }

            // Split: Player can distribute move value to different pieces
            ActionKind::Split { .. } => {
                self.action_split(current_player, _action)?;
            },

            // Remove: Player removes one card in his hand of no other action is possible
            ActionKind::Remove => {
                self.action_remove(current_player, _action)?;
            },

            // Grab: Player draws one card from target_player using Card::Two
            ActionKind::Grab { .. } => {
                self.action_grab(current_player, _action)?;
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
            for (player_index, _) in &entry.cards_dealt {
                self.players[*player_index].cards.clear();
            }

            self.trading_phase = false;
            self.round -= 1;
        }

        let entry_player_index = self.index_of_color(entry.action.player);
        let played_card = entry.action.card;
        
        let played_card_index = entry.played_card_index.unwrap_or(0);

        match entry.action.action {
            ActionKind::Place { .. }=> {
                let placed_piece_owner = entry
                    .placed_piece_owner
                    .expect("Place undo requires placed_piece_owner");

                let start = self.board.start_field(placed_piece_owner);

                self.board.tiles[start] = None;
                self.players[placed_piece_owner].pieces_to_place += 1;

                if let Some(beaten_piece_owner) = entry.beaten_piece_owner {
                    self.board.tiles[start] = Some(Piece {
                        owner: beaten_piece_owner,
                        left_start: true,
                    });

                    self.players[beaten_piece_owner].pieces_to_place -= 1;
                }

                self.discard.pop();
                self.players[entry_player_index].cards.insert(played_card_index, played_card.unwrap());

                self.current_player_index = entry_player_index;
            },

            ActionKind::Interchange { a, b } => {
                let (a_owner, b_owner) = entry
                    .interchanged_piece_owner
                    .ok_or("Missing interchanged_piece_owner in history")?;

                self.board.tiles[a] = Some(Piece {
                    owner: a_owner,
                    left_start: true,
                });

                self.board.tiles[b] = Some(Piece {
                    owner: b_owner,
                    left_start: true,
                });

                self.discard.pop();
                self.players[entry_player_index].cards.push(played_card.unwrap());

                self.current_player_index = entry_player_index;
            },

            ActionKind::Move { from, to } => {

                let moved_piece = self.board.tiles[to]
                    .take()
                    .ok_or("Expected moved piece on target tile")?;

                self.board.tiles[from] = Some(Piece { 
                    owner: moved_piece.owner, 
                    left_start: entry.left_start_before 
                });

                if from < self.board.ring_size && to >= self.board.ring_size {
                    self.players[moved_piece.owner].pieces_in_house -= 1;
                }

                if let Some(beaten_piece_owner) = entry.beaten_piece_owner {
                    self.board.tiles[to] = Some(Piece {
                        owner: beaten_piece_owner,
                        left_start: true,
                    });

                    self.players[beaten_piece_owner].pieces_to_place -= 1;
                }

                self.discard.pop();
                self.players[entry_player_index].cards.insert(played_card_index, played_card.unwrap());

                self.current_player_index = entry_player_index;
            },

            ActionKind::Trade => {

                // Check if trade phase just ended
                if entry.trade_buffer_before.len() == (self.players.len() - 1) {

                    let mut trades: Vec<_> = entry.trade_buffer_before.clone();
                    trades.push((
                        entry_player_index,
                        self.teammate_index(entry_player_index)
                            .expect("Teammate must exist"),
                        played_card.unwrap(),
                    ));

                    // Reverse trade
                    for (_, to, card) in trades {
                        self.players[to].remove_card(card);
                    }

                    self.trading_phase = true;

                }

                self.players[entry_player_index]
                    .cards
                    .push(played_card.unwrap());

                self.trade_buffer = entry.trade_buffer_before;
                self.current_player_index = entry_player_index;
            },

            ActionKind::TradeGrab { .. } => {
                // Check if trade phase just ended
                if entry.trade_buffer_before.len() == (self.players.len() - 1) {

                    let mut trades: Vec<_> = entry.trade_buffer_before.clone();
                    trades.push((
                        entry.grabbed_from_player.unwrap(),
                        entry_player_index,
                        entry.grabbed_card.unwrap(),
                    ));

                    // Reverse trade
                    for (_, to, card) in trades {
                        let pos = self.players[to]
                            .cards
                            .iter()
                            .position(|&c| c == card)
                            .expect("Traded card must exist in recipient hand");

                        self.players[to].cards.remove(pos);
                    }

                    self.trading_phase = true;

                }


                    let grabbed_from_player = entry.grabbed_from_player.unwrap();
                    let grabbed_card_index = entry.grabbed_card_index.unwrap();
                    let grabbed_card = entry.grabbed_card.unwrap();

                    self.players[grabbed_from_player].cards
                        .insert(grabbed_card_index, grabbed_card);




                self.trade_buffer = entry.trade_buffer_before;
                self.current_player_index = entry_player_index;
            }

            ActionKind::Split { from, to } => {

                let moved_piece = self.board.tiles[to]
                    .take()
                    .ok_or("No piece to undo split")?;

                self.board.tiles[from] = Some(Piece {
                    owner: moved_piece.owner,
                    left_start: entry.left_start_before
                });

                if from < self.board.ring_size && to >= self.board.ring_size {
                    self.players[moved_piece.owner].pieces_in_house -= 1;
                }

                if let Some(beaten_piece_owner) = entry.beaten_piece_owner {
                    self.board.tiles[to] = Some(Piece {
                        owner: beaten_piece_owner,
                        left_start: true,
                    });

                    self.players[beaten_piece_owner].pieces_to_place -= 1;
                }

                // Return card if split just began
                if entry.split_rest_before.is_none() {
                    self.discard.pop();
                    self.players[entry_player_index].cards.insert(played_card_index, played_card.unwrap());
                }

                self.split_rest = entry.split_rest_before;
                self.current_player_index = entry_player_index;                
            },

            ActionKind::Remove => {
                self.discard.pop();
                self.players[entry_player_index].cards.push(played_card.unwrap());

                self.current_player_index = entry_player_index;
            },

            ActionKind::Grab { .. } => {
                let grabbed_from_player = entry
                    .grabbed_from_player
                    .ok_or("Undo grab failed: missing grabbed_from_player")?;

                let grabbed_card = entry
                    .grabbed_card
                    .ok_or("Undo grab failed: missing grabbed_card")?;

                let grabbed_card_index = entry
                    .grabbed_card_index
                    .ok_or("Undo grab failed: missing grabbed_card_index")?;

                self.players[grabbed_from_player]
                    .cards
                    .insert(grabbed_card_index, grabbed_card);

                let card_index = self.players[entry_player_index]
                    .cards
                    .iter()
                    .position(|c| *c == grabbed_card)
                    .ok_or("Undo grab failed: card not found in current player hand")?;

                self.players[entry_player_index].cards.remove(card_index);
                self.discard.pop();
                self.players[entry_player_index].cards.insert(played_card_index, played_card.unwrap());

                self.prev_player();
            }
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
                ActionKind::Split { .. } => {
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

        // Reset deck & discard
        self.deck = Deck::new();
        self.deck.shuffle();
        self.discard.clear();

        
        let round_index = (self.round - 1) % 4;
        let cards_to_deal = CARDS_PER_ROUND[round_index as usize];

        // Deal cards
        for _ in 0..cards_to_deal {
            for player in &mut self.players {
                player.cards.push(
                    self.deck.draw().expect("Deck should contain enough cards"),
                );
            }
        }

            self.trading_phase = true;

        // Starting player rotates by round
        self.current_player_index = (self.round - 1) % self.players.len();
        
        self.round += 1;

        if let Some(entry) = self.history.last_mut() {
            entry.cards_dealt = self.players
                .iter()
                .enumerate()
                .map(|(i, p)| (i, p.cards.clone()))
                .collect();
        }
    }
    
    fn is_winner(&self) -> bool {
        let current_index = self.current_player_index;

        if let Some(teams) = &self.teams {
            
            if let Some(team) = teams.iter().find(|t| t.contains(&current_index)) {
                // Team wins if all members have 4 pieces in house
                return team.iter().all(|&i| self.players[i].pieces_in_house == 4);
            }
        } else {
            // Free-for-all: player wins if he has 4 pieces in house
            return self.players[current_index].pieces_in_house == 4;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod helper_tests {
        use super::*;

        mod game_variant_tests {
            use super::*;

            #[test]
            fn game_variant_player_counts() {
                assert_eq!(Game::new(GameVariant::TwoVsTwo).players.len(), 4);
                assert_eq!(Game::new(GameVariant::ThreeVsThree).players.len(), 6);
                assert_eq!(Game::new(GameVariant::TwoVsTwoVsTwo).players.len(), 6);

                for n in 2..=6 {
                    assert_eq!(
                        Game::new(GameVariant::FreeForAll(n)).players.len(),
                        n
                    );
                }
            }

            #[test]
            fn teams_exist_only_in_team_modes() {
                assert!(Game::new(GameVariant::TwoVsTwo).teams.is_some());
                assert!(Game::new(GameVariant::ThreeVsThree).teams.is_some());
                assert!(Game::new(GameVariant::TwoVsTwoVsTwo).teams.is_some());
                assert!(Game::new(GameVariant::FreeForAll(4)).teams.is_none());
            }

        }

        mod teammate_indices_tests {
            use super::*;

            #[test]
            fn teammate_indices_in_2v2() {
                let game = Game::new(GameVariant::TwoVsTwo);

                assert_eq!(game.teammate_indices(0), vec![2]);
                assert_eq!(game.teammate_indices(2), vec![0]);
                assert_eq!(game.teammate_indices(1), vec![3]);
            }

            #[test]
            fn teammate_indices_in_3v3() {
                let game = Game::new(GameVariant::ThreeVsThree);

                assert_eq!(game.teammate_indices(0), vec![2, 4]);
                assert_eq!(game.teammate_indices(2), vec![0, 4]);
                assert_eq!(game.teammate_indices(4), vec![0, 2]);
                assert_eq!(game.teammate_indices(1), vec![3, 5]);
            }

            #[test]
            fn teammate_indices_in_2v2v2() {
                let game = Game::new(GameVariant::TwoVsTwoVsTwo);

                assert_eq!(game.teammate_indices(0), vec![3]);
                assert_eq!(game.teammate_indices(3), vec![0]);
                assert_eq!(game.teammate_indices(1), vec![4]);
                assert_eq!(game.teammate_indices(2), vec![5]);
            }

            #[test]
            fn teammate_indices_empty_in_ffa() {
                let game = Game::new(GameVariant::FreeForAll(4));

                for i in 0..4 {
                    assert!(game.teammate_indices(i).is_empty());
                    assert!(game.teammate_index(i).is_none());
                }
            }
        }

        mod can_control_piece_tests {
            use super::*;

            #[test]
            fn can_control_own_piece() {
                let game = Game::new(GameVariant::TwoVsTwo);
                assert!(game.can_control_piece(0, 0));
            }

            #[test]
            fn cannot_control_teammate_without_all_pieces_home() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.players[0].pieces_in_house = 3;

                assert!(!game.can_control_piece(0, 2));
            }

            #[test]
            fn can_control_teammate_with_all_pieces_home() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.players[0].pieces_in_house = 4;

                assert!(game.can_control_piece(0, 2));
            }

            #[test]
            fn cannot_control_in_ffa() {
                let mut game = Game::new(GameVariant::FreeForAll(4));
                game.players[0].pieces_in_house = 4;

                assert!(!game.can_control_piece(0, 1));
            }
        }

        mod can_place_for_tests {
            use super::*;

            #[test]
            fn can_place_for_self() {
                let game = Game::new(GameVariant::TwoVsTwo);
                assert!(game.can_place_for(0, 0));
            }

            #[test]
            fn cannot_place_for_teammate_without_all_pieces_home() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.players[0].pieces_in_house = 3;

                assert!(!game.can_place_for(0, 2));
            }

            #[test]
            fn can_place_for_teammate_with_all_pieces_home() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.players[0].pieces_in_house = 4;

                assert!(game.can_place_for(0, 2));
            }

            #[test]
            fn cannot_place_for_anyone_in_ffa() {
                let mut game = Game::new(GameVariant::FreeForAll(4));
                game.players[0].pieces_in_house = 4;

                assert!(!game.can_place_for(0, 1));
            }

        }

        mod next_player_test {
            use super::*;

            #[test]
            fn next_player_wraps_correctly_for_all_variants() {
                // 2 vs 2
                let mut game_2v2 = Game::new(GameVariant::TwoVsTwo);
                game_2v2.current_player_index = game_2v2.players.len() - 1;
                game_2v2.next_player();
                assert_eq!(game_2v2.current_player_index, 0);

                // 3 vs 3
                let mut game_3v3 = Game::new(GameVariant::ThreeVsThree);
                game_3v3.current_player_index = game_3v3.players.len() - 1;
                game_3v3.next_player();
                assert_eq!(game_3v3.current_player_index, 0);

                // 2 vs 2 vs 2
                let mut game_2v2v2 = Game::new(GameVariant::TwoVsTwoVsTwo);
                game_2v2v2.current_player_index = game_2v2v2.players.len() - 1;
                game_2v2v2.next_player();
                assert_eq!(game_2v2v2.current_player_index, 0);

                // Free-for-All
                for n in 2..=6 {
                    let mut game_ffa = Game::new(GameVariant::FreeForAll(n));
                    game_ffa.current_player_index = game_ffa.players.len() - 1;
                    game_ffa.next_player();
                    assert_eq!(game_ffa.current_player_index, 0);
                }
            }

        }

        mod index_of_color_tests {
            use super::*;

            #[test]
            fn index_of_color_is_correct_for_all_variants() {
                // 2 vs 2
                let game_2v2 = Game::new(GameVariant::TwoVsTwo);
                for (i, player) in game_2v2.players.iter().enumerate() {
                    assert_eq!(game_2v2.index_of_color(player.color), i);
                }

                // 3 vs 3
                let game_3v3 = Game::new(GameVariant::ThreeVsThree);
                for (i, player) in game_3v3.players.iter().enumerate() {
                    assert_eq!(game_3v3.index_of_color(player.color), i);
                }

                // 2 vs 2 vs 2
                let game_2v2v2 = Game::new(GameVariant::TwoVsTwoVsTwo);
                for (i, player) in game_2v2v2.players.iter().enumerate() {
                    assert_eq!(game_2v2v2.index_of_color(player.color), i);
                }

                // Free-for-All
                for n in 2..=6 {
                    let game_ffa = Game::new(GameVariant::FreeForAll(n));
                    for (i, player) in game_ffa.players.iter().enumerate() {
                        assert_eq!(game_ffa.index_of_color(player.color), i);
                    }
                }
            }

        }

        mod play_tests {
            use super::*;

            fn setup_game() -> Game {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.players[0].cards = vec![Card::Ace, Card::Two, Card::Three];
                game.players[1].cards = vec![Card::Four, Card::Five, Card::Six];
                game.players[2].cards = vec![Card::Seven, Card::Eight, Card::Nine];
                game.players[3].cards = vec![Card::Ten, Card::Jack, Card::Queen];
                game
            }

            #[test]
            fn play_valid_place() {
                let mut game = setup_game();
                game.trading_phase = false;

                let input = "R 1 P 0"; // Red places with Ace

                assert!(game.play(input).is_ok());

                assert!(!game.players[0].cards.contains(&Card::Ace));
                assert_eq!(game.current_player_index, 1);
            }

            #[test]
            fn play_invalid_card_string() {
                let mut game = setup_game();
                let input = "R X P 0";

                assert!(game.play(input).is_err());
                assert_eq!(game.current_player_index, 0);
            }

            #[test]
            fn play_invalid_player() {
                let mut game = setup_game();
                let input = "S 5 T";

                assert!(game.play(input).is_err());
                assert_eq!(game.current_player_index, 0);
            }

            #[test]
            fn play_invalid_action() {
                let mut game = setup_game();
                let input = "R 5 Z";

                assert!(game.play(input).is_err());
                assert_eq!(game.current_player_index, 0);
            }

            #[test]
            fn play_trade_sequence() {
                let mut game = setup_game();

                let inputs = ["R 2 T", "G 5 T", "B 7 T", "Y 10 T"];
                for input in inputs.iter() {
                    assert!(game.play(input).is_ok());
                }

                assert!(!game.trading_phase);
                assert_eq!(game.players[0].cards.len(), 3);
                assert_eq!(game.players[1].cards.len(), 3);
                assert!(game.players[0].cards.contains(&Card::Seven));
            }

            #[test]
            fn play_place_action() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Joker];

                assert!(game.play("R 0 P 0").is_ok());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                assert_eq!(game.players[0].pieces_to_place, 3)
            }

            #[test]
            fn play_move_action() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Four];
                game.board.tiles[60] = Some(Piece { 
                    owner: 0, 
                    left_start: true 
                });

                assert!(game.play("R 4 M 60 56").is_ok());
                assert!(game.board.tiles[60].is_none());
                assert_eq!(game.board.tiles[56].as_ref().unwrap().owner, 0);
            }

            #[test]
            fn play_remove_action() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Five];
                game.players[1].cards = vec![Card::Ace];

                assert!(game.play("R 5 R").is_ok());
                assert!(game.players[0].cards.is_empty());
                assert_eq!(game.current_player_index, 1);
            }

            #[test]
            fn play_interchange_action() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Joker];

                game.board.tiles[60] = Some(Piece { 
                    owner: 0, 
                    left_start: true 
                });

                game.board.tiles[56] = Some(Piece { 
                    owner: 1, 
                    left_start: true 
                });

                assert!(game.play("R 0 I 60 56").is_ok());
                assert_eq!(game.board.tiles[56].as_ref().unwrap().owner, 0);
                assert_eq!(game.board.tiles[60].as_ref().unwrap().owner, 1);
            }

            #[test]
            fn play_split_action() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;
                game.players[0].cards = vec![Card::Seven];
                game.players[1].cards = vec![Card::Seven];

                // Setup pieces for split
                game.board.tiles[0] = Some(Piece { owner: 0, left_start: true });
                game.board.tiles[3] = Some(Piece { owner: 1, left_start: true });

                let input1 = "R 7 S 0 5";
                let input2 = "R 7 S 5 7";

                assert!(game.play(input1).is_ok());
                assert_eq!(game.split_rest, Some(2));

                assert!(game.play(input2).is_ok());
                assert_eq!(game.split_rest, None);
                assert_eq!(game.current_player_index, 1);
            }

            #[test]
            fn play_invalid_split() {
                let mut game = setup_game();
                game.players[0].cards = vec![Card::Five];

                let input = "R 5 S 0 10";

                assert!(game.play(input).is_err());
                assert_eq!(game.split_rest, None);
            }
        }
    
        mod check_any_action_possible_tests {
            use super::*;

            #[test]
            fn no_cards_no_action_possible() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards.clear();

                assert!(!game.check_if_any_action_possible());
            }

            #[test]
            fn cards_but_no_pieces_no_action_possible() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Five];
                game.players[0].pieces_to_place = 0;

                assert!(!game.check_if_any_action_possible());
            }

            #[test]
            fn place_possible_on_empty_start() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Ace];
                game.players[0].pieces_to_place = 4;

                assert!(game.check_if_any_action_possible());
            }

            #[test]
            fn place_possible_by_beating_opponent() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                let start = game.board.start_field(0) as usize;
                game.board.tiles[start] = Some(Piece {
                    owner: 1,
                    left_start: true,
                });

                game.players[0].cards = vec![Card::Ace];
                game.players[0].pieces_to_place = 4;

                assert!(game.check_if_any_action_possible());
            }

            #[test]
            fn place_partner_piece_possible() {
                let mut game = Game::new_3v3();
                game.trading_phase = false;

                game.players[0].pieces_in_house = 4;
                game.players[0].cards = vec![Card::Ace];

                assert!(game.check_if_any_action_possible());
            }

            #[test]
            fn normal_move_possible() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Five];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                assert!(game.check_if_any_action_possible());
            }

            #[test]
            fn move_blocked_no_move_possible() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Five];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                game.board.tiles[3] = Some(Piece {
                    owner: 1,
                    left_start: false,
                });

                assert!(!game.check_if_any_action_possible());
            }

            #[test]
            fn seven_split_possible() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                assert!(game.check_if_any_action_possible());
            }

            #[test]
            fn seven_but_no_split_possible() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];
                game.players[0].pieces_to_place = 0;

                // no pieces on board

                assert!(!game.check_if_any_action_possible());
            }

            #[test]
            fn seven_split_blocked() {
                let mut game = Game::new_2v2();
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: false,
                });

                game.board.tiles[1] = Some(Piece {
                        owner: 1,
                        left_start: false,
                });

                assert!(!game.check_if_any_action_possible());                
            }

            #[test]
            fn seven_split_separate_moves_not_possible() {
                let mut game = Game::new_2v2();
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: false,
                });

                game.board.tiles[4] = Some(Piece {
                        owner: 1,
                        left_start: false,
                });

                game.board.tiles[5] = Some(Piece {
                    owner: 0,
                    left_start: false,
                });

                game.board.tiles[9] = Some(Piece {
                        owner: 1,
                        left_start: false,
                });

                assert!(!game.check_if_any_action_possible());
            }

            #[test]
            fn seven_split_single_moves_nearly_all_blocked() {
                let mut game = Game::new_2v2();
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];

                for position in 0..6 {
                    game.board.tiles [3 * position] = Some(Piece { 
                        owner: (2 * position) % 4, 
                        left_start: false 
                    });

                    game.board.tiles[3 * position + 2] = Some(Piece { 
                        owner: 1, 
                        left_start: false, 
                    });
                }

                assert!(!game.check_if_any_action_possible());

                game.board.tiles[18] = Some(Piece { 
                    owner: 0, 
                    left_start: false 
                });

                game.board.tiles[20] = Some(Piece { 
                    owner: 1, 
                    left_start: false 
                });

                assert!(game.check_if_any_action_possible());
            }

            #[test]
            fn seven_split_only_team_piece_possible() {
                let mut game = Game::new_2v2();
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];

                game.board.tiles[0] = Some(Piece { owner: 0, left_start: false });
                game.board.tiles[1] = Some(Piece { owner: 1, left_start: false });

                game.board.tiles[5] = Some(Piece { owner: 2, left_start: true });

                assert!(game.check_if_any_action_possible());
            }

            #[test]
            fn seven_cannot_enter_house_without_left_start() {
                let mut game = Game::new_2v2();
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];

                game.board.tiles[0] = Some(Piece { owner: 0, left_start: false });
                game.board.tiles[4] = Some(Piece { owner: 1, left_start: false });

                game.board.tiles[5] = Some(Piece { owner: 2, left_start: false });
                game.board.tiles[9] = Some(Piece { owner: 1, left_start: false });

                assert!(!game.check_if_any_action_possible());
            }



            #[test]
            fn jack_swap_possible() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Jack];

                game.board.tiles[0] = Some(Piece { owner: 0, left_start: true });
                game.board.tiles[5] = Some(Piece { owner: 1, left_start: true });

                assert!(game.check_if_any_action_possible());
            }

            #[test]
            fn jack_but_no_swap_possible() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Jack];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: false,
                });

                assert!(!game.check_if_any_action_possible());
            }

            #[test]
            fn team_piece_move_possible_in_3v3() {
                let mut game = Game::new(GameVariant::ThreeVsThree);
                game.trading_phase = false;

                game.players[0].pieces_in_house = 4;
                game.players[0].cards = vec![Card::Five];

                game.board.tiles[15] = Some(Piece {
                    owner: 4,
                    left_start: true,
                });

                assert!(game.check_if_any_action_possible());
            }


        }
    }
    mod action_tests {
        use super::*;

        fn setup_game(variant: GameVariant) -> Game {
                let mut game = Game::new(variant);
                game.trading_phase = false;
                game
        }

        #[test]
        fn wrong_player_cannot_act() {
            let mut game = Game::new(GameVariant::TwoVsTwo);
            game.trading_phase = false;

            let action = Action {
                player: Color::Green,
                action: ActionKind::Place {target_player: 1},
                card: Some(Card::Ace),
            };
      
            assert!(game.action(Some(Card::Ace), action).is_err());
            assert_eq!(game.current_player_index, 0);
        }

        #[test]
        fn invalid_action_does_not_change_state() {
            let mut game = Game::new(GameVariant::TwoVsTwo);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Ace];

            let board_before = game.board.tiles.clone();
            let discard_before = game.discard.clone();
            let history_len_before = game.history.len();
            let red_cards_before = game.players[0].cards.clone();

            let action = Action {
                player: Color::Red,
                action: ActionKind:: Place { target_player: 0 },
                card: Some(Card::Two),
            };

            assert!(game.action(Some(Card::Two), action).is_err());

            assert_eq!(game.board.tiles, board_before);
            assert_eq!(game.discard, discard_before);
            assert_eq!(game.history.len(), history_len_before);
            assert_eq!(game.players[0].cards, red_cards_before);
            assert_eq!(game.current_player_index, 0);
        }

         #[test]
        fn cannot_act_during_other_phase() {
            let mut game = Game::new(GameVariant::TwoVsTwo);

            game.players[0].cards = vec![Card::Ace];

            let action = Action {
                player: Color::Red,
                action: ActionKind::Remove,
                card: Some(Card::Ace),
            };

            assert!(game.action(Some(Card::Ace), action).is_err());
            assert_eq!(game.current_player_index, 0);
            assert!(game.players[0].cards.contains(&Card::Ace));
        }

        mod action_place_tests {
            use super::*;

            #[test]
            fn place_on_empty_start_all_variants() {
                let variants = [
                    GameVariant::TwoVsTwo,
                    GameVariant::ThreeVsThree,
                    GameVariant::TwoVsTwoVsTwo,
                ];

                // Free-for-All fails because start is already occupied by default
                for n in 2..=6 {
                    let mut game = setup_game(GameVariant::FreeForAll(n));
                    let player_index = 0;
                    game.players[player_index].cards = vec![Card::Ace, Card::King, Card::Joker];

                    let start = game.board.start_field(player_index) as usize;
                    let card = Card::Ace;
                    let action = Action {
                        player: game.players[player_index].color,
                        action: ActionKind::Place { target_player: player_index },
                        card: Some(card),
                    };

                    assert!(game.action(Some(card), action).is_err());
                    assert!(game.board.tiles[start].is_some());
                    assert!(game.players[player_index].cards.contains(&card));
                    assert!(!game.discard.contains(&card));
                }

                for variant in variants {
                    let mut game = setup_game(variant);
                    let player_index = 0;
                    game.players[player_index].cards = vec![Card::Ace, Card::King, Card::Joker];

                    let start = game.board.start_field(player_index) as usize;
                    let card = Card::Ace;
                    let action = Action {
                        player: game.players[player_index].color,
                        action: ActionKind::Place { target_player: player_index },
                        card: Some(card),
                    };

                    assert!(game.action(Some(card), action).is_ok());
                    assert!(game.board.tiles[start].is_some());
                    assert!(!game.players[player_index].cards.contains(&card));
                    assert!(game.discard.contains(&card));
                }
            }

            #[test]
            fn cannot_place_without_pieces_to_place_all_variants() {
                let variants = [
                    GameVariant::TwoVsTwo,
                    GameVariant::ThreeVsThree,
                    GameVariant::TwoVsTwoVsTwo,
                ];

                for n in 2..=6 {
                    let mut game = setup_game(GameVariant::FreeForAll(n));
                    let player_index = 0;
                    game.players[player_index].cards = vec![Card::Ace];
                    game.players[player_index].pieces_to_place = 0;

                    let action = Action {
                        player: game.players[player_index].color,
                        action: ActionKind::Place { target_player: player_index },
                        card: Some(Card::Ace),
                    };

                    assert!(game.action(Some(Card::Ace), action).is_err());
                }

                for variant in variants {
                    let mut game = setup_game(variant);
                    let player_index = 0;
                    game.players[player_index].cards = vec![Card::Ace];
                    game.players[player_index].pieces_to_place = 0;

                    let action = Action {
                        player: game.players[player_index].color,
                        action: ActionKind::Place { target_player: player_index },
                        card: Some(Card::Ace),
                    };

                    assert!(game.action(Some(Card::Ace), action).is_err());
                }
            }

            #[test]
            fn cannot_place_on_protected_piece_all_variants() {
                let variants = [
                    GameVariant::TwoVsTwo,
                    GameVariant::ThreeVsThree,
                    GameVariant::TwoVsTwoVsTwo,
                ];

                for n in 2..=6 {
                    let mut game = setup_game(GameVariant::FreeForAll(n));
                    let player_index = 0;
                    game.players[player_index].cards = vec![Card::Ace];

                    let start = game.board.start_field(player_index);
                    game.board.tiles[start] = Some(Piece {
                        owner: player_index,
                        left_start: false,
                    });

                    let action = Action {
                        player: game.players[player_index].color,
                        action: ActionKind::Place { target_player: player_index },
                        card: Some(Card::Ace),
                    };

                    assert!(game.action(Some(Card::Ace), action).is_err());
                    assert_eq!(game.board.tiles[start].as_ref().unwrap().owner, player_index);
                }

                for variant in variants {
                    let mut game = setup_game(variant);
                    let player_index = 0;
                    game.players[player_index].cards = vec![Card::Ace];

                    let start = game.board.start_field(player_index);
                    game.board.tiles[start] = Some(Piece {
                        owner: player_index,
                        left_start: false,
                    });

                    let action = Action {
                        player: game.players[player_index].color,
                        action: ActionKind::Place { target_player: player_index },
                        card: Some(Card::Ace),
                    };

                    assert!(game.action(Some(Card::Ace), action).is_err());
                    assert_eq!(game.board.tiles[start].as_ref().unwrap().owner, player_index);
                }
            }

            #[test]
            fn place_partner_piece_all_variants() {
                let variants = [
                    GameVariant::TwoVsTwo,
                    GameVariant::ThreeVsThree,
                    GameVariant::TwoVsTwoVsTwo,
                ];

                for variant in variants {
                    let mut game = setup_game(variant);
                    let player_index = 0;
                    game.players[player_index].pieces_in_house = 4;
                    game.players[player_index].cards = vec![Card::Ace, Card::Five];

                    if let Some(teammate_index) = game.teammate_index(player_index) {
                        game.players[teammate_index].pieces_to_place = 4;
                        let start = game.board.start_field(teammate_index);

                        let action = Action {
                            player: game.players[player_index].color,
                            action: ActionKind::Place { target_player: teammate_index },
                            card: Some(Card::Ace),
                        };

                        assert!(game.action(Some(Card::Ace), action).is_ok());
                        assert_eq!(game.board.tiles[start].as_ref().unwrap().owner, teammate_index);
                        assert_eq!(game.players[teammate_index].pieces_to_place, 3);
                        assert!(!game.players[player_index].cards.contains(&Card::Ace));
                    }
                }
            }

            #[test]
            fn invalid_card_cannot_place() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Ace, Card::King, Card::Joker];

                let invalid_card = Card::Two;
                let action = Action {
                    player: Color::Red,
                    action: ActionKind:: Place { target_player: 0 },
                    card: Some(invalid_card)
                };

                assert!(game.action(Some(Card::Two), action).is_err());
            }

            #[test]
            fn cannot_place_on_partner_protected_piece() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].pieces_in_house = 4;
                game.players[0].cards = vec![Card::Ace];

                let start = game.board.start_field(2);

                game.board.tiles[start] = Some(Piece {
                    owner: 2,
                    left_start: false,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind:: Place { target_player: 2 },
                    card: Some(Card::Ace),
                };

                assert!(game.action(Some(Card::Ace), action).is_err());
            }

            #[test]
            fn place_beat_opponent() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Ace, Card::King, Card::Joker];

                let start = game.board.start_field(0) as usize;
                let card = Some(Card::Ace);
                let action = Action {
                    player: Color::Red,
                    action: ActionKind:: Place { target_player: 0 },
                    card: Some(Card::Ace),
                };

                game.board.tiles[start] = Some(Piece {
                    owner: 1,
                    left_start: true
                });

                assert!(game.action(card, action).is_ok());
                assert_eq!(game.board.tiles[start].as_ref().unwrap().owner, 0);
            }

            #[test]
            fn invalid_place_does_not_change_state() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Ace];

                let start = game.board.start_field(0) as usize;

                // block start with own protected piece
                game.board.tiles[start] = Some(Piece {
                    owner: 0,
                    left_start: false,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind:: Place { target_player: 0 },
                    card: Some(Card::Ace),
                };

                let current_player = game.current_player_index;
                let cards_before = game.players[0].cards.clone();

                assert!(game.action(Some(Card::Ace), action).is_err());

                assert_eq!(game.current_player_index, current_player);
                assert_eq!(game.players[0].cards, cards_before);
                assert!(game.discard.is_empty());
            }
        }
        
        mod action_interchange_tests {
            use super::*;

            #[test]
            fn interchange_success_all_variants() {
                let variants = [
                    GameVariant::TwoVsTwo,
                    GameVariant::ThreeVsThree,
                    GameVariant::TwoVsTwoVsTwo,
                ];

                for variant in variants {
                    let mut game = setup_game(variant);
                    let player_index = 0;
                    let opponent_index = 1;

                    game.players[player_index].cards = vec![Card::Jack, Card::Joker];
                    game.players[opponent_index].cards = vec![Card::Jack, Card::Joker];

                    game.board.tiles[1] = Some(Piece { owner: player_index, left_start: true });
                    game.board.tiles[2] = Some(Piece { owner: opponent_index, left_start: true });

                    let action = Action {
                        player: game.players[player_index].color,
                        action: ActionKind::Interchange { a: 1, b: 2 },
                        card: Some(Card::Jack),
                    };

                    assert!(game.action(Some(Card::Jack), action).is_ok());
                    assert_eq!(game.board.tiles[1].as_ref().unwrap().owner, opponent_index);
                    assert_eq!(game.board.tiles[2].as_ref().unwrap().owner, player_index);
                    assert!(!game.players[player_index].cards.contains(&Card::Jack));
                    assert!(game.discard.contains(&Card::Jack));
                }

                // Free-for-All
                for n in 2..=6 {
                    let mut game = setup_game(GameVariant::FreeForAll(n));
                    let player_index = 0;
                    let opponent_index = 1;

                    game.players[player_index].cards = vec![Card::Jack];
                    game.players[opponent_index].cards = vec![Card::Jack];

                    game.board.tiles[0] = Some(Piece { owner: player_index, left_start: true });
                    game.board.tiles[1] = Some(Piece { owner: opponent_index, left_start: true });

                    let action = Action {
                        player: game.players[player_index].color,
                        action: ActionKind::Interchange { a: 0, b: 1 },
                        card: Some(Card::Jack),
                    };

                    assert!(game.action(Some(Card::Jack), action).is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, opponent_index);
                    assert_eq!(game.board.tiles[1].as_ref().unwrap().owner, player_index);
                }
            }

            #[test]
            fn cannot_interchange_invalid_or_empty_tile_all_variants() {
                let variants = [
                    GameVariant::TwoVsTwo,
                    GameVariant::ThreeVsThree,
                    GameVariant::TwoVsTwoVsTwo,
                ];

                for variant in variants {
                    let mut game = setup_game(variant);
                    let player_index = 0;

                    game.players[player_index].cards = vec![Card::Jack];

                    // Tile empty
                    let action = Action { 
                        player: game.players[player_index].color,
                        action: ActionKind::Interchange { a: 10, b: 11 },
                        card: Some(Card::Jack),
                    };
                    assert!(game.action(Some(Card::Jack), action).is_err());

                    // Invalid card
                    game.board.tiles[1] = Some(Piece { owner: player_index, left_start: true });
                    game.board.tiles[2] = Some(Piece { owner: 1, left_start: true });
                    let action2 = Action { 
                        player: game.players[player_index].color,
                        action: ActionKind::Interchange { a: 1, b: 2 },
                        card: Some(Card::Two),
                    };
                    assert!(game.action(Some(Card::Two), action2).is_err());
                }
            }

            #[test]
            fn invalid_card() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Jack, Card::Joker];
                game.players[1].cards = vec![Card::Jack, Card::Joker];

                game.board.tiles[1] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                game.board.tiles[2] = Some(Piece {
                    owner: 1,
                    left_start: true,
                });

                let action = Action { 
                    player: Color::Red,
                    action: ActionKind::Interchange { a: 1, b: 2 },
                    card: Some(Card::Two),
                };

                assert!(game.action(Some(Card::Two), action).is_err()); 
            }

            #[test]
            fn house_tile() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Jack, Card::Joker];
                game.players[1].cards = vec![Card::Jack, Card::Joker];

                game.board.tiles[64] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                game.board.tiles[2] = Some(Piece {
                    owner: 1,
                    left_start: true,
                });

                let action = Action { 
                    player: Color::Red,
                    action: ActionKind::Interchange { a: 64, b: 2 },
                    card: Some(Card::Jack),
                };

                assert!(game.action(Some(Card::Jack), action).is_err());
            }

            #[test]
            fn not_own_piece() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Jack, Card::Joker];
                game.players[1].cards = vec![Card::Jack, Card::Joker];

                game.board.tiles[1] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                game.board.tiles[2] = Some(Piece {
                    owner: 1,
                    left_start: true,
                });

                let action = Action { 
                    player: Color::Red,
                    action: ActionKind::Interchange { a: 2, b: 1 },
                    card: Some(Card::Jack),
                };

                assert!(game.action(Some(Card::Jack), action).is_err());
            }

            #[test]
            fn protected_piece() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Jack, Card::Joker];
                game.players[1].cards = vec![Card::Jack, Card::Joker];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: false,
                });

                game.board.tiles[2] = Some(Piece {
                    owner: 1,
                    left_start: true,
                });

                let action = Action { 
                    player: Color::Red,
                    action: ActionKind::Interchange { a: 0, b: 2 },
                    card: Some(Card::Jack),
                };

                assert!(game.action(Some(Card::Jack), action).is_err());
            }

            #[test]
            fn partner_interchange_all_variants() {
                let variants = [
                    GameVariant::TwoVsTwo,
                    GameVariant::ThreeVsThree,
                    GameVariant::TwoVsTwoVsTwo,
                ];

                for variant in variants {
                    let mut game = setup_game(variant);
                    let player_index = 0;

                    // Partner exists only if teams
                    if let Some(teammate_index) = game.teammate_index(player_index) {
                        game.players[player_index].pieces_in_house = 4;
                        game.players[player_index].cards = vec![Card::Jack];
                        game.board.tiles[1] = Some(Piece { owner: teammate_index, left_start: true });
                        game.board.tiles[2] = Some(Piece { owner: player_index, left_start: true });

                        let action = Action {
                            player: game.players[player_index].color,
                            action: ActionKind::Interchange { a: 1, b: 2 },
                            card: Some(Card::Jack),
                        };

                        assert!(game.action(Some(Card::Jack), action).is_ok());
                        assert_eq!(game.board.tiles[1].as_ref().unwrap().owner, player_index);
                        assert_eq!(game.board.tiles[2].as_ref().unwrap().owner, teammate_index);
                    }
                }
            }

            #[test]
            fn cannot_interchange_partner_if_less_than_4_in_house() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].pieces_in_house = 3;

                game.players[0].cards = vec![Card::Jack];
                game.players[2].cards = vec![Card::Jack];

                game.board.tiles[1] = Some(Piece { owner: 2, left_start: true });
                game.board.tiles[2] = Some(Piece { owner: 0, left_start: true });

                let action1 = Action {
                    player: Color::Red,
                    action: ActionKind::Interchange { a: 1, b: 2 },
                    card: Some(Card::Jack),
                };

                let action2 = Action {
                    player: Color::Red,
                    action: ActionKind::Interchange { a: 2, b: 1 },
                    card: Some(Card::Jack),
                };

                assert!(game.action(Some(Card::Jack), action1).is_err());
                assert!(game.action(Some(Card::Jack), action2).is_ok());
            }

            #[test]
            fn partner_piece_protected_cannot_interchange() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].pieces_in_house = 4;

                game.players[0].cards = vec![Card::Jack];
                game.players[2].cards = vec![Card::Jack];

                game.board.tiles[1] = Some(Piece { owner: 2, left_start: false });
                game.board.tiles[2] = Some(Piece { owner: 0, left_start: true });
                game.board.tiles[3] = Some(Piece { owner: 2, left_start: true });

                let action1 = Action {
                    player: Color::Red,
                    action: ActionKind::Interchange { a: 1, b: 2 },
                    card: Some(Card::Jack),
                };

                let action2 = Action {
                    player: Color::Red,
                    action: ActionKind::Interchange {a: 3, b: 2 },
                    card: Some(Card::Jack),
                };

                assert!(game.action(Some(Card::Jack), action1).is_err());
                assert!(game.action(Some(Card::Jack), action2).is_ok());
            }

        }
    
        mod action_move_tests {
            use super::*;

            #[test]
            fn valid_move_forward_all_variants() {
                let variants = [
                    GameVariant::TwoVsTwo,
                    GameVariant::ThreeVsThree,
                    GameVariant::TwoVsTwoVsTwo,
                ];

                for variant in variants {
                    let mut game = setup_game(variant);
                    let player_index = 0;

                    game.players[player_index].cards = vec![Card::Five];

                    // Ausgangsfigur
                    game.board.tiles[0] = Some(Piece {
                        owner: player_index,
                        left_start: false,
                    });

                    let action = Action {
                        player: game.players[player_index].color,
                        action: ActionKind::Move { from: 0, to: 5 },
                        card: Some(Card::Five),
                    };

                    assert!(game.action(Some(Card::Five), action).is_ok());
                    assert!(game.board.tiles[0].is_none());
                    assert_eq!(game.board.tiles[5].as_ref().unwrap().owner, player_index);
                }

                // Free-for-All
                for n in 2..=6 {
                    let mut game = setup_game(GameVariant::FreeForAll(n));
                    game.players[0].cards = vec![Card::Five];
                    game.board.tiles[0] = Some(Piece { owner: 0, left_start: false });

                    let action = Action {
                        player: game.players[0].color,
                        action: ActionKind::Move { from: 0, to: 5 },
                        card: Some(Card::Five),
                    };

                    assert!(game.action(Some(Card::Five), action).is_ok());
                    assert!(game.board.tiles[0].is_none());
                    assert_eq!(game.board.tiles[5].as_ref().unwrap().owner, 0);
                }
            }

            #[test]
            fn valid_move_into_house() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Five, Card::Ten];

                game.board.tiles[60] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 60, to: 64 },
                    card: Some(Card::Five),
                };

                assert!(game.action(Some(Card::Five), action).is_ok());
                assert!(game.board.tiles[60].is_none());
                assert_eq!(game.board.tiles[64].as_ref().unwrap().owner, 0);
                assert_eq!(game.players[0].pieces_in_house, 1);
            }

            #[test]
            fn valid_move_in_house() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Ace, Card::Ten];
                game.players[0].pieces_in_house = 1;

                game.board.tiles[64] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 64, to: 65 },
                    card: Some(Card::Ace),
                };

                assert!(game.action(Some(Card::Ace), action).is_ok());
                assert!(game.board.tiles[64].is_none());
                assert_eq!(game.board.tiles[65].as_ref().unwrap().owner, 0);
                assert_eq!(game.players[0].pieces_in_house, 1);
            }

            #[test]
            fn valid_move_backward() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Four, Card::Ten];

                game.board.tiles[10] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 10, to: 6 },
                    card: Some(Card::Four),
                };

                assert!(game.action(Some(Card::Four), action).is_ok());
                assert!(game.board.tiles[10].is_none());
                assert_eq!(game.board.tiles[6].as_ref().unwrap().owner, 0);
            }

            #[test]
            fn valid_move_backward_with_joker() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Joker, Card::Ten];

                game.board.tiles[10] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 10, to: 6 },
                    card: Some(Card::Joker),
                };

                assert!(game.action(Some(Card::Joker), action).is_ok());
                assert!(game.board.tiles[10].is_none());
                assert_eq!(game.board.tiles[6].as_ref().unwrap().owner, 0);
            }

            #[test]
            fn invalid_move_with_jack() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Jack, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 0, to: 5 },
                    card: Some(Card::Jack),
                };

                assert!(game.action(Some(Card::Jack), action).is_err());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                assert!(game.board.tiles[5].is_none());
            }

            #[test]
            fn invalid_move_into_house() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Two];

                // Piece hasn't left start yet
                game.board.tiles[0] = Some(Piece { 
                    owner: 0, 
                    left_start: false 
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 0, to: 64 },
                    card: Some(Card::Two),
                };

                assert!(game.action(Some(Card::Two), action).is_err());
            }

            #[test]
            fn invalid_move_past_protected_piece_all_variants() {
                let variants = [
                    GameVariant::TwoVsTwo,
                    GameVariant::ThreeVsThree,
                    GameVariant::TwoVsTwoVsTwo,
                ];

                for variant in variants {
                    let mut game = setup_game(variant);
                    let player_index = 0;

                    game.players[player_index].cards = vec![Card::Three];
                    game.board.tiles[0] = Some(Piece { owner: player_index, left_start: true });
                    game.board.tiles[3] = Some(Piece { owner: 1, left_start: false });

                    let action = Action {
                        player: game.players[player_index].color,
                        action: ActionKind::Move { from: 0, to: 5 },
                        card: Some(Card::Three),
                    };

                    assert!(game.action(Some(Card::Three), action).is_err());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, player_index);
                    assert_eq!(game.board.tiles[3].as_ref().unwrap().owner, 1);
                }
            }

            #[test]
            fn invalid_move_past_house_piece() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Five, Card::Ten];

                game.board.tiles[60] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                game.board.tiles[64] = Some(Piece {
                    owner: 1,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 60 , to: 65 },
                    card: Some(Card::Five),
                };

                assert!(game.action(Some(Card::Five), action).is_err());
                assert_eq!(game.board.tiles[60].as_ref().unwrap().owner, 0);
                assert_eq!(game.board.tiles[64].as_ref().unwrap().owner, 1);
                assert!(game.board.tiles[65].is_none());
            }

            #[test]
            fn invalid_move_not_own_piece() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Five, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 1,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 0, to: 5 },
                    card: Some(Card::Five),
                };

                assert!(game.action(Some(Card::Five), action).is_err());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 1);
                assert!(game.board.tiles[5].is_none());
            }

            #[test]
            fn invalid_move_not_allowed_by_card() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Three, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 0, to: 5 },
                    card: Some(Card::Three),
                };

                assert!(game.action(Some(Card::Three), action).is_err());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                assert!(game.board.tiles[5].is_none());
            }

            #[test]
            fn invalid_move_empty_from_tile() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Five, Card::Ten];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 0, to: 5 },
                    card: Some(Card::Five),
                };

                assert!(game.action(Some(Card::Five), action).is_err());
                assert!(game.board.tiles[0].is_none());
                assert!(game.board.tiles[5].is_none());
            }

            #[test]
            fn invalid_move_path_cannot_be_calculated() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Five, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                game.board.tiles[1] = Some(Piece {
                    owner: 1,
                    left_start: false,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 0, to: 5 },
                    card: Some(Card::Five),
                };

                assert!(game.action(Some(Card::Five), action).is_err());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                assert_eq!(game.board.tiles[1].as_ref().unwrap().owner, 1);
            }

            #[test]
            fn beat_opponent_piece() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Five, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                game.board.tiles[5] = Some(Piece {
                    owner: 1,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 0, to: 5 },
                    card: Some(Card::Five),
                };

                assert!(game.action(Some(Card::Five), action).is_ok());
                assert!(game.board.tiles[0].is_none());
                assert_eq!(game.board.tiles[5].as_ref().unwrap().owner, 0);
                assert_eq!(game.player_mut_by_color(Color::Green).pieces_to_place, 5);
            }

            #[test]
            fn move_partner_piece_all_variants() {
                let variants = [
                    GameVariant::TwoVsTwo,
                    GameVariant::ThreeVsThree,
                    GameVariant::TwoVsTwoVsTwo,
                ];

                for variant in variants {
                    let mut game = setup_game(variant);
                    let player_index = 0;

                    // partner only exists in team games
                    if let Some(teammate_index) = game.teammate_index(player_index) {
                        game.players[player_index].pieces_in_house = 4;
                        game.players[player_index].cards = vec![Card::Five];
                        game.board.tiles[0] = Some(Piece { owner: teammate_index, left_start: true });

                        let action = Action {
                            player: game.players[player_index].color,
                            action: ActionKind::Move { from: 0, to: 5 },
                            card: Some(Card::Five),
                        };

                        assert!(game.action(Some(Card::Five), action).is_ok());
                        assert!(game.board.tiles[0].is_none());
                        assert_eq!(game.board.tiles[5].as_ref().unwrap().owner, teammate_index);
                    }
                }
            }

            #[test]
            fn move_partner_piece_into_house() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;
                game.current_player_index = 2;

                game.players[2].pieces_in_house = 4;
                game.players[0].cards = vec![Card::Two];
                game.players[2].cards = vec![Card::Two];

                game.board.tiles[63] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Blue,
                    action: ActionKind::Move { from: 63, to: 64 },
                    card: Some(Card::Two),
                };

                assert!(game.action(Some(Card::Two), action).is_ok());
                assert!(game.board.tiles[63].is_none());
                assert_eq!(game.board.tiles[64].as_ref().unwrap().owner, 0);
                assert_eq!(game.player_by_color(Color::Red).pieces_in_house, 1);
            }

            #[test]
            fn cannot_move_partner_piece_if_not_in_house() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].pieces_in_house = 3;
                game.players[0].cards = vec![Card::Five];
                game.players[2].cards = vec![Card::Five];

                game.board.tiles[0] = Some(Piece {
                    owner: 2,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 0, to: 5 },
                    card: Some(Card::Five),
                };

                assert!(game.action(Some(Card::Five), action).is_err());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 2);
                assert!(game.board.tiles[5].is_none());
            }

            #[test]
            fn cannot_move_partner_piece_past_protected_piece() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].pieces_in_house = 4;
                game.players[0].cards = vec![Card::Five];
                game.players[2].cards = vec![Card::Five];

                game.board.tiles[0] = Some(Piece {
                    owner: 2,
                    left_start: true,
                });

                game.board.tiles[3] = Some(Piece {
                    owner: 0,
                    left_start: false,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Move { from: 0, to: 5 },
                    card: Some(Card::Five),
                };

                assert!(game.action(Some(Card::Five), action).is_err());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 2);
                assert_eq!(game.board.tiles[3].as_ref().unwrap().owner, 0);
                assert!(game.board.tiles[5].is_none());
            }
        }

        mod action_split_tests {
            use super::*;

            #[test]
            fn split_within_limits_all_variants() {
                let variants = [
                    GameVariant::TwoVsTwo,
                    GameVariant::ThreeVsThree,
                    GameVariant::TwoVsTwoVsTwo,
                ];

                for variant in variants {
                    let mut game = Game::new(variant);
                    game.trading_phase = false;

                    // Setze Karten und Ausgangsfiguren
                    game.players[0].cards = vec![Card::Seven, Card::Ten];

                    game.board.tiles[30] = Some(Piece { owner: 0, left_start: true });

                    let action1 = Action {
                        player: game.players[0].color,
                        action: ActionKind::Split { from: 30, to: 34 },
                        card: Some(Card::Seven),
                    };

                    assert!(game.action(Some(Card::Seven), action1).is_ok());
                    assert!(game.board.tiles[30].is_none());
                    assert_eq!(game.board.tiles[34].as_ref().unwrap().owner, 0);
                    assert_eq!(game.board.tiles[34].as_ref().unwrap().left_start, true);
                    assert_eq!(game.split_rest, Some(3));
                }
            }

            #[test]
            fn split_within_limits() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven, Card::Ten];

                game.board.tiles[63] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                game.board.tiles[4] = Some(Piece {
                    owner: 2,
                    left_start: false,
                });

                let action1 = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 63, to: 67 },
                    card: Some(Card::Seven),
                };

                let action2 = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 4, to: 6 },
                    card: Some(Card::Seven)
                };

                assert!(game.action(Some(Card::Seven), action1).is_ok());
                assert!(game.board.tiles[63].is_none());
                assert_eq!(game.board.tiles[67].as_ref().unwrap().owner, 0);
                assert_eq!(game.board.tiles[67].as_ref().unwrap().left_start, true);
                assert_eq!(game.players[0].pieces_in_house, 1);
                assert_eq!(game.split_rest, Some(2));

                let _ = game.action(Some(Card::Seven), action1);

                assert!(game.action(Some(Card::Seven), action2).is_ok());
                assert!(game.board.tiles[4].is_none());
                assert_eq!(game.board.tiles[6].as_ref().unwrap().owner, 2);
                assert_eq!(game.board.tiles[6].as_ref().unwrap().left_start, true);
                assert_eq!(game.players[0].pieces_in_house, 1);
                assert_eq!(game.split_rest, None);
                assert_eq!(game.current_player_index, 1);
            }

            #[test]
            fn split_outside_limits() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 10 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_err());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
            }

            #[test]
            fn split_with_joker() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Joker, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 5 },
                    card: Some(Card::Joker),
                };

                assert!(game.action(Some(Card::Joker), action).is_ok());
                assert!(game.board.tiles[0].is_none());
                assert_eq!(game.board.tiles[5].as_ref().unwrap().owner, 0);
                assert_eq!(game.split_rest, Some(2));
            }

            #[test]
            fn split_beaten_piece_correct_history() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[1].pieces_to_place = 3;

                game.players[0].cards = vec![Card::Seven, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: false,
                });

                game.board.tiles[3] = Some(Piece {
                    owner: 1,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 5 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_ok());
                assert!(game.board.tiles[0].is_none());
                assert_eq!(game.board.tiles[5].as_ref().unwrap().owner, 0);
                assert_eq!(game.player_mut_by_color(Color::Green).pieces_to_place, 4);

                let first_entry = &game.history[game.history.len() - 2];
                assert_eq!(first_entry.action.action, ActionKind::Split { from: 0, to: 3 });
                assert_eq!(first_entry.beaten_piece_owner, Some(1));
                assert_eq!(first_entry.left_start_before, false);

                let second_entry = &game.history[game.history.len() - 1];
                assert_eq!(second_entry.action.action, ActionKind::Split { from: 3, to: 5 });
                assert_eq!(second_entry.beaten_piece_owner, None);
                assert_eq!(second_entry.left_start_before, true);
            }

            #[test]
            fn split_complete_turn() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 7 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_ok());
                assert!(game.board.tiles[0].is_none());
                assert_eq!(game.board.tiles[7].as_ref().unwrap().owner, 0);
                assert_eq!(game.split_rest, None);
                assert_eq!(game.current_player_index, 1);
            }

            #[test]
            fn split_invalid_card() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Five, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 5 },
                    card: Some(Card::Five),
                };

                assert!(game.action(Some(Card::Five), action).is_err());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
            }

            #[test]
            fn split_not_own_piece() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 1,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 5 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_err());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 1);
            }

            #[test]
            fn split_path_blocked_by_protected_piece() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                game.board.tiles[3] = Some(Piece {
                    owner: 1,
                    left_start: false,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 5 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_err());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
            }

            #[test]
            fn split_path_blocked_by_house_piece() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven, Card::Ten];

                game.board.tiles[60] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                game.board.tiles[64] = Some(Piece {
                    owner: 1,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 60, to: 65 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_err());
                assert_eq!(game.board.tiles[60].as_ref().unwrap().owner, 0);
            }

            #[test]
            fn split_empty_tile() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven, Card::Ten];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 5 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_err());
                assert!(game.board.tiles[0].is_none());
            }

            #[test]
            fn split_multiple_times_within_limits() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let first_action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 4 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), first_action).is_ok());
                assert_eq!(game.split_rest, Some(3));

                let second_action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 4, to: 7 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), second_action).is_ok());
                assert_eq!(game.split_rest, None);
                assert_eq!(game.current_player_index, 1);
            }

            #[test]
            fn split_multiple_times_correct_history() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven, Card::Ten];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let first_action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 4 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), first_action).is_ok());
                assert_eq!(game.split_rest, Some(3));

                let second_action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 4, to: 7 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), second_action).is_ok());
                assert_eq!(game.split_rest, None);
                assert_eq!(game.current_player_index, 1);

                let first_entry = &game.history[game.history.len() - 2];
                assert_eq!(first_entry.action.action, ActionKind::Split { from: 0, to: 4 });

                let second_entry = &game.history[game.history.len() - 1];
                assert_eq!(second_entry.action.action, ActionKind::Split { from: 4, to: 7 });
            }
            
            #[test]
            fn split_can_move_partner_piece_without_pieces_in_house() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];
                game.players[0].pieces_in_house = 0;

                game.board.tiles[0] = Some(Piece {
                    owner: 2,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 3 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_ok());
                assert!(game.board.tiles[0].is_none());
                assert_eq!(game.board.tiles[3].as_ref().unwrap().owner, 2);
            }

            #[test]
            fn split_can_beat_partner_piece() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];
                game.players[2].pieces_to_place = 3;

                game.board.tiles[0] = Some(Piece { owner: 0, left_start: true });
                game.board.tiles[3] = Some(Piece { owner: 2, left_start: true });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 5 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_ok());
                assert_eq!(game.players[2].pieces_to_place, 4);
            }

            #[test]
            fn split_enter_house_only_counts_once() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];

                game.board.tiles[63] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 63, to: 66 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_ok());
                assert_eq!(game.players[0].pieces_in_house, 1);
            }

            #[test]
            fn split_cannot_enter_wrong_house() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];

                game.board.tiles[15] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });

                
                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 15, to: 68 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_err());
            }
        
            #[test]
            fn split_in_ffa_can_move_opponents_piece() {
                let mut game = Game::new_free_for_all(2);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];

                game.board.tiles[16] = Some(Piece {
                    owner: 1,
                    left_start: false,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 16, to: 23 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_ok());
                assert_eq!(game.board.tiles[23].unwrap().left_start, true);
            }

            #[test]
            fn split_in_ffa_can_move_opponents_piece_in_house() {
                let mut game = Game::new_free_for_all(2);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];

                game.board.tiles[16] = Some(Piece {
                    owner: 1,
                    left_start: true,
                });

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 16, to: 39 },
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action).is_ok());
            }
        }

        mod action_trade_tests {
            use std::vec;

            use super::*;

            #[test]
            fn trade_succeeds() {
                let mut game = Game::new(GameVariant::TwoVsTwo);

                game.players[0].cards = vec![Card::Five, Card::Ten];
                game.players[1].cards = vec![Card::Two, Card::Three];
                game.players[2].cards = vec![Card::Seven, Card::Eight];
                game.players[3].cards = vec![Card::Nine, Card::Ten];

                let action_red = Action {
                    player: Color::Red,
                    action: ActionKind::Trade,
                    card: Some(Card::Five),
                };

                assert!(game.action(Some(Card::Five), action_red).is_ok());
                assert_eq!(game.players[0].cards.len(), 1);
                assert_eq!(game.players[0].cards[0], Card::Ten);
                assert_eq!(game.trade_buffer.len(), 1);

                let action_green = Action {
                    player: Color::Green,
                    action: ActionKind::Trade,
                    card: Some(Card::Two),
                };

                assert!(game.action(Some(Card::Two), action_green).is_ok());
                assert_eq!(game.players[1].cards.len(), 1);
                assert_eq!(game.players[1].cards[0], Card::Three);
                assert_eq!(game.trade_buffer.len(), 2);

                let action_blue = Action {
                    player: Color::Blue,
                    action: ActionKind::Trade,
                    card: Some(Card::Seven),
                };

                assert!(game.action(Some(Card::Seven), action_blue).is_ok());
                assert_eq!(game.players[2].cards.len(), 1);
                assert_eq!(game.players[2].cards[0], Card::Eight);
                assert_eq!(game.trade_buffer.len(), 3);

                let action_yellow = Action {
                    player: Color::Yellow,
                    action: ActionKind::Trade,
                    card: Some(Card::Nine),
                };

                assert!(game.action(Some(Card::Nine), action_yellow).is_ok());

                // Swap buffer is emptied and players get cards
                assert!(game.trade_buffer.is_empty());
                assert!(!game.trading_phase);

                assert_eq!(game.players[0].cards.len(), 2);
                assert!(game.players[0].cards.contains(&Card::Seven));

                assert_eq!(game.players[1].cards.len(), 2);
                assert!(game.players[1].cards.contains(&Card::Nine));

                assert_eq!(game.players[2].cards.len(), 2);
                assert!(game.players[2].cards.contains(&Card::Five));

                assert_eq!(game.players[3].cards.len(), 2);
                assert!(game.players[3].cards.contains(&Card::Two));
            }

            #[test]
            fn trade_outside_trading_phase_fails() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Trade,
                    card: Some(Card::Five),
                };

                assert!(game.action(Some(Card::Five), action).is_err());
            }

            #[test]
            fn trade_duplicate_card_fails() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                
                game.players[0].cards = vec![Card::Five, Card::Ten];

                let action1 = Action {
                    player: Color::Red,
                    action: ActionKind::Trade,
                    card: Some(Card::Five),
                };
                let action2 = Action {
                    player: Color::Red,
                    action: ActionKind::Trade,
                    card: Some(Card::Ten),
                };

                assert!(game.action(Some(Card::Five), action1).is_ok());
                assert!(game.action(Some(Card::Ten), action2).is_err());
            }
        }
    
        mod action_remove_tests {
            use super::*;

            #[test]
            fn remove_card_success() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Two, Card::Five];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Remove,
                    card: Some(Card::Two),
                };

                assert!(game.action(Some(Card::Two), action).is_ok());

                assert!(!game.players[0].cards.contains(&Card::Two));

                assert!(game.players[0].cards.contains(&Card::Five));
            
                assert!(game.discard.contains(&Card::Two));

                assert_eq!(game.current_player_index, 1);
            }

            #[test]
            fn remove_card_not_in_hand() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Remove,
                    card: Some(Card::Two),
                };

                assert!(game.action(Some(Card::Two), action).is_err());
            }

            #[test]
            fn cannot_remove_during_trading_phase() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = true;

                game.players[0].cards = vec![Card::Ace];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Remove,
                    card: Some(Card::Ace),
                };

                assert!(game.action(Some(Card::Ace), action).is_err());
            }

            #[test]
            fn invalid_remove_creates_no_history_entry() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Five];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Remove,
                    card: Some(Card::Ace),
                };

                let history_len = game.history.len();

                assert!(game.action(Some(Card::Ace), action).is_err());
                assert_eq!(game.history.len(), history_len);
            }
        }
        

        mod action_grab_tests {
            use super::*;

            #[test]
            fn grab_two_card_success() {
                let mut game = Game::new(GameVariant::FreeForAll(3));
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Two];
                game.players[1].cards = vec![Card::Ace, Card::Five];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind::Grab {
                        target_player: Color::Green,
                        target_card: 0,
                    },
                    card: Some(Card::Two),
                };

                assert!(game.action(Some(Card::Two), action).is_ok());

                assert_eq!(game.players[1].cards, vec![Card::Five]);

                assert_eq!(game.players[0].cards, vec![Card::Ace]);

                let entry = game.history.last().unwrap();
                assert_eq!(entry.grabbed_from_player, Some(1));
                assert_eq!(entry.grabbed_card, Some(Card::Ace));
                assert_eq!(entry.grabbed_card_index, Some(0));

                assert_eq!(game.current_player_index, 1);
            }

            #[test]
            fn grab_not_allowed_in_team_game() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Two];
                game.players[1].cards = vec![Card::Ace];

                let action = Action {
                    player: game.players[0].color,
                    action: ActionKind::Grab {
                        target_player: game.players[1].color,
                        target_card: 0,
                    },
                    card: Some(Card::Two),
                };

                assert!(game.action(Some(Card::Two), action).is_err());
            }

            #[test]
            fn grab_not_allowed_with_other_card() {
                let mut game = Game::new(GameVariant::FreeForAll(3));
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Three];
                game.players[1].cards = vec![Card::Ace];

                let action = Action {
                    player: game.players[0].color,
                    action: ActionKind::Grab {
                        target_player: game.players[1].color,
                        target_card: 0,
                    },
                    card: Some(Card::Three),
                };

                assert!(game.action(Some(Card::Three), action).is_err());
            }

            #[test]
            fn grab_invalid_card_index_fails() {
                let mut game = Game::new(GameVariant::FreeForAll(3));
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Two];
                game.players[1].cards = vec![Card::Ace];

                let action = Action {
                    player: game.players[0].color,
                    action: ActionKind::Grab {
                        target_player: game.players[1].color,
                        target_card: 1, // not in scope
                    },
                    card: Some(Card::Two),
                };

                assert!(game.action(Some(Card::Two), action).is_err());

                assert_eq!(game.players[1].cards, vec![Card::Ace]);
                assert_eq!(game.players[0].cards, vec![Card::Two]);
                assert!(game.history.is_empty());
            }

            #[test]
            fn grab_does_not_modify_board_or_pieces() {
                let mut game = Game::new(GameVariant::FreeForAll(3));
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Two];
                game.players[1].cards = vec![Card::Ace];

                let board_before = game.board.tiles.clone();

                let action = Action {
                    player: game.players[0].color,
                    action: ActionKind::Grab {
                        target_player: game.players[1].color,
                        target_card: 0,
                    },
                    card: Some(Card::Two),
                };

                let _ = game.action(Some(Card::Two), action);

                assert_eq!(game.board.tiles, board_before);
            }
        }
    
        mod action_trade_grab_tests {
            use super::*;

            #[test]
            fn trade_grab_successful() {
                let mut game = Game::new_free_for_all(3);
                
                game.players[0].cards = vec![Card::Ace];
                game.players[1].cards = vec![Card::Two];
                game.players[2].cards = vec![Card::Three, Card::Four];

                let action = Action {
                    player: game.players[0].color,
                    card: None,
                    action: ActionKind::TradeGrab { target_card: 0 } // Three of player 3
                };

                assert!(game.action(None, action).is_ok());

                assert_eq!(game.players[2].cards.len(), 1);
                assert!(!game.players[2].cards.contains(&Card::Three));
                assert_eq!(game.trade_buffer, [(2, 0, Card::Three)]);
            }

            #[test]
            fn trade_grab_full_round_successful() {
                let mut game = Game::new_free_for_all(2);
                
                game.players[0].cards = vec![Card::Ace, Card::Two];
                game.players[1].cards = vec![Card::Three, Card::Four];

                let action1 = Action {
                    player: game.players[0].color,
                    card: None,
                    action: ActionKind::TradeGrab { target_card: 0 }
                };

                let action2 = Action {
                    player: game.players[1].color,
                    card: None,
                    action: ActionKind::TradeGrab { target_card: 1 }
                };
                
                // First trade grab
                assert!(game.action(None, action1).is_ok());

                assert_eq!(game.players[1].cards.len(), 1);
                assert_eq!(game.players[0].cards.len(), 2);
                assert!(!game.players[1].cards.contains(&Card::Three));
                assert_eq!(game.trade_buffer, [(1, 0, Card::Three)]);

                // Second trade grab + exchange cards
                assert!(game.action(None, action2).is_ok());
                assert_eq!(game.players[1].cards.len(), 2);
                assert_eq!(game.players[0].cards.len(), 2);
                assert!(game.players[0].cards.contains(&Card::Ace));
                assert!(game.players[0].cards.contains(&Card::Three));
                assert!(game.players[1].cards.contains(&Card::Two));
                assert!(game.players[1].cards.contains(&Card::Four));
                assert!(!game.trading_phase);
            }

            #[test]
            fn trade_grab_fails_outside_trading_phase() {
                let mut game = Game::new(GameVariant::FreeForAll(3));
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Ace];
                game.players[2].cards = vec![Card::Seven];

                let action = Action {
                    player: game.players[0].color,
                    card: None,
                    action: ActionKind::TradeGrab { target_card: 0 },
                };

                assert!(game.action_trade_grab(0, action).is_err());
            }

            #[test]
            fn trade_grab_fails_if_card_is_present() {
                let mut game = Game::new(GameVariant::FreeForAll(3));
                game.trading_phase = true;

                game.players[0].cards = vec![Card::Ace];
                game.players[2].cards = vec![Card::Seven];

                let action = Action {
                    player: game.players[0].color,
                    card: Some(Card::Ace),
                    action: ActionKind::TradeGrab { target_card: 0 },
                };

                assert!(game.action_trade_grab(0, action).is_err());
            }

            #[test]
            fn trade_grab_not_allowed_in_team_games() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = true;

                game.players[0].cards = vec![Card::Ace];
                game.players[1].cards = vec![Card::King];

                let action = Action {
                    player: game.players[0].color,
                    card: None,
                    action: ActionKind::TradeGrab { target_card: 0 },
                };

                assert!(game.action_trade_grab(0, action).is_err());
            }

            #[test]
            fn trade_grab_fails_with_invalid_card_index() {
                let mut game = Game::new(GameVariant::FreeForAll(3));
                game.trading_phase = true;

                game.players[0].cards = vec![Card::Ace];
                game.players[2].cards = vec![Card::Seven];

                let action = Action {
                    player: game.players[0].color,
                    card: None,
                    action: ActionKind::TradeGrab { target_card: 5 },
                };

                assert!(game.action_trade_grab(0, action).is_err());
            }

            #[test]
            fn trade_grab_creates_correct_history_entry() {
                let mut game = Game::new(GameVariant::FreeForAll(3));
                game.trading_phase = true;

                game.players[0].cards = vec![Card::Ace];
                game.players[2].cards = vec![Card::Seven];

                let action = Action {
                    player: game.players[0].color,
                    card: None,
                    action: ActionKind::TradeGrab { target_card: 0 },
                };

                game.action_trade_grab(0, action).unwrap();

                let entry = game.history.last().unwrap();

                assert_eq!(entry.grabbed_from_player, Some(2));
                assert_eq!(entry.grabbed_card, Some(Card::Seven));
                assert_eq!(entry.grabbed_card_index, Some(0));
                assert_eq!(entry.trade_buffer_before.len(), 0);
            }
        }
    }    

    mod undo_tests {
        use super::*;

        mod undo_action_tests {
            use super::*;

            #[test]
            fn undo_move_with_empty_history_fails() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                assert!(game.undo_action().is_err());
            }

            mod undo_place_tests {
                use super::*;

                #[test]
                fn undo_place_piece() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Ace, Card::Five];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind:: Place { target_player: 0 },
                        card: Some(Card::Ace),
                    };

                    assert!(game.action(Some(Card::Ace), action).is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                    assert_eq!(game.players[0].pieces_to_place, 3);
                    assert!(!game.players[0].cards.contains(&Card::Ace));

                    assert!(game.undo_action().is_ok());
                    assert!(game.board.tiles[0].is_none());
                    assert_eq!(game.players[0].pieces_to_place, 4);
                    assert!(game.players[0].cards.contains(&Card::Ace));
                }

                #[test]
                fn undo_place_restores_current_player() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Ace, Card::Five];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind:: Place { target_player: 0 },
                        card: Some(Card::Ace),
                    };

                    assert!(game.action(Some(Card::Ace), action).is_ok());
                    assert_ne!(game.current_player_index, 0);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.current_player_index, 0);
                }

                #[test]
                fn undo_place_restores_left_start_flag() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;
                    game.players[0].cards = vec![Card::Ace];

                    let start = game.board.start_field(0) as usize;

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Place { target_player: 0 },
                        card: Some(Card::Ace),
                    };

                    assert!(game.action(Some(Card::Ace), action).is_ok());
                    assert_eq!(game.board.tiles[start].as_ref().unwrap().left_start, false);

                    assert!(game.undo_action().is_ok());
                    assert!(game.board.tiles[start].is_none());
                }

                #[test]
                fn invalid_place_creates_no_history_entry() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    // Card not in player's hand
                    let action = Action {
                        player: Color::Red,
                        action: ActionKind:: Place { target_player: 0 },
                        card: Some(Card::Ace),
                    };

                    let history_len = game.history.len();

                    assert!(game.action(Some(Card::Ace), action).is_err());
                    assert_eq!(game.history.len(), history_len);
                }

                #[test]
                fn undo_place_removes_card_from_discard() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;
                    game.players[0].cards = vec![Card::Ace, Card::Two];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Place { target_player: 0 },
                        card: Some(Card::Ace),
                    };

                    assert!(game.action(Some(Card::Ace), action).is_ok());
                    assert!(game.discard.contains(&Card::Ace));

                    assert!(game.undo_action().is_ok());
                    assert!(!game.discard.contains(&Card::Ace));
                }

                #[test]
                fn undo_place_teammate_piece() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].pieces_in_house = 4;
                    game.players[0].cards = vec![Card::Ace, Card::Two];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind:: Place { target_player: 2 },
                        card: Some(Card::Ace),
                    };

                    assert!(game.action(Some(Card::Ace), action).is_ok());
                    assert_eq!(game.board.tiles[32].as_ref().unwrap().owner, 2);
                    assert!(game.board.tiles[0].is_none());
                    assert_eq!(game.players[2].pieces_to_place, 3);
                    assert!(!game.players[0].cards.contains(&Card::Ace));

                    assert!(game.undo_action().is_ok());
                    assert!(game.board.tiles[32].is_none());
                    assert_eq!(game.players[2].pieces_to_place, 4);
                    assert!(game.players[0].cards.contains(&Card::Ace));

                }

                #[test]
                fn undo_place_with_beaten_piece() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Ace, Card::Five];

                    game.board.tiles[0] = Some(Piece { 
                        owner: 1, 
                        left_start: true,
                    });

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind:: Place { target_player: 0 },
                        card: Some(Card::Ace),
                    };

                    assert!(game.action(Some(Card::Ace), action).is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                    assert_eq!(game.players[0].pieces_to_place, 3);
                    assert!(!game.players[0].cards.contains(&Card::Ace));

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 1);
                    assert_eq!(game.players[0].pieces_to_place, 4);
                    assert!(game.players[0].cards.contains(&Card::Ace));
                }

                #[test]
                fn undo_place_3v3() {
                    let mut game = Game::new(GameVariant::ThreeVsThree);
                    game.trading_phase = false;
                    game.current_player_index = 4;

                    game.players[4].cards = vec![Card::Ace, Card::Two];

                    let start = game.board.start_field(4);

                    let action = Action {
                        player: game.players[4].color,
                        action: ActionKind::Place { target_player: 4 },
                        card: Some(Card::Ace),
                    };

                    assert!(game.action(Some(Card::Ace), action).is_ok());
                    assert!(game.undo_action().is_ok());

                    assert!(game.board.tiles[start].is_none());
                    assert!(game.players[4].cards.contains(&Card::Ace));
                }

                #[test]
                fn undo_place_3v3_partner_piece() {
                    let mut game = Game::new(GameVariant::ThreeVsThree);
                    game.trading_phase = false;
                    game.current_player_index = 4;

                    game.players[4].cards = vec![Card::Ace, Card::Two];
                    game.players[4].pieces_in_house = 4;

                    let start = game.board.start_field(2);

                    let action = Action {
                        player: game.players[4].color,
                        action: ActionKind::Place { target_player: 2 },
                        card: Some(Card::Ace),
                    };

                    assert!(game.action(Some(Card::Ace), action).is_ok());
                    assert_eq!(game.board.tiles[start].unwrap().owner, 2);
                    
                    assert!(game.undo_action().is_ok());
                    assert!(game.board.tiles[start].is_none());
                    assert!(game.players[4].cards.contains(&Card::Ace));
                }
            }
        
            mod undo_move_tests {
                use super::*;

                #[test]
                fn undo_move_piece() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Five, Card::Six];
                    game.board.tiles[0] = Some(Piece { 
                        owner: 0, 
                        left_start: false, 
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Some(Card::Five),
                        action: ActionKind::Move { from: 0, to: 5 },
                    };

                    assert!(game.action(Some(Card::Five), action).is_ok());
                    assert_eq!(game.board.tiles[5].as_ref().unwrap().owner, 0);
                    assert_eq!(game.board.tiles[5].as_ref().unwrap().left_start, true);
                    assert!(!game.players[0].cards.contains(&Card::Five));

                    assert!(game.undo_action().is_ok());
                    assert!(game.board.tiles[5].is_none());
                    assert!(game.players[0].cards.contains(&Card::Five));
                    assert_eq!(game.board.check_tile(0).unwrap().left_start, false);
                }

                #[test]
                fn double_undo_move_into_and_in_house() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Two, Card::Ace];
                    game.board.tiles[0] = Some(Piece { 
                        owner: 0, 
                        left_start: true 
                    });

                    let action1 = Action {
                        player: Color::Red,
                        card: Some(Card::Two),
                        action: ActionKind::Move { from: 0,to: 65 },
                    };

                    let action2 = Action {
                        player: Color::Red,
                        card: Some(Card::Ace),
                        action: ActionKind::Move { from: 65,to: 66 }
                    };

                    assert!(game.action(Some(Card::Two), action1).is_ok());
                    assert_eq!(game.players[0].pieces_in_house, 1);

                    game.current_player_index = 0;
                    assert!(game.action(Some(Card::Ace), action2).is_ok());

                    // First undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.board.tiles[65].as_ref().unwrap().owner, 0);
                    assert!(game.board.tiles[66].is_none());
                    assert_eq!(game.players[0].pieces_in_house, 1);

                    // Second undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                    assert!(game.board.tiles[65].is_none());
                    assert_eq!(game.players[0].pieces_in_house, 0);
                }

                #[test]
                fn undo_move_into_house_restores_card() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Two, Card::Three];

                    game.board.tiles[0] = Some(Piece {
                        owner: 0,
                        left_start: true,
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Some(Card::Two),
                        action: ActionKind::Move { from: 0, to: 65 },
                    };

                    assert!(game.action(Some(Card::Two), action).is_ok());
                    assert!(!game.players[0].cards.contains(&Card::Two));

                    assert!(game.undo_action().is_ok());
                    assert!(game.players[0].cards.contains(&Card::Two));
                    assert_eq!(game.players[0].pieces_in_house, 0);
                }

                #[test]
                fn undo_move_beating_opponent() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Two, Card:: Three];

                    game.board.tiles[0] = Some(Piece { 
                        owner: 0, 
                        left_start: false 
                    });

                    game.board.tiles[2] = Some(Piece { 
                        owner: 1, 
                        left_start: true 
                    });
                    
                    let _action = Action {
                        player: Color::Red,
                        card: Some(Card::Two),
                        action: ActionKind::Move { from: 0, to: 2},
                    };

                    assert!(game.action(Some(Card::Two), _action).is_ok());

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.board.tiles[2].as_ref().unwrap().owner, 1);
                    assert_eq!(game.board.tiles[2].as_ref().unwrap().left_start, true);

                    assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().left_start, false);
                }

                #[test]
                fn undo_move_teammate_piece_into_house() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;
                    game.current_player_index = 2;

                    game.players[2].cards = vec![Card::Two, Card::Ace];
                    game.players[2].pieces_in_house = 4;


                    game.board.tiles[0] = Some(Piece { 
                        owner: 0, 
                        left_start: true 
                    });

                    let action = Action {
                        player: Color::Blue,
                        card: Some(Card::Two),
                        action: ActionKind::Move { from: 0, to: 65 },
                    };

                    assert!(game.action(Some(Card::Two), action).is_ok());

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.players[0].pieces_in_house, 0);
                    assert!(game.players[2].cards.contains(&Card::Two));
                }

                #[test]
                fn undo_move_restores_current_player() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Five, Card::Six];

                    game.board.tiles[0] = Some(Piece {
                        owner: 0,
                        left_start: false,
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Some(Card::Five),
                        action: ActionKind::Move { from: 0, to: 5 },
                    };

                    assert!(game.action(Some(Card::Five), action).is_ok());
                    assert_ne!(game.current_player_index, 0);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.current_player_index, 0);
                }

                #[test]
                fn invalid_move_creates_no_history_entry() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Five];

                    // Kein Piece auf Feld 0
                    let action = Action {
                        player: Color::Red,
                        card: Some(Card::Five),
                        action: ActionKind::Move { from: 0, to: 5 },
                    };

                    let history_len = game.history.len();

                    assert!(game.action(Some(Card::Five), action).is_err());
                    assert_eq!(game.history.len(), history_len);
                }
            }

            mod undo_split_tests {
                use super::*;

                #[test]
                fn undo_split_piece() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Seven, Card::Eight];

                    game.board.tiles[0] = Some(Piece { 
                        owner: 0, 
                        left_start: false, 
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Some(Card::Seven),
                        action: ActionKind::Split { from: 0, to: 5 },
                    };

                    assert!(game.action(Some(Card::Seven), action).is_ok());
                    assert_eq!(game.split_rest, Some(2));
                    assert_eq!(game.current_player_index, 0);
                    assert_eq!(game.history.last().unwrap().split_rest_before, None);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.split_rest, None);
                    assert!(game.board.tiles[5].is_none());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                }

                #[test]
                fn undo_split_into_house() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Seven, Card::Eight];
                    game.players[0].pieces_in_house = 0;

                    game.board.tiles[63] = Some(Piece {
                        owner: 0,
                        left_start: true,
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Some(Card::Seven),
                        action: ActionKind::Split { from: 63, to: 65 },
                    };

                    assert!(game.action(Some(Card::Seven), action).is_ok());
                    assert_eq!(game.players[0].pieces_in_house, 1);
                    assert_eq!(game.split_rest, Some(4));

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.players[0].pieces_in_house, 0);
                    assert_eq!(game.board.tiles[63].as_ref().unwrap().owner, 0);
                    assert_eq!(game.split_rest, None);
                }

                #[test]
                fn undo_split_beating_opponent() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Seven, Card::Eight];

                    game.board.tiles[0] = Some(Piece { 
                        owner: 0, 
                        left_start: false, 
                    });

                    game.players[1].pieces_to_place = 3;
                    game.board.tiles[3] = Some(Piece { 
                        owner: 1, 
                        left_start: true, 
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Some(Card::Seven),
                        action: ActionKind::Split { from: 0, to: 5 },
                    };

                    assert!(game.action(Some(Card::Seven), action).is_ok());
                    assert_eq!(game.split_rest, Some(2));
                    assert_eq!(game.players[1].pieces_to_place, 4);

                    // First undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.split_rest, Some(4));
                    assert!(game.board.tiles[5].is_none());
                    assert_eq!(game.board.tiles[3].as_ref().unwrap().owner, 0);
                    assert!(game.board.tiles[0].is_none());

                    // Second undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.split_rest, None);
                    assert_eq!(game.board.tiles[3].as_ref().unwrap().owner, 1);
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                }

                #[test]
                fn undo_split_teammate() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Seven, Card::Eight];

                    game.board.tiles[0] = Some(Piece { 
                        owner: 0, 
                        left_start: false, 
                    });

                    game.board.tiles[7] = Some(Piece { 
                        owner: 2, 
                        left_start: true 
                    });

                    let action1 = Action {
                        player: Color::Red,
                        card: Some(Card::Seven),
                        action: ActionKind::Split { from: 0, to: 5 },
                    };

                    let action2 = Action {
                        player: Color::Red,
                        card: Some(Card::Seven),
                        action: ActionKind::Split { from: 7, to: 9 },
                    };

                    assert!(game.action(Some(Card::Seven), action1).is_ok());
                    assert_eq!(game.split_rest, Some(2));

                    assert!(game.action(Some(Card::Seven), action2).is_ok());
                    assert_eq!(game.split_rest, None);
                    assert_eq!(game.current_player_index, 1);

                    // First undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.split_rest, Some(2));
                    assert!(game.board.tiles[9].is_none());
                    assert_eq!(game.board.tiles[7].as_ref().unwrap().owner, 2);

                    // Second undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.split_rest, None);
                    assert!(game.board.tiles[5].is_none());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                }

                #[test]
                fn undo_split_restores_joker_card(){
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Joker, Card::Ace];

                    game.board.tiles[0] = Some(Piece {
                        owner: 0,
                        left_start: false,
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Some(Card::Joker),
                        action: ActionKind::Split { from: 0, to: 7 },
                    };

                    assert!(game.action(Some(Card::Joker), action).is_ok());
                    assert!(!game.players[0].cards.contains(&Card::Joker));
                    assert_eq!(game.split_rest, None);

                    assert!(game.undo_action().is_ok());
                    assert!(game.players[0].cards.contains(&Card::Joker));
                    assert_eq!(game.split_rest, None);
                }

                #[test]
                fn undo_split_does_not_change_player() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Seven, Card::Eight];

                    game.board.tiles[0] = Some(Piece {
                        owner: 0,
                        left_start: false,
                    });

                    let action = Action {
                        player: Color::Red,
                        card: Some(Card::Seven),
                        action: ActionKind::Split { from: 0, to: 5 },
                    };

                    assert!(game.action(Some(Card::Seven), action).is_ok());
                    assert_eq!(game.current_player_index, 0);
                    assert_eq!(game.split_rest, Some(2));

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.current_player_index, 0);
                    assert_eq!(game.split_rest, None);
                }

                #[test]
                fn invalid_split_creates_no_history_entry() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Seven];

                    let action = Action {
                        player: Color::Red,
                        card: Some(Card::Seven),
                        action: ActionKind::Split { from: 0, to: 5 },
                    };

                    let history_len = game.history.len();

                    assert!(game.action(Some(Card::Seven), action).is_err());
                    assert_eq!(game.history.len(), history_len);
                }

            }
        
            mod undo_remove_tests {
                use super::*;

                #[test]
                fn undo_remove_card() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Two, Card::Five];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Remove,
                        card: Some(Card::Two),
                    };

                    assert!(game.action(Some(Card::Two), action).is_ok());
                    assert!(!game.players[0].cards.contains(&Card::Two));
                    assert!(game.discard.contains(&Card::Two));

                    assert!(game.undo_action().is_ok());
                    assert!(game.players[0].cards.contains(&Card::Two));
                    assert!(!game.discard.contains(&Card::Two));
                }

                #[test]
                fn undo_remove_restores_current_player() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Two, Card::Three];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Remove,
                        card: Some(Card::Two),
                    };

                    let current_player = game.current_player_index;

                    assert!(game.action(Some(Card::Two), action).is_ok());
                    assert_ne!(game.current_player_index, current_player);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.current_player_index, current_player);
                }
            }
        
            mod undo_interchange_tests {
                use super::*;

                #[test]
                fn undo_interchange_successful() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Jack];

                    game.board.tiles[0] = Some(Piece { 
                        owner: 0, 
                        left_start: true 
                    });

                    game.board.tiles[1] = Some(Piece { 
                        owner: 1, 
                        left_start: true 
                    });

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Interchange { a: 0, b: 1},
                        card: Some(Card::Jack),
                    };

                    assert!(game.action(Some(Card::Jack), action).is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 1);
                    assert_eq!(game.board.tiles[1].as_ref().unwrap().owner, 0);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                    assert_eq!(game.board.tiles[1].as_ref().unwrap().owner, 1);
                    assert!(game.players[0].cards.contains(&Card::Jack));
                    assert!(!game.discard.contains(&Card::Jack));
                    assert_eq!(game.current_player_index, 0);
                }
            }

            mod undo_trade_tests {
                use super::*;

                #[test]
                fn undo_trade_basic() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = true;

                    game.players[0].cards = vec![Card::Five, Card::Ten];
                    game.players[1].cards = vec![Card::Two, Card::Three];
                    game.players[2].cards = vec![Card::Seven, Card::Eight];
                    game.players[3].cards = vec![Card::Nine, Card::Ten];

                    let action_red = Action {
                        player: Color::Red,
                        action: ActionKind::Trade,
                        card: Some(Card::Five),
                    };

                    assert!(game.action(Some(Card::Five), action_red).is_ok());
                    assert_eq!(game.players[0].cards.len(), 1);
                    assert_eq!(game.players[0].cards[0], Card::Ten);
                    assert_eq!(game.trade_buffer.len(), 1);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.players[0].cards.len(), 2);
                    assert!(game.players[0].cards.contains(&Card::Five));
                    assert!(game.players[0].cards.contains(&Card::Ten));
                    assert_eq!(game.trade_buffer.len(), 0);
                }

                #[test]
                fn undo_trade_full_trade_phase() {
                    let mut game = Game::new(GameVariant::TwoVsTwo);
                    game.trading_phase = true;

                    game.players[0].cards = vec![Card::Five, Card::Ten];
                    game.players[1].cards = vec![Card::Two, Card::Three];
                    game.players[2].cards = vec![Card::Seven, Card::Eight];
                    game.players[3].cards = vec![Card::Nine, Card::Ten];

                    let action_red = Action {
                        player: Color::Red,
                        action: ActionKind::Trade,
                        card: Some(Card::Five),
                    };

                    assert!(game.action(Some(Card::Five), action_red).is_ok());
                    assert_eq!(game.players[0].cards.len(), 1);
                    assert_eq!(game.players[0].cards[0], Card::Ten);
                    assert_eq!(game.trade_buffer.len(), 1);

                    let action_green = Action {
                        player: Color::Green,
                        action: ActionKind::Trade,
                        card: Some(Card::Two),
                    };

                    assert!(game.action(Some(Card::Two), action_green).is_ok());
                    assert_eq!(game.players[1].cards.len(), 1);
                    assert_eq!(game.players[1].cards[0], Card::Three);
                    assert_eq!(game.trade_buffer.len(), 2);

                    let action_blue = Action {
                        player: Color::Blue,
                        action: ActionKind::Trade,
                        card: Some(Card::Seven),
                    };

                    assert!(game.action(Some(Card::Seven), action_blue).is_ok());
                    assert_eq!(game.players[2].cards.len(), 1);
                    assert_eq!(game.players[2].cards[0], Card::Eight);
                    assert_eq!(game.trade_buffer.len(), 3);

                    let action_yellow = Action {
                        player: Color::Yellow,
                        action: ActionKind::Trade,
                        card: Some(Card::Nine),
                    };

                    assert!(game.action(Some(Card::Nine), action_yellow).is_ok());
                    assert_eq!(game.players[3].cards.len(), 2);
                    assert_eq!(game.players[3].cards[0], Card::Ten);

                    // Swap buffer is emptied and players get cards
                    assert!(game.trade_buffer.is_empty());
                    assert!(!game.trading_phase);
                    assert_eq!(game.players[0].cards.len(), 2);
                    assert_eq!(game.players[1].cards.len(), 2);
                    assert_eq!(game.players[2].cards.len(), 2);
                    assert_eq!(game.players[3].cards.len(), 2);
                    assert!(game.players[0].cards.contains(&Card::Seven));
                    assert!(game.players[1].cards.contains(&Card::Nine));
                    assert!(game.players[2].cards.contains(&Card::Five));
                    assert!(game.players[3].cards.contains(&Card::Two));

                    // Undo last trade (yellow)
                    assert!(game.undo_action().is_ok());
                    assert!(game.trading_phase);
                    assert_eq!(game.trade_buffer.len(), 3);
                    assert_eq!(game.players[3].cards.len(), 2);
                    assert!(game.players[3].cards.contains(&Card::Nine));

                    // Undo third trade (blue)
                    assert!(game.undo_action().is_ok());
                    assert!(game.trading_phase);
                    assert_eq!(game.trade_buffer.len(), 2);
                    assert_eq!(game.players[2].cards.len(), 2);
                    assert!(game.players[2].cards.contains(&Card::Seven));

                    // Undo second trade (green)
                    assert!(game.undo_action().is_ok());
                    assert!(game.trading_phase);
                    assert_eq!(game.trade_buffer.len(), 1);
                    assert_eq!(game.players[1].cards.len(), 2);
                    assert!(game.players[1].cards.contains(&Card::Two));

                    // Undo first trade (red)
                    assert!(game.undo_action().is_ok());
                    assert!(game.trading_phase);
                    assert_eq!(game.trade_buffer.len(), 0);
                    assert_eq!(game.players[0].cards.len(), 2);
                    assert!(game.players[0].cards.contains(&Card::Five));               
                }
            }
        
            mod undo_grab_tests {
                use super::*;

                #[test]
                fn undo_grab_restores_hands() {
                    let mut game = Game::new(GameVariant::FreeForAll(3));
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Two];
                    game.players[1].cards = vec![Card::Ace, Card::King];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Grab {
                            target_player: Color::Green,
                            target_card: 1,
                        },
                        card: Some(Card::Two),
                    };

                    assert!(game.action(Some(Card::Two), action).is_ok());

                    assert_eq!(game.players[0].cards, vec![Card::King]);
                    assert_eq!(game.players[1].cards, vec![Card::Ace]);

                    assert!(game.undo_action().is_ok());

                    assert_eq!(game.players[0].cards, vec![Card::Two]);
                    assert_eq!(game.players[1].cards, vec![Card::Ace, Card::King]);
                }

                #[test]
                fn undo_grab_restores_card_at_original_index() {
                    let mut game = Game::new(GameVariant::FreeForAll(3));
                    game.trading_phase = false;

                    game.current_player_index = 0;

                    game.players[0].cards = vec![Card::Two];
                    game.players[1].cards = vec![Card::Ace, Card::King, Card::Queen];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Grab {
                            target_player: Color::Green,
                            target_card: 1, // King
                        },
                        card: Some(Card::Two),
                    };

                    assert!(game.action(Some(Card::Two), action).is_ok());
                    assert!(game.undo_action().is_ok());

                    assert_eq!(
                        game.players[1].cards,
                        vec![Card::Ace, Card::King, Card::Queen]
                    );
                }

                #[test]
                fn undo_grab_restores_current_player() {
                    let mut game = Game::new(GameVariant::FreeForAll(3));
                    game.trading_phase = false;

                    game.current_player_index = 0;

                    game.players[0].cards = vec![Card::Two];
                    game.players[1].cards = vec![Card::Ace];

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Grab {
                            target_player: Color::Green,
                            target_card: 0,
                        },
                        card: Some(Card::Two),
                    };

                    assert!(game.action(Some(Card::Two), action).is_ok());
                    assert_ne!(game.current_player_index, 0);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.current_player_index, 0);
                }

                #[test]
                fn undo_grab_removes_history_entry() {
                    let mut game = Game::new(GameVariant::FreeForAll(3));
                    game.trading_phase = false;

                    game.players[0].cards = vec![Card::Two];
                    game.players[1].cards = vec![Card::Ace];

                    let history_len_before = game.history.len();

                    let action = Action {
                        player: Color::Red,
                        action: ActionKind::Grab {
                            target_player: Color::Green,
                            target_card: 0,
                        },
                        card: Some(Card::Two),
                    };

                    assert!(game.action(Some(Card::Two), action).is_ok());
                    assert_eq!(game.history.len(), history_len_before + 1);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.history.len(), history_len_before);
                }
            }
            
            mod undo_trade_grab_tests {
                use super::*;

                #[test]
                fn undo_trade_grab_basic() {
                    let mut game = Game::new(GameVariant::FreeForAll(2));
                    game.trading_phase = true;

                    game.players[0].cards = vec![Card::Ace, Card::Two];
                    game.players[1].cards = vec![Card::Three, Card::Four];

                    let action = Action {
                        player: game.players[0].color,
                        action: ActionKind::TradeGrab { target_card: 0 },
                        card: None,
                    };

                    assert!(game.action(None, action).is_ok());
                    assert_eq!(game.players[0].cards.len(), 2);
                    assert_eq!(game.players[1].cards.len(), 1);
                    assert_eq!(game.trade_buffer.len(), 1);
                    assert_eq!(game.trade_buffer, [(1, 0, Card::Three)]);

                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.players[0].cards.len(), 2);
                    assert_eq!(game.players[1].cards.len(), 2);
                    assert!(game.trade_buffer.is_empty());

                    assert_eq!(game.players[1].cards, vec![Card::Three, Card::Four]);
                    assert_eq!(game.trade_buffer.len(), 0);
                    assert_eq!(game.current_player_index, 0);
                }

                #[test]
                fn undo_trade_grab_full() {
                    let mut game = Game::new(GameVariant::FreeForAll(2));
                    game.trading_phase = true;

                    game.players[0].cards = vec![Card::Ace, Card::Two];
                    game.players[1].cards = vec![Card::Three, Card::Four];

                    let action1 = Action {
                        player: game.players[0].color,
                        action: ActionKind::TradeGrab { target_card: 0 },
                        card: None,
                    };

                    let action2 = Action {
                        player: game.players[1].color,
                        action: ActionKind::TradeGrab { target_card: 0 },
                        card: None,
                    };

                    assert!(game.action(None, action1).is_ok());
                    assert_eq!(game.players[0].cards.len(), 2);
                    assert_eq!(game.players[1].cards.len(), 1);
                    assert_eq!(game.trade_buffer.len(), 1);
                    assert_eq!(game.trade_buffer, [(1, 0, Card::Three)]);

                    assert!(game.action(None, action2).is_ok());
                    assert_eq!(game.players[0].cards.len(), 2);
                    assert_eq!(game.players[1].cards.len(), 2);
                    assert_eq!(game.trade_buffer.len(), 0);
                    assert_eq!(game.players[0].cards, vec![Card::Two, Card::Three]);
                    assert_eq!(game.players[1].cards, vec![Card::Four, Card::Ace]);

                    // First undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.players[0].cards.len(), 2);
                    assert_eq!(game.players[1].cards.len(), 1);
                    assert_eq!(game.trade_buffer, [(1, 0, Card::Three)]);

                    // Second undo
                    assert!(game.undo_action().is_ok());
                    assert_eq!(game.players[0].cards.len(), 2);
                    assert_eq!(game.players[1].cards.len(), 2);
                    assert!(game.trade_buffer.is_empty());

                    assert_eq!(game.players[1].cards, vec![Card::Three, Card::Four]);
                    assert_eq!(game.trade_buffer.len(), 0);
                    assert_eq!(game.current_player_index, 0);
                }


            }
        
            
        }

        mod undo_turn_tests {
            use super::*;

            #[test]
            fn undo_turn_single_action() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Ace, Card::Two];

                let action = Action {
                    player: Color::Red,
                    action: ActionKind:: Place { target_player: 0 },
                    card: Some(Card::Ace),
                };

                assert!(game.action(Some(Card::Ace), action).is_ok());

                // Sanity check
                let start = game.board.start_field(0) as usize;
                assert!(game.board.tiles[start].is_some());
                assert_eq!(game.current_player_index, 1);

                // Undo full turn
                assert!(game.undo_turn().is_ok());

                assert!(game.board.tiles[start].is_none());
                assert!(game.players[0].cards.contains(&Card::Ace));
                assert_eq!(game.current_player_index, 0);
            }

            #[test]
            fn undo_turn_split() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Seven, Card::Eight];

                game.board.tiles[0] = Some(Piece {
                    owner: 0,
                    left_start: false,
                });

                game.board.tiles[3] = Some(Piece { 
                    owner: 1, 
                    left_start: true 
                });

                let action1 = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 0, to: 5 },
                    card: Some(Card::Seven),
                };

                let action2 = Action {
                    player: Color::Red,
                    action: ActionKind::Split { from: 5, to: 7 },
                    card: Some(Card::Seven),
                };

                // Simulate turns
                assert!(game.action(Some(Card::Seven), action1).is_ok());
                assert_eq!(game.split_rest, Some(2));
                assert!(game.action(Some(Card::Seven), action2).is_ok());
                assert_eq!(game.split_rest, None);
                assert_eq!(game.current_player_index, 1);
                
                assert!(game.undo_turn().is_ok());
                assert_eq!(game.board.tiles[0].as_ref().unwrap().owner, 0);
                assert_eq!(game.board.tiles[3].as_ref().unwrap().owner, 1);
                assert!(game.board.tiles[7].is_none());
                assert_eq!(game.current_player_index, 0);
                assert_eq!(game.split_rest, None);
            }

            #[test]
            fn undo_turn_full_trade() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = true;

                game.players[0].cards = vec![Card::Ace];
                game.players[1].cards = vec![Card::Two];
                game.players[2].cards = vec![Card::Three];
                game.players[3].cards = vec![Card::Four];

                let action1 = Action {
                    player: Color::Red,
                    action: ActionKind::Trade,
                    card: Some(Card::Ace),
                };

                let action2 = Action {
                    player: Color::Green,
                    action: ActionKind::Trade,
                    card: Some(Card::Two),
                };

                let action3 = Action {
                    player: Color::Blue,
                    action: ActionKind::Trade,
                    card: Some(Card::Three),
                };

                let action4 = Action {
                    player: Color::Yellow,
                    action: ActionKind::Trade,
                    card: Some(Card::Four),
                };

                assert!(game.action(Some(Card::Ace), action1).is_ok());
                assert!(game.action(Some(Card::Two), action2).is_ok());
                assert!(game.action(Some(Card::Three), action3).is_ok());
                assert!(game.action(Some(Card::Four), action4).is_ok());
                assert!(!game.trading_phase);

                assert!(game.undo_turn().is_ok());
                assert!(game.trading_phase);
                assert_eq!(game.current_player_index, 0);
                assert!(game.trade_buffer.is_empty());
                
            }
        }

        mod undo_sequence_tests {
            use super::*;

            #[test]
            fn undo_sequence_multiple_turns() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = false;

                game.players[0].cards = vec![Card::Ace, Card::Two];
                game.players[1].cards = vec![Card::Ace, Card::Two];

                let action_red = Action {
                    player: Color::Red,
                    action: ActionKind:: Place { target_player: 0 },
                    card: Some(Card::Ace),
                };

                let action_green = Action {
                    player: Color::Green,
                    action: ActionKind:: Place { target_player: 1 },
                    card: Some(Card::Ace),
                };

                assert!(game.action(Some(Card::Ace), action_red).is_ok());
                assert!(game.action(Some(Card::Ace), action_green).is_ok());

                // Undo both turns
                assert!(game.undo_sequence(2).is_ok());

                let red_start = game.board.start_field(0);
                let green_start = game.board.start_field(1);

                assert!(game.board.tiles[red_start].is_none());
                assert!(game.board.tiles[green_start].is_none());

                assert!(game.players[0].cards.contains(&Card::Ace));
                assert!(game.players[1].cards.contains(&Card::Ace));
                assert_eq!(game.current_player_index, 0);
            }

            #[test]
            fn undo_sequence_trade_and_place() {
                let mut game = Game::new(GameVariant::TwoVsTwo);
                game.trading_phase = true;

                game.players[0].cards = vec![Card::Five];
                game.players[1].cards = vec![Card::Two];
                game.players[2].cards = vec![Card::Ace];
                game.players[3].cards = vec![Card::Nine];

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
                        card: Some(card),
                    };
                    assert!(game.action(Some(card), action).is_ok());
                }

                let place_action = Action {
                    player: Color::Red,
                    card: Some(Card::Ace),
                    action: ActionKind:: Place { target_player: 0 },
                };

                assert!(game.action(Some(Card::Ace), place_action).is_ok());

                // Undo entire trade
                assert!(game.undo_sequence(2).is_ok());

                assert!(game.trading_phase);
                assert_eq!(game.trade_buffer.len(), 0);

                assert!(game.players[0].cards.contains(&Card::Five));
                assert!(game.players[1].cards.contains(&Card::Two));
                assert!(game.players[2].cards.contains(&Card::Ace));
                assert!(game.players[3].cards.contains(&Card::Nine));
            }
        }
    }
}
