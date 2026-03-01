use crate::game::{Game, Color, Piece};
use crate::game::game::DogGame;


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

    for (player_index, _player) in game.players.iter().enumerate() {
        let start = game.board.start_field(player_index);
        let house = game.board.house_by_player(player_index);
        draw_segment(game, &board, player_index, start, &house);
    }

    println!();
}

/// Draws a segment of the board for a player
fn draw_segment(
    game: &Game,
    board: &[Option<Piece>],
    player_index: usize,
    start: usize,
    house: &[usize],
) {
    let label = format!("{:?} ({})", game.players[player_index].color, player_index);
    

    print!("{:<15}: Track: ", label);

    // 16 tiles per player segment
    for i in 0..16 {
        let idx = (start + i) % board.len();
        print!("{} ", cell_char(game, board, idx));
    }

    print!("| House: ");
    for &idx in house {
        print!("{} ", cell_char(game, board, idx));
    }

    println!();
}

/// Returns the character representation of a cell
fn cell_char(game: &Game, board: &[Option<Piece>], idx: usize) -> char {
    match &board[idx] {
        Some(piece) => {
            let color = game.players[piece.owner].color;
            match color {
                Color::Red => 'R',
                Color::Green => 'G',
                Color::Blue => 'B',
                Color::Yellow => 'Y',
                Color::Purple => 'P',
                Color::Orange => 'O',
            }
        }
        None => '.',
    }
}

fn draw_current_player(game: &Game) {
    let p = game.current_player();
    let player_index = game.index_of_color(p.color);

    println!("Aktiver Spieler: {:?}", p.color);
    println!(
        "Pieces: in house = {}, to place = {}",
        p.pieces_in_house,
        p.pieces_to_place
    );

    if let Some(steps_left) = game.split_rest {
        println!("Split-Rest: {} Schritte verbleibend", steps_left);
    }

    let positions: Vec<usize> = game
        .board_state()
        .iter()
        .enumerate()
        .filter(|(_, tile)| tile.as_ref().map_or(false, |piece| piece.owner == player_index))
        .map(|(idx, _)| idx)
        .collect();

    println!("Figurenpositionen: {:?}", positions);
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
