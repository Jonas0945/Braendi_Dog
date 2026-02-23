use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;
use rand::rng;

use crate::game::game::{Game, DogGame};
use crate::Action;
use crate::ActionKind::Split;
use crate::ai::evaluator::{Evaluator, EvalContext, EvalPerspective, Score};

pub trait Bot {
    fn new() -> Self;

    fn choose_action(&mut self, game: &mut Game, actions: Vec<Action>) -> Option<Action>;
}

pub struct RandomBot {
    rng: ThreadRng,
}

impl RandomBot {
    pub fn new() -> Self {
        Self { 
            rng: rng(),
        }
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
        Self { evaluator: Evaluator::new() }
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

        let player_index = game.current_player_index;

        let teammate_indices = game.teammate_indices(player_index);

        let opponent_indices: Vec<usize> = (0..game.players.len())
            .filter(|i| {
                *i != player_index &&
                !teammate_indices.contains(i)
            })
            .collect();

        let mut best_score = Score::MIN;
        let mut best_action: Option<Action> = None;

        for action in actions {

            // Aktion simulieren
            if game.action(action.card, action.clone()).is_err() {
                continue;
            }

            let ctx = EvalContext {
                game,
                perspective: EvalPerspective {
                    player_index,
                    partner_indices: teammate_indices.clone(),
                    opponent_indices: opponent_indices.clone(),
                },
            };

            let score = self.evaluator.evaluate(&ctx);

            match action.action {
                Split { .. } => game.undo_turn().expect("Undo must succeed"),
                _ => game.undo_action().expect("Undo must succeed"),
            };

            // kompletter Zug rückgängig
            game.undo_turn().expect("Undo must succeed");

            if score > best_score {
                best_score = score;
                best_action = Some(action.clone());
            }
        }

        best_action
    }
}

#[cfg(test)]
mod random_bot_tests {
    use super::*;
    use crate::ai::generator::generate_all_legal_actions;
    use crate::game::card::Card;
    use crate::game::game::GameVariant;
    use crate::game::Game;

    #[test]
    fn bot_selects_valid_action() {
        let mut game = Game::new(GameVariant::TwoVsTwo);
        game.trading_phase = false;

        game.players[0].cards = vec![Card::Ace];

        let mut bot = RandomBot::new();

        let actions = generate_all_legal_actions(&game);

        let action = bot.choose_action(&mut game, actions);
        assert!(action.is_some(), "Bot sollte eine Aktion wählen");
    }

    #[test]
    fn bot_returns_none_when_no_actions() {
        let mut game = Game::new(GameVariant::TwoVsTwo);
        game.trading_phase = false;

        game.players[0].cards = vec![];

        let mut bot = RandomBot::new();
        let actions = generate_all_legal_actions(&game);

        let action = bot.choose_action(&mut game, actions);
        assert!(action.is_none(), "Bot sollte None zurückgeben, wenn keine Aktionen möglich sind");
    }
}
