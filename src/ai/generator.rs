use crate::{Action, ActionKind, game::{Game, board::Point, game::GameVariant}};
use crate::game::card::Card;

pub fn generate_all_legal_actions(game: &Game) -> Vec<Action> {
    let mut total_actions = Vec::new();

    let player_color = game.current_player().color;
    let player_index = game.current_player_index;

    let place_cards: Vec<Card> = game.current_player().cards
        .iter()
        .cloned()
        .filter(|c| c.is_place_card())
        .collect();

    /*let move_cards: Vec<Card> = game.current_player().cards
        .iter()
        .cloned()
        .filter(|c| c.is_move_card())
        .collect();*/

    // Trade phase moves 
    if game.trading_phase {
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

                        total_actions.push(action);
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

                    total_actions.push(action);
                }
            },
        }

        return total_actions;
    }

    // Split only options
    if let Some(split_rest) = game.split_rest {

        match game.game_variant {
            GameVariant::FreeForAll(_) => {
                for (from_index, tile) in game.board.tiles.iter().enumerate() {
                    let Some(piece) = tile else { continue };

                    for to_index in 0..game.board.tiles.len() {
                            
                        let Some(distance) = game.board
                            .distance_between(from_index, to_index, piece.owner)
                        
                        else {
                            continue
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

                        total_actions.push(action);
                    }
                }
            }
            _ => {
                let mut allowed_owners = game.teammate_indices(player_index);
                allowed_owners.push(player_index);

                for (from_index, tile) in game.board.tiles.iter().enumerate() {
                    let Some(piece) = tile else { continue };

                    if !allowed_owners.contains(&piece.owner) {
                        continue
                    };

                    for to_index in 0..game.board.tiles.len() {
                        let Some(distance) = game.board
                            .distance_between(from_index, to_index, piece.owner)
                        else {
                            continue
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

                        total_actions.push(action);
                    }
                }
            }
        }

        return total_actions;
    }

    // Collect every normal action

    // Place own pieces
    if game.current_player().pieces_to_place != 0 {
        let start_field = game.board.start_field(player_index) as Point;

        if !game.board.is_blocked(start_field) {
            for card in &place_cards {
                total_actions.push(Action { 
                    player: player_color, 
                    card: Some(*card), 
                    action: ActionKind::Place { target_player: player_index } 
                });
            }
        }
    }

    //Place team pieces
    if game.current_player().pieces_in_house == 4 {
        for teammate in game.teammate_indices(player_index) {
            let teammate_start_field = game.board.start_field(teammate);

            if !game.board.is_blocked(teammate_start_field) {
                for card in &place_cards {
                    total_actions.push(Action { 
                        player: player_color, 
                        card: Some(*card), 
                        action: ActionKind::Place { target_player: teammate } 
                    });    
                }
            }
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

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
}