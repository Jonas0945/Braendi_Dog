use braendi_dog::{Game, DogGame, render};

fn main() {
    let mut game = Game::new();

    game.new_round();

    loop {
        render(&game);

        // Read player action
        use std::io::{self, Write};
        print!("Aktion eingeben (z.B. Place, Move, Split, Trade, Remove, Undo, exit): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            println!("Spiel beendet.");
            break;
        }

        if input.eq_ignore_ascii_case("undo") {
            match game.undo_action() {
                Ok(_) => println!("Letzte Aktion wurde zurückgenommen."),
                Err(e) => println!("Undo fehlgeschlagen: {}", e),
            }
            continue;
        }

        match game.play(input) {
            Ok(_) => {
                if game.is_winner() {
                    render(&game);
                    println!("Spieler {:?} gewinnt!", game.current_player().color);
                    break;
                }
            }
            Err(e) => {
                println!("Fehler: {}", e);
            }
        }
    }
}