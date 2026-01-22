use crate::game::{Game};
use crate::game::card::Card;

pub type Score = i32;

fn piece_has_any_move(game: &Game, from: usize, owner: usize) -> bool {
    for card in &game.players[owner].cards {
        if !card.is_move_card() {
            continue;
        }

        for dist in card.possible_distances() {
            if game.can_piece_move_distance(from, dist, false) {
                return true;
            }

            if card.is_backward_move_card()
                && game.can_piece_move_distance(from, dist, true)
            {
                return true;
            }
        }
    }

    false
}


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
    House,          // Evaluates current pieces_in_house
    BoardProgress,  // Evaluates distance towards house_tiles
    Mobility,       // Evaluates if piece is in range of being blocked
    Risk,           // Evaluates if piece is in range of being captured
    Teamplay,       // Evaluates if teamplay is more beneficial
}

impl EvalFeature {
    
    
    fn evaluate(&self, context: &EvalContext) -> Score {
        match self {
            EvalFeature::House => {
                /*
                Own piece in house: +1000
                Partner piece in house: +400
                Opponent piece in house: -800
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

            EvalFeature::BoardProgress => {
                /*
                Own piece: +5 per tile
                Team piece: +2 per tile
                Opponent piece: -4 per tile
                 */

                let game = context.game;
                let p = &context.perspective;
                let board = &game.board;

                let ring_size = board.ring_size as i32;           
                let mut score: Score = 0;

                for (from_index, tile) in board.tiles.iter().enumerate() {
                    let Some(piece) = tile else { continue };

                    let owner = piece.owner;

                    if from_index >= board.ring_size {
                        continue;
                    }

                    let house_entry = board.start_field(owner);

                    let Some(distance) = 
                        board.distance_between(from_index, house_entry, owner)
                    else {
                        continue;
                    };

                    let progress = ring_size - distance as i32;

                    if owner == p.player_index {
                        score += progress * 5;
                    } else if p.partner_indices.contains(&owner) {
                        score += progress * 2;
                    } else if p.opponent_indices.contains(&owner) {
                        score -= progress * 4;
                    }
                }

                score
            },

            EvalFeature::Mobility => {
                /*
                Piece can move: +10
                Piece blocked: -15
                Only one piece on board: -20
                 */
                let game = context.game;
                let p = &context.perspective;
                let board = &game.board;

                let mut relevant_own = 4;
                let mut movable_own = 0;
                let mut blocked_own = 0;

                for (index, tile) in board.tiles.iter().enumerate() {
                    let Some(piece) = tile else { continue };

                    let owner = piece.owner;

                    if owner != p.player_index {
                        continue;
                    }

                    if index >= game.board.ring_size {
                        continue;
                    }

                    // Piece is already in house
                    relevant_own -= 1;

                    if piece_has_any_move(game, index, owner) {
                        movable_own += 1;
                    } else {
                        blocked_own += 1;
                    }
                }

                let mut score = 0;

                score += movable_own * 10;
                score -= blocked_own * 15;

                // Only one piece on board
                if movable_own == 1 && relevant_own >= 2  {
                    score -= 20;
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
                EvalFeature::BoardProgress,
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
    use crate::game::Piece;

    mod eval_house_tests {
        use super::*;

        #[test]
        fn house_feature_basic_scoring() {
            let mut game = Game::new(GameVariant::TwoVsTwo);
            game.players[0].pieces_in_house = 2;
            game.players[1].pieces_in_house = 1;
            game.players[2].pieces_in_house = 1;
            game.players[3].pieces_in_house = 3;

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::House;
            let score = feature.evaluate(&context);

            assert_eq!(score, 2 * 1000 + 1 * 400 - (1 + 3) * 800);
        }
    }

    mod eval_board_progress_tests {
        use super::*;

        #[test]
        fn board_progress_basic_scoring() {
            let mut game = Game::new(GameVariant::TwoVsTwo);

            game.board.tiles[62] = Some(Piece { 
                owner: 0, 
                left_start: true 
            });

            game.board.tiles [20] = Some(Piece { 
                owner: 2, 
                left_start: true 
            });

            game.board.tiles [5] = Some(Piece { 
                owner: 1, 
                left_start: true 
            });            

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::BoardProgress;
            let score = feature.evaluate(&context);

            assert_eq!(score, 5 * (64 - 2) + 2 * (64 - 12) - 4 * (64 - 11));
        }
    }

    mod eval_mobility_tests {
        use super::*;

        #[test]
        fn mobility_basic_scoring() {
            let mut game = Game::new(GameVariant::TwoVsTwo);

            game.players[0].cards = vec![
                Card::Two,
                Card::Three,
                Card::Five,
            ];

            game.board.tiles[10] = Some(Piece { 
                owner: 0, 
                left_start: true 
            });

            game.board.tiles[20] = Some(Piece { 
                owner: 0, 
                left_start: true 
            });

            game.board.tiles[21] = Some(Piece { 
                owner: 1, 
                left_start: false 
            });

            game.board.tiles[30] = Some(Piece { 
                owner: 0, 
                left_start: true 
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Mobility;
            let score = feature.evaluate(&context);

            // Two pieces can move, one blocked
            assert_eq!(score, 2 * 10 - 1 * 15);
        }

        #[test]
        fn mobility_only_one_piece_penalty() {
            let mut game = Game::new(GameVariant::TwoVsTwo);

            game.players[0].cards = vec![
                Card::Two,
                Card::Three,
                Card::Five,
            ];

            game.board.tiles[10] = Some(Piece { 
                owner: 0, 
                left_start: true 
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Mobility;
            let score = feature.evaluate(&context);

            // Only one piece on board penalty
            assert_eq!(score, 1 * 10 - 20);
        }

        #[test]
        fn mobility_no_relevant_pieces() {
            let game = Game::new(GameVariant::TwoVsTwo);

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Mobility;
            let score = feature.evaluate(&context);

            // No relevant pieces should yield zero score
            assert_eq!(score, 0);
        }

        #[test]
        fn all_pieces_in_house_no_penalty() {
            let mut game = Game::new(GameVariant::TwoVsTwo);

            game.players[0].pieces_in_house = 4;

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Mobility;
            let score = feature.evaluate(&context);

            // All pieces in house should yield zero score
            assert_eq!(score, 0);
        }

        #[test]
        fn mobility_all_pieces_blocked() {
            let mut game = Game::new(GameVariant::TwoVsTwo);

            game.players[0].cards = vec![
                Card::Two,
            ];

            game.board.tiles[10] = Some(Piece { 
                owner: 0, 
                left_start: true 
            });

            game.board.tiles[12] = Some(Piece { 
                owner: 1, 
                left_start: false 
            });

            game.board.tiles[20] = Some(Piece { 
                owner: 0, 
                left_start: true 
            });

            game.board.tiles[22] = Some(Piece { 
                owner: 1, 
                left_start: false 
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Mobility;
            let score = feature.evaluate(&context);

            // Both pieces blocked
            assert_eq!(score, -2 * 15);
        }
    }
}