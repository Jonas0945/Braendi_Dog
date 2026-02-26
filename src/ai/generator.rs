use crate::{Action, ActionKind, game::{Game, board::Point, game::GameVariant}};
use crate::game::card::Card;

pub fn generate_all_legal_actions(game: &Game) -> Vec<Action> {
    let mut total_actions = Vec::new();

    let player_color = game.current_player().color;

    // Collect & return trade actions 
    if game.trading_phase {
        total_actions.extend(collect_trade_actions(game));

        return total_actions;
    }

    // Collect & return split actions while split_rest is active 
    if game.split_rest.is_some() {
        total_actions.extend(collect_split_actions(game));

        return total_actions;
    }

    // Collect place actions
    total_actions.extend(collect_place_actions(game));

    // Move collection
    total_actions.extend(collect_forward_move_actions(game));
    total_actions.extend(collect_backward_move_actions(game));

    // Split collection
    total_actions.extend(collect_split_actions(game));

    // Collect interchange actions
    total_actions.extend(collect_interchange_actions(game));

    // Collect grab actions
    match game.game_variant {
        GameVariant::FreeForAll(_) => {
            total_actions.extend(collect_grab_actions(game));
        },
        _ => {},
    }

    // Remove option = no other action possible 
    if total_actions.is_empty() {
        for card in &game.current_player().cards {
            let action = Action {
                player: player_color,
                card: Some(*card),
                action: ActionKind::Remove,
            };

            total_actions.push(action);
        }
    }


    total_actions
}

pub fn collect_place_actions(game: &Game) -> Vec<Action> {
    let mut place_actions = Vec::new();
    
    let player_color = game.current_player().color;
    let player_index = game.current_player_index;

    let place_cards: Vec<Card> = game.current_player().cards
        .iter()
        .cloned()
        .filter(|c| c.is_place_card())
        .collect();

    for card in &place_cards {
        // Place own pieces
        if game.current_player().pieces_to_place != 0 {
            let start_field = game.board.start_field(player_index) as Point;

            if !game.board.is_blocked(start_field) {
                place_actions.push(Action { 
                    player: player_color, 
                    card: Some(*card), 
                    action: ActionKind::Place { target_player: player_index } 
                });
            }
        }

        if game.current_player().pieces_in_house == 4 {
            for teammate in game.teammate_indices(player_index) {
                let teammate_start_field = game.board.start_field(teammate);

                if !game.board.is_blocked(teammate_start_field) {
                    place_actions.push(Action { 
                        player: player_color, 
                        card: Some(*card), 
                        action: ActionKind::Place { target_player: teammate } 
                    });    
                }
            }
        }
    }

    place_actions
}

pub fn collect_trade_actions(game: &Game) -> Vec<Action> {
    let mut trade_actions = Vec::new();

    let player_color = game.current_player().color;
    let player_index = game.current_player_index;

    match game.game_variant {
        GameVariant::FreeForAll(_) => {
            for target_index in 0..game.players.len() {
                if target_index == player_index {
                    continue;
                }

                // Trade grab every possible card of target player
                for card_index in 0..game.players[target_index].cards.len() {
                    let action = Action {
                        player: player_color,
                        card: None,
                        action: crate::ActionKind::TradeGrab { target_card: card_index }
                    };

                    trade_actions.push(action);
                }
            }
        },

        _ => {
            for card in &game.current_player().cards {
                let action = Action {
                    player: player_color,
                    card: Some(*card),
                    action: ActionKind::Trade,
                };

                trade_actions.push(action);
            }
        },
    }

    trade_actions
}

pub fn collect_split_actions(game: &Game) -> Vec<Action> {
    let mut split_actions = Vec::new();
    let split_rest = game.split_rest.unwrap_or(7);

    let player_color = game.current_player().color;
    let player_index = game.current_player_index;

    let split_cards: Vec<Card> = game.current_player().cards
        .iter()
        .cloned()
        .filter(|c| *c == Card::Seven)
        .collect();

    let allowed_owners: Vec<usize> = match game.game_variant {
        GameVariant::FreeForAll(_) => (0..game.players.len()).collect(),
        _ => {
            let mut team_owners = game.teammate_indices(player_index);
            team_owners.push(player_index);
            team_owners
        }
    };

    for _card in split_cards {
        for (from, tile) in game.board.tiles.iter().enumerate() {
            let Some(piece) = tile else { continue };

            if !allowed_owners.contains(&piece.owner) {
                continue;
            }

            let range = match piece.left_start {
                false => game.board.ring_size,
                true => game.board.tiles.len()
            };      

            for to in 0..range {
                let Some(distance) = game.board.distance_between(from, to, piece.owner) else {
                    continue;
                };

                if distance == 0 || distance > split_rest {
                    continue;
                }

                if !game.can_piece_move_from_to(from, to, false) {
                    continue;
                }

                let action = Action {
                    player: player_color,
                    card: Some(Card::Seven),
                    action: ActionKind::Split { from, to },
                };

                split_actions.push(action);
            }
        }   
    }
    

    split_actions

}

pub fn collect_grab_actions(game: &Game) -> Vec<Action> {
    let mut grab_actions = Vec::new();

    let player_color = game.current_player().color;
    let player_index = game.current_player_index;

    let grab_cards: Vec<Card> = game.current_player().cards
        .iter()
        .cloned()
        .filter(|c| *c == Card::Two)
        .collect();

    for grab_card in grab_cards {
        for (target_index, target_player) in game.players.iter().enumerate() {
            if target_index == player_index {
                continue
            };

            let target_color = target_player.color;

            for target_card_index in 0..target_player.cards.len() {
                grab_actions.push(Action { 
                    player: player_color, 
                    card: Some(grab_card), 
                    action: ActionKind::Grab { 
                        target_player: target_color,
                        target_card: target_card_index,    
                    }, 
                });
            }
        }
    }

    grab_actions
}

pub fn collect_forward_move_actions(game: &Game) -> Vec<Action> {
    let mut forward_move_actions = Vec::new();

    let player_color = game.current_player().color;
    let player_index = game.current_player_index;

    let forward_move_cards: Vec<Card> = game.current_player().cards
        .iter()
        .cloned()
        .filter(|c| c.is_forward_move_card())
        .collect();

    for card in forward_move_cards {
        for dist in card.possible_distances() {

            for (from, tile) in game.board.tiles.iter().enumerate() {
                let Some(piece) = tile else { continue };
                if !game.can_control_piece(player_index, piece.owner) {
                    continue;
                }

                if !game.can_piece_move_distance(from, dist, false) {
                    continue;
                }

                let range = match piece.left_start {
                    false => game.board.ring_size,
                    true => game.board.tiles.len()
                }; 

                for to in 0..range {
                    if game.board.distance_between(from, to, piece.owner) != Some(dist) {
                        continue;
                    }

                    if !game.can_piece_move_from_to(from, to, false) {
                        continue;
                    }

                    forward_move_actions.push(Action {
                        player: player_color,
                        card: Some(card),
                        action: ActionKind::Move { from, to },
                    });
                }
            }
        }
    }

    forward_move_actions
}

pub fn collect_backward_move_actions(game: &Game) -> Vec<Action> {
    let mut backward_move_actions = Vec::new();

    let player_color = game.current_player().color;
    let player_index = game.current_player_index;

    let backward_move_cards: Vec<Card> = game.current_player().cards
        .iter()
        .cloned()
        .filter(|c| c.is_backward_move_card())
        .collect();

    for card in backward_move_cards {
        for (from, tile) in game.board.tiles.iter().enumerate() {

            // Cannot move backwards in-house
            if from >= game.board.ring_size {
                continue;
            }
            
            let Some(piece) = tile else { continue };
            
            if !game.can_control_piece(player_index, piece.owner) {
                continue;
            }

            let distance = 4;
            let to = (from + game.board.ring_size - distance as usize) % game.board.ring_size;

            if !game.can_piece_move_from_to(from, to, true) {
                continue;
            }

            backward_move_actions.push(Action {
                player: player_color,
                card: Some(card),
                action: ActionKind::Move { from, to },
            }); 
        }
    }

    backward_move_actions
}

pub fn collect_interchange_actions(game: &Game) -> Vec<Action> {
    let mut interchange_actions = Vec::new();

    let player_color = game.current_player().color;
    let player_index = game.current_player_index;

    let interchange_cards: Vec<Card> = game.current_player().cards
        .iter()
        .cloned()
        .filter(|c| c.is_interchange_card())
        .collect();


    for card in interchange_cards {
        for (a_index, a_tile) in game.board.tiles.iter().enumerate() {
            
            if a_index >= game.board.ring_size {
                continue;
            }

            let Some(a_piece) = a_tile else { continue };

            if !a_piece.left_start {
                continue;
            }

            if !game.can_control_piece(player_index, a_piece.owner) {
                continue;
            }

            for (b_index, b_tile) in game.board.tiles.iter().enumerate() {
                if a_index == b_index {
                    continue;
                }

                if b_index >= game.board.ring_size {
                    continue;
                }

                let Some(b_piece) = b_tile else { continue };

                // Cannot interchange with identical owners
                if a_piece.owner == b_piece.owner {
                    continue;
                }

                if !b_piece.left_start {
                    continue;
                }

                interchange_actions.push(Action {
                    player: player_color,
                    card: Some(card),
                    action: ActionKind::Interchange { a: a_index, b: b_index },
                });
            }
        }
    }


    interchange_actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::piece::Piece;
    use crate::game::player::PlayerType;

    mod collect_place_tests {
        use crate::Piece;

        use super::*;

        #[test]
        fn own_place_free_field() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Ace];

            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 1);
            assert_eq!(actions[0].player, game.player_by_index(0).color);
            assert_eq!(actions[0].card, Some(Card::Ace));
        }

        #[test]
        fn own_place_blocked_field() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Ace];
            game.board.tiles[0] = Some(Piece {
                owner: 0,
                left_start: false,
            });

            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn own_place_multiple_cards() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Ace, Card::Joker];

            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 2);
            assert_eq!(actions[0].player, game.player_by_index(0).color);
            assert_eq!(actions[0].card, Some(Card::Ace));

            assert_eq!(actions[1].player, game.player_by_index(0).color);
            assert_eq!(actions[1].card, Some(Card::Joker));
        }

        #[test]
        fn own_place_no_card() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Two];
            
            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn own_place_no_pieces_to_place() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].pieces_to_place = 0;
            game.players[0].pieces_in_house = 3;
            game.players[0].cards = vec![Card::Ace];
            
            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 0);
        }



        #[test]
        fn team_place_free_field() {
            let mut game = Game::new(GameVariant::ThreeVsThree, vec![PlayerType::Human; 6]);
            game.trading_phase = false;

            game.players[0].pieces_to_place = 0;
            game.players[0].pieces_in_house = 4;
            game.players[0].cards = vec![Card::Ace];

            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 2);
            assert_eq!(actions[0].player, game.player_by_index(0).color);
            assert_eq!(actions[0].card, Some(Card::Ace));
            if let ActionKind::Place { target_player } = actions[0].action {
                assert_eq!(target_player, 2);
            } else {
                panic!("Expected Place action");
            }

            if let ActionKind::Place { target_player } = actions[1].action {
                assert_eq!(target_player, 4);
            } else {
                panic!("Expected Place action");
            }
        }

        #[test]
        fn team_place_no_cards() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].pieces_to_place = 0;
            game.players[0].pieces_in_house = 4;
            game.players[0].cards = vec![Card::Two];
            
            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn team_place_multiple_cards() {
            let mut game = Game::new(GameVariant::ThreeVsThree, vec![PlayerType::Human; 6]);
            game.trading_phase = false;

            game.players[0].pieces_to_place = 0;
            game.players[0].pieces_in_house = 4;
            game.players[0].cards = vec![Card::Ace, Card::Ace];

            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 4);
        }

        #[test]
        fn team_place_all_fields_blocked() {
            let mut game = Game::new(GameVariant::ThreeVsThree, vec![PlayerType::Human; 6]);
            game.trading_phase = false;

            game.players[0].pieces_to_place = 0;
            game.players[0].pieces_in_house = 4;
            game.players[0].cards = vec![Card::Ace];

            game.board.tiles[32] = Some(Piece {
                owner: 1,
                left_start: false,
            });

            game.board.tiles[64] = Some(Piece {
                owner: 3,
                left_start: false,
            });

            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn team_place_one_field_blocked() {
            let mut game = Game::new(GameVariant::ThreeVsThree, vec![PlayerType::Human; 6]);
            game.trading_phase = false;

            game.players[0].pieces_to_place = 0;
            game.players[0].pieces_in_house = 4;
            game.players[0].cards = vec![Card::Ace];

            game.board.tiles[64] = Some(Piece {
                owner: 3,
                left_start: false,
            });

            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 1);
        }    
    }

    mod collect_trade_tests {
        use super::*;

        #[test]
        fn ffa_single_opponent_single_card() {
            let mut game = Game::new(GameVariant::FreeForAll(4), vec![PlayerType::Human; 4]);
            game.trading_phase = true;

            game.players[3].cards = vec![Card::Ace];

            let actions = collect_trade_actions(&game);

            assert_eq!(actions.len(), 1);
            assert_eq!(actions[0].player, game.player_by_index(0).color);
            assert_eq!(actions[0].card, None);

            match actions[0].action {
                ActionKind::TradeGrab { target_card } => assert_eq!(target_card, 0),
                _ => panic!("Expected TradeGrab"),
            }
        }

        #[test]
        fn ffa_multiple_opponents_multiple_cards() {
            let mut game = Game::new(GameVariant::FreeForAll(3), vec![PlayerType::Human; 3]);
            game.trading_phase = true;

            game.players[1].cards = vec![Card::Ace, Card::Two];
            game.players[2].cards = vec![Card::Three];

            let actions = collect_trade_actions(&game);

            assert_eq!(actions.len(), 3);

            let expected = vec![
                (1, 0),
                (1, 1),
                (2, 0),
            ];

            for (i, action) in actions.iter().enumerate() {
                assert_eq!(action.player, game.player_by_index(0).color);
                assert_eq!(action.card, None);

                match action.action {
                    ActionKind::TradeGrab { target_card } => {
                        assert_eq!(target_card, expected[i].1);
                    }
                    _ => panic!("Expected TradeGrab"),
                }
            }
        }

        #[test]
        fn team_normal_single_card() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = true;

            game.players[0].cards = vec![Card::Ace];

            let actions = collect_trade_actions(&game);

            assert_eq!(actions.len(), 1);

            let action = &actions[0];
            assert_eq!(action.player, game.player_by_index(0).color);
            assert_eq!(action.card, Some(Card::Ace));

            match action.action {
                ActionKind::Trade => {},
                _ => panic!("Expected Trade"),
            }
        }

        #[test]
        fn team_normal_multiple_cards() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = true;

            game.players[0].cards = vec![Card::Ace, Card::Joker];

            let actions = collect_trade_actions(&game);

            assert_eq!(actions.len(), 2);

            
            assert_eq!(actions[0].player, game.player_by_index(0).color);
            assert_eq!(actions[0].card, Some(Card::Ace));
            if let ActionKind::Trade = actions[0].action {} else { panic!("Expected Trade"); }

            assert_eq!(actions[1].player, game.player_by_index(0).color);
            assert_eq!(actions[1].card, Some(Card::Joker));
            if let ActionKind::Trade = actions[1].action {} else { panic!("Expected Trade"); }
        }

        #[test]
        fn team_normal_no_cards() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = true;

            game.players[0].cards = vec![];

            let actions = collect_trade_actions(&game);
            assert_eq!(actions.len(), 0);
        }
    }

    mod collect_split_tests {
        use super::*;

        #[test]
        fn ffa_two_pieces_move_only_ring() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Seven];

            let actions = collect_split_actions(&game);

            assert_eq!(actions.len(), 14);
        }

        #[test]
        fn ffa_two_piece_move_left_start_different() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Seven];
            game.board.tiles[0] = Some(Piece { owner: 0, left_start: true }); // now has 11 Options (7 ring + 4 in-house)

            let actions = collect_split_actions(&game);

            assert_eq!(actions.len(), 18); // 11 + 7
        }

        #[test]
        fn ffa_two_pieces_move_with_split_rest() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;
            game.split_rest = Some(4);

            game.players[0].cards = vec![Card::Seven];

            let actions = collect_split_actions(&game);

            assert_eq!(actions.len(), 8);
        }

        #[test]
        fn ffa_two_pieces_move_with_split_rest_and_blocking_piece() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;
            game.split_rest = Some(4);

            game.players[0].cards = vec![Card::Seven];
            game.board.tiles[1] = Some(Piece { owner: 0, left_start: false });

            let actions = collect_split_actions(&game);

            assert_eq!(actions.len(), 8);
        }

        #[test]
        fn ffa_three_pieces_move_with_split_rest() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;
            game.split_rest = Some(4);

            game.players[0].cards = vec![Card::Seven];
            game.board.tiles[1] = Some(Piece { owner: 0, left_start: true });

            let actions = collect_split_actions(&game);

            assert_eq!(actions.len(), 12);
        }

        #[test]
        fn no_split_without_seven_card() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Ace];

            let actions = collect_split_actions(&game);

            assert!(actions.is_empty());
        }

        #[test]
        fn multiple_sevens_duplicate_actions() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;
            game.players[0].cards = vec![Card::Seven, Card::Seven];

            let actions = collect_split_actions(&game);

            assert_eq!(actions.len(), 28);
        }

        #[test]
        fn all_actions_are_valid_splits() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;
            let actions = collect_split_actions(&game);

            for action in actions {
                assert_eq!(action.card, Some(Card::Seven));

                match action.action {
                    ActionKind::Split { from, to } => {
                        assert_ne!(from, to);
                    }
                    _ => panic!("Expected Split action"),
                }
            }
        }

        #[test]
        fn team_mode_does_not_move_enemy_pieces() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Seven];

            game.board.tiles[0] = Some(Piece {
                owner: 0,
                left_start: false,
            });

            game.board.tiles[5] = Some(Piece {
                owner: 1,
                left_start: false,
            });

            game.board.tiles[7] = Some(Piece {
                owner: 2,
                left_start: false,
            });

            let actions = collect_split_actions(&game);

            for action in actions {
                if let ActionKind::Split { from, .. } = action.action {
                    let piece = game.board.tiles[from].as_ref().unwrap();
                    assert!(piece.owner == 0 || piece.owner == 2);
                }
            }
        }

        #[test]
        fn split_rest_zero_produces_no_actions() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Seven];
            game.split_rest = Some(0);

            let actions = collect_split_actions(&game);

            assert!(actions.is_empty());
        }

        #[test]
        fn fully_blocked_piece_has_no_split_actions() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
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

            let actions = collect_split_actions(&game);

            assert!(actions.is_empty());
        }
    }

    mod collect_grab_tests {
        use super::*;

        #[test]
        fn no_grab_without_two_card() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Ace];

            let actions = collect_grab_actions(&game);

            assert!(actions.is_empty());
        }

        #[test]
        fn grab_single_card_from_single_opponent() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Two];

            game.players[1].cards = vec![Card::Ace];

            let actions = collect_grab_actions(&game);

            assert_eq!(actions.len(), 1);

            let action = &actions[0];
            assert_eq!(action.player, game.players[0].color);
            assert_eq!(action.card, Some(Card::Two));

            match action.action {
                ActionKind::Grab { target_player, target_card } => {
                    assert_eq!(target_player, game.players[1].color);
                    assert_eq!(target_card, 0);
                }
                _ => panic!("Expected Grab action"),
            }
        }

        #[test]
        fn grab_multiple_cards_from_one_opponent() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Two];
            game.players[1].cards = vec![Card::Ace, Card::King, Card::Queen];

            let actions = collect_grab_actions(&game);

            assert_eq!(actions.len(), 3);

            for action in actions {
                match action.action {
                    ActionKind::Grab { target_player, target_card } => {
                        assert_eq!(target_player, game.players[1].color);
                        assert!(target_card < 3);
                    }
                    _ => panic!("Expected Grab action"),
                }
            }
        }

        #[test]
        fn grab_from_multiple_opponents() {
            let mut game = Game::new(GameVariant::FreeForAll(3), vec![PlayerType::Human; 3]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Two];

            game.players[1].cards = vec![Card::Ace];
            game.players[2].cards = vec![Card::King, Card::Queen];

            let actions = collect_grab_actions(&game);

            assert_eq!(actions.len(), 3);

            let mut targets = Vec::new();

            for action in actions {
                if let ActionKind::Grab { target_player, .. } = action.action {
                    targets.push(target_player);
                }
            }

            assert!(targets.contains(&game.players[1].color));
            assert!(targets.contains(&game.players[2].color));
        }

        #[test]
        fn multiple_twos_duplicate_grab_actions() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Two, Card::Two];
            game.players[1].cards = vec![Card::Ace, Card::King];

            let actions = collect_grab_actions(&game);

            // 2 Twos × 2 target cards
            assert_eq!(actions.len(), 4);
        }

        #[test]
        fn grab_never_targets_self() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);

            game.players[0].cards = vec![Card::Two];
            game.players[0].cards.push(Card::Ace);

            let actions = collect_grab_actions(&game);

            for action in actions {
                if let ActionKind::Grab { target_player, .. } = action.action {
                    assert_ne!(target_player, game.players[0].color);
                }
            }
        }
    }

    mod collect_forward_move_tests {
        use super::*;

        #[test]
        fn single_piece_single_move_card() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Ace];
            game.board.tiles[0] = Some(Piece { 
                owner: 0, 
                left_start: false });

            let actions = collect_forward_move_actions(&game);

            assert!(actions.len() == 2);
            assert_eq!(actions[0].player, game.current_player().color);
            assert_eq!(actions[0].card, Some(Card::Ace));
            assert!(matches!(actions[0].action, ActionKind::Move { from: 0, to: 1 }));
            assert_eq!(actions[1].player, game.current_player().color);
            assert_eq!(actions[1].card, Some(Card::Ace));
            assert!(matches!(actions[1].action, ActionKind::Move { from: 0, to: 11 }));
        }

        #[test]
        fn single_piece_left_start() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Ace];
            game.board.tiles[0] = Some(Piece { 
                owner: 0, 
                left_start: true });

            let actions = collect_forward_move_actions(&game);

            assert!(actions.len() == 3);
            assert_eq!(actions[0].player, game.current_player().color);
            assert_eq!(actions[0].card, Some(Card::Ace));
            assert!(matches!(actions[0].action, ActionKind::Move { from: 0, to: 1 }));
            assert_eq!(actions[1].player, game.current_player().color);
            assert_eq!(actions[1].card, Some(Card::Ace));
            assert!(matches!(actions[1].action, ActionKind::Move { from: 0, to: 32 }));
            assert_eq!(actions[2].player, game.current_player().color);
            assert_eq!(actions[2].card, Some(Card::Ace));
            assert!(matches!(actions[2].action, ActionKind::Move { from: 0, to: 11 }));
        }

        #[test]
        fn multiple_pieces_multiple_distances() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;
            game.players[0].cards = vec![Card::Two];
            game.board.tiles[0] = Some(Piece { owner: 0, left_start: false });
            game.board.tiles[5] = Some(Piece { owner: 0, left_start: false });

            let actions = collect_forward_move_actions(&game);

            assert!(actions.len() == 2);
            assert!(matches!(actions[0].action, ActionKind::Move { from: 0, to: 2 }));
            assert!(matches!(actions[1].action, ActionKind::Move { from: 5, to: 7 }));
        }

        #[test]
        fn blocked_field_no_action() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;
            game.players[0].cards = vec![Card::Ace];
            game.board.tiles[0] = Some(Piece { owner: 0, left_start: false });
            game.board.tiles[1] = Some(Piece { owner: 1, left_start: false });

            let actions = collect_forward_move_actions(&game);

            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn joker_can_move_every_distance() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;
            game.players[0].cards = vec![Card::Joker];
            game.board.tiles[0] = Some(Piece { owner: 0, left_start: false });

            let actions = collect_forward_move_actions(&game);

            assert!(actions.len() == 13);
            for act in &actions {
                assert_eq!(act.card, Some(Card::Joker));
            }
        }

        #[test]
        fn only_own_pieces_movable() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;
            game.players[0].cards = vec![Card::Ace];
            game.board.tiles[0] = Some(Piece { owner: 0, left_start: false });
            game.board.tiles[12] = Some(Piece { owner: 1, left_start: false });

            let actions = collect_forward_move_actions(&game);
            assert!(actions.len() == 2);
        }
    }

    mod collect_backward_move_tests {
        use super::*;

        #[test]
        fn single_piece_single_move_card() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Four];
            game.board.tiles[0] = Some(Piece { 
                owner: 0, 
                left_start: false });

            let actions = collect_backward_move_actions(&game);

            assert!(actions.len() == 1);
            assert_eq!(actions[0].player, game.current_player().color);
            assert_eq!(actions[0].card, Some(Card::Four));
            assert!(matches!(actions[0].action, ActionKind::Move { from: 0, to: 28 }));
        }

        #[test]
        fn multiple_pieces_single_move() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Joker];
            game.board.tiles[0] = Some(Piece { owner: 0, left_start: false });
            game.board.tiles[5] = Some(Piece { owner: 0, left_start: false });

            let actions = collect_backward_move_actions(&game);

            assert_eq!(actions.len(), 2);
            assert!(actions.iter().any(|a| matches!(a.action, ActionKind::Move { from: 0, to: 28 })));
            assert!(actions.iter().any(|a| matches!(a.action, ActionKind::Move { from: 5, to: 1 })));
        }

        #[test]
        fn blocked_backward_move() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Four];
            game.board.tiles[0] = Some(Piece { owner: 0, left_start: false });
            game.board.tiles[28] = Some(Piece { owner: 1, left_start: false });

            let actions = collect_backward_move_actions(&game);

            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn left_start_backward_move() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Four];
            game.board.tiles[0] = Some(Piece { owner: 0, left_start: true });

            let actions = collect_backward_move_actions(&game);

            assert_eq!(actions.len(), 1);
            assert!(matches!(actions[0].action, ActionKind::Move { from: 0, to: 28 }));
        }

        #[test]
        fn single_move_multiple_cards() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Four, Card::Four];
            game.board.tiles[0] = Some(Piece { owner: 0, left_start: true });

            let actions = collect_backward_move_actions(&game);

            assert_eq!(actions.len(), 2);
            assert!(matches!(actions[0].action, ActionKind::Move { from: 0, to: 28 }));
        }

        #[test]
        fn joker_backward_moves() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Joker];
            game.board.tiles[0] = Some(Piece { owner: 0, left_start: false });

            let actions = collect_backward_move_actions(&game);

            assert_eq!(actions.len(), 1);
            assert!(matches!(actions[0].action, ActionKind::Move { from: 0, to: 28 }));
        }

        #[test]
        fn in_house_backward_move_not_allowed() {
            let mut game = Game::new(GameVariant::FreeForAll(2), vec![PlayerType::Human; 2]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Four];
            game.board.tiles[0] = None;
            game.board.tiles[32] = Some(Piece { owner: 0, left_start: false }); // In-house

            let actions = collect_backward_move_actions(&game);

            assert_eq!(actions.len(), 0);
        }
    }

    mod collect_interchange_tests {
        use super::*;

        #[test]
        fn no_interchange_single_piece() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Jack];

            game.board.tiles[0] = Some(Piece { owner: 0, left_start: true });

            let actions = collect_interchange_actions(&game);
            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn interchange_with_teammate_and_opponent() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Jack];

            game.board.tiles[0] = Some(Piece { owner: 0, left_start: true }); 
            game.board.tiles[4] = Some(Piece { owner: 2, left_start: true }); 
            game.board.tiles[7] = Some(Piece { owner: 1, left_start: true }); 

            let actions = collect_interchange_actions(&game);

            assert_eq!(actions.len(), 2);
            assert_eq!(actions[0].player, game.current_player().color);
            assert_eq!(actions[0].card, Some(Card::Jack));
            assert!(matches!(actions[0].action, ActionKind::Interchange { a: 0, b: 4 }));
        }

        #[test]
        fn no_interchange_solely_own_pieces() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Jack];

            game.board.tiles[0] = Some(Piece { owner: 0, left_start: true }); 
            game.board.tiles[4] = Some(Piece { owner: 0, left_start: true });

            let actions = collect_interchange_actions(&game);

            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn no_interchange_with_blocked_piece_and_in_house_piece() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Jack];

            game.board.tiles[0] = Some(Piece { owner: 0, left_start: true }); 
            game.board.tiles[4] = Some(Piece { owner: 1, left_start: false });
            game.board.tiles[68] = Some(Piece { owner: 1, left_start: true });

            let actions = collect_interchange_actions(&game);

            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn mulitple_interchanges() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Jack];

            game.board.tiles[0] = Some(Piece { owner: 0, left_start: true }); 
            game.board.tiles[4] = Some(Piece { owner: 0, left_start: true }); 
            game.board.tiles[7] = Some(Piece { owner: 1, left_start: true }); 

            let actions = collect_interchange_actions(&game);

            assert_eq!(actions.len(), 2);
        }

        #[test]
        fn no_interchange_without_own_piece() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Jack];

            game.board.tiles[1] = Some(Piece { owner: 1, left_start: true });
            game.board.tiles[3] = Some(Piece { owner: 2, left_start: true });

            let actions = collect_interchange_actions(&game);
            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn interchange_without_own_piece_full_house() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Jack];
            game.players[0].pieces_in_house = 4;

            game.board.tiles[1] = Some(Piece { owner: 1, left_start: true });
            game.board.tiles[3] = Some(Piece { owner: 2, left_start: true });

            let actions = collect_interchange_actions(&game);
            assert_eq!(actions.len(), 1);
        }
    }
}