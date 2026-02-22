use braendi_dog::{BeginGameMesage, ServerNachrich};
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use braendi_dog::game::{Game, GameVariant};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
//use crate::bin::client;
pub struct Client{
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf,
    #[allow(dead_code)]
    server_addresse: String,
    game: Option<Game>,
}
impl Client{
    pub async fn new(addr: &str) -> Self{
        let socket = TcpStream::connect(addr).await.unwrap();
        let (reader, writer) = socket.into_split();
        Client{reader, writer, server_addresse: addr.to_owned(), game: None,  }
    }
    pub async fn verbinde_client(addr: &str)-> tokio::io::Result<TcpStream>{
    println!("Versuche mit {} zu verbinden", addr);

    let socket = TcpStream::connect(addr).await?;

    println!("Erfolgreich verbunden");

    Ok(socket)
}
pub async fn make_play(&mut self, play: &str) -> Result<(), Box<dyn std::error::Error>>{
        let mut play_json = serde_json::to_vec(&play).unwrap();
        play_json.push(b'\n'); // terminator
        self.writer.write_all(&play_json).await?;
        Ok(())
    }

    pub async fn nachrichten_empfangen_loop(&mut self) -> tokio::io::Result<()>{
        let mut reader = tokio::io::BufReader::new(&mut self.reader);
        let mut line = String::new();

        while tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut line).await? !=0{
            println!("RAW empfangen: {:?}", line);
            let message = line.trim();
            if message.is_empty(){
                line.clear();
                continue;
            }

            match serde_json::from_str::<ServerNachrich>(message){
                Ok(ServerNachrich::State(ga)) => {
                    self.game =Some(ga);
                    //TODO: Gui aktualisieren
                }
                 Ok(ServerNachrich::Fehler(e)) => {
                    eprintln!("Server meldet Fehler: {}", e);
                }
                Err(e) => {
                    eprintln!("Ungültige Server‑Nachricht: {}", e);
                }
            }
            line.clear();
        }
        Ok(())
    }

}
pub async fn starte_client(server_adresse: &str) -> Result<TcpStream, Box<dyn std::error::Error>>{
    let mut socket = TcpStream::connect(server_adresse).await?;
    println!("Client hier, Verbunden mit server");

   // socket.write("Moin Servus Moin\n".as_bytes()).await?;
   // socket.write("testmest".as_bytes()).await?;
      
    let variante = GameVariant::ThreeVsThree;
    let json_bytes= serde_json::to_vec(&variante).unwrap();

    socket.write_all(&json_bytes).await?;
    Ok(socket)
}
pub async fn join_running_game(server_adresse: &str, player_name: String) -> Result<Client, Box<dyn std::error::Error>>{
    let mut socket = TcpStream::connect(server_adresse).await?;
    println!("client verbunden");
    let mut join_msg = serde_json::to_vec(&BeginGameMesage::SpielBeitreten { player_name }).unwrap();
    join_msg.push(b'\n');
    socket.write_all(&join_msg).await?;
    let (reader, writer) = socket.into_split();
        
    Ok(Client{reader, writer, server_addresse: server_adresse.to_string(), game: None})

}

pub async fn create_game(server_adresse: &str, player_name: String, variante: GameVariant)-> Result<Client, Box<dyn std::error::Error>>{
    let mut socket = TcpStream::connect(server_adresse).await?;

    println!("Erstellender Client verbunden");
    let mut join_msg = serde_json::to_vec(&BeginGameMesage::ErstelleSpiel { variant: variante, player_name: player_name}).unwrap();
    join_msg.push(b'\n');
    println!("Sende ErstelleSpiel: {:?}", String::from_utf8_lossy(&join_msg));
    socket.write_all(&join_msg).await?;
    let (reader, writer) = socket.into_split();
    Ok(Client{reader, writer, server_addresse: server_adresse.to_string(), game: None})
}
 







//pub async fn send_move()

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // kurz warten bis Server bereit ist
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    println!("=== Erstelle Spiel als Jor ===");
    let mut c1 = create_game("127.0.0.1:8080", "Jor".to_string(), GameVariant::ThreeVsThree).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("=== Clients treten bei ===");
    let mut c2 = join_running_game("127.0.0.1:8080", "Mor".to_string()).await?;
    let mut c3 = join_running_game("127.0.0.1:8080", "gor".to_string()).await?;
    let _c4 = join_running_game("127.0.0.1:8080", "grgr".to_string()).await?;
    let _c5 = join_running_game("127.0.0.1:8080", "gerger".to_string()).await?;
    let _c6 = join_running_game("127.0.0.1:8080", "horst".to_string()).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    println!("=== Test 1: Stein setzen (G 0 P) ===");
    c1.make_play("G 0 P").await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    println!("=== Test 2: Falscher Spieler versucht Zug (sollte Fehler geben) ===");
    c2.make_play("Y 4 M 16 20").await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    println!("=== Test 3: Richtiger Zug von Yellow ===");
    // welcher client yellow ist hängt von der Slot-Zuweisung ab
    c2.make_play("Y 4 M 16 20").await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    println!("=== Test 4: Jack (Interchange) ===");
    c3.make_play("B 11 I 40 45").await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    println!("=== Test 5: Joker-Trade ===");
    c1.make_play("Y 0 T").await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    println!("=== Empfange Nachrichten von c1 ===");
    // Timeout damit der loop nicht ewig blockiert
    tokio::select! {
        res = c1.nachrichten_empfangen_loop() => {
            println!("Empfangs-Loop beendet: {:?}", res);
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
            println!("Timeout – Test abgeschlossen");
        }
    }

    println!("=== Alle Tests abgeschlossen ===");
    Ok(())
}