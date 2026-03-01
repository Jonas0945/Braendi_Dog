use rand::rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::Action;
use crate::ai::evaluator::{EvalContext, EvalPerspective, Evaluator, Score};
use crate::ai::generator::*;
use crate::game::action::ActionKind;
use crate::game::game::{DogGame, Game};

/// Comments by Sebastian Servos
/// This module defines the Bot trait and two implementations: RandomBot and EvalBot.
/// The Bot trait has a method choose_action that takes the current game state and a list of legal actions, and returns the chosen action.
/// RandomBot simply selects a random action from the list, while EvalBot simulates each possible action, evaluates the resulting game state using a heuristic evaluator, and selects the action with the highest score.
/// EvalBot also handles the trading phase by simulating partner actions for each traded card.
/// If no action can be chosen (e.g. all simulations fail), it falls back to selecting the first available action.

pub trait Bot {
    fn new() -> Self;

    // Returns the chosen action, or None if no actions are available.
    fn choose_action(&mut self, game: &mut Game, actions: Vec<Action>) -> Option<Action>;
}

pub struct RandomBot {
    rng: ThreadRng,
}

impl RandomBot {
    pub fn new() -> Self {
        Self { rng: rng() }
    }
}

impl Bot for RandomBot {
    fn new() -> Self {
        RandomBot::new()
    }

    fn choose_action(&mut self, _game: &mut Game, mut actions: Vec<Action>) -> Option<Action> {
        if actions.is_empty() {
            return None;
        }

        actions.shuffle(&mut self.rng);
        actions.into_iter().next()
    }
}

pub struct EvalBot {
    evaluator: Evaluator,
}

impl EvalBot {
    pub fn new() -> Self {
        Self {
            evaluator: Evaluator::new(),
        }
    }
}

impl Bot for EvalBot {
    fn new() -> Self {
        EvalBot::new()
    }

    fn choose_action(&mut self, game: &mut Game, actions: Vec<Action>) -> Option<Action> {
        if actions.is_empty() {
            return None;
        }

        // If all available actions are Remove actions, pick the card with lowest value to remove.
        if actions
            .iter()
            .all(|a| matches!(a.action, ActionKind::Remove))
        {
            return actions
                .iter()
                .min_by_key(|a| a.card.map(|c| c.value()).unwrap_or(u8::MAX))
                .cloned();
        }

        let player_index = game.current_player_index;
        let teammate_indices = game.teammate_indices(player_index);
        let opponent_indices: Vec<usize> = (0..game.players.len())
            .filter(|i| *i != player_index && !teammate_indices.contains(i))
            .collect();

        let mut best_score = Score::MIN;
        let mut best_action: Option<Action> = None;

        // Trade phase: simulate partner action for every traded card
        if game.trading_phase {
            if teammate_indices.is_empty() {
                return actions.into_iter().next();
            }
            let partner_index = teammate_indices[0];
            let mut sim_game = game.clone();
            sim_game.trading_phase = false; // Skip trading phase in simulation
            let partner_teammate_indices = sim_game.teammate_indices(partner_index);

            // Give all cards to partner & generate all actions
            let player_cards = sim_game.players[player_index].cards.clone();
            sim_game.players[partner_index].cards = player_cards;
            sim_game.current_player_index = partner_index;
            let partner_actions = generate_all_legal_actions(&sim_game);

            for action in partner_actions {
                if sim_game.action(action.card, action.clone()).is_err() {
                    continue;
                }

                let sim_context = EvalContext {
                    game: &sim_game,
                    perspective: EvalPerspective {
                        player_index: partner_index,
                        partner_indices: partner_teammate_indices.clone(),
                        opponent_indices: opponent_indices.clone(),
                    },
                };

                let score = self.evaluator.evaluate(&sim_context);

                match action.action {
                    ActionKind::Split { .. } => sim_game.undo_turn().expect("Undo must succeed"),
                    _ => sim_game.undo_action().expect("Undo must succeed"),
                };

                if score > best_score {
                    best_score = score;

                    best_action = Some(Action {
                        player: sim_game.player_by_index(player_index).color,
                        card: action.card,
                        action: ActionKind::Trade,
                    });
                }
            }
        } else {
            let all_actions = actions.clone();
            for action in all_actions {
                // Simulate action
                if game.action(action.card, action.clone()).is_err() {
                    continue;
                }

                let context = EvalContext {
                    game,
                    perspective: EvalPerspective {
                        player_index,
                        partner_indices: teammate_indices.clone(),
                        opponent_indices: opponent_indices.clone(),
                    },
                };

                let score = self.evaluator.evaluate(&context);

                match action.action {
                    ActionKind::Split { .. } => game.undo_turn().expect("Undo must succeed"),
                    _ => game.undo_action().expect("Undo must succeed"),
                };

                if score > best_score {
                    best_score = score;
                    best_action = Some(action.clone());
                }
            }
        }

        // Fallback: if no action was chosen (e.g. all simulations failed), pick the first available action.
        if best_action.is_none() {
            return actions.into_iter().next();
        }

        best_action
    }
}

#[cfg(test)]
mod random_bot_tests {
    use super::*;
    use crate::ai::generator::generate_all_legal_actions;
    use crate::game::Game;
    use crate::game::card::Card;
    use crate::game::game::GameVariant;
    use crate::game::player::PlayerType;

    #[test]
    fn bot_selects_valid_action() {
        let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
        game.trading_phase = false;

        game.players[0].cards = vec![Card::Ace];

        let mut bot = RandomBot::new();

        let actions = generate_all_legal_actions(&game);

        let action = bot.choose_action(&mut game, actions);
        assert!(action.is_some(), "Bot sollte eine Aktion wählen");
    }

    #[test]
    fn bot_returns_none_when_no_actions() {
        let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
        game.trading_phase = false;

        game.players[0].cards = vec![];

        let mut bot = RandomBot::new();
        let actions = generate_all_legal_actions(&game);

        let action = bot.choose_action(&mut game, actions);
        assert!(
            action.is_none(),
            "Bot sollte None zurückgeben, wenn keine Aktionen möglich sind"
        );
    }
}
