use crate::game::{Game, DogGame, Color, Piece};

pub fn render(game: &Game) {
    clear_screen();
    print_header();
    draw_board(game);
    draw_current_player(game);
    draw_hand(game);
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn print_header() {
    println!("=========================================");
    println!("      BRANDY DOG – TERMINAL TEST UI      ");
    println!("=========================================\n");
}

fn draw_board(game: &Game) {
    let board = game.board_state();

    println!("Board State:\n");

    draw_segment("Red    (0–15)", board, 0..16, 64..68);
    draw_segment("Green  (16–31)", board, 16..32, 68..72);
    draw_segment("Blue   (32–47)", board, 32..48, 72..76);
    draw_segment("Yellow (48–63)", board, 48..64, 76..80);

    println!();
}

fn draw_segment(label: &str, board: &[Option<Piece>; 80], track_range: std::ops::Range<usize>, house_range: std::ops::Range<usize>) {
    print!("{:<15}: Track: ", label);
    for i in track_range {
        print!("{} ", cell_char(board, i));
    }
    print!("| House: ");
    for i in house_range {
        print!("{} ", cell_char(board, i));
    }
    println!();
}

fn cell_char(board: &[Option<Piece>; 80], idx: usize) -> char {
    match &board[idx] {
        Some(piece) => match piece.color {
            Color::Red => 'R',
            Color::Green => 'G',
            Color::Blue => 'B',
            Color::Yellow => 'Y',
        },
        None => '.',
    }
}

fn draw_current_player(game: &Game) {
    let p = game.current_player();

    println!("Aktiver Spieler: {:?}", p.color);
    println!(
        "Pieces: in house = {}, to place = {}",
        p.pieces_in_house,
        p.pieces_to_place
    );
    println!();
}

fn draw_hand(game: &Game) {
    let p = game.current_player();

    print!("Hand: ");
    for card in &p.cards {
        print!("[{}] ", card.value());
    }
    println!("\n");
}