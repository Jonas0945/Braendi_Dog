// 56––57––58––59––60––61––62––63–– 0–– 1–– 2–– 3–– 4–– 5–– 6––7
// |                               |                           |
// 55                              64                          8
// |                               |                           |
// 54                              65                          9
// |                               |                           |
// 53                              66                          10
// |                               |                           |
// 52                              67                          11
// |                                                           |
// 51                            RED                           12
// |                                                           |
// 50                                                          13
// |                                                           |
// 49                                                          14
// |              YELLOW                                       |
// 48––76––77––78––79                                          15
// |                                           GREEN           |
// 47                                          71––70––69––68––16
// |                                                           |
// 46                                                          17
// |                                                           |
// 45                                                          18
// |                                                           |
// 44                                                          19
// |                             BLUE                          |
// 43                          75                              20
// |                           |                               |
// 42                          74                              21
// |                           |                               |
// 41                          73                              22
// |                           |                               |
// 40                          72                              23
// |                           |                               |
// 39––38––37––36––35––34––33––32––31––30––29––28––27––26––25––24
use super::color::Color;
use super::piece::Piece;

pub type Point = u8; // 0–79

pub const HOUSE_TILES: [Point; 16] = [
    64, 65, 66, 67, 68, 69, 70, 71,
    72, 73, 74, 75, 76, 77, 78, 79
];

pub const PLAYER_HOUSE_BY_COLOR: [(Color, [Point; 4]); 4] = [
    (Color::Red,    [64, 65, 66, 67]),
    (Color::Green,  [68, 69, 70, 71]),
    (Color::Blue,   [72, 73, 74, 75]),
    (Color::Yellow, [76, 77, 78, 79]),
];

pub const PLAYER_HOUSE_BY_TILE: [(Point, [Point; 4]); 4] = [
    (0,    [64, 65, 66, 67]),
    (16,  [68, 69, 70, 71]),
    (32,   [72, 73, 74, 75]),
    (48, [76, 77, 78, 79]),
];

pub struct Board {
    pub tiles: [Option<Piece>; 80],
}

impl Board {
    pub fn new() -> Self {
        Self {
            tiles: [None; 80],
        }
    }
    pub fn get_board(&self) -> &[Option<Piece>; 80] {
        &self.tiles
    }

    pub fn check_tile(&self, p: Point) -> Option<Piece> {
        self.tiles[p as usize]
    }

    pub fn start_field(color: Color) -> Point {
        match color {
            Color::Red => 0,
            Color::Green => 15,
            Color::Blue => 31, 
            Color::Yellow => 47,
        }
    }

    pub fn house_entry(&self, color: Color) -> Point {
        match color {
            Color::Red => 0,
            Color::Green => 16,
            Color::Blue => 32, 
            Color::Yellow => 48,
        }
    }

    pub fn house_by_color(&self, color: Color) -> [u8; 4] {
        match color {
            Color::Red =>    [64, 65, 66, 67],
            Color::Green =>  [68, 69, 70, 71],
            Color::Blue =>   [72, 73, 74, 75],
            Color::Yellow => [76, 77, 78, 79],
        }
    }

    pub fn house_gateway(& self, color: Color) -> Point {
        match color {
            Color::Red => 64,
            Color::Green => 68,
            Color::Blue => 72,
            Color::Yellow => 76,
        }
    }

    pub fn distance_between(&self, from: u8, to: u8, color: Color) -> Option<u8> {
        let ring_size = 64;

        // Check for out-of-bounds positions
        if from > 79 || to > 79 {
            return None;
        }

        let house = self.house_by_color(color);

        // Piece already in house
        if from >= ring_size {
            
            // Check correct in-house movement
            if !house.contains(&from) || !house.contains(&to) || to < from {
                return None;
            }

            return Some(to - from);
        }
        
        // Moving from the ring into the house
        if to >= ring_size {
            
            if !house.contains(&to) {
                return None; 
            }

            let house_entry = self.house_entry(color);

            let distance_to_house_entry = (house_entry + ring_size - from) % ring_size;

            let steps_in_house = to - self.house_gateway(color);

            // Total distance
            return Some(distance_to_house_entry as u8 + 1 + steps_in_house as u8);
        } 
        
        // Moving around the ring
        else {

            let distance = (to + ring_size - from) % ring_size;
            Some(distance as u8)
        }
    }

    pub fn passed_tiles(&self, from: u8, to:u8, color: Color, backward: bool) -> Option<Vec<Point>> {
        let ring_size = 64;
        let mut tiles = Vec::new();
        let mut current_position = from;

        let distance = if backward {
            self.distance_between(to, from, color)?
        } else {
            self.distance_between(from, to, color)?
        };

        // Piece already in house
        if from >= ring_size {
            
            // Backward move not allowed in-house
            if backward {
                return None;
            }

            for pos in (from + 1)..= to {
                tiles.push(pos);
            }

            return Some(tiles);

        
        }
        
        // Moving from the ring into the house
        if to >= ring_size {

            // Backward move into house not allowed 
            if backward {
                return None;
            }

            let house_entry = self.house_entry(color);
            let house_gateway = self.house_gateway(color);

            // Add tiles to house entry
            while current_position != house_entry {
                current_position = (current_position + 1) % ring_size;
                tiles.push(current_position);
            }

            // Add first tile in house
            tiles.push(house_gateway);

            for pos in (house_gateway + 1)..= to {
                tiles.push(pos);
            }

            return Some(tiles);
        }

        // Normal ring movement   
        for _ in 0..distance {
            current_position = if backward {
                (current_position + ring_size - 1) % ring_size
            } else {
                (current_position + 1) % ring_size
            };
            tiles.push(current_position);
        }

        Some(tiles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod distance_between_tests {
        use super::*;

        #[test]
        fn ring() {
            let board = Board::new();

            assert_eq!(board.distance_between(0, 3, Color::Red), Some(3));
            assert_eq!(board.distance_between(62, 2, Color::Red), Some(4));
        }

        #[test]
        fn into_house() {
            let board = Board::new();

            assert_eq!(board.distance_between(62, 64, Color::Red), Some(3));
        }

        #[test]
        fn wrong_house() {
            let board = Board::new();

            assert_eq!(board.distance_between(62, 64, Color::Green), None);
        }

        #[test]
        fn inside_house() {
            let board = Board::new();
            let house = board.house_by_color(Color::Green);

            assert_eq!(board.distance_between(house[0], house[2], Color::Green), Some(2));
        }

        #[test]
        fn invalid_positions() {
            let board = Board::new();
            
            assert_eq!(board.distance_between(80, 0, Color::Red), None);
            assert_eq!(board.distance_between(0, 80, Color::Red), None);
            assert_eq!(board.distance_between(81, 90, Color::Blue), None);
        }
    }
    
    mod passed_tiles_tests {
        use super::*;

        #[test]
        fn ring_forward() {
            let board = Board::new();
            
            assert_eq!(
                board.passed_tiles(0, 3, Color::Red, false),
                Some(vec![1, 2, 3])
            );
            assert_eq!(
                board.passed_tiles(62, 2, Color::Red, false),
                Some(vec![63, 0, 1, 2])
            );
        }

        #[test]
        fn ring_backward() {
            let board = Board::new();
            
            assert_eq!(
                board.passed_tiles(3, 0, Color::Red, true),
                Some(vec![2, 1, 0])
            );
            assert_eq!(
                board.passed_tiles(2, 62, Color::Red, true),
                Some(vec![1, 0, 63, 62])
            );
        }

        #[test]
        fn inside_house_forward() {
            let board = Board::new();
            let house = board.house_by_color(Color::Blue);

            assert_eq!(
                board.passed_tiles(house[0], house[2], Color::Blue, false),
                Some(vec![house[1], house[2]])
            );
        }

        #[test]
        fn inside_house_backward() {
            let board = Board::new();
            let house = board.house_by_color(Color::Blue);

            assert_eq!(
                board.passed_tiles(house[2], house[1], Color::Blue, true),
                None
            );
        }

        #[test]
        fn into_house_forward() {
            let board = Board::new();

            assert_eq!(
                board.passed_tiles(62, 64, Color::Red, false),
                Some(vec![63, 0, 64])
            );
        }

        #[test]
        fn into_house_backward() {
            let board = Board::new();

            assert_eq!(
                board.passed_tiles(64, 62, Color::Red, true),
                None
            );
        }

        #[test]
        fn invalid_positions() {
            let board = Board::new();
            
            assert_eq!(board.passed_tiles(80, 0, Color::Red, false), None);
            assert_eq!(board.passed_tiles(0, 80, Color::Red, false), None);
            assert_eq!(board.passed_tiles(81, 90, Color::Blue, false), None);
        }
    }
}