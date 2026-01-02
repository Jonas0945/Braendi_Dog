/// Pro Spieler werden 16 Felder ausgehend vom Start bis zum nächsten Startfeld kalkuliert
/// Die Ringgröße entscheidet über die Position der HOUSE_TILES, statt sie fest als Konstante zu schreiben.


use super::piece::Piece;

pub type Point = usize; // 0–79
pub const HOUSE_SIZE: usize = 4;

pub struct Board {
    pub tiles: Vec<Option<Piece>>,
    pub ring_size: usize,
}

impl Board {
    pub fn new(num_players: usize) -> Self {
        let ring_size = num_players * 16;
        let total_tiles = ring_size + num_players * HOUSE_SIZE;

        Self {
            tiles: vec![None; total_tiles],
            ring_size,
        }
    }

    pub fn get_board(&self) -> &Vec<Option<Piece>> {
        &self.tiles
    }

    pub fn check_tile(&self, p: Point) -> Option<Piece> {
        self.tiles[p]
    }

    pub fn start_field(&self,player_index: usize) -> Point {
        player_index * 16
    }

    pub fn house_gateway(&self, player_index: usize) -> Point {
        self.ring_size + player_index * HOUSE_SIZE
    }

    pub fn house_by_player(&self, player_index: usize) -> Vec<usize> {
        let start = self.house_gateway(player_index);
        (start..start + HOUSE_SIZE).collect()
    }

    pub fn distance_between(&self, from: usize, to: usize, player_index: usize) -> Option<u8> {
        let ring_size = self.ring_size;
        let num_players = ring_size / 16;
        let total_tiles = self.tiles.len();

        if player_index >= num_players {
            return None;
        }

        // Check for out-of-bounds positions
        if from >= total_tiles || to >= total_tiles {
            return None;
        }

        let house = self.house_by_player(player_index);

        // Piece already in house
        if from >= ring_size {
            
            // Check correct in-house movement
            if !house.contains(&from) || !house.contains(&to) || to < from {
                return None;
            }

            return (to - from).try_into().ok();
        }
        
        // Moving from the ring into the house
        if to >= ring_size {
            
            if !house.contains(&to) {
                return None; 
            }

            let house_entry = self.start_field(player_index); // Equals start_field
            let distance_to_house_entry = (house_entry + ring_size - from) % ring_size;
            let steps_in_house = to - self.house_gateway(player_index);

            // Total distance
            return (distance_to_house_entry + 1 + steps_in_house)
                .try_into()
                .ok();
        } 
        
        // Moving around the ring
            let distance = (to + ring_size - from) % ring_size;
            Some(distance as u8)
    }

    pub fn passed_tiles(&self, from: usize, to: usize, player_index: usize, backward: bool) -> Option<Vec<Point>> {
        let ring_size = self.ring_size;
        let num_players = ring_size / 16;

        if player_index >= num_players {
            return None;
        }

        let total_tiles = self.tiles.len();
        let mut passed_tiles = Vec::new();
        let mut current_position = from;

        if from >= total_tiles || to >= total_tiles {
            return None;
        }

        let distance = if backward {
            self.distance_between(to, from, player_index)?
        } else {
            self.distance_between(from, to, player_index)?
        };

        // Piece already in house
        if from >= ring_size {
            
            // Backward move not allowed in-house
            if backward {
                return None;
            }

            for pos in (from + 1)..= to {
                passed_tiles.push(pos);
            }

            return Some(passed_tiles);
        }
        
        // Moving from the ring into the house
        if to >= ring_size {

            // Backward move into house not allowed 
            if backward {
                return None;
            }

            let house_entry = self.start_field(player_index);
            let house_gateway = self.house_gateway(player_index);

            // Add tiles to house entry
            while current_position != house_entry {
                current_position = (current_position + 1) % ring_size;
                passed_tiles.push(current_position);
            }

            // Add first tile in house
            passed_tiles.push(house_gateway);

            for pos in (house_gateway + 1)..= to {
                passed_tiles.push(pos);
            }

            return Some(passed_tiles);
        }

        // Normal ring movement   
        for _ in 0..distance {
            current_position = if backward {
                (current_position + ring_size - 1) % ring_size
            } else {
                (current_position + 1) % ring_size
            };
            passed_tiles.push(current_position);
        }

        Some(passed_tiles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod board_tests {
        use super::*;

        #[test]
        fn board_sizes_for_2_to_6_players() {
            for players in 2..=6 {
                let board = Board::new(players);

                assert_eq!(board.ring_size, players * 16);
                assert_eq!(
                    board.tiles.len(),
                    players * 16 + players * HOUSE_SIZE
                );
            }
        }

        #[test]
        fn start_and_house_fields_do_not_overlap() {
            for players in 2..=6 {
                let board = Board::new(players);

                for p in 0..players {
                    let start = board.start_field(p);
                    assert!(start < board.ring_size);

                    let house = board.house_by_player(p);
                    for h in house {
                        assert!(h >= board.ring_size);
                    }
                }
            }
        }

        #[test]
        fn ring_wrap_works_for_all_player_counts() {
            for players in 2..=6 {
                let board = Board::new(players);
                let last = board.ring_size - 1;

                assert_eq!(
                    board.distance_between(last, 0, 0),
                    Some(1)
                );
            }
        }

        #[test]
        fn invalid_player_index_returns_none() {
            let board = Board::new(4);

            assert_eq!(board.distance_between(0, 1, 4), None);
            assert_eq!(board.passed_tiles(0, 1, 4, false), None);
        }
    }

    mod distance_between_tests {
        use super::*;

        #[test]
        fn ring() {
            let board = Board::new(4);

            assert_eq!(board.distance_between(0, 3, 0), Some(3));
            assert_eq!(board.distance_between(62, 2, 0), Some(4));
        }

        #[test]
        fn into_house() {
            let board = Board::new(4);

            assert_eq!(board.distance_between(62, 64, 0), Some(3));
        }

        #[test]
        fn wrong_house() {
            let board = Board::new(4);

            assert_eq!(board.distance_between(62, 64, 1), None);
        }

        #[test]
        fn inside_house() {
            let board = Board::new(4);
            let house = board.house_by_player(1);

            assert_eq!(board.distance_between(house[0], house[2], 1), Some(2));
        }

        #[test]
        fn invalid_positions() {
            let board = Board::new(4);
            
            assert_eq!(board.distance_between(80, 0, 0), None);
            assert_eq!(board.distance_between(0, 80, 0), None);
            assert_eq!(board.distance_between(81, 90, 2), None);
        }

        #[test]
        fn every_player_can_enter_own_house() {
            for players in 2..=6 {
                let board = Board::new(players);

                for p in 0..players {
                    let from = (board.start_field(p) + board.ring_size - 2) % board.ring_size;
                    let to = board.house_gateway(p);

                    assert!(board.distance_between(from, to, p).is_some());
                }
            }
        }

        #[test]
        fn house_entry_only_via_own_start_field() {
            let board = Board::new(4);

            for p in 0..4 {
                let wrong_from = (board.start_field(p) + 1) % board.ring_size;
                let to = board.house_gateway(p);

                assert_eq!(
                    board.distance_between(wrong_from, to, p),
                    Some(board.ring_size as u8)
                );
            }
        }

    }
    
    mod passed_tiles_tests {
        use super::*;

        #[test]
        fn ring_forward() {
            let board = Board::new(4);
            
            assert_eq!(
                board.passed_tiles(0, 3, 0, false),
                Some(vec![1, 2, 3])
            );
            assert_eq!(
                board.passed_tiles(62, 2, 0, false),
                Some(vec![63, 0, 1, 2])
            );
        }

        #[test]
        fn ring_backward() {
            let board = Board::new(4);
            
            assert_eq!(
                board.passed_tiles(3, 0, 0, true),
                Some(vec![2, 1, 0])
            );
            assert_eq!(
                board.passed_tiles(2, 62, 0, true),
                Some(vec![1, 0, 63, 62])
            );
        }

        #[test]
        fn zero_distance_move() {
            let board = Board::new(4);

            assert_eq!(board.distance_between(5, 5, 0), Some(0));
            assert_eq!(board.passed_tiles(5, 5, 0, false), Some(vec![]));
        }

        #[test]
        fn inside_house_forward() {
            let board = Board::new(4);
            let house = board.house_by_player(2);

            assert_eq!(
                board.passed_tiles(house[0], house[2], 2, false),
                Some(vec![house[1], house[2]])
            );
        }

        #[test]
        fn inside_house_backward() {
            let board = Board::new(4);
            let house = board.house_by_player(2);

            assert_eq!(
                board.passed_tiles(house[2], house[1], 2, true),
                None
            );
        }

        #[test]
        fn into_house_forward() {
            let board = Board::new(4);

            assert_eq!(
                board.passed_tiles(62, 64, 0, false),
                Some(vec![63, 0, 64])
            );
        }

        #[test]
        fn into_house_backward() {
            let board = Board::new(4);

            assert_eq!(
                board.passed_tiles(64, 62, 0, true),
                None
            );
        }

        #[test]
        fn invalid_positions() {
            let board = Board::new(4);
            
            assert_eq!(board.passed_tiles(80, 0, 0, false), None);
            assert_eq!(board.passed_tiles(0, 80, 0, false), None);
            assert_eq!(board.passed_tiles(81, 90, 2, false), None);
        }

        #[test]
        fn distance_matches_passed_tiles_length() {
            for players in 2..=6 {
                let board = Board::new(players);

                for p in 0..players {
                    let from = board.start_field(p);
                    let to = (from + 5) % board.ring_size;

                    let dist = board.distance_between(from, to, p).unwrap();
                    let passed = board.passed_tiles(from, to, p, false).unwrap();

                    assert_eq!(passed.len(), dist as usize);
                }
            }
        }

        #[test]
        fn passed_tiles_always_end_on_target() {
            let board = Board::new(4);

            let cases = [
                (0, 5, 0, false),
                (5, 0, 0, true),
                (62, 64, 0, false),
            ];

            for (from, to, p, backward) in cases {
                let passed = board.passed_tiles(from, to, p, backward).unwrap();
                assert_eq!(passed.last(), Some(&to));
            }
        }

        #[test]
        fn cannot_pass_through_foreign_house_tiles() {
            let board = Board::new(4);

            let from = 15;
            let to = 20;

            let passed = board.passed_tiles(from, to, 0, false).unwrap();

            for tile in passed {
                assert!(tile < board.ring_size);
            }
        }

        #[test]
        fn backward_is_inverse_of_forward_on_ring() {
            let board = Board::new(4);

            let from = 10;
            let to = 25;

            let forward = board.passed_tiles(from, to - 1, 0, false).unwrap();
            let backward = board.passed_tiles(to, from + 1, 0, true).unwrap();

            let reversed: Vec<_> = backward.into_iter().rev().collect();
            assert_eq!(forward, reversed);
        }
    }
}