use braendi_dog::{Game, DogGame, Action, render};
use std::fs::OpenOptions;
use std::str::FromStr;

fn main() {
    let mut game = Game::new(new_2v2);
    let log_file_path = "game_log.txt";

    game.new_round();

    loop {
        render(&game);

        // Read player action
        use std::io::{self, Write};
        print!("Aktion eingeben (z.B. Place, Move, Split, Trade, Remove, Undo, exit): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Fehler beim Lesen der Eingabe.");
            continue;
        }
        let input = input.trim();

        // Exit
        if input.eq_ignore_ascii_case("exit") {
            println!("Spiel beendet.");
            break;
        }

        // Undo
        if input.eq_ignore_ascii_case("undo") {
            match game.undo_action() {
                Ok(_) => println!("Letzte Aktion wurde zurückgenommen."),
                Err(e) => println!("Undo fehlgeschlagen: {}", e),
            }
            continue;
        }

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_file_path) {
            let _ = writeln!(file, "{}", input);
        }

        match Action::from_str(input) {
            Ok(action) => {
                match game.action(action.card, action) {
                    Ok(_) => {
                        println!("Zug erfolgreich!");
                    }
                    Err(e) => {
                        println!("Regelverstoß: {}", e);
                        println!("Drücke Enter zum Fortfahren...");
                        let _ = io::stdin().read_line(&mut String::new());
                    }
                }
            }
            Err(e) => {
                println!("Fehler beim Parsen: {}", e);
                println!("Drücke Enter zum Fortfahren...");
                let _ = io::stdin().read_line(&mut String::new());
            }
        }

        // 5. Prüfen auf Sieg
        if game.is_winner() {
            println!("Spieler {:?} gewinnt!", game.current_player().color);
            break;
        }
    }
}