use crate::{Action, ActionKind, game::{Game, board::Point, game::GameVariant}};
use crate::game::card::Card;

pub fn generate_all_legal_actions(game: &Game) -> Vec<Action> {
    let mut total_actions = Vec::new();

    let player_color = game.current_player().color;
    let player_index = game.current_player_index;

    /*let move_cards: Vec<Card> = game.current_player().cards
        .iter()
        .cloned()
        .filter(|c| c.is_move_card())
        .collect();*/

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
    /*for card in &move_cards {
        for &dist in card.possible_distances().iter() {
            for (from, tile) in game.board.tiles.iter().enumerate() {
                let piece = match tile {
                    Some(p) if game.can_control_piece(player_index, p.owner) => p, 
                    _ => continue,
                };


                let can_forward = game.can_piece_move_distance(from, dist, false);
                let can_backward = game.can_piece_move_distance(from, dist, true);

                match card {
                    Card::Four if !can_forward && !can_backward => continue,
                    _ if !can_forward => continue,
                    _ => {},
                };

                for to in 0..game.board.tiles.len() {
                    let valid = game.board.distance_between(from, to, piece.owner) 
                        == Some(dist);

                        if !valid {
                            continue
                        };


                }



            }
        }
    }*/

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
        for (from_index, tile) in game.board.tiles.iter().enumerate() {
            let Some(piece) = tile else { continue };

            if !allowed_owners.contains(&piece.owner) {
                continue;
            }

            let range = match piece.left_start {
                false => game.board.ring_size,
                true => game.board.tiles.len()
            };      

            for to_index in 0..range {
                let Some(distance) = game.board.distance_between(from_index, to_index, piece.owner) else {
                    continue;
                };

                if distance == 0 || distance > split_rest {
                    continue;
                }

                if !game.can_piece_move_from_to(from_index, to_index, false) {
                    continue;
                }

                let action = Action {
                    player: player_color,
                    card: Some(Card::Seven),
                    action: ActionKind::Split { from: from_index, to: to_index },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::piece::Piece;

    mod collect_place_tests {
        use crate::Piece;

        use super::*;

        #[test]
        fn own_place_free_field() {
            let mut game = Game::new(GameVariant::TwoVsTwo);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Ace];

            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 1);
            assert_eq!(actions[0].player, game.player_by_index(0).color);
            assert_eq!(actions[0].card, Some(Card::Ace));
        }

        #[test]
        fn own_place_blocked_field() {
            let mut game = Game::new(GameVariant::TwoVsTwo);
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
            let mut game = Game::new(GameVariant::TwoVsTwo);
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
            let mut game = Game::new(GameVariant::TwoVsTwo);
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Two];
            
            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn own_place_no_pieces_to_place() {
            let mut game = Game::new(GameVariant::TwoVsTwo);
            game.trading_phase = false;

            game.players[0].pieces_to_place = 0;
            game.players[0].pieces_in_house = 3;
            game.players[0].cards = vec![Card::Ace];
            
            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 0);
        }



        #[test]
        fn team_place_free_field() {
            let mut game = Game::new(GameVariant::ThreeVsThree);
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
            let mut game = Game::new(GameVariant::TwoVsTwo);
            game.trading_phase = false;

            game.players[0].pieces_to_place = 0;
            game.players[0].pieces_in_house = 4;
            game.players[0].cards = vec![Card::Two];
            
            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn team_place_multiple_cards() {
            let mut game = Game::new(GameVariant::ThreeVsThree);
            game.trading_phase = false;

            game.players[0].pieces_to_place = 0;
            game.players[0].pieces_in_house = 4;
            game.players[0].cards = vec![Card::Ace, Card::Ace];

            let actions = collect_place_actions(&game);

            assert_eq!(actions.len(), 4);
        }

        #[test]
        fn team_place_all_fields_blocked() {
            let mut game = Game::new(GameVariant::ThreeVsThree);
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
            let mut game = Game::new(GameVariant::ThreeVsThree);
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
            let mut game = Game::new(GameVariant::FreeForAll(4));
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
            let mut game = Game::new(GameVariant::FreeForAll(3));
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
            let mut game = Game::new(GameVariant::TwoVsTwo);
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
            let mut game = Game::new(GameVariant::TwoVsTwo);
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
            let mut game = Game::new(GameVariant::TwoVsTwo);
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
            let mut game = Game::new(GameVariant::FreeForAll(2));
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Seven];

            let actions = collect_split_actions(&game);

            assert_eq!(actions.len(), 14);
        }

        #[test]
        fn ffa_two_piece_move_left_start_different() {
            let mut game = Game::new(GameVariant::FreeForAll(2));
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Seven];
            game.board.tiles[0] = Some(Piece { owner: 0, left_start: true }); // now has 11 Options (7 ring + 4 in-house)

            let actions = collect_split_actions(&game);

            assert_eq!(actions.len(), 18); // 11 + 7
        }

        #[test]
        fn ffa_two_pieces_move_with_split_rest() {
            let mut game = Game::new(GameVariant::FreeForAll(2));
            game.trading_phase = false;
            game.split_rest = Some(4);

            game.players[0].cards = vec![Card::Seven];

            let actions = collect_split_actions(&game);

            assert_eq!(actions.len(), 8);
        }

        #[test]
        fn ffa_two_pieces_move_with_split_rest_and_blocking_piece() {
            let mut game = Game::new(GameVariant::FreeForAll(2));
            game.trading_phase = false;
            game.split_rest = Some(4);

            game.players[0].cards = vec![Card::Seven];
            game.board.tiles[1] = Some(Piece { owner: 0, left_start: false });

            let actions = collect_split_actions(&game);

            assert_eq!(actions.len(), 8);
        }

        #[test]
        fn ffa_three_pieces_move_with_split_rest() {
            let mut game = Game::new(GameVariant::FreeForAll(2));
            game.trading_phase = false;
            game.split_rest = Some(4);

            game.players[0].cards = vec![Card::Seven];
            game.board.tiles[1] = Some(Piece { owner: 0, left_start: true });

            let actions = collect_split_actions(&game);

            assert_eq!(actions.len(), 12);
        }

        #[test]
        fn no_split_without_seven_card() {
            let mut game = Game::new(GameVariant::FreeForAll(2));
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Ace];

            let actions = collect_split_actions(&game);

            assert!(actions.is_empty());
        }

        #[test]
        fn multiple_sevens_duplicate_actions() {
            let mut game = Game::new(GameVariant::FreeForAll(2));
            game.trading_phase = false;
            game.players[0].cards = vec![Card::Seven, Card::Seven];

            let actions = collect_split_actions(&game);

            assert_eq!(actions.len(), 28);
        }

        #[test]
        fn all_actions_are_valid_splits() {
            let mut game = Game::new(GameVariant::FreeForAll(2));
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
            let mut game = Game::new(GameVariant::TwoVsTwo);
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
            let mut game = Game::new(GameVariant::FreeForAll(2));
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Seven];
            game.split_rest = Some(0);

            let actions = collect_split_actions(&game);

            assert!(actions.is_empty());
        }

        #[test]
        fn fully_blocked_piece_has_no_split_actions() {
            let mut game = Game::new(GameVariant::TwoVsTwo);
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
            let mut game = Game::new(GameVariant::FreeForAll(2));
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Ace];

            let actions = collect_grab_actions(&game);

            assert!(actions.is_empty());
        }

        #[test]
        fn grab_single_card_from_single_opponent() {
            let mut game = Game::new(GameVariant::FreeForAll(2));
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
            let mut game = Game::new(GameVariant::FreeForAll(2));
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
            let mut game = Game::new(GameVariant::FreeForAll(3));
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
            let mut game = Game::new(GameVariant::FreeForAll(2));
            game.trading_phase = false;

            game.players[0].cards = vec![Card::Two, Card::Two];
            game.players[1].cards = vec![Card::Ace, Card::King];

            let actions = collect_grab_actions(&game);

            // 2 Twos × 2 target cards
            assert_eq!(actions.len(), 4);
        }

        #[test]
        fn grab_never_targets_self() {
            let mut game = Game::new(GameVariant::FreeForAll(2));

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
}