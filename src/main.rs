//use anyhow::Ok;
use braendi_dog::{Game, DogGame, Action, game::game::GameVariant, render};
use std::io::{self, Write};
use std::fs::OpenOptions;
use std::str::FromStr;


fn select_game_variant() -> GameVariant {
    loop {
        println!("Spielvariante auswählen:");
        println!("1) 2 vs 2 (4 Spieler)");
        println!("2) 3 vs 3 (6 Spieler)");
        println!("3) 2 vs 2 vs 2 (6 Spieler)");
        println!("4) Free For All (2 bis 6 Spieler)");
        print!("Eingabe: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Eingabefehler.");
            continue;
        }

        match input.trim() {
            "1" => return GameVariant::TwoVsTwo,
            "2" => return GameVariant::ThreeVsThree,
            "3" => return GameVariant::TwoVsTwoVsTwo,
            "4" => {
                loop {
                    println!("Spieleranzahl auswählen (2-6):");
                    print!("Eingabe: ");
                    io::stdout().flush().unwrap();

                    let mut input = String::new();
                    if io::stdin().read_line(&mut input).is_err() {
                        println!("Eingabefehler.");
                        continue;
                    }

                    match input.trim() {
                        "2" => return GameVariant::FreeForAll(2),
                        "3" => return GameVariant::FreeForAll(3),
                        "4" => return GameVariant::FreeForAll(4),
                        "5" => return GameVariant::FreeForAll(5),
                        "6" => return GameVariant::FreeForAll(6),
                        _ => println!("Ungültige Anzahl.\n"),
                    }
                }
            }
            _ => println!("Ungültige Auswahl.\n"),
        }
    }
}

fn main() {
    let variant = select_game_variant();
    let mut game = Game::new(variant);
    let log_file_path = "game_log.txt";

    game.new_round();

    loop {
        render(&game);

        // Read player action

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

/*
# [tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    tokio::spawn(async {
    net::server::start_server("0.0.0.0:8080").await;}
);
    net::client::starte_client("127.0.0.1:8080").await?;
    Ok(())
}*/