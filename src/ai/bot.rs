use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;
use rand::rng;


use crate::Action;



pub struct RandomBot {
    rng: ThreadRng,
}

impl RandomBot {
    pub fn new() -> Self {
        RandomBot { 
            rng: rng(),
        }
    }
}

pub trait Bot {
    fn new() -> Self;

    fn choose_action(&mut self, actions: Vec<Action>) -> Option<Action>;
}

impl Bot for RandomBot {
    fn new() -> Self {
        RandomBot::new()
    }

    fn choose_action(&mut self, mut actions: Vec<Action>) -> Option<Action> {

        if actions.is_empty() {
            return None;
        }

        actions.shuffle(&mut self.rng);
        actions.into_iter().next()
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

        let action = bot.choose_action(actions);
        assert!(action.is_some(), "Bot sollte eine Aktion wählen");
    }

    #[test]
    fn bot_returns_none_when_no_actions() {
        let mut game = Game::new(GameVariant::TwoVsTwo);
        game.trading_phase = false;

        game.players[0].cards = vec![];

        let mut bot = RandomBot::new();
        let actions = generate_all_legal_actions(&game);

        let action = bot.choose_action(actions);
        assert!(action.is_none(), "Bot sollte None zurückgeben, wenn keine Aktionen möglich sind");
    }
}
