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
// |                                           GREEN           |
// 48                                          71––70––69––68––15
// |              YELLOW                                       |
// 47––76––77––78––79                                          16
// |                                                           |
// 46                                                          17
// |                                                           |
// 45                                                          18
// |                                                           |
// 44                                                          19
// |                                  BLUE                     |
// 43                              75                          20
// |                                |                          |
// 42                              74                          21
// |                                |                          |
// 41                              73                          22
// |                                |                          |
// 40                              72                          23
// |                                |                          |
// 39––38––37––36––35––34––33––32––31––30––29––28––27––26––25––24
use super::color::Color;
use super::piece::Piece;

pub type Point = u8; // 0–79


pub const PLAYER_HOUSE: [(Color, [Point; 4]); 4] = [
    (Color::Red,    [64, 65, 66, 67]),
    (Color::Green,  [68, 69, 70, 71]),
    (Color::Blue,   [72, 73, 74, 75]),
    (Color::Yellow, [76, 77, 78, 79]),
];

pub fn house_entry_for(color: Color) -> Point {
    match color {
        Color::Red => 56,
        Color::Green => 68,
        Color::Blue => 72,
        Color::Yellow => 76,
    }
}

pub struct Board {
    tiles: [Option<Piece>; 80],
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

    pub fn check_piece(&self, piece: Piece) -> Option<Point> {
        for (i, tile) in self.tiles.iter().enumerate() {
            if let Some(p) = tile {
                if p.color() == piece.color() && p.id() == piece.id() {
                    return Some(i as Point);
                }
            }
        }
        None
    }

    pub fn start(&mut self, piece: Piece) -> Option<Piece> {
        let entry = house_entry_for(piece.color()); 
        let old = self.tiles[entry as usize];
        self.tiles[entry as usize] = Some(piece);
        old
    }
}