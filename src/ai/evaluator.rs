use crate::game::game::GameVariant;
use crate::game::Game;
use crate::game::board_view::collect_board_pieces;

/// Comments by Sebastian Servos
/// This module defines the evaluation logic for the EvalBot. 
/// It includes the EvalFeature enum, which represents different aspects of the game state that can be evaluated (e.g. progress towards winning, mobility, risk). 
/// Each feature has a method to evaluate its score based on the current game state and the bot's perspective. 
/// The Evaluator struct combines multiple features to assign an overall score to a game state from the perspective of a player. 
/// The score is used by EvalBot to choose the best action during its turn. 
/// The module also includes helper functions to determine if pieces have legal moves and if they are under threat from opponents.

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

// Forward threat: opponent can reach target within 13 tiles
// Backward threat: opponent can reach target exactly 4 tiles back
fn is_threat(game: &Game, opponent_position: usize, opponent_index: usize, target_position:usize, target_index: usize) -> bool {
    let board = &game.board;

    if target_position >= board.ring_size {
        return false;
    }

    let Some(target_piece) = board.tiles[target_position].as_ref() else {
        return false;
    };

    if !target_piece.left_start {
        return false;
    }

    if opponent_position >= board.ring_size {
        return false;
    }

    // Check forward moves
    if let Some(forward_distance) = board.distance_between(opponent_position, target_position, opponent_index) {
        if forward_distance <= 13 {
            if let Some(forward_path) = board.passed_tiles(opponent_position, target_position, opponent_index, false) {
                if board.is_path_free(&forward_path) {
                    return true;
                }
            }
        }
    }

    // Check backward move
    if let Some(backward_distance) = board.distance_between(target_position, opponent_position, target_index) {
        if backward_distance == 4 {
            if let Some(backward_path) = board.passed_tiles(opponent_position, target_position, opponent_index, true) {
                if board.is_path_free(&backward_path) {
                    return true;
                }
            }
        }
    }

    false
}

// Perspective for evaluation: which player is the bot, who are the teammates and who are the opponents.
pub struct EvalPerspective {
    pub player_index: usize,
    pub partner_indices: Vec<usize>,
    pub opponent_indices: Vec<usize>,
}

// Context for evaluation: the game state and the perspective.
pub struct EvalContext<'a> {
    pub game: &'a Game,
    pub perspective: EvalPerspective,
}

// Aspect of the evaluation that assigns a score to a game state from the perspective of a player.
enum EvalFeature {
    HouseProgress,  // Evaluates current pieces_in_house
    HouseMobility,  // Evaluates if pieces can enter/move in house
    BoardProgress,  // Evaluates distance towards house_tiles
    BoardMobility,  // Evaluates if piece is in range of being blocked
    Risk,           // Evaluates if piece is in range of being captured
    Teamplay,       // Evaluates if teamplay is more beneficial
}

impl EvalFeature {
    fn evaluate(&self, context: &EvalContext) -> Score {
        let game = context.game;
        let p = &context.perspective;
        let board = &game.board;
        let ring_size = board.ring_size;

        let mut score = 0;

        let board_pieces = collect_board_pieces(game);

        let own_board_pieces: Vec<_> = board_pieces
            .iter()
            .filter(|bp| bp.owner == p.player_index)
            .collect();

        let partner_board_pieces: Vec<_> = board_pieces
            .iter()
            .filter(|bp| p.partner_indices.contains(&bp.owner))
            .collect();

        let opponent_board_pieces: Vec<_> = board_pieces
            .iter()
            .filter(|bp| p.opponent_indices.contains(&bp.owner))
            .collect();
        
        match self {
            EvalFeature::HouseProgress => {
                /*
                Own piece in house: +1000
                Partner piece in house: +400
                Opponent piece in house: -800
                */

                let own_house = game.players[p.player_index].pieces_in_house as Score;
                
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

            EvalFeature::HouseMobility => {
                /*
                Every piece is at the correct position: +40
                Open space between pieces (further up gets worse score): - 25 * depth of tile 
                */

                let house_tiles = game.board.house_by_player(p.player_index);
                let pieces_in_house = game.players[p.player_index].pieces_in_house as usize;

                if pieces_in_house == 0 {
                    return score
                };

                let mut seen_pieces = 0;

                for (depth, &tile_index) in house_tiles.iter().enumerate().rev() {
                    if seen_pieces == pieces_in_house {
                        continue;
                    }

                    if game.board.tiles[tile_index].is_some() {
                        seen_pieces += 1;
                    } else {
                        score -= depth as Score * 25;
                    }
                }

                // Pieces at the correct position
                if score == 0 {
                    score += 40;
                } 

                score
            },

            EvalFeature::BoardProgress => {
                /*
                Own piece: +5 per tile
                Team piece: +2 per tile
                Opponent piece: -4 per tile
                 */

                for piece in &board_pieces {
                    if piece.position >= ring_size {
                        continue
                    };

                    let owner = piece.owner;
                    let house_entry = board.start_field(owner);

                    let Some(distance) = 
                        board.distance_between(piece.position, house_entry, owner)
                    else {
                        continue;
                    };

                    let progress = ring_size as i32 - distance as i32;

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

            EvalFeature::BoardMobility => {
                /*
                Piece can move: +10
                Piece blocked: -15
                Only one piece on board: -20
                 */

                let mut relevant_pieces = 4;
                let mut movable_pieces = 0;
                let mut blocked_pieces = 0;

                for own in &own_board_pieces {
                    if own.position >= ring_size {
                        relevant_pieces -= 1;
                        continue;
                    }

                    if piece_has_any_move(game, own.position, own.owner) {
                        movable_pieces += 1;
                    } else {
                        blocked_pieces += 1;
                    }
                }

                score += movable_pieces * 10;
                score -= blocked_pieces * 15;

                // Only one piece on board
                if movable_pieces == 1 && relevant_pieces >= 2  {
                    score -= 20;
                }

                score
            },

            EvalFeature::Risk => {
                /*
                Piece can be taken by other piece (up to 13 forwards, exactly 4 backwards): -40 per threat
                */

                for own in &own_board_pieces {
                    if own.position >= ring_size || !own.left_start {
                        continue;
                    }

                    for opponent in &opponent_board_pieces {
                        if opponent.position >= ring_size {
                            continue;
                        }

                        if is_threat(game, opponent.position, opponent.owner, own.position, own.owner) {
                            score -= 40;
                        }
                    }
                }

                score
            },

            EvalFeature::Teamplay => {
                /*
                Team piece is in house: +30
                Team piece is safe on ring (just started or no threat): +10
                Team piece can be taken: -40 per threat
                */

                match game.game_variant {
                    GameVariant::FreeForAll(_) => return score,
                    _ => {}
                }

                for partner in &partner_board_pieces {
                    if !partner.left_start {
                        score += 10;
                        continue;
                    }

                    if partner.position >= ring_size {
                        score += 30;
                        continue;
                    }

                    let mut threat_count = 0;

                    for opponent in &opponent_board_pieces {
                        if opponent.position >= ring_size {
                            continue;
                        }

                        if is_threat(game, opponent.position, opponent.owner, partner.position, partner.owner) {
                            threat_count += 1;
                        }
                    }

                    if threat_count >= 1 {
                        score -= threat_count * 40;
                    } else {
                        score += 10;
                    }
                }

                score
            },
        }
    }
}

// Evaluator that combines multiple features to assign a score to a game state from the perspective of a player.
pub struct Evaluator {
    features: Vec<EvalFeature>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            features: vec![
                EvalFeature::HouseProgress,
                EvalFeature::HouseMobility,
                EvalFeature::BoardProgress,
                EvalFeature::BoardMobility,
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
    use crate::game::card::Card;
    use crate::game::player::PlayerType;

    mod eval_house_progress_tests {
        use super::*;

        #[test]
        fn house_progress_feature_basic_scoring() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);
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

            let feature = EvalFeature::HouseProgress;
            let score = feature.evaluate(&context);

            assert_eq!(score, 2 * 1000 + 1 * 400 - (1 + 3) * 800);
        }
    }

    mod eval_house_mobility_tests {
        use super::*;

        #[test]
        fn house_mobility_no_pieces_in_house() {
            let game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::HouseMobility;
            let score = feature.evaluate(&context);

            assert_eq!(score, 0);
        }

        #[test]
        fn house_mobility_all_filled() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            let house = game.board.house_by_player(0);
            for &idx in &house {
                game.board.tiles[idx] = Some(Piece {
                    owner: 0,
                    left_start: true,
                });
            }

            game.players[0].pieces_in_house = house.len() as u8;

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::HouseMobility;
            let score = feature.evaluate(&context);

            assert_eq!(score, 40);
        }

        #[test]
        fn house_mobility_single_gap() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.players[0].pieces_in_house = 1;
            let house = game.board.house_by_player(0);

            game.board.tiles[house[2]] = Some(Piece {
                owner: 0,
                left_start: true,
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::HouseMobility;
            let score = feature.evaluate(&context);

            assert_eq!(score, -75);

        }
        
        #[test]
        fn house_mobility_double_gap() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.players[0].pieces_in_house = 2;
            let house = game.board.house_by_player(0);

            game.board.tiles[house[2]] = Some(Piece {
                owner: 0,
                left_start: true,
            });

            game.board.tiles[house[0]] = Some(Piece {
                owner: 0,
                left_start: true,
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::HouseMobility;
            let score = feature.evaluate(&context);

            assert_eq!(score, -25 - 75);

        }

        #[test]
        fn house_triple_piece_no_gap() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.players[0].pieces_in_house = 3;
            let house = game.board.house_by_player(0);

            game.board.tiles[house[3]] = Some(Piece {
                owner: 0,
                left_start: true,
            });

            game.board.tiles[house[2]] = Some(Piece {
                owner: 0,
                left_start: true,
            });

            game.board.tiles[house[1]] = Some(Piece {
                owner: 0,
                left_start: true,
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::HouseMobility;
            let score = feature.evaluate(&context);

            assert_eq!(score, 40);

        }

        #[test]
        fn house_triple_piece_single_best_gap() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.players[0].pieces_in_house = 3;
            let house = game.board.house_by_player(0);

            game.board.tiles[house[3]] = Some(Piece {
                owner: 0,
                left_start: true,
            });

            game.board.tiles[house[2]] = Some(Piece {
                owner: 0,
                left_start: true,
            });

            game.board.tiles[house[0]] = Some(Piece {
                owner: 0,
                left_start: true,
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::HouseMobility;
            let score = feature.evaluate(&context);

            assert_eq!(score, -25);

        }

        #[test]
        fn house_triple_piece_single_middle_gap() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.players[0].pieces_in_house = 3;
            let house = game.board.house_by_player(0);

            game.board.tiles[house[3]] = Some(Piece {
                owner: 0,
                left_start: true,
            });

            game.board.tiles[house[1]] = Some(Piece {
                owner: 0,
                left_start: true,
            });

            game.board.tiles[house[0]] = Some(Piece {
                owner: 0,
                left_start: true,
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::HouseMobility;
            let score = feature.evaluate(&context);

            assert_eq!(score, -50);

        }
    }
    mod eval_board_progress_tests {
        use super::*;

        #[test]
        fn board_progress_basic_scoring() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

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

    mod eval_board_mobility_tests {
        use super::*;

        #[test]
        fn board_mobility_basic_scoring() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

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

            let feature = EvalFeature::BoardMobility;
            let score = feature.evaluate(&context);

            // Two pieces can move, one blocked
            assert_eq!(score, 2 * 10 - 1 * 15);
        }

        #[test]
        fn board_mobility_only_one_piece_penalty() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

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

            let feature = EvalFeature::BoardMobility;
            let score = feature.evaluate(&context);

            // Only one piece on board penalty
            assert_eq!(score, 1 * 10 - 20);
        }

        #[test]
        fn board_mobility_no_relevant_pieces() {
            let game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::BoardMobility;
            let score = feature.evaluate(&context);

            // No relevant pieces should yield zero score
            assert_eq!(score, 0);
        }

        #[test]
        fn all_pieces_in_house_no_penalty() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.players[0].pieces_in_house = 4;

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::BoardMobility;
            let score = feature.evaluate(&context);

            // All pieces in house should yield zero score
            assert_eq!(score, 0);
        }

        #[test]
        fn board_mobility_all_pieces_blocked() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

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

            let feature = EvalFeature::BoardMobility;
            let score = feature.evaluate(&context);

            // Both pieces blocked
            assert_eq!(score, -2 * 15);
        }
    }

    mod eval_risk_tests {
        use super::*;

        #[test]
        fn risk_no_threats() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.board.tiles[0] = Some(Piece { owner: 0, left_start: true });
            game.board.tiles[32] = Some(Piece { owner: 1, left_start: true });
            game.board.tiles[48] = Some(Piece { owner: 2, left_start: true });
            game.board.tiles[16] = Some(Piece { owner: 3, left_start: true });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Risk;
            let score = feature.evaluate(&context);
            assert_eq!(score, 0);
        }

        #[test]
        fn risk_forward_threat() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.board.tiles[0] = Some(Piece { owner: 1, left_start: true });
            game.board.tiles[10] = Some(Piece { owner: 0, left_start: true });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Risk;
            let score = feature.evaluate(&context);
            assert_eq!(score, -40);
        }

        #[test]
        fn risk_backward_threat() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.board.tiles[20] = Some(Piece { owner: 0, left_start: true });
            game.board.tiles[24] = Some(Piece { owner: 1, left_start: true });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Risk;
            let score = feature.evaluate(&context);
            assert_eq!(score, -40);
        }

        #[test]
        fn risk_multiple_threats() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.board.tiles[12] = Some(Piece { owner: 0, left_start: true });
            game.board.tiles[4] = Some(Piece { owner: 1, left_start: true });
            game.board.tiles[0] = Some(Piece { owner: 3, left_start: true });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Risk;
            let score = feature.evaluate(&context);
            assert_eq!(score, -80);
        }

        #[test]
        fn risk_piece_in_house_and_blocked_safe() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.board.tiles[64] = Some(Piece { owner: 0, left_start: true });
            game.board.tiles[4] = Some(Piece { owner: 0, left_start: false });

            game.board.tiles[63] = Some(Piece { owner: 1, left_start: true });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Risk;
            let score = feature.evaluate(&context);
            assert_eq!(score, 0);
        }
    }

    mod eval_teamplay_tests {
        use super::*;

        #[test]
        fn teamplay_free_for_all_returns_zero() {
            let mut game = Game::new(GameVariant::FreeForAll(4), vec![PlayerType::Human; 4]);

            game.board.tiles[4] = Some(Piece { owner: 0, left_start: false });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Teamplay;
            let score = feature.evaluate(&context);

            assert_eq!(score, 0);
        }

        #[test]
        fn teamplay_partner_piece_in_house_scores_bonus() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.board.tiles[64] = Some(Piece {
                owner: 2,
                left_start: true,
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Teamplay;
            let score = feature.evaluate(&context);

            assert_eq!(score, 30);
        }

        #[test]
        fn teamplay_safe_partner_piece_on_ring_scores_small_bonus() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.board.tiles[10] = Some(Piece {
                owner: 2,
                left_start: true,
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Teamplay;
            let score = feature.evaluate(&context);

            assert_eq!(score, 10);
        }

        #[test]
        fn teamplay_partner_piece_with_single_threat_is_penalized() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.board.tiles[10] = Some(Piece {
                owner: 2,
                left_start: true,
            });

            game.board.tiles[8] = Some(Piece {
                owner: 1,
                left_start: true,
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Teamplay;
            let score = feature.evaluate(&context);

            assert_eq!(score, -40);
        }

        #[test]
        fn teamplay_partner_piece_with_multiple_threats() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.board.tiles[10] = Some(Piece {
                owner: 2,
                left_start: true,
            });

            game.board.tiles[8] = Some(Piece {
                owner: 1,
                left_start: true,
            });

            game.board.tiles[6] = Some(Piece {
                owner: 3,
                left_start: true,
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Teamplay;
            let score = feature.evaluate(&context);

            assert_eq!(score, -80);
        }

        #[test]
        fn teamplay_multiple_partner_pieces_are_aggregated() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.board.tiles[10] = Some(Piece {
                owner: 2,
                left_start: true,
            });

            game.board.tiles[68] = Some(Piece {
                owner: 2,
                left_start: true,
            });

            game.board.tiles[8] = Some(Piece {
                owner: 1,
                left_start: true,
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Teamplay;
            let score = feature.evaluate(&context);

            assert_eq!(score, -10);
        }


        #[test]
        fn teamplay_piece_block() {
            let mut game = Game::new(GameVariant::TwoVsTwo, vec![PlayerType::Human; 4]);

            game.board.tiles[10] = Some(Piece {
                owner: 2,
                left_start: true,
            });

            game.board.tiles[8] = Some(Piece {
                owner: 0,
                left_start: false,
            });

            game.board.tiles[6] = Some(Piece {
                owner: 1,
                left_start: true,
            });

            let context = EvalContext {
                game: &game,
                perspective: EvalPerspective {
                    player_index: 0,
                    partner_indices: vec![2],
                    opponent_indices: vec![1, 3],
                },
            };

            let feature = EvalFeature::Teamplay;
            let score = feature.evaluate(&context);

            assert_eq!(score, 10);
        }
    }
}