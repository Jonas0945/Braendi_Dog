use crate::{Action, ActionKind, game::{Game, game::GameVariant}};
use crate::game::card::Card;

pub fn generate_all_legal_actions(game: &Game) -> Vec<Action> {
    let mut total_actions = Vec::new();

    let player_color = game.current_player().color;
    let player_index = game.current_player_index;

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