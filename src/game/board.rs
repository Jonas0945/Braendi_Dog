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

        if from >= ring_size {
            // Piece already in house
            // Check correct in-house movement
            if !house.contains(&from) || !house.contains(&to) || to < from {
                return None;
            }

            Some ((to - from) as u8)
            
        } else if to >= ring_size {
            // Moving from the ring into the house

            if !house.contains(&to) {
                return None; 
            }

            let house_entry = self.house_entry(color);
            let distance_to_house_entry = (house_entry + ring_size - from) % ring_size;

            let steps_in_house = to - self.house_gateway(color);

            // Total distance
            Some(distance_to_house_entry as u8 + 1 + steps_in_house as u8)

        } else {

            // Moving around the ring
            Some(((to + ring_size - from) % ring_size) as u8)
        }
    }

    pub fn passed_tiles(&self, from: u8, to:u8, color: Color) -> Option<Vec<Point>> {
        let ring_size = 64;
        let distance = self.distance_between(from, to, color)?;
        let mut tiles = Vec::new();

        if from >= ring_size {
            // Piece already in house

            for pos in from..= to {
                tiles.push(pos);
            }

        } else if to >= ring_size {
            // Moving from the ring into the house

            let mut current_position = from; 
            let house_entry = self.house_entry(color);

            // Add tiles to house entry
            while current_position != house_entry {
                current_position = (current_position + 1) % ring_size;
                tiles.push(current_position);
            }

            // Add tiles inside house 
            let house = self.house_by_color(color);

            for pos in house[0]..= to {
                tiles.push(pos);
            }
        } else {
            // Normal ring movement

            let mut current_position = from;

            for _ in 0..distance {
                current_position = (current_position + 1) % ring_size;
                tiles.push(current_position);
            }
        }

        Some(tiles)
    }
  
}

#[cfg(test)]
mod tests {
    use super::*;

    mod distance_tests {
        use super::*;

        #[test]
        fn ring_only() {
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
        fn wrong_direction_inside_house() {
            let board = Board::new();
            let house = board.house_by_color(Color::Green);

            assert_eq!(board.distance_between(house[2], house[1], Color::Green), None);
        }
    }
    
    mod passed_tiles_tests {
        use super::*;

        #[test]
        fn ring_only() {
            let board = Board::new();
            let tiles = board.passed_tiles(60, 2, Color::Red).unwrap();
            assert_eq!(tiles, vec![61, 62, 63, 0, 1, 2]);
        }

        #[test]
        fn into_house() {
            let board = Board::new();
            let tiles = board.passed_tiles(60, 66, Color::Red).unwrap();
            assert_eq!(tiles, vec![61, 62, 63, 0, 64, 65, 66]);
        }


    }


    






}