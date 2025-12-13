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
#[derive(Clone )]
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

    pub fn start_field(color: Color) -> Point {
        house_entry_for(color)
    }

    fn house_gateway_for(&self, color: Color) -> Point {
        match color {
            Color::Red => 63,
            Color::Green => 15,
            Color::Blue => 31,
            Color::Yellow => 47,
        }
    }

    pub fn is_in_house(&self, p: Point, color: Color) -> bool {
        if let Some((_, house_fields)) = PLAYER_HOUSE.iter().find(|(c, _)| *c == color) {
            return house_fields.contains(&p);
        }
        false
    }

    pub fn calculate_target(&self, current_pos: Point, steps: i8, color: Color) -> Option<Point> {
        let ring_size = 64; 

        if current_pos >= ring_size {
            if steps < 0 { return None; }
            
            if !self.is_in_house(current_pos, color) { return None; }

            let new_pos = current_pos + steps as u8;
            
            let (_, house_fields) = PLAYER_HOUSE.iter().find(|(c, _)| *c == color).unwrap();
            let last_house_field = house_fields[3]; 
            
            if new_pos > last_house_field {
                return None; 
            }
            return Some(new_pos);
        }

        
        if steps < 0 {
            let ring_steps = steps.abs() as u8;
            let new_pos = (current_pos + ring_size - ring_steps) % ring_size;
            return Some(new_pos);
        }

        let steps_u8 = steps as u8;
        let gateway = self.house_gateway_for(color);
        
        let distance_to_gateway = (gateway + ring_size - current_pos) % ring_size;

       
        if steps_u8 > distance_to_gateway {
            
            let steps_remaining = steps_u8 - distance_to_gateway - 1;
            
            let (_, house_fields) = PLAYER_HOUSE.iter().find(|(c, _)| *c == color).unwrap();
            let first_house_field = house_fields[0];
            
            let target_in_house = first_house_field + steps_remaining;
            
           
            let last_house_field = house_fields[3];
            if target_in_house > last_house_field {
                return None;
            }
            return Some(target_in_house);
        } else {
            
            let new_pos = (current_pos + steps_u8) % ring_size;
            return Some(new_pos);
        }
    }

    pub fn start(&mut self, piece: Piece) -> Option<Piece> {
        let entry = house_entry_for(piece.color()); 
        let old = self.tiles[entry as usize];
        self.tiles[entry as usize] = Some(piece);
        old
    }

}
