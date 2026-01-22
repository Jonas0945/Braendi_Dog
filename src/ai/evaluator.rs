use crate::game::{Game};

pub type Score = i32;

pub struct EvalPerspective {
    pub player_index: usize,
    pub partner_indices: Vec<usize>,
    pub opponent_indices: Vec<usize>,
}

pub struct EvalContext<'a> {
    pub game: &'a Game,
    pub perspective: EvalPerspective,
}

enum EvalFeature {
    House,
    Progress,
    Mobility,
    Risk,
    Teamplay,
}

impl EvalFeature {
    fn evaluate(&self, context: &EvalContext) -> Score {
        match self {
            EvalFeature::House => {
                /*
                Own piece in house: +1000
                Partner piece in house: +400
                Opponent piece in house: -800
                Result: Self > team > opponent
                */
                
                let game = context.game;
                let p = &context.perspective;

                let mut score: Score = 0;

                let own_house = 
                    game.players[p.player_index].pieces_in_house as Score;
                
                score += own_house * 1000;

                for &partner in &p.partner_indices {
                    let count = game.players[partner].pieces_in_house as Score;
                    score += count * 400;
                }

                for &opponent in &p.opponent_indices {
                    let count = game.players[opponent].pieces_in_house as Score;
                    score -= count * 800;
                }
                
                score
            },
            _ => todo!()
        }
    }
}

pub struct Evaluator {
    features: Vec<EvalFeature>,
}

impl Evaluator {
    pub fn new_default() -> Self {
        Self {
            features: vec![
                EvalFeature::House,
                EvalFeature::Progress,
                EvalFeature::Mobility,
                EvalFeature::Risk,
                EvalFeature::Teamplay,
            ],
        }
    }

    pub fn evaluate(&self, ctx: &EvalContext) -> Score {
        self.features
            .iter()
            .map(|f| f.evaluate(ctx))
            .sum()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::game::GameVariant;

    mod eval_house_tests {
        use super::*;

        #[test]
        fn house_feature_basic_scoring() {
            let mut game = Game::new(GameVariant::TwoVsTwo);
            game.players[0].pieces_in_house = 2;
            game.players[1].pieces_in_house = 1;
            game.players[2].pieces_in_house = 1;
            game.players[3].pieces_in_house = 3;

            let ctx = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::House;
            let score = feature.evaluate(&ctx);

            assert_eq!(score, 2 * 1000 + 1 * 400 - (1 + 3) * 800);
        }
    }
}