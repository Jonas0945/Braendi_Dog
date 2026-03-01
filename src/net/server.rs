// Kommentare bei Jonas
// Dieses Modul implementiert einen einfachen asynchronen TCP-Server, der
// mehrere Clients verwaltet und ein Spiel steuert.

use crate::ai::bot::{Bot, EvalBot, RandomBot};
use crate::ai::generator::generate_all_legal_actions;
use crate::{BeginGameMesage, DogGame, Game, game::player::PlayerType};
use crate::{Color, ServerNachrich};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::tcp::OwnedWriteHalf;

// Eindeutige Kennung für einen client, Bots bekommen hohe IDs damit sie in der logik von Humans unterschieden werden können
type ClientID = usize;
const BOT_CLIENT_ID_BASE: usize = usize::MAX / 2;

pub struct GameServer {
    pub game: Arc<Mutex<Option<Game>>>,
    pub clients: Arc<tokio::sync::Mutex<HashMap<ClientID, OwnedWriteHalf>>>,
    pub client_to_index: Arc<Mutex<HashMap<ClientID, Color>>>,
    pub action_queue: Arc<std::sync::Mutex<VecDeque<(ClientID, String)>>>,
    pub next_id: Arc<AtomicUsize>,
}

impl GameServer {
    // Erstellt einen frischen Server ohne laufendes Spiel und ohne Clients.
    // Die Felder werden mit `Arc` und `Mutex` umwickelt, damit sie von mehreren
    // Tokio-Tasks gleichzeitig sicher genutzt werden können.
    pub fn new() -> Self {
        GameServer {
            game: Arc::new(Mutex::new(None)),
            clients: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            client_to_index: Arc::new(Mutex::new(HashMap::new())),
            action_queue: Arc::new(Mutex::new(VecDeque::new())),
            next_id: Arc::new(AtomicUsize::new(0)),
        }
    }
    // Startet den TCP Server, beinhaltet die Serverlogik
    //zwei Hauptaufgaben:
    ///1. accept scleife, die neue Clients Verarbeitet und die Verbindung in clients einträgt
    /// 2. Game loop, welcher actionqueue abarbeitet und den Zustand samt Bots aktualisiert und broadcastet

    pub async fn start_server(&self, addresse: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addresse).await?;
        println!("Server hier, listner an {:?} gebunden", addresse);

        let game_ref = self.game.clone();
        let clients_ref = self.clients.clone();
        let client_to_index_ref = self.client_to_index.clone();
        let action_queue_ref = self.action_queue.clone();
        let next_id_ref = self.next_id.clone();
        // erster loop
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((socket, addr)) => {
                        println!("Neue Connection: {}", addr);
                        let (reader, writer) = socket.into_split();

                        let netzwerk_id = next_id_ref.fetch_add(1, Ordering::Relaxed);
                        {
                            let mut clients_guard = clients_ref.lock().await;
                            clients_guard.insert(netzwerk_id, writer);
                        }

                        let inner_game_ref = game_ref.clone();
                        let inner_client_to_index_ref = client_to_index_ref.clone();
                        let inner_action_queue_ref = action_queue_ref.clone();
                        let inner_clients_ref = clients_ref.clone();

                        tokio::spawn(async move {
                            let mut reader = tokio::io::BufReader::new(reader);
                            let mut hello_line = String::new();

                            match reader.read_line(&mut hello_line).await {
                                Ok(n) if n > 0 => {
                                    println!("Raw line empfangen ({} bytes): {:?}", n, hello_line);
                                }
                                _ => return,
                            }

                            let hello_trimmed = hello_line.trim();
                            let hello_message: BeginGameMesage =
                                match serde_json::from_str(hello_trimmed) {
                                    Ok(msg) => msg,
                                    Err(e) => {
                                        println!("Unbekannte Nachricht: {}", e);
                                        return;
                                    }
                                };

                            let (maybe_state_msg, maybe_broadcast_state) = {
                                let mut game_guard = inner_game_ref.lock().unwrap();
                                //Spiel erstellen oder beitreten
                                match hello_message {
                                    BeginGameMesage::ErstelleSpiel {
                                        variant,
                                        player_name,
                                        player_types,
                                    } => {
                                        if game_guard.is_none() {
                                            let mut new_game = Game::new(variant, player_types);
                                            new_game.new_round();

                                            *game_guard = Some(new_game);

                                            let game = game_guard.as_mut().unwrap();
                                            let mut human_slot_assigned = false;
                                            for (i, player) in game.players.iter_mut().enumerate() {
                                                match player.player_type {
                                                    PlayerType::Human => {
                                                        if !human_slot_assigned {
                                                            player.name = player_name.clone();
                                                            inner_client_to_index_ref
                                                                .lock()
                                                                .unwrap()
                                                                .insert(netzwerk_id, player.color);
                                                            human_slot_assigned = true;
                                                        } else {
                                                            player.name = "Wartet...".to_string(); //noch nicht verbundene Spieler
                                                        }
                                                    }
                                                    PlayerType::RandomBot | PlayerType::EvalBot => {
                                                        inner_client_to_index_ref
                                                            .lock()
                                                            .unwrap()
                                                            .insert(
                                                                BOT_CLIENT_ID_BASE + i,
                                                                player.color,
                                                            );
                                                    }
                                                }
                                            }

                                            println!("Spiel wurde von {} erstellt.", player_name);
                                        } else {
                                            println!("Fehler: das Spiel läuft schon");
                                            return;
                                        }

                                        let msg1 =
                                            serde_json::to_string(&ServerNachrich::Welcome(0))
                                                .unwrap()
                                                + "\n";
                                        let game_clone = game_guard.as_ref().unwrap().clone();
                                        let msg2 = serde_json::to_string(&ServerNachrich::State(
                                            game_clone,
                                        ))
                                        .unwrap()
                                            + "\n";

                                        (Some(msg1 + &msg2), None)
                                    }
                                    BeginGameMesage::SpielBeitreten { player_name } => {
                                        // Beitritt nur möglich wenn Spiel existiert
                                        //und ein freier Menschenslot da ist
                                        if let Some(game) = game_guard.as_mut() {
                                            let free_slot = game.players.iter().position(|p| {
                                                p.player_type == PlayerType::Human
                                                    && p.name == "Wartet..."
                                            });

                                            if let Some(slot) = free_slot {
                                                game.players[slot].name = player_name.clone();

                                                let player = game.players[slot].clone();
                                                inner_client_to_index_ref
                                                    .lock()
                                                    .unwrap()
                                                    .insert(netzwerk_id, player.color);
                                                println!(
                                                    "Client '{}' hat sich angemeldet als Slot {}",
                                                    player_name, slot
                                                );

                                                let msg1 = serde_json::to_string(
                                                    &ServerNachrich::Welcome(slot),
                                                )
                                                .unwrap()
                                                    + "\n";
                                                let game_clone = game.clone();
                                                let msg2 = serde_json::to_string(
                                                    &ServerNachrich::State(game_clone.clone()),
                                                )
                                                .unwrap()
                                                    + "\n";

                                                (Some(msg1 + &msg2), Some(game_clone))
                                            } else {
                                                println!(
                                                    "Kein freier Human-Slot mehr in der Lobby."
                                                );
                                                (None, None)
                                            }
                                        } else {
                                            println!("Kein Spiel vorhanden.");
                                            (None, None)
                                        }
                                    }
                                }
                            };

                            if let Some(state) = maybe_broadcast_state {
                                let state_str =
                                    serde_json::to_string(&ServerNachrich::State(state)).unwrap()
                                        + "\n";
                                let mut clients_guard = inner_clients_ref.lock().await;
                                for (id, writer) in clients_guard.iter_mut() {
                                    if *id != netzwerk_id {
                                        let _ = writer.write_all(state_str.as_bytes()).await;
                                    }
                                }
                            }

                            if let Some(state_msg) = maybe_state_msg {
                                if let Some(writer) =
                                    inner_clients_ref.lock().await.get_mut(&netzwerk_id)
                                {
                                    if let Err(e) = writer.write_all(state_msg.as_bytes()).await {
                                        println!("Fehler beim Senden: {}", e);
                                    }
                                }
                            }

                            // Hauptleseschleife für diesen Client. nichtleere Nachrichten werden in die actionqueue hinzugefügt und später abgearbeitet
                            loop {
                                let mut action_line = String::new();
                                match reader.read_line(&mut action_line).await {
                                    Ok(n) if n > 0 => {
                                        let trimmed = action_line.trim();
                                        if let Ok(action_str) =
                                            serde_json::from_str::<String>(trimmed)
                                        {
                                            inner_action_queue_ref
                                                .lock()
                                                .unwrap()
                                                .push_back((netzwerk_id, action_str));
                                        }
                                    }
                                    _ => break, // Verbindung geschlossen oder Fehler
                                }
                            }
                        });
                    }
                    Err(e) => println!("Verbindungsfehler: {}", e),
                }
            }
        });

        // Klonen der Referenzen damit die schleife auf die gleichen
        // Strukturen zugreift wie der Listener Task
        let game_loop_game_ref = self.game.clone();
        let game_loop_clients_ref = self.clients.clone();
        let game_loop_queue_ref = self.action_queue.clone();
        let game_loop_clients_to_index_ref = self.client_to_index.clone();

        // game loop, läuft dauerhaft
        tokio::spawn(async move {
            loop {
                //Warteschlange einsammeln
                let mut actions: Vec<(usize, String)> = Vec::new();
                {
                    let mut queue: std::sync::MutexGuard<'_, VecDeque<(usize, String)>> =
                        game_loop_queue_ref.lock().unwrap();
                    while let Some(ac) = queue.pop_front() {
                        actions.push(ac);
                    }
                }

                // Nachrichten, die später direkt an einzelne Clients geschickt
                // werden sollen (z.B. Fehlermeldungen).
                let mut outbox: Vec<(usize, String)> = Vec::new();
                let mut state_changed = false;

                // Flag, ob in diesem Loop schon ein menschlicher Spieler
                // gezogen hat. Wird für Botthreads und die Tick-Pausen
                // genutzt.
                let mut human_played_this_tick = false;

                // actions werden verarbeitet
                if !actions.is_empty() {
                    let mut game_guard = game_loop_game_ref.lock().unwrap();
                    if let Some(game) = game_guard.as_mut() {
                        for (clientid, actionsstr) in actions {
                            let player_color = game_loop_clients_to_index_ref
                                .lock()
                                .unwrap()
                                .get(&clientid)
                                .cloned();
                            let current_color = game.current_player().color;

                            let is_undo = actionsstr.eq_ignore_ascii_case("undo");

                            if let Some(col) = player_color {
                                if current_color == col || is_undo {
                                    match game.play(&actionsstr) {
                                        Ok(()) => {
                                            state_changed = true;
                                            human_played_this_tick = true;
                                        }
                                        Err(e) => {
                                            let err_json = serde_json::to_string(
                                                &ServerNachrich::Fehler(format!("Fehler: {}", e)),
                                            )
                                            .unwrap()
                                                + "\n";
                                            outbox.push((clientid, err_json));
                                        }
                                    }
                                } else {
                                    let err_json = serde_json::to_string(&ServerNachrich::Fehler(
                                        "Nicht dein Zug!".to_string(),
                                    ))
                                    .unwrap()
                                        + "\n";
                                    outbox.push((clientid, err_json));
                                }
                            } else {
                                let err_json = serde_json::to_string(&ServerNachrich::Fehler(
                                    "Unbekannter Spieler!".to_string(),
                                ))
                                .unwrap()
                                    + "\n";
                                outbox.push((clientid, err_json));
                            }
                        }
                    }
                }

                let mut sleep_duration = 0;

                // Falls in diesem Tick kein Mensch gezogen hat, wird ein Botzug
                // oder eine Sonderaktion (Undo/Next) ausgeführt.
                if !human_played_this_tick {
                    let mut force_undo = false;
                    let mut force_next = false;

                    // Zuerst prüfen wir, ob der aktuelle Spieler ein Bot ist und
                    // ob er überhaupt legale Aktionen hat.
                    {
                        let mut game_guard = game_loop_game_ref.lock().unwrap();
                        if let Some(game) = game_guard.as_mut() {
                            let current_idx = game.current_player_index;
                            if game.players[current_idx].player_type != PlayerType::Human {
                                let actions_vec = generate_all_legal_actions(&game);
                                if actions_vec.is_empty() {
                                    if game.split_rest.is_some() {
                                        println!("Bot steckt im Split fest! Führe Undo aus.");
                                        force_undo = true;
                                    } else {
                                        force_next = true;
                                    }
                                }
                            }
                        }
                    }

                    if force_undo {
                        // Der Bot war im Split gefangen. Wir machen einfach ein
                        // Undo und werfen die Karte ab, um den Loop zu durchbrechen.
                        let mut game_guard = game_loop_game_ref.lock().unwrap();
                        let game = game_guard.as_mut().unwrap();

                        let mut card_to_burn = None;
                        if let Some(last_entry) = game.history.last() {
                            card_to_burn = last_entry.action.card;
                        }

                        let _ = game.undo_turn();

                        if let Some(c) = card_to_burn {
                            let idx = game.current_player_index;
                            game.players[idx].remove_card(c);
                            game.discard.push(c);
                            println!("Bot-Loop durchbrochen: Karte {:?} abgeworfen.", c);
                        }

                        game.next_player();
                        state_changed = true;
                        sleep_duration = 1500;
                    } else if force_next {
                        // Bot muss passen
                        let mut game_guard = game_loop_game_ref.lock().unwrap();
                        let game = game_guard.as_mut().unwrap();
                        let idx = game.current_player_index;

                        if !game.players[idx].cards.is_empty() {
                            let min_index = game.players[idx]
                                .cards
                                .iter()
                                .enumerate()
                                .min_by_key(|&(_, c)| c.value())
                                .map(|(i, _)| i)
                                .unwrap_or(0);

                            let card = game.players[idx].cards.remove(min_index);
                            game.discard.push(card);
                            println!("Bot muss passen und wirft Karte ab.");
                        }

                        game.next_player();
                        state_changed = true;
                        sleep_duration = 1000;
                    } else {
                        // Normale Botentscheidung je nach Typ
                        let bot_action = {
                            let game_clone_opt = {
                                let guard = game_loop_game_ref.lock().unwrap();
                                guard.as_ref().map(|g| g.clone())
                            };

                            if let Some(mut game_clone) = game_clone_opt {
                                let current_idx = game_clone.current_player_index;
                                match game_clone.players[current_idx].player_type {
                                    PlayerType::Human => None,
                                    PlayerType::RandomBot => {
                                        let actions = generate_all_legal_actions(&game_clone);
                                        RandomBot::new().choose_action(&mut game_clone, actions)
                                    }
                                    PlayerType::EvalBot => {
                                        let actions = generate_all_legal_actions(&game_clone);
                                        tokio::task::block_in_place(|| {
                                            EvalBot::new().choose_action(&mut game_clone, actions)
                                        })
                                    }
                                }
                            } else {
                                None
                            }
                        };

                        if let Some(chosen) = bot_action {
                            let play_str = chosen.to_string();
                            let mut game_guard = game_loop_game_ref.lock().unwrap();
                            if let Some(game) = game_guard.as_mut() {
                                match game.play(&play_str) {
                                    Ok(()) => state_changed = true,
                                    Err(e) => println!("Bot play error: {}", e),
                                }
                            }
                            sleep_duration = 1500;
                        }
                    }
                } else {
                    // Mensch hat gespielt; wir geben ein bisschen mehr Zeit bevor der nächste Tick geprüft wird
                    let is_trading = game_loop_game_ref
                        .lock()
                        .unwrap()
                        .as_ref()
                        .map(|g| g.trading_phase)
                        .unwrap_or(false);
                    if !is_trading {
                        sleep_duration = 600;
                    }
                }

                let mut clients_guard = game_loop_clients_ref.lock().await;

                // Wenn sich der Spielzustand geändert hat, senden wir den
                // neuen State an *alle* verbundenen Clients.
                if state_changed {
                    let state_msg = {
                        let game_guard = game_loop_game_ref.lock().unwrap();
                        game_guard.as_ref().map(|game| {
                            serde_json::to_string(&ServerNachrich::State(game.clone())).unwrap()
                                + "\n"
                        })
                    };
                    if let Some(msg) = state_msg {
                        for writer in clients_guard.values_mut() {
                            let _ = writer.write_all(msg.as_bytes()).await;
                        }
                    }
                }

                // Alle individuellen Fehler-/Informationsnachrichten an Clients
                for (clientid, message) in outbox {
                    if let Some(writer) = clients_guard.get_mut(&clientid) {
                        let _ = writer.write_all(message.as_bytes()).await;
                    }
                }

                if sleep_duration > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(sleep_duration)).await;
                }
                // Kleine Pause am Ende jeder Iteration, um 50ms-Takt zu sichern.
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        });
        Ok(())
    }
}
