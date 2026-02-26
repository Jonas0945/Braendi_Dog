use crate::{Color, ServerNachrich};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::{BeginGameMesage, Game, DogGame, game::player::{Player, PlayerType}};
use crate::ai::generator::generate_all_legal_actions;
use crate::ai::bot::{RandomBot, EvalBot, Bot};
use tokio::net::tcp::OwnedWriteHalf;

type ClientID = usize;

pub struct GameServer {
    pub game: Arc<Mutex<Option<Game>>>,
    pub clients: Arc<tokio::sync::Mutex<HashMap<ClientID, OwnedWriteHalf>>>,
    pub client_to_index: Arc<Mutex<HashMap<ClientID, Color>>>,
    pub action_queue: Arc<std::sync::Mutex<VecDeque<(ClientID, String)>>>,
    pub next_id: Arc<AtomicUsize>,
}

impl GameServer {
    pub fn new() -> Self {
        GameServer {
            game: Arc::new(Mutex::new(None)),
            clients: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            client_to_index: Arc::new(Mutex::new(HashMap::new())),
            action_queue: Arc::new(Mutex::new(VecDeque::new())),
            next_id: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub async fn start_server(&self, addresse: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addresse).await?;
        println!("Server hier, listner an {:?} gebunden", addresse);

        let game_ref = self.game.clone();
        let clients_ref = self.clients.clone();
        let client_to_index_ref = self.client_to_index.clone();
        let action_queue_ref = self.action_queue.clone();
        let next_id_ref = self.next_id.clone();

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

                            let maybe_state_msg: Option<String> = {
                                let mut game_guard = inner_game_ref.lock().unwrap();

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

                                            if let Some(player_0) =
                                                game_guard.as_ref().unwrap().players.get(0)
                                            {
                                                inner_client_to_index_ref
                                                    .lock()
                                                    .unwrap()
                                                    .insert(netzwerk_id, player_0.color);
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
                                        Some(msg1 + &msg2)
                                    }
                                    BeginGameMesage::SpielBeitreten { player_name } => {
                                        if let Some(game) = game_guard.as_ref() {
                                            let max_allowed = game.players.len();
                                            let used_colors: Vec<_> = inner_client_to_index_ref
                                                .lock()
                                                .unwrap()
                                                .values()
                                                .cloned()
                                                .collect();
                                            let free_slot = (0..max_allowed).find(|&i| {
                                                !used_colors.contains(&game.players[i].color)
                                            });

                                            if let Some(slot) = free_slot {
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
                                                    &ServerNachrich::State(game_clone),
                                                )
                                                .unwrap()
                                                    + "\n";
                                                Some(msg1 + &msg2)
                                            } else {
                                                println!("Kein freier Platz mehr.");
                                                None
                                            }
                                        } else {
                                            println!("Kein Spiel vorhanden.");
                                            None
                                        }
                                    }
                                }
                            };

                            if let Some(state_msg) = maybe_state_msg {
                                if let Some(writer) =
                                    inner_clients_ref.lock().await.get_mut(&netzwerk_id)
                                {
                                    if let Err(e) = writer.write_all(state_msg.as_bytes()).await {
                                        println!("Fehler beim Senden: {}", e);
                                    }
                                }
                            }

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
                                    _ => break,
                                }
                            }
                        });
                    }
                    Err(e) => println!("Verbindungsfehler: {}", e),
                }
            }
        });

        let game_loop_game_ref = self.game.clone();
        let game_loop_clients_ref = self.clients.clone();
        let game_loop_queue_ref = self.action_queue.clone();
        let game_loop_clients_to_index_ref = self.client_to_index.clone();

        tokio::spawn(async move {
            loop {
                let mut actions: Vec<(usize, String)> = Vec::new();
                {
                    let mut queue: std::sync::MutexGuard<'_, VecDeque<(usize, String)>> =
                        game_loop_queue_ref.lock().unwrap();
                    while let Some(ac) = queue.pop_front() {
                        actions.push(ac);
                    }
                }

                let mut outbox: Vec<(usize, String)> = Vec::new();
                let mut state_changed = false;

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

                            if let Some(col) = player_color {
                                if current_color == col {
                                    match game.play(&actionsstr) {
                                        Ok(()) => {
                                            state_changed = true;
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

                // Bot turn handling: if current player is a bot, choose and play an action
                let mut sleep_after = false;
                {
                    let mut game_guard = game_loop_game_ref.lock().unwrap();
                    if let Some(game) = game_guard.as_mut() {
                        let current_idx = game.current_player_index;
                        let player_type = game.players[current_idx].player_type;

                        match player_type {
                            PlayerType::Human => {}
                            PlayerType::RandomBot => {
                                let actions_vec = generate_all_legal_actions(&game);
                                if actions_vec.is_empty() {
                                    game.next_player();
                                    state_changed = true;
                                } else {
                                    let mut bot = RandomBot::new();
                                    if let Some(chosen) = bot.choose_action(game, actions_vec) {
                                        let play_str = chosen.to_string();
                                        match game.play(&play_str) {
                                            Ok(()) => state_changed = true,
                                            Err(e) => println!("Bot play error: {}", e),
                                        }
                                    }
                                }
                                sleep_after = true;
                            }
                            PlayerType::EvalBot => {
                                let actions_vec = generate_all_legal_actions(&game);
                                if actions_vec.is_empty() {
                                    game.next_player();
                                    state_changed = true;
                                } else {
                                    let mut bot = EvalBot::new();
                                    if let Some(chosen) = bot.choose_action(game, actions_vec) {
                                        let play_str = chosen.to_string();
                                        match game.play(&play_str) {
                                            Ok(()) => state_changed = true,
                                            Err(e) => println!("EvalBot play error: {}", e),
                                        }
                                    }
                                }
                                sleep_after = true;
                            }
                        }
                    }
                }

                let mut clients_guard = game_loop_clients_ref.lock().await;

                if state_changed {
                    let state_msg = {
                        let game_guard = game_loop_game_ref.lock().unwrap();
                        game_guard.as_ref().map(|game| {
                            serde_json::to_string(&ServerNachrich::State(game.clone())).unwrap()
                                + "\n"
                        })
                    };
                    if let Some(msg) = state_msg {
                        broadcast(&mut clients_guard, &msg).await;
                    }
                }

                for (clientid, message) in outbox {
                    if let Some(writer) = clients_guard.get_mut(&clientid) {
                        let _ = writer.write_all(message.as_bytes()).await;
                    }
                }
                if sleep_after {
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        });
        Ok(())
    }
}

async fn broadcast(clients: &mut HashMap<ClientID, OwnedWriteHalf>, message: &str) {
    for writer in clients.values_mut() {
        let _ = writer.write_all(message.as_bytes()).await;
    }
}
