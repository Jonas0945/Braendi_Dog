use crate::{Action, ActionKind, game::{Game, game::GameVariant}};

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
    if game.split_rest.is_some() {

        match game.game_variant {
            GameVariant::FreeForAll(_) => todo!(),
            _ => todo!(),
        }
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