//Kommentare by Jonas
// Einfacher TCP-Client für das Spiel. 

use crate::game::{Game, GameVariant, };
use crate::game::player::PlayerType;
use crate::{BeginGameMesage};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

#[derive(Debug)]
pub struct Client {
    pub reader: Option<OwnedReadHalf>,
    pub writer: OwnedWriteHalf,
    #[allow(dead_code)]
    pub server_addresse: String,
    pub game: Option<Game>,
}

impl Client {
    pub async fn new(addr: &str) -> Self {
        let socket = TcpStream::connect(addr).await.unwrap();
        let (reader, writer) = socket.into_split();
        Client {
            reader: Some(reader),
            writer,
            server_addresse: addr.to_owned(),
            game: None,
        }
    }

    /// Hilfsmethode, die nur einen nackten `TcpStream` zurückgibt. Wird
    /// beim Einrichtungsdialog verwendet, falls man erst entscheiden möchte,
    /// ob man ein neues Spiel erstellt oder einem beitritt.
    pub async fn verbinde_client(addr: &str) -> tokio::io::Result<TcpStream> {
        println!("Versuche mit {} zu verbinden", addr);
        let socket = TcpStream::connect(addr).await?;
        println!("Erfolgreich verbunden");
        Ok(socket)
    }

    /// Sendet einen json zug an den Server
    pub async fn make_play(&mut self, play: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut play_json = serde_json::to_vec(&play).unwrap();
        play_json.push(b'\n'); // terminator
        self.writer.write_all(&play_json).await?;
        Ok(())
    }
}


pub async fn starte_client(server_adresse: &str) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect(server_adresse).await?;
    println!("Client hier, Verbunden mit server");

    let variante = GameVariant::ThreeVsThree;
    let json_bytes = serde_json::to_vec(&variante).unwrap();

    socket.write_all(&json_bytes).await?;
    Ok(socket)
}
//Verbinded sich und sendet SPielBeitreten, gibt Client zurück, der Züge verschicken und Nachrichten empfangen kann
pub async fn join_running_game(
    server_adresse: &str,
    player_name: String,
) -> Result<Client, Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect(server_adresse).await?;
    println!("client verbunden");
    let mut join_msg =
        serde_json::to_vec(&BeginGameMesage::SpielBeitreten { player_name }).unwrap();
    join_msg.push(b'\n');
    socket.write_all(&join_msg).await?;
    let (reader, writer) = socket.into_split();

    Ok(Client {
        reader: Some(reader),
        writer,
        server_addresse: server_adresse.to_string(),
        game: None,
    })
}

//Verbindet sich mit dem Server und fordert die Erstellung eines neuen
// Spiels mit den übergebenen Spielern an.
pub async fn create_game(
    server_adresse: &str,
    player_name: String,
    variante: GameVariant,
    player_types: Vec<PlayerType>,
) -> Result<Client, Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect(server_adresse).await?;

    println!("Erstellender Client verbunden");
    let mut join_msg = serde_json::to_vec(&BeginGameMesage::ErstelleSpiel {
        variant: variante,
        player_name: player_name,
        player_types: player_types,
    })
    .unwrap();
    join_msg.push(b'\n');
    println!(
        "Sende ErstelleSpiel: {:?}",
        String::from_utf8_lossy(&join_msg)
    );
    socket.write_all(&join_msg).await?;
    let (reader, writer) = socket.into_split();

    Ok(Client {
        reader: Some(reader),
        writer,
        server_addresse: server_adresse.to_string(),
        game: None,
    })
}
