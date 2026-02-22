use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use braendi_dog::ServerNachrich;
use tokio::net::TcpListener;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt};

use braendi_dog::{BeginGameMesage, Game, game::player::Player};
use tokio::net::tcp::OwnedWriteHalf;




//use crate::bin::server;
type ClientID = usize;

pub struct GameServer{
    pub game: Arc<Mutex<Option<Game>>>,
    pub clients: Arc<tokio::sync::Mutex<Vec<tokio::net::tcp::OwnedWriteHalf>>>,
    // HashMap die Client-IDs auf Spieler abbildet
    pub client_to_index: Arc<Mutex<HashMap<ClientID, Player>>>,
    // Queue für eingehende Aktionen von Clients
    pub action_queue: Arc<std::sync::Mutex<VecDeque<(ClientID, String)>>>
}
impl GameServer{
    pub fn new() -> Self{
        GameServer { 
            game: Arc::new(Mutex::new(None)), 
            clients: Arc::new(tokio::sync::Mutex::new(Vec::new())), 
            client_to_index: Arc::new(Mutex::new(HashMap::new())),
            action_queue: Arc::new(Mutex::new(VecDeque::new()))
        }
    }

    pub async fn start_server(&self, addresse : &str) -> Result<(), Box<dyn std::error::Error>>{
    let listener = TcpListener::bind(addresse).await?;
    println!("Server hier, listner an {:?} gebunden", addresse);

    // Referenzen werden geklont für Hintergrund task
    let game_ref = self.game.clone();
    let clients_ref = self.clients.clone();
    let client_to_index_ref = self.client_to_index.clone();
    let action_queue_ref = self.action_queue.clone();
    
    //Hintergrund task
    tokio::spawn(async move{
        loop{
            match listener.accept().await{
                Ok((socket, addr)) =>{
                    println!("Neue Connection: {}", addr);
                    // Erlaubt einzelzugriff auf reader und writer
                    let (reader, writer) = socket.into_split();

                    // Netzwerkkennung für diesen Client – ermittelt im Moment des Verbindungsaufbaus
                    let netzwerk_id: ClientID;
                    {
                        let mut clients_guard = clients_ref.lock().await;
                        clients_guard.push(writer);
                        netzwerk_id = clients_guard.len() - 1;
                    }

                    // Referenzen für den Client-Task klonen
                    let inner_game_ref = game_ref.clone();
                    let inner_client_to_index_ref = client_to_index_ref.clone();
                    let inner_action_queue_ref = action_queue_ref.clone();
                    let inner_clients_ref = clients_ref.clone();

                    //Task für einen Client
                    tokio::spawn(
                        
                        async move {
                            
                        
                        let mut reader = tokio::io::BufReader::new(reader);
                        let mut hello_line = String::new();

                        match reader.read_line(&mut hello_line).await {
                            Ok(n) if n > 0 => {
                                println!("Raw line empfangen ({} bytes): {:?}", n, hello_line);
                            }
                            _ => return,
                        }

                        let hello_trimmed = hello_line.trim();
                        let hello_message: BeginGameMesage = match serde_json::from_str(hello_trimmed){
                            Ok(msg) => msg,
                            Err(e) =>{
                                println!("Unbekannte Nachricht: {}", e);
                                return;
                            }
                        };

                        // Wir extrahieren hier optional eine Nachricht, die nach dem
                        // Halten des `game_guard` versendet werden soll. Der Ownership
                        // von `game_guard` wird nach dem Block fallen gelassen, bevor
                        // wir erneut `await` verwenden.
                        let maybe_state_msg: Option<String> = {
                            let mut game_guard = inner_game_ref.lock().unwrap();

                            match hello_message {
                                BeginGameMesage::ErstelleSpiel { variant, player_name } => {
                                    if game_guard.is_none() {
                                        let new_game = Game::new(variant);
                                        
                                        *game_guard = Some(new_game);

                                        // erster Spieler belegt Slot 0
                                        if let Some(player_0) =
                                            game_guard.as_ref().unwrap().players.get(0)
                                        {
                                            inner_client_to_index_ref
                                                .lock()
                                                .unwrap()
                                                .insert(netzwerk_id, player_0.clone());

                                            // debug: zeige aktuelle Zuordnung
                                            println!(
                                                "Creator (netzwerk_id={}) zugewiesen: {:?}; map={:?}",
                                                netzwerk_id,
                                                player_0.color,
                                                inner_client_to_index_ref.lock().unwrap()
                                            );
                                        }

                                        println!(
                                            "Spiel wurde von {} erstellt: Spiel Debug: {:?}",
                                            player_name,
                                            game_guard
                                        );
                                    } else {
                                        println!("Fehler: das Spiel läuft schon");
                                        return;
                                    }

                                    // Bereite Zustandstext vor, aber halte `game_guard` nicht
                                    // über ein await.
                                    let game_clone = game_guard.as_ref().unwrap().clone();
                                    let msg =
                                        serde_json::to_string(&ServerNachrich::State(game_clone))
                                            .unwrap()
                                            + "\n";
                                    Some(msg)
                                }
                                BeginGameMesage::SpielBeitreten { player_name } => {
                                    if let Some(game) = game_guard.as_ref() {
                                        let max_allowed = game.players.len();

                                        // Farben bereits vergeben ermitteln
                                        let used_colors: Vec<_> = inner_client_to_index_ref
                                            .lock()
                                            .unwrap()
                                            .values()
                                            .map(|p| p.color)
                                            .collect();

                                        let free_slot = (0..max_allowed).find(|&i| {
                                            !used_colors.contains(&game.players[i].color)
                                        });

                                        if let Some(slot) = free_slot {
                                            let player = game.players[slot].clone();
                                            inner_client_to_index_ref
                                                .lock()
                                                .unwrap()
                                                .insert(netzwerk_id, player.clone());

                                            println!(
                                                "Client '{}' hat sich als {:?} angemeldet (netzwerk_id={})",
                                                player_name, player.color, netzwerk_id
                                            );
                                        } else {
                                            println!(
                                                "Kein freier Platz mehr für '{}' (max={})",
                                                player_name, max_allowed
                                            );
                                        }
                                    } else {
                                        println!(
                                            "Kein Spiel vorhanden, '{}' kann nicht beitreten",
                                            player_name
                                        );
                                    }

                                    None
                                }
                            }
                        };

                        // send msg outside of the lock scope
                        if let Some(state_msg) = maybe_state_msg {
                            if let Some(writer) = inner_clients_ref
                                .lock()
                                .await
                                .get_mut(netzwerk_id)
                            {
                                if let Err(e) = writer.write_all(state_msg.as_bytes()).await {
                                    println!(
                                        "Fehler beim Senden an Client {}: {}",
                                        netzwerk_id, e
                                    );
                                }
                            }
                        }
                        
                        // Lese kontinuierlich Aktionen von diesem Client (eine JSON‑Zeile pro Aktion)
                        loop {
                            let mut action_line = String::new();
                            match reader.read_line(&mut action_line).await {
                                Ok(n) if n > 0 => {
                                    let trimmed = action_line.trim();
                                    if let Ok(action_str) = serde_json::from_str::<String>(trimmed) {
                                        println!("Aktion empfangen: {}", action_str);
                                        inner_action_queue_ref.lock().unwrap().push_back((netzwerk_id, action_str));
                                    } else {
                                        println!("Ungültige Aktion: {}", trimmed);
                                    }
                                }
                                _ => {
                                     println!("verbindung geschlossen");
                                    break;
                                }
                            }
                        }
                    });
                   
                }
                Err(e) => println!("Verbindungsfehler: {}", e),            }
        }
    });
   //Task der die queue abarbeitet
   let game_loop_game_ref = self.game.clone();
   let game_loop_clients_ref = self.clients.clone();
   let game_loop_queue_ref = self.action_queue.clone();
   let game_loop_clients_to_index_ref = self.client_to_index.clone();

   tokio::spawn(async move{
    //server ticks
    loop{
        let mut actions: Vec<(usize, String)> = Vec::new();
        {
            let mut queue: std::sync::MutexGuard<'_, VecDeque<(usize, String)>> = game_loop_queue_ref.lock().unwrap();
            while let Some(ac) = queue.pop_front() {
                actions.push(ac);
            }
        }

        let mut outbox: Vec<(usize, String)> = Vec::new();

        if !actions.is_empty(){
            let mut game_guard = game_loop_game_ref.lock().unwrap();
            if let Some(game) = game_guard.as_mut(){
                for (clientid, actionsstr) in actions{
                    let player = game_loop_clients_to_index_ref.lock().unwrap().get(&clientid).cloned();
                    let current_player = game.current_player();

                    if let Some(p) = player {
                        if *current_player == p {
                            match game.play(&actionsstr) {
                                Ok(()) => println!("Aktion von {} erfolgreich {}", clientid, actionsstr),
                                Err(e) => {
                                    let err_json = serde_json::to_string(&ServerNachrich::Fehler(format!("Fehler: {}", e))).unwrap() + "\n";
                                    outbox.push((clientid, err_json));
                                }
                            }
                        } else {
                            let err_json = serde_json::to_string(&ServerNachrich::Fehler("Nicht dein Zug!".to_string())).unwrap() + "\n";
                            outbox.push((clientid, err_json));
                        }
                    } else {
                        let err_json = serde_json::to_string(&ServerNachrich::Fehler("Unbekannter Spieler!".to_string())).unwrap() + "\n";
                        outbox.push((clientid, err_json));
                    }

                    
                    
                }
                
            }
        }

        if !outbox.is_empty(){
            
                let state_msg = {
                    let game_guard = game_loop_game_ref.lock().unwrap();
                    game_guard.as_ref().map(|game| {
                        serde_json::to_string(&ServerNachrich::State(game.clone())).unwrap() + "\n"
                    })
                };
               
            
            let mut clients_guard = game_loop_clients_ref.lock().await;

            if let Some(msg) = state_msg{
                broadcast(&mut clients_guard, &msg).await;
            }
            for (clientid, message) in outbox{
                if let Some(writer) = clients_guard.get_mut(clientid){
                    
                    if let Err(e) = writer.write_all(message.as_bytes()).await {
                        println!("Fehler beim Senden an Client {}: {}", clientid, e);
                    }
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
   }
    
   );
    Ok(())
}


   
   
}

async fn broadcast(clients: &mut Vec<OwnedWriteHalf>, message: &str){
    
    for writer in clients{
        if let Err(e )= writer.write_all(message.as_bytes()).await{
            println!("Fehler beim broadcast {}" ,e)  ;
        }
        
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = GameServer::new();
    
    // Server startet alle Hintergrund-Tasks und gibt Ok() zurück
    server.start_server("127.0.0.1:8080").await?;
    
    println!("Server läuft im Hintergrund. Drücke Strg+C zum Beenden.");

    // Wartet unendlich, bis das Betriebssystem ein Beenden-Signal (Strg+C) sendet
    tokio::signal::ctrl_c().await?;
    
    println!("Server wird heruntergefahren...");
    Ok(())
}