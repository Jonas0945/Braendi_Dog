use braendi_dog::ai::generator::generate_all_legal_actions;
use braendi_dog::{Game, DogGame, Action, game::game::GameVariant, render};
use braendi_dog::ai::bot::{Bot, RandomBot};
use std::io::{self, Write};
use std::str::FromStr;

// Wir binden das GUI-Modul ein (das wir gleich erstellen)
pub mod gui;

fn main() -> iced::Result {
    // Startet die GUI aus der Datei gui.rs
    gui::launch()
}

fn select_bots(num_players: usize) -> Vec<bool> {
    let mut bot_flags = vec![false; num_players];

    println!("Für jeden Spieler angeben, ob Bot (y/n):");

    for i in 0..num_players {
        loop {
            print!("Spieler {}: ", i);
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            match input.trim().to_lowercase().as_str() {
                "y" => {
                    bot_flags[i] = true;
                    break;
                }
                "n" => {
                    bot_flags[i] = false;
                    break;
                }
                _ => println!("Bitte y oder n eingeben."),
            }
        }
    }

    bot_flags
}

fn ascii_main() {
    let variant = select_game_variant();
    let mut game = Game::new(variant);

    let num_players = game.players.len();
    let bot_flags = select_bots(num_players);
    let mut bot = RandomBot::new();

    let mut last_action: Option<Action> = None;

    game.new_round();

    loop {
        render(&game);

        if let Some(last) = &last_action {
            println!("Letzter Zug: {:?}", last);
        }

        let player_index = game.current_player_index;
        let player_color = game.current_player().color;

        let action: Action = if bot_flags[player_index] {
            let actions = generate_all_legal_actions(&game);

            if actions.is_empty() {
                println!("Keine Aktionen verfügbar für {:?}", player_color);
                game.next_player();
                continue;
            } else {
                let chosen = bot.choose_action(actions);
                println!("Bot {:?} wählt: {:?}", player_color, chosen);
                chosen.unwrap()
            }
            
        } else {
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

            match Action::from_str(input) {
                Ok(a) => a,
                Err(e) => {
                    println!("Fehler beim Parsen: {}", e);
                    continue;
                }
            }
            
        };

        match game.action(action.card, action) {
            Ok(_) => {
                println!("Zug erfolgreich!");
                last_action = Some(action);
            },
            Err(e) => {
                println!("Regelverstoß: {}", e);
                let _ = io::stdin().read_line(&mut String::new());
            }
        }

        if game.is_winner() {
            println!("Spieler {:?} gewinnt!", game.current_player().color);
            break;
        }
    }
}
