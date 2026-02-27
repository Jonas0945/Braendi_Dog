use crate as braendi_dog;
use braendi_dog::client::{Client, join_running_game};
use braendi_dog::server::GameServer;
use braendi_dog::{Action, ActionKind, Card, Color as GameColor, DogGame, Game, GameVariant, ServerNachrich, };
use braendi_dog::game::player::PlayerType;
use futures::SinkExt;
use iced::widget::{
    button, canvas, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{
    Application, Color as IcedColor, Command, Element, Length, Point, Renderer, Settings, Size,
    Subscription, Theme, event, executor, mouse, window,
};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::net::UdpSocket;
use std::time::Instant;

type SharedClient = Arc<tokio::sync::Mutex<Client>>;

pub fn launch() -> iced::Result {
    DogApp::run(Settings::default())
}

async fn start_server(addr: String) -> Result<String, String> {
    let server = GameServer::new();
    server
        .start_server(&addr)
        .await
        .map_err(|e| e.to_string())?;
     tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let lan_ip = UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| { s.connect("8.8.8.8:80")?; s.local_addr() })
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());

    let port = addr.split(':').last().unwrap_or("8333");
    Ok(format!("{}:{}", lan_ip, port))
}
async fn join_server(addr: String, player_name: String) -> Result<SharedClient, String> {
    let client = join_running_game(&addr, player_name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Arc::new(tokio::sync::Mutex::new(client)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameVariantKind {
    TwoVsTwo,
    ThreeVsThree,
    TwoVsTwoVsTwo,
    FreeForAll,
}
impl GameVariantKind {
    const ALL: [GameVariantKind; 4] = [
        GameVariantKind::TwoVsTwo,
        GameVariantKind::ThreeVsThree,
        GameVariantKind::TwoVsTwoVsTwo,
        GameVariantKind::FreeForAll,
    ];
}
impl std::fmt::Display for GameVariantKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GameVariantKind::TwoVsTwo => "2 vs 2",
                GameVariantKind::ThreeVsThree => "3 vs 3",
                GameVariantKind::TwoVsTwoVsTwo => "2 vs 2 vs 2",
                GameVariantKind::FreeForAll => "Free For All",
            }
        )
    }
}

#[derive(Debug, Clone)]
enum GameAction {
    Place,
    Move,
    Remove,
    Trade,
    Grab,
    Interchange,
    TradeGrab,
    Undo,
}

#[derive(Debug, Clone)]
enum PendingAction {
    Move { from: Option<usize> },
    Interchange { from: Option<usize> },
    Grab,
    TradeGrab,
}

enum Screen {
    Start,
    BotSetup,
    Game,
    Rules,
    GameOver { winner: GameColor },
}

#[derive(Debug, Clone)]
struct MoveAnimation {
    from: usize,
    to: usize,
    color: IcedColor,
    progress: f32,
    from_zwinger_of_player: Option<usize>,
}
#[derive(Debug, Clone)]
struct Confetti {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    color: IcedColor,
    size: f32,
    rotation: f32,
    rot_speed: f32,
}

struct DogApp {
    game: Option<Game>,
    screen: Screen,
    window_size: Size,
    selected_variant: Option<GameVariantKind>,
    ffa_players_input: String,
    selected_card: Option<Card>,
    pending_action: Option<PendingAction>,
    msg: String,
    join_ip_input: String,
    bind_addr_input: String,
    player_name_input: String,
    client: Option<SharedClient>,

    // NEU: Damit die GUI weiß, wer WIR am PC eigentlich sind!
    local_player_index: Option<usize>,

    selected_opponent: Option<usize>,
    selected_opponent_card: Option<usize>,
    animation: Option<MoveAnimation>,
    last_tick: Instant,
    confetti: Vec<Confetti>,
    _audio_stream: Option<OutputStream>,
    audio_sink: Option<Sink>,
    player_types: Vec<PlayerType>,
    hosting: bool,
}

#[derive(Debug, Clone)]
enum Message {
    WindowResized(Size),
    Tick(Instant),
    VariantSelected(GameVariantKind),
    FreeForAllPlayersChanged(String),
    StartGame,
    ShowRules,
    BackToStart,
    CardSelected(Card),
    BoardClicked(usize),
    GameActionBtn(GameAction),
    CancelPendingAction,
    JoinIpChanged(String),
    BindAddrChanged(String),
    PlayerNameChanged(String),
    HostGame,
    PlayResult(Result<(), String>),
    ServerStarted(Result<String, String>),
    JoinGame,
    JoinResult(Result<SharedClient, String>),
    HostJoined(Result<SharedClient, String>),
    OpponentSelected(usize),
    OpponentCardSelected(usize),
    OpponentCardBack,
    IncomingNetwork(braendi_dog::ServerNachrich),
    GoToBotSetup,
    PlayerTypeChanged(usize, PlayerType),
    ConfirmBotSetup,
    None,
}

impl Application for DogApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut app = DogApp {
            game: None,
            screen: Screen::Start,
            window_size: Size::new(1024.0, 768.0),
            selected_variant: None,
            ffa_players_input: String::new(),
            selected_card: None,
            pending_action: None,
            msg: String::from("Willkommen! Wähle einen Spielmodus."),
            selected_opponent: None,
            selected_opponent_card: None,
            animation: None,
            last_tick: Instant::now(),
            confetti: Vec::new(),
            join_ip_input: String::from("127.0.0.1:8333"),
            bind_addr_input: String::from("0.0.0.0:8333"),
            player_name_input: String::new(),
            client: None,
            local_player_index: None,
            _audio_stream: None,
            audio_sink: None,
            player_types: Vec::new(),
            hosting: false,
        };
        app.play_audio("lobby.mp3", 0.15, true);
        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("Brändi Dog")
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![event::listen().map(|event| {
            if let iced::Event::Window(_, window::Event::Resized { width, height }) = event {
                Message::WindowResized(Size::new(width as f32, height as f32))
            } else {
                Message::None
            }
        })];
        let needs_tick = self.animation.is_some() || matches!(self.screen, Screen::GameOver { .. });
        if needs_tick {
            subs.push(window::frames().map(|_| Message::Tick(Instant::now())));
        }

if let Some(client_arc) = &self.client {
            let client_clone = Arc::clone(client_arc);
            
            // NEU: Ein Guard, der den Reader rettet, falls Iced den Task abbricht
            struct ReaderGuard {
                client: Arc<tokio::sync::Mutex<braendi_dog::client::Client>>,
                reader: Option<tokio::net::tcp::OwnedReadHalf>,
            }
            impl Drop for ReaderGuard {
                fn drop(&mut self) {
                    if let Some(r) = self.reader.take() {
                        let client = self.client.clone();
                        tokio::spawn(async move {
                            client.lock().await.reader = Some(r);
                        });
                    }
                }
            }

            let net_sub = iced::subscription::channel(
                std::any::TypeId::of::<braendi_dog::ServerNachrich>(),
                100,
                |mut ui_sender| async move {
                    let maybe_reader = {
                        let mut guard = client_clone.lock().await;
                        guard.reader.take()
                    };
                    
                    if let Some(reader) = maybe_reader {
                        let mut guard = ReaderGuard {
                            client: client_clone.clone(),
                            reader: Some(reader),
                        };
                        
                        use tokio::io::AsyncBufReadExt;
                        let mut buf_reader = tokio::io::BufReader::new(guard.reader.as_mut().unwrap());
                        let mut line = String::new();
                        loop {
                            line.clear();
                            match buf_reader.read_line(&mut line).await {
                                Ok(0) => break, // EOF
                                Ok(_) => {
                                    let trimmed = line.trim();
                                    if trimmed.is_empty() { continue; }
                                    if let Ok(server_msg) = serde_json::from_str::<braendi_dog::ServerNachrich>(trimmed) {
                                        let _ = ui_sender.send(Message::IncomingNetwork(server_msg)).await;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    }
                    // Verhindert das sofortige Schließen des Channels, falls der Reader gerade woanders ist
                    loop { tokio::time::sleep(std::time::Duration::from_secs(3600)).await; }
                },
            );
            subs.push(net_sub);
        }
        Subscription::batch(subs)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::None => {}
            Message::WindowResized(size) => {
                self.window_size = size;
            }
            Message::Tick(now) => {
                let delta = now.duration_since(self.last_tick).as_secs_f32();
                self.last_tick = now;
                if let Some(anim) = &mut self.animation {
                    anim.progress += delta / 0.4;
                    if anim.progress >= 1.0 {
                        self.animation = None;
                    }
                }
                if matches!(self.screen, Screen::GameOver { .. }) {
                    for c in &mut self.confetti {
                        c.x += c.vx * delta * 60.0;
                        c.y += c.vy * delta * 60.0;
                        c.rotation += c.rot_speed * delta * 60.0;
                        c.vx += (rand::random::<f32>() - 0.5) * 0.2;
                        if c.y > self.window_size.height + 50.0 {
                            c.y = -50.0;
                            c.x = rand::random::<f32>() * self.window_size.width;
                        }
                    }
                }
            }
            Message::ShowRules => {
                self.screen = Screen::Rules;
            }
            Message::BackToStart => {
                self.screen = Screen::Start;
                self.game = None;
                self.client = None;
                self.local_player_index = None;
                self.player_types = Vec::new();
                self.play_audio("lobby.mp3", 0.15, true);
            }
            Message::OpponentSelected(idx) => {
                self.selected_opponent = Some(idx);
                self.selected_opponent_card = None;
                if let Some(game) = &self.game {
                    self.msg = format!("Gegner {:?} gewählt.", game.players[idx].color);
                }
            }
            Message::OpponentCardSelected(card_idx) => {
                self.selected_opponent_card = Some(card_idx);
                match self.pending_action {
                    Some(PendingAction::Grab) => {
                        return self.execute_grab(false);
                    }
                    Some(PendingAction::TradeGrab) => {
                        return self.execute_grab(true);
                    }
                    _ => {
                        self.msg = format!("Karte {} gewählt.", card_idx + 1);
                    }
                }
            }
            Message::OpponentCardBack => {
                self.selected_opponent = None;
                self.selected_opponent_card = None;
            }
            Message::VariantSelected(kind) => {
                self.selected_variant = Some(kind);
                if kind != GameVariantKind::FreeForAll {
                    self.ffa_players_input.clear();
                }
            }
            Message::FreeForAllPlayersChanged(value) => {
                if value.chars().all(|c| c.is_ascii_digit()) {
                    self.ffa_players_input = value;
                }
            }
            Message::GoToBotSetup => {
                if let Some(variant) = self.build_game_variant() {
                    // Anzahl Spieler bestimmen
                    let count = match variant {
                        GameVariant::TwoVsTwo => 4,
                        GameVariant::ThreeVsThree => 6,
                        GameVariant::TwoVsTwoVsTwo => 6,
                        GameVariant::FreeForAll(n) => n,
                    };
                    // Alle als Human vorbelegen
                    self.player_types = vec![PlayerType::Human; count];
                    self.hosting = false;
                    self.screen = Screen::BotSetup;
                }
            }
            Message::PlayerTypeChanged(idx, pt) => {
                if let Some(slot) = self.player_types.get_mut(idx) {
                    *slot = pt;
                }
            }
            Message::ConfirmBotSetup => {
                  if self.hosting {
        // Server starten, player_types mitnehmen
        self.msg = "Starte Server...".into();
        let addr = self.bind_addr_input.clone();
        return Command::perform(start_server(addr), Message::ServerStarted);
    } else {
        if let Some(variant) = self.build_game_variant() {
            let mut game = Game::new(variant, self.player_types.clone());
            game.new_round();
            self.game = Some(game);
            self.local_player_index = None;
            self.screen = Screen::Game;
        }
    }
                /*
                if let Some(variant) = self.build_game_variant() {
                    let mut game = Game::new(variant, self.player_types.clone());
                    game.new_round();
                    self.game = Some(game);
                    self.local_player_index = None;
                    self.screen = Screen::Game;
                }*/
            }
            Message::StartGame => {
                return self.update(Message::GoToBotSetup);
                
              /*  if let Some(variant) = self.build_game_variant() {
                    let mut game = Game::new(variant);
                    game.new_round();
                    self.game = Some(game);
                    self.local_player_index = None; // Lokales Spiel = Perspektive rotiert
                    self.hosting = false;
                    self.screen = Screen::Game;
                }*/
            }
            Message::CardSelected(card) => {
                self.selected_card = Some(card);
                self.pending_action = None;
            }
            Message::CancelPendingAction => {
                self.pending_action = None;
            }
            Message::GameActionBtn(action_type) => {
                return self.handle_btn_click(action_type);
            }
            Message::BoardClicked(tile_index) => {
                return self.handle_board_click(tile_index);
            }
            Message::JoinIpChanged(ip) => {
                self.join_ip_input = ip;
            }
            Message::BindAddrChanged(addr) => {
                self.bind_addr_input = addr;
            }
            Message::PlayerNameChanged(name) => {
                self.player_name_input = name;
            }
            Message::HostGame => {
                if self.player_name_input.is_empty() {
                    self.msg = "Bitte gib deinen Namen ein!".into();
                    return Command::none();
                }
                if self.bind_addr_input.trim().is_empty() {
                    self.msg = "Bitte gib eine Bind-Adresse ein!".into();
                    return Command::none();
                }
                if let Some(variant) = self.build_game_variant() {
                    let count = match variant {
                        GameVariant::TwoVsTwo => 4,
                        GameVariant::ThreeVsThree => 6,
                        GameVariant::TwoVsTwoVsTwo => 6,
                        GameVariant::FreeForAll(n) => n,
                    };
                    self.player_types = vec![PlayerType::Human; count];
                    self.hosting = true;
                    self.screen = Screen::BotSetup;
                }
            }
            Message::ServerStarted(Ok(addr)) => {
                // tell the user where the server is bound so they can share it
                self.msg = format!("Server läuft unter {}", addr);
                let name = self.player_name_input.clone();
                let variant = self.build_game_variant().unwrap();
                let player_types = self.player_types.clone();
                return Command::perform(
                    async move {
                        braendi_dog::client::create_game(&addr, name, variant, player_types)
                            .await
                            .map(|c| Arc::new(tokio::sync::Mutex::new(c)))
                            .map_err(|e| e.to_string())
                    },
                    Message::HostJoined,
                );
            }
            Message::JoinGame => {
                if self.player_name_input.is_empty() {
                    self.msg = "Bitte gib deinen Namen ein!".into();
                    return Command::none();
                }
                self.msg = "Verbinde...".into();
                let addr = self.join_ip_input.clone();
                let name = self.player_name_input.clone();
                return Command::perform(join_server(addr, name), Message::JoinResult);
            }
            Message::JoinResult(result) | Message::HostJoined(result) => match result {
                Ok(client) => {
                    self.client = Some(client);
                    self.screen = Screen::Game;
                    self.msg = "Warte auf Brettdaten...".into();
                }
                Err(e) => {
                    self.msg = format!("Fehler beim Beitreten: {}", e);
                    self.screen = Screen::Start;
                }
            },
            Message::ServerStarted(Err(e)) => {
                self.msg = format!("Server-Fehler: {}", e);
            }
            Message::PlayResult(Ok(())) => {
                self.msg = "Warte auf Server...".into();
            }
            Message::PlayResult(Err(e)) => {
                self.msg = format!("Fehler: {}", e);
            }

            Message::IncomingNetwork(net_msg) => {
                match net_msg {
                    braendi_dog::ServerNachrich::Welcome(idx) => {
                        // FIX: Server verrät uns unsere Farbe!
                        self.local_player_index = Some(idx);
                        self.msg = format!("Verbunden! Du bist Spieler {}.", idx);
                    }
                    braendi_dog::ServerNachrich::State(new_game) => {
                        // NEU: Animation triggern, wenn eine neue Aktion in der History ist!
                        if let Some(old_game) = &self.game {
                            if new_game.history.len() > old_game.history.len() {
                                if let Some(last_entry) = new_game.history.last() {
                                    let mut is_move = false;
                                    let mut from_idx = 0;
                                    let mut to_idx = 0;
                                    let mut from_zwinger = None;

                                    match last_entry.action.action {
                                        ActionKind::Move { from, to } | ActionKind::Split { from, to } => {
                                            is_move = true;
                                            from_idx = from;
                                            to_idx = to;
                                        }
                                        ActionKind::Place { target_player } => {
                                            is_move = true;
                                            to_idx = new_game.board.start_field(target_player);
                                            from_zwinger = Some(target_player);
                                        }
                                        _ => {}
                                    }

                                    if is_move {
                                        let anim_color_enum = match last_entry.action.action {
                                            ActionKind::Place { target_player } => new_game.players[target_player].color,
                                            ActionKind::Move { to, .. } | ActionKind::Split { to, .. } => {
                                                if let Some(piece) = &new_game.board.tiles[to] {
                                                    new_game.players[piece.owner].color
                                                } else {
                                                    last_entry.action.player
                                                }
                                            }
                                            _ => last_entry.action.player,
                                        };

                                        let anim_color_iced = match anim_color_enum {
                                            GameColor::Red => IcedColor::from_rgb(0.8, 0.2, 0.2),
                                            GameColor::Green => IcedColor::from_rgb(0.2, 0.8, 0.2),
                                            GameColor::Blue => IcedColor::from_rgb(0.2, 0.2, 0.8),
                                            GameColor::Yellow => IcedColor::from_rgb(0.8, 0.8, 0.2),
                                            GameColor::Purple => IcedColor::from_rgb(0.5, 0.0, 0.5),
                                            GameColor::Orange => IcedColor::from_rgb(1.0, 0.65, 0.0),
                                        };

                                        self.animation = Some(MoveAnimation {
                                            from: from_idx,
                                            to: to_idx,
                                            color: anim_color_iced,
                                            progress: 0.0,
                                            from_zwinger_of_player: from_zwinger,
                                        });
                                        self.last_tick = Instant::now();
                                    }
                                }
                            }
                        }

                        // Spiel-Status normal übernehmen
                        self.game = Some(new_game);
                        
                        if let Some(game) = &self.game {
                            let my_idx = self.local_player_index.unwrap_or(game.current_player_index);
                            if my_idx == game.current_player_index {
                                self.msg = "Du bist am Zug!".to_string();
                            } else {
                                self.msg = format!(
                                    "Spieler {:?} (P{}) ist am Zug.",
                                    game.current_player().color,
                                    game.current_player_index
                                );
                            }
                            if game.is_winner() {
                                let mut winner_color = game.current_player().color;
                                for p in &game.players {
                                    if p.pieces_in_house == 4 {
                                        winner_color = p.color;
                                        break;
                                    }
                                }
                                self.trigger_win(winner_color);
                            }
                        }
                    }
                    braendi_dog::ServerNachrich::Fehler(err_msg) => {
                        self.msg = err_msg;
                    }
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Start => self.render_start(),
            Screen::BotSetup => self.render_bot_setup(),
            Screen::Game => self.render_game(),
            Screen::Rules => self.render_rules(),
            Screen::GameOver { winner } => self.render_game_over(winner),
        }
    }
}

impl DogApp {
    fn play_audio(&mut self, filename: &str, volume: f32, loop_audio: bool) {
        if let Ok((stream, stream_handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                if let Ok(file) = File::open(filename) {
                    let reader = BufReader::new(file);
                    if let Ok(source) = Decoder::new(reader) {
                        sink.set_volume(volume);
                        if loop_audio {
                            sink.append(source.repeat_infinite());
                        } else {
                            sink.append(source);
                        }
                        sink.play();
                        self._audio_stream = Some(stream);
                        self.audio_sink = Some(sink);
                    }
                }
            }
        }
    }

    fn trigger_win(&mut self, winner: GameColor) {
        self.screen = Screen::GameOver { winner };
        self.animation = None;
        self.last_tick = Instant::now();
        self.play_audio("win.mp3", 0.5, false);
        self.confetti.clear();
        for _ in 0..150 {
            let color = match rand::random::<u8>() % 6 {
                0 => IcedColor::from_rgb(0.9, 0.2, 0.2),
                1 => IcedColor::from_rgb(0.2, 0.9, 0.2),
                2 => IcedColor::from_rgb(0.2, 0.4, 1.0),
                3 => IcedColor::from_rgb(0.9, 0.9, 0.2),
                4 => IcedColor::from_rgb(0.8, 0.2, 0.8),
                _ => IcedColor::from_rgb(1.0, 0.5, 0.0),
            };
            self.confetti.push(Confetti {
                x: rand::random::<f32>() * 2000.0,
                y: rand::random::<f32>() * -1000.0,
                vx: (rand::random::<f32>() - 0.5) * 4.0,
                vy: 2.0 + rand::random::<f32>() * 4.0,
                color,
                size: 8.0 + rand::random::<f32>() * 8.0,
                rotation: rand::random::<f32>() * std::f32::consts::TAU,
                rot_speed: (rand::random::<f32>() - 0.5) * 0.2,
            });
        }
    }

    fn execute_grab(&mut self, trade: bool) -> Command<Message> {
        let Some(card) = self.selected_card else {
            return Command::none();
        };
        let (Some(target_player_idx), Some(target_card)) =
            (self.selected_opponent, self.selected_opponent_card)
        else {
            return Command::none();
        };
        let game = self.game.as_mut().unwrap();
        let my_idx = self.local_player_index.unwrap_or(game.current_player_index);
        let target_color = game.players[target_player_idx].color;

        let action = if trade {
            Action {
                player: game.players[my_idx].color,
                card: None,
                action: ActionKind::TradeGrab { target_card },
            }
        } else {
            Action {
                player: game.players[my_idx].color,
                card: Some(card),
                action: ActionKind::Grab {
                    target_player: target_color,
                    target_card,
                },
            }
        };
        return self.do_action(card, action);
    }

    fn get_possible_moves(&self, my_idx: usize) -> Vec<usize> {
        let Some(game) = &self.game else {
            return vec![];
        };

        match &self.pending_action {
            Some(PendingAction::Move { from: Some(from_idx) }) => {
                let Some(card) = self.selected_card else { return vec![]; };
                let mut targets = Vec::new();
                let board_len = game.board.tiles.len();

                let piece_owner = game.board.tiles[*from_idx]
                    .as_ref()
                    .map(|p| p.owner)
                    .unwrap_or(my_idx);

                let distances: Vec<i8> = if matches!(card, Card::Seven) {
                    let max_steps = game.split_rest.unwrap_or(7) as i8;
                    (1..=max_steps).collect()
                } else {
                    let mut dists: Vec<i8> = card.possible_distances().into_iter().map(|x| x as i8).collect();
                    if matches!(card, Card::Joker | Card::Four) {
                        dists.push(-4); 
                    }
                    dists
                };

                for dist in distances {
                    let backward = dist < 0;
                    let abs_dist = dist.abs() as u8;

                    if !game.can_piece_move_distance(*from_idx, abs_dist, backward) {
                        continue;
                    }

                    for to_idx in 0..board_len {
                        if let Some(piece) = &game.board.tiles[*from_idx] {
                            if *from_idx < game.board.ring_size && to_idx >= game.board.ring_size && !piece.left_start {
                                continue; 
                            }
                        }

                        let ok = if backward {
                            game.board.distance_between(to_idx, *from_idx, piece_owner) == Some(abs_dist)
                        } else {
                            game.board.distance_between(*from_idx, to_idx, piece_owner) == Some(abs_dist)
                        };

                        if ok {
                            targets.push(to_idx);
                        }
                    }
                }
                targets.sort_unstable();
                targets.dedup();
                targets
            }
            
            Some(PendingAction::Interchange { from: Some(first_idx) }) => {
                let mut targets = Vec::new();
                
                let first_owner = game.board.tiles[*first_idx].as_ref().map(|p| p.owner);
                
                for (to_idx, tile) in game.board.tiles.iter().enumerate() {
                    if let Some(piece) = tile {
                        if to_idx < game.board.ring_size && piece.left_start {
                            if Some(piece.owner) != first_owner {
                                targets.push(to_idx);
                            }
                        }
                    }
                }
                targets
            }
            _ => vec![],
        }
    }

    fn build_game_variant(&self) -> Option<GameVariant> {
        match self.selected_variant? {
            GameVariantKind::TwoVsTwo => Some(GameVariant::TwoVsTwo),
            GameVariantKind::ThreeVsThree => Some(GameVariant::ThreeVsThree),
            GameVariantKind::TwoVsTwoVsTwo => Some(GameVariant::TwoVsTwoVsTwo),
            GameVariantKind::FreeForAll => {
                let players: usize = self.ffa_players_input.parse().ok()?;
                if players < 2 || players > 6 {
                    return None;
                }
                Some(GameVariant::FreeForAll(players))
            }
        }
    }

    fn render_start(&self) -> Element<'_, Message> {
        let msg_color = if self.msg.contains("Fehler") || self.msg.contains("Bitte") {
            IcedColor::from_rgb(1.0, 0.4, 0.4)
        } else {
            IcedColor::from_rgb(0.6, 0.9, 0.6)
        };
        let status_msg = text(&self.msg)
            .size(18)
            .style(msg_color)
            .horizontal_alignment(iced::alignment::Horizontal::Center);

        let mut host_controls = column![
            text("Neues Spiel erstellen:")
                .size(18)
                .style(IcedColor::WHITE)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            pick_list(
                &GameVariantKind::ALL[..],
                self.selected_variant,
                Message::VariantSelected
            )
            .placeholder("Spielmodus wählen...")
            .width(Length::Fixed(300.0))
            .padding(15),
            text_input("Bind-Adresse (z.B. 0.0.0.0:8080)", &self.bind_addr_input)
                .on_input(Message::BindAddrChanged)
                .padding(15)
                .width(Length::Fixed(300.0)),
        ]
        .spacing(10)
        .align_items(iced::Alignment::Center);
        if self.selected_variant == Some(GameVariantKind::FreeForAll) {
            host_controls = host_controls
                .push(iced::widget::Space::with_height(Length::Fixed(5.0)))
                .push(
                    text_input("Anzahl Spieler (2-6)", &self.ffa_players_input)
                        .on_input(Message::FreeForAllPlayersChanged)
                        .padding(15)
                        .width(Length::Fixed(300.0)),
                );
        }

        let can_start = self.build_game_variant().is_some();
        host_controls = host_controls
            .push(iced::widget::Space::with_height(Length::Fixed(10.0)))
            .push(
                row![
                    button(
                        text("Lokal Starten")
                            .size(18)
                            .horizontal_alignment(iced::alignment::Horizontal::Center)
                    )
                    .padding([12, 30])
                    .on_press_maybe(can_start.then_some(Message::StartGame)),
                    button(
                        text("Spiel Hosten")
                            .size(18)
                            .horizontal_alignment(iced::alignment::Horizontal::Center)
                    )
                    .padding([12, 30])
                    .style(iced::theme::Button::Positive)
                    .on_press_maybe(can_start.then_some(Message::HostGame))
                ]
                .spacing(15),
            );

            if self.msg.starts_with("Server läuft") {
        host_controls = host_controls.push(
            text(&self.msg)
                .size(16)
                .style(IcedColor::from_rgb(0.3, 1.0, 0.5))
                .horizontal_alignment(iced::alignment::Horizontal::Center)
        );
    }

        let join_controls = column![
            text("Oder bestehendem Spiel beitreten:")
                .size(18)
                .style(IcedColor::WHITE)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            text_input("Dein Spielername...", &self.player_name_input)
                .on_input(Message::PlayerNameChanged)
                .padding(15)
                .width(Length::Fixed(300.0)),
            text_input("Server-Adresse (z.B. 127.0.0.1:8080)", &self.join_ip_input)
                .on_input(Message::JoinIpChanged)
                .padding(15)
                .width(Length::Fixed(300.0)),
            iced::widget::Space::with_height(Length::Fixed(5.0)),
            button(
                text("Spiel Beitreten")
                    .size(20)
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
            )
            .padding([15, 60])
            .style(iced::theme::Button::Primary)
            .on_press(Message::JoinGame)
        ]
        .spacing(10)
        .align_items(iced::Alignment::Center);

        container(
            container(
                column![
                    text("Brändi Dog")
                        .size(80)
                        .style(IcedColor::from_rgb(0.9, 0.75, 0.2))
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                    text("Willkommen am Spieltisch")
                        .size(22)
                        .style(IcedColor::from_rgb(0.7, 0.7, 0.7))
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                    iced::widget::Space::with_height(Length::Fixed(20.0)),
                    status_msg,
                    iced::widget::Space::with_height(Length::Fixed(20.0)),
                    host_controls,
                    iced::widget::Space::with_height(Length::Fixed(30.0)),
                    join_controls,
                    iced::widget::Space::with_height(Length::Fixed(30.0)),
                    button(
                        text("Spielregeln")
                            .size(16)
                            .horizontal_alignment(iced::alignment::Horizontal::Center)
                    )
                    .padding([10, 30])
                    .style(iced::theme::Button::Secondary)
                    .on_press(Message::ShowRules),
                ]
                .align_items(iced::Alignment::Center),
            )
            .padding(50)
            .style(|_: &Theme| container::Appearance {
                background: Some(iced::Background::Color(IcedColor::from_rgba(
                    0.0, 0.0, 0.0, 0.6,
                ))),
                border: iced::Border {
                    radius: 20.0.into(),
                    width: 3.0,
                    color: IcedColor::from_rgb(0.6, 0.4, 0.2),
                },
                ..Default::default()
            }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(|_: &Theme| container::Appearance {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.1, 0.3, 0.2))),
            ..Default::default()
        })
        .into()
    }
    fn render_bot_setup(&self) -> Element<'_, Message> {
        let title = text("Spieler konfigurieren")
            .size(40)
            .style(IcedColor::from_rgb(0.9, 0.75, 0.2))
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .width(Length::Fill);

        let mut player_rows = column![].spacing(12);

        let colors = [
            GameColor::Red, GameColor::Green, GameColor::Blue,
            GameColor::Yellow, GameColor::Purple, GameColor::Orange,
        ];

        for (idx, pt) in self.player_types.iter().enumerate() {
            let color_name = format!("{:?}", colors[idx % colors.len()]);
            let label = text(format!("Spieler {} ({}):", idx + 1, color_name))
                .size(18)
                .style(IcedColor::WHITE)
                .width(Length::Fixed(200.0));

            let btn_human = button(text("Mensch").size(16))
                .padding([8, 16])
                .style(if *pt == PlayerType::Human {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::PlayerTypeChanged(idx, PlayerType::Human));

            let btn_random = button(text("Zufalls-Bot").size(16))
                .padding([8, 16])
                .style(if *pt == PlayerType::RandomBot {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::PlayerTypeChanged(idx, PlayerType::RandomBot));

            let btn_eval = button(text("Eval-Bot").size(16))
                .padding([8, 16])
                .style(if *pt == PlayerType::EvalBot {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::PlayerTypeChanged(idx, PlayerType::EvalBot));

            player_rows = player_rows.push(
                row![label, btn_human, btn_random, btn_eval]
                    .spacing(10)
                    .align_items(iced::Alignment::Center),
            );
        }

        let bottom_btns = row![
            button(text("Zurück").size(18))
                .padding([12, 30])
                .style(iced::theme::Button::Destructive)
                .on_press(Message::BackToStart),
            button(text("Spiel starten!").size(18))
                .padding([12, 30])
                .style(iced::theme::Button::Positive)
                .on_press(Message::ConfirmBotSetup),
        ]
        .spacing(20);

        container(
            container(
                column![
                    title,
                    iced::widget::Space::with_height(Length::Fixed(30.0)),
                    player_rows,
                    iced::widget::Space::with_height(Length::Fixed(30.0)),
                    bottom_btns,
                ]
                .align_items(iced::Alignment::Center),
            )
            .padding(50)
            .style(|_: &Theme| container::Appearance {
                background: Some(iced::Background::Color(IcedColor::from_rgba(0.0, 0.0, 0.0, 0.6))),
                border: iced::Border {
                    radius: 20.0.into(),
                    width: 3.0,
                    color: IcedColor::from_rgb(0.6, 0.4, 0.2),
                },
                ..Default::default()
            }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(|_: &Theme| container::Appearance {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.1, 0.3, 0.2))),
            ..Default::default()
        })
        .into()
    }

    fn render_rules(&self) -> Element<'_, Message> {
        let rules_text_content = "Viel Text ..."; // BITTE WIEDER REGELN EINFÜGEN!
        container(
            container(
                column![
                    text("Spielregeln")
                        .size(50)
                        .style(IcedColor::from_rgb(0.9, 0.75, 0.2))
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                        .width(Length::Fill),
                    iced::widget::Space::with_height(Length::Fixed(20.0)),
                    scrollable(
                        text(rules_text_content)
                            .size(18)
                            .style(IcedColor::from_rgb(0.9, 0.9, 0.9))
                            .width(Length::Fill)
                    )
                    .height(Length::Fill),
                    iced::widget::Space::with_height(Length::Fixed(20.0)),
                    container(
                        button(
                            text("Zurück zum Menü")
                                .size(20)
                                .horizontal_alignment(iced::alignment::Horizontal::Center)
                        )
                        .padding([10, 40])
                        .style(iced::theme::Button::Destructive)
                        .on_press(Message::BackToStart)
                    )
                    .width(Length::Fill)
                    .center_x()
                ]
                .padding(40),
            )
            .width(Length::Fixed(850.0))
            .height(Length::FillPortion(8))
            .style(|_: &Theme| container::Appearance {
                background: Some(iced::Background::Color(IcedColor::from_rgba(
                    0.0, 0.0, 0.0, 0.7,
                ))),
                border: iced::Border {
                    radius: 20.0.into(),
                    width: 3.0,
                    color: IcedColor::from_rgb(0.6, 0.4, 0.2),
                },
                ..Default::default()
            }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .padding(40)
        .style(|_: &Theme| container::Appearance {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.1, 0.3, 0.2))),
            ..Default::default()
        })
        .into()
    }

    fn render_game_over(&self, winner: GameColor) -> Element<'_, Message> {
        let (color_name, glow_color) = match winner {
            GameColor::Red => ("TEAM ROT", IcedColor::from_rgb(1.0, 0.3, 0.3)),
            GameColor::Green => ("TEAM GRÜN", IcedColor::from_rgb(0.3, 1.0, 0.3)),
            GameColor::Blue => ("TEAM BLAU", IcedColor::from_rgb(0.3, 0.5, 1.0)),
            GameColor::Yellow => ("TEAM GELB", IcedColor::from_rgb(1.0, 1.0, 0.3)),
            GameColor::Purple => ("TEAM LILA", IcedColor::from_rgb(0.8, 0.3, 1.0)),
            GameColor::Orange => ("TEAM ORANGE", IcedColor::from_rgb(1.0, 0.6, 0.2)),
        };
        container(
            row![
                container(
                    canvas(ConfettiView {
                        confetti: &self.confetti
                    })
                    .width(Length::Fill)
                    .height(Length::Fill)
                )
                .width(Length::Fill)
                .height(Length::Fill)
            ]
            .push(
                container(
                    container(
                        container(
                            column![
                                text("HERZLICHEN GLÜCKWUNSCH!")
                                    .size(40)
                                    .style(IcedColor::WHITE)
                                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                                iced::widget::Space::with_height(Length::Fixed(20.0)),
                                text(format!("{} HAT GEWONNEN!", color_name))
                                    .size(70)
                                    .style(glow_color)
                                    .horizontal_alignment(iced::alignment::Horizontal::Center),
                                iced::widget::Space::with_height(Length::Fixed(60.0)),
                                button(
                                    text("Zurück zum Hauptmenü")
                                        .size(24)
                                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                                )
                                .padding([15, 50])
                                .style(iced::theme::Button::Primary)
                                .on_press(Message::BackToStart)
                            ]
                            .align_items(iced::Alignment::Center),
                        )
                        .padding(60)
                        .style(move |_: &Theme| container::Appearance {
                            background: Some(iced::Background::Color(IcedColor::from_rgba(
                                0.0, 0.0, 0.0, 0.85,
                            ))),
                            border: iced::Border {
                                radius: 20.0.into(),
                                width: 4.0,
                                color: glow_color,
                            },
                            ..Default::default()
                        }),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y(),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(iced::theme::Container::Transparent),
            ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &Theme| container::Appearance {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.1, 0.3, 0.2))),
            ..Default::default()
        })
        .into()
    }

    fn render_game(&self) -> Element<'_, Message> {
        let Some(game) = self.game.as_ref() else {
            return container(text(&self.msg).size(30).style(IcedColor::WHITE))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into();
        };

        let ui_scale = (self.window_size.width / 1024.0).max(0.7);
        let sidebar_width = 320.0 * ui_scale;

        let my_idx = self.local_player_index.unwrap_or(game.current_player_index);

        let board = canvas(BoardView {
            game,
            highlights: self.get_possible_moves(my_idx),
            animation: self.animation.clone(),
            my_idx,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let main_area = row![
            container(board)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(iced::theme::Container::Transparent),
            container(self.make_sidebar(game, ui_scale, my_idx))
                .width(Length::Fixed(sidebar_width))
                .padding(20.0 * ui_scale)
                .style(|_: &Theme| container::Appearance {
                    background: Some(iced::Background::Color(IcedColor::from_rgba(
                        0.0, 0.0, 0.0, 0.3
                    ))),
                    text_color: Some(IcedColor::WHITE),
                    ..Default::default()
                })
        ]
        .spacing(0)
        .height(Length::Fill);

        container(column![
            self.build_grab_bar(ui_scale, my_idx)
                .unwrap_or_else(|| container(text("")).padding(0).into()),
            main_area,
            container(self.make_hand_view(game, my_idx))
                .padding(0)
                .height(Length::Fixed(180.0))
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &Theme| container::Appearance {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.1, 0.3, 0.2))),
            ..Default::default()
        })
        .into()
    }

    fn build_grab_bar(&self, scale: f32, _my_idx: usize) -> Option<Element<'_, Message>> {
        if let Some(pending) = &self.pending_action {
            match pending {
                PendingAction::Grab => {
                    if self.selected_opponent.is_none() {
                        let mut row_buttons = row![].spacing(5.0 * scale);
                        for (idx, player) in self.game.as_ref().unwrap().players.iter().enumerate()
                        {
                            if idx == _my_idx {
                                continue;
                            }
                            row_buttons = row_buttons.push(
                                button(text(format!("{:?}", player.color)).size(14.0 * scale))
                                    .on_press(Message::OpponentSelected(idx)),
                            );
                        }
                        Some(container(row_buttons).padding(5.0 * scale).into())
                    } else {
                        let opponent_idx = self.selected_opponent.unwrap();
                        let opponent_cards =
                            &self.game.as_ref().unwrap().players[opponent_idx].cards;
                        let mut row_buttons = row![].spacing(5.0 * scale);
                        for (idx, _) in opponent_cards.iter().enumerate() {
                            row_buttons = row_buttons.push(
                                button(text(format!("{}", idx + 1)).size(14.0 * scale))
                                    .on_press(Message::OpponentCardSelected(idx)),
                            );
                        }
                        row_buttons = row_buttons.push(
                            button(text("Zurück").size(14.0 * scale))
                                .style(iced::theme::Button::Destructive)
                                .on_press(Message::OpponentCardBack),
                        );
                        Some(container(row_buttons).padding(5.0 * scale).into())
                    }
                }
                PendingAction::TradeGrab => {
                    let opponent_idx = self.selected_opponent.unwrap();
                    let opponent_cards = &self.game.as_ref().unwrap().players[opponent_idx].cards;
                    let mut row_buttons = row![].spacing(5.0 * scale);
                    for (idx, _) in opponent_cards.iter().enumerate() {
                        row_buttons = row_buttons.push(
                            button(text(format!("{}", idx + 1)).size(14.0 * scale))
                                .on_press(Message::OpponentCardSelected(idx)),
                        );
                    }
                    Some(container(row_buttons).padding(5.0 * scale).into())
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn make_hand_view<'a>(&self, game: &'a Game, my_idx: usize) -> Element<'a, Message> {
        container(
            canvas(HandView {
                game,
                selected_card: self.selected_card,
                my_idx,
            })
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Transparent)
        .into()
    }

    fn debug_view(&self, scale: f32) -> Element<'_, Message> {
        let font_size = 14.0 * scale;
        let game_debug = if let Some(game) = &self.game {
            column![
                text("GAME").size(font_size).style(IcedColor::WHITE),
                text(format!("round: {}", game.round))
                    .size(font_size)
                    .style(IcedColor::from_rgb(0.8, 0.8, 0.8)),
                text(format!("trading_phase: {}", game.trading_phase))
                    .size(font_size)
                    .style(IcedColor::from_rgb(0.8, 0.8, 0.8)),
                text(format!("current_player: {}", game.current_player_index))
                    .size(font_size)
                    .style(IcedColor::from_rgb(0.8, 0.8, 0.8)),
            ]
            .spacing(4.0 * scale)
        } else {
            column![text("GAME: none").size(font_size)].spacing(4.0 * scale)
        };
        column![
            text("DEBUG").size(16.0 * scale).style(IcedColor::WHITE),
            text(format!("sel_card: {:?}", self.selected_card))
                .size(font_size)
                .style(IcedColor::from_rgb(0.8, 0.8, 0.8)),
            text(format!("pending: {:?}", self.pending_action))
                .size(font_size)
                .style(IcedColor::from_rgb(0.8, 0.8, 0.8)),
            text("----------------")
                .size(font_size)
                .style(IcedColor::from_rgb(0.5, 0.5, 0.5)),
            game_debug,
        ]
        .spacing(4.0 * scale)
        .into()
    }

    fn make_sidebar(&self, game: &Game, scale: f32, my_idx: usize) -> Element<'_, Message> {
        let font_std = 16.0 * scale;
        let info = column![
            text(format!("Runde: {}", game.round))
                .size(font_std)
                .style(IcedColor::WHITE),
            text(format!("Du bist: {:?}", game.players[my_idx].color))
                .size(font_std)
                .style(IcedColor::WHITE),
            text(&self.msg)
                .size(14.0 * scale)
                .style(IcedColor::from_rgb(0.9, 0.9, 0.9)),
        ]
        .spacing(5.0 * scale);

        let mut btns = column![].spacing(8.0 * scale);

        // FIX: Schalte alle Buttons aus, wenn wir nicht am Zug sind!
        let is_my_turn = my_idx == game.current_player_index;

        if !is_my_turn {
            btns = btns.push(
                text("Warte auf anderen Spieler...")
                    .size(18.0 * scale)
                    .style(IcedColor::from_rgb(1.0, 0.8, 0.2)),
            );
        } else {
            if !game.trading_phase {
                if let Some(rest) = game.split_rest {
                    btns = btns.push(
                        text(format!("Split: {} Schritte übrig!", rest))
                            .size(18.0 * scale)
                            .style(IcedColor::from_rgb(1.0, 0.8, 0.2)),
                    );
                    btns = btns.push(
                        button(text("Weiter aufteilen (Move)").size(font_std))
                            .on_press(Message::GameActionBtn(GameAction::Move))
                            .width(Length::Fill),
                    );
                    // NEU: Der Split-Abbrechen Button
                    btns = btns.push(
                        button(text("Split abbrechen (Undo)").size(font_std))
                            .style(iced::theme::Button::Destructive)
                            .on_press(Message::GameActionBtn(GameAction::Undo))
                            .width(Length::Fill),
                    );
                }else {
                    let can_do_anything = game.check_if_any_action_possible();
                    if !can_do_anything {
                        btns = btns.push(
                            text("Kein gültiger Zug möglich!")
                                .size(16.0 * scale)
                                .style(IcedColor::from_rgb(1.0, 0.5, 0.5)),
                        );
                        btns = btns.push(
                            button(text("Abwerfen (Remove)").size(font_std))
                                .on_press(Message::GameActionBtn(GameAction::Remove))
                                .width(Length::Fill),
                        );
                    } else {
                        if let Some(card) = self.selected_card {
                            let is_place_card =
                                matches!(card, Card::Ace | Card::King | Card::Joker);
                            let is_move_card = !card.possible_distances().is_empty()
                                || matches!(card, Card::Seven | Card::Joker);
                            let is_jack = matches!(card, Card::Jack);
                            let is_interchange_card = matches!(card, Card::Jack | Card::Joker);
                            let is_grab_card = matches!(card, Card::Two);
                            let is_ffa =
                                matches!(self.selected_variant, Some(GameVariantKind::FreeForAll));

                            if is_move_card && !is_jack {
                                btns = btns.push(
                                    button(text("Ziehen / Split (Move)").size(font_std))
                                        .on_press(Message::GameActionBtn(GameAction::Move))
                                        .width(Length::Fill),
                                );
                            }
                            if is_place_card {
                                btns = btns.push(
                                    button(text("Legen (Place)").size(font_std))
                                        .on_press(Message::GameActionBtn(GameAction::Place))
                                        .width(Length::Fill),
                                );
                            }
                            if is_interchange_card {
                                btns = btns.push(
                                    button(text("Tauschen (Interchange)").size(font_std))
                                        .on_press(Message::GameActionBtn(GameAction::Interchange))
                                        .width(Length::Fill),
                                );
                            }
                            if is_grab_card && is_ffa {
                                btns = btns.push(
                                    button(text("Klauen (Grab)").size(font_std))
                                        .on_press(Message::GameActionBtn(GameAction::Grab))
                                        .width(Length::Fill),
                                );
                            }
                        } else {
                            btns = btns.push(
                                text("Wähle eine Karte aus, um mögliche Aktionen zu sehen.")
                                    .size(14.0 * scale)
                                    .style(IcedColor::from_rgb(0.7, 0.7, 0.7)),
                            );
                        }
                    }
                }
            } else {
                if matches!(self.selected_variant, Some(GameVariantKind::FreeForAll)) {
                    btns = btns.push(
                        button(text("Tausch-Klau (TradeGrab)").size(font_std))
                            .on_press(Message::GameActionBtn(GameAction::TradeGrab))
                            .width(Length::Fill),
                    );
                } else {
                    btns = btns.push(
                        button(text("Handel (Trade)").size(font_std))
                            .on_press(Message::GameActionBtn(GameAction::Trade))
                            .width(Length::Fill),
                    );
                }
            }

            if self.pending_action.is_some() {
                btns = btns.push(
                    button(text("Abbrechen").size(font_std))
                        .style(iced::theme::Button::Destructive)
                        .on_press(Message::CancelPendingAction),
                );
            }
        }

        column![
            info,
            btns,
            container(self.debug_view(scale))
                .padding(10.0 * scale)
                .style(move |_: &Theme| container::Appearance {
                    background: Some(iced::Background::Color(IcedColor::from_rgba(
                        0.0, 0.0, 0.0, 0.3
                    ))),
                    text_color: Some(IcedColor::from_rgb(0.7, 0.7, 0.7)),
                    border: iced::Border {
                        radius: (5.0 * scale).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
        ]
        .spacing(30.0 * scale)
        .into()
    }

    fn handle_btn_click(&mut self, action_type: GameAction) -> Command<Message> {
        let game = self.game.as_ref().unwrap();
        let my_idx = self.local_player_index.unwrap_or(game.current_player_index);
        
        if my_idx != game.current_player_index {
            self.msg = "Es ist nicht dein Zug!".into();
            return Command::none();
        }

        // NEU: Sonderbehandlung für Undo (benötigt keine ausgewählte Karte!)
        if matches!(action_type, GameAction::Undo) {
            let play_str = "undo".to_string();
            self.selected_card = None;
            self.pending_action = None;
            
            if let Some(client) = &self.client {
                let client = Arc::clone(client);
                return Command::perform(
                    async move {
                        client.lock().await.make_play(&play_str).await.map_err(|e| e.to_string())
                    },
                    Message::PlayResult,
                );
            } else {
                if let Some(game) = self.game.as_mut() {
                    let _ = game.undo_turn();
                }
                return Command::none();
            }
        }

        // Ab hier geht der Code weiter für Aktionen, die zwingend eine Karte erfordern
        if self.selected_card.is_none() {
            self.msg = "Erst Karte wählen!".into();
            return Command::none();
        }
        let card = self.selected_card.unwrap();
        let current_color = game.players[my_idx].color;

        match action_type {
            GameAction::Undo => unreachable!(), // Wird bereits oben abgefangen
            GameAction::Grab => {
                self.pending_action = Some(PendingAction::Grab);
                self.selected_opponent = None;
                self.selected_opponent_card = None;
                self.msg = "Wähle einen Gegner zum Klauen.".into();
            }
            GameAction::TradeGrab => {
                let prev_idx = if my_idx == 0 {
                    game.players.len() - 1
                } else {
                    my_idx - 1
                };
                self.pending_action = Some(PendingAction::TradeGrab);
                self.selected_opponent = Some(prev_idx);
                self.selected_opponent_card = None;
                self.msg = format!(
                    "Tausch-Klau: Wähle Karte von {:?}.",
                    game.players[prev_idx].color
                );
            }
            GameAction::Move => {
                self.pending_action = Some(PendingAction::Move { from: None });
                self.msg = "Wähle Figur (Start).".into();
            }
            GameAction::Interchange => {
                self.pending_action = Some(PendingAction::Interchange { from: None });
                self.msg = "Wähle erste Figur zum Tauschen.".into();
            }
            GameAction::Place => {
                let act = Action {
                    player: current_color,
                    card: Some(card),
                    action: ActionKind::Place {
                        target_player: my_idx,
                    },
                };
                return self.do_action(card, act);
            }
            GameAction::Remove => {
                let act = Action {
                    player: current_color,
                    card: Some(card),
                    action: ActionKind::Remove,
                };
                return self.do_action(card, act);
            }
            GameAction::Trade => {
                let act = Action {
                    player: current_color,
                    card: Some(card),
                    action: ActionKind::Trade,
                };
                return self.do_action(card, act);
            }
        }
        Command::none()
    }

    fn handle_board_click(&mut self, tile_idx: usize) -> Command<Message> {
        if self.selected_card.is_none() {
            self.msg = format!("Feld {} geklickt. Wähle erst Karte!", tile_idx);
            return Command::none();
        }
        let card = self.selected_card.unwrap();

        let game = self.game.as_ref().unwrap();
        let my_idx = self.local_player_index.unwrap_or(game.current_player_index);
        if my_idx != game.current_player_index {
            self.msg = "Es ist nicht dein Zug!".into();
            return Command::none();
        }
        let current_color = game.players[my_idx].color;

        if let Some(pending) = self.pending_action.clone() {
            match pending {
                PendingAction::Grab | PendingAction::TradeGrab => {
                    self.msg = "Wähle einen Gegner und eine Karte oben.".into();
                }
                PendingAction::Move { from } => {
                    if let Some(start_idx) = from {
                        let action_kind = if matches!(card, Card::Seven) {
                            ActionKind::Split {
                                from: start_idx,
                                to: tile_idx,
                            }
                        } else {
                            ActionKind::Move {
                                from: start_idx,
                                to: tile_idx,
                            }
                        };
                        let act = Action {
                            player: current_color,
                            card: Some(card),
                            action: action_kind,
                        };
                        return self.do_action(card, act);
                    } else {
                        self.pending_action = Some(PendingAction::Move {
                            from: Some(tile_idx),
                        });
                        self.msg = format!("Start: {}. Wähle Ziel!", tile_idx);
                    }
                }
                PendingAction::Interchange { from } => match from {
                    None => {
                        self.pending_action = Some(PendingAction::Interchange {
                            from: Some(tile_idx),
                        });
                        self.msg = format!("Figur 1: {}. Wähle Figur 2.", tile_idx);
                    }
                    Some(first_idx) => {
                        let act = Action {
                            player: current_color,
                            card: Some(card),
                            action: ActionKind::Interchange {
                                a: first_idx,
                                b: tile_idx,
                            },
                        };
                        return self.do_action(card, act);
                    }
                },
            }
        } else {
            self.msg = "Wähle rechts erst eine Aktion (z.B. Move).".to_string();
        }
        Command::none()
    }

    fn do_action(&mut self, card: Card, mut action: Action) -> Command<Message> {
        let mut is_move = false;
        let mut from_idx = 0;
        let mut to_idx = 0;
        let mut from_zwinger = None;
        match action.action {
            ActionKind::Move { from, to } => {
                is_move = true;
                from_idx = from;
                to_idx = to;
            }
            ActionKind::Split { from, to } => {
                is_move = true;
                from_idx = from;
                to_idx = to;
            }
            ActionKind::Place { target_player } => {
                is_move = true;
                to_idx = self.game.as_ref().unwrap().board.start_field(target_player);
                from_zwinger = Some(target_player);
            }
            _ => {}
        };
        if matches!(action.action, ActionKind::TradeGrab { .. }) {
            action.card = None;
        } else {
            action.card = Some(card);
        }

        let my_idx = self
            .local_player_index
            .unwrap_or(self.game.as_ref().unwrap().current_player_index);
        let current_color_enum = self.game.as_ref().unwrap().players[my_idx].color;

        let anim_color_enum = match action.action {
            ActionKind::Place { target_player } => {
                self.game.as_ref().unwrap().players[target_player].color
            }
            ActionKind::Move { from, .. } | ActionKind::Split { from, .. } => {
                if let Some(piece) = &self.game.as_ref().unwrap().board.tiles[from] {
                    self.game.as_ref().unwrap().players[piece.owner].color
                } else {
                    current_color_enum
                }
            }
            _ => current_color_enum,
        };

        let anim_color_iced = match anim_color_enum {
            GameColor::Red => IcedColor::from_rgb(0.8, 0.2, 0.2),
            GameColor::Green => IcedColor::from_rgb(0.2, 0.8, 0.2),
            GameColor::Blue => IcedColor::from_rgb(0.2, 0.2, 0.8),
            GameColor::Yellow => IcedColor::from_rgb(0.8, 0.8, 0.2),
            GameColor::Purple => IcedColor::from_rgb(0.5, 0.0, 0.5),
            GameColor::Orange => IcedColor::from_rgb(1.0, 0.65, 0.0),
        };

        if let Some(client) = &self.client {
            let client = Arc::clone(client);
            let play_str = action.to_string();
            self.selected_card = None;
            self.pending_action = None;
            self.selected_opponent = None;
            self.selected_opponent_card = None;
            Command::perform(
                async move {
                    client
                        .lock()
                        .await
                        .make_play(&play_str)
                        .await
                        .map_err(|e| e.to_string())
                },
                Message::PlayResult,
            )
        } else {
            if let Some(game) = self.game.as_mut() {
                match game.action(action.card, action) {
                    Ok(_) => {
                        if is_move {
                            self.animation = Some(MoveAnimation {
                                from: from_idx,
                                to: to_idx,
                                color: anim_color_iced,
                                progress: 0.0,
                                from_zwinger_of_player: from_zwinger,
                            });
                            self.last_tick = Instant::now();
                        }
                        if game.is_winner() {
                            self.trigger_win(current_color_enum);
                        }
                    }
                    Err(e) => self.msg = format!("Fehler: {}", e),
                }
                self.selected_card = None;
                self.pending_action = None;
                self.selected_opponent = None;
                self.selected_opponent_card = None;
            }
            Command::none()
        }
    }
}

struct ConfettiView<'a> {
    confetti: &'a [Confetti],
}
impl<'a> canvas::Program<Message> for ConfettiView<'a> {
    type State = ();
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        for c in self.confetti {
            let rect =
                canvas::Path::rectangle(Point::new(c.x, c.y), Size::new(c.size, c.size * 1.5));
            frame.with_save(|frame| {
                frame.translate(iced::Vector::new(c.x + c.size / 2.0, c.y + c.size * 0.75));
                frame.rotate(c.rotation);
                frame.translate(iced::Vector::new(-c.x - c.size / 2.0, -c.y - c.size * 0.75));
                frame.fill(&rect, c.color);
            });
        }
        vec![frame.into_geometry()]
    }
}

struct BoardView<'a> {
    game: &'a Game,
    highlights: Vec<usize>,
    animation: Option<MoveAnimation>,
    my_idx: usize,
}
fn get_tile_position(
    index: usize,
    total_players: usize,
    center: Point,
    scale: f32,
    rotation_angle: f32,
) -> Point {
    let r_ring = 250.0 * scale;
    let ring_size = total_players * 16;
    if index < ring_size {
        let angle = (index as f32 / ring_size as f32) * std::f32::consts::TAU;
        let final_angle = angle + rotation_angle;
        Point::new(
            center.x + r_ring * final_angle.cos(),
            center.y + r_ring * final_angle.sin(),
        )
    } else {
        let house_global_index = index - ring_size;
        let player_idx = house_global_index / 4;
        let step = house_global_index % 4;
        let start_idx = player_idx * 16;
        let angle = (start_idx as f32 / ring_size as f32) * std::f32::consts::TAU;
        let final_angle = angle + rotation_angle;
        let r_current = r_ring - (30.0 * scale) - (step as f32 * 35.0 * scale);
        Point::new(
            center.x + r_current * final_angle.cos(),
            center.y + r_current * final_angle.sin(),
        )
    }
}
fn is_hit(cursor: Point, pos: Point, radius: f32) -> bool {
    let dx = cursor.x - pos.x;
    let dy = cursor.y - pos.y;
    (dx * dx + dy * dy).sqrt() < radius
}

impl<'a> canvas::Program<Message> for BoardView<'a> {
    type State = ();
    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let cursor_position = if let Some(p) = cursor.position_in(bounds) {
            p
        } else {
            return (canvas::event::Status::Ignored, None);
        };
        if let canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
            let total_players = self.game.players.len();
            let total_tiles = total_players * 16 + total_players * 4;
            let ring_size = self.game.board.ring_size;
            let scale = bounds.width.min(bounds.height) / 850.0;
            let my_angle = (self.my_idx as f32 * 16.0 / ring_size as f32) * std::f32::consts::TAU;
            let rotation = std::f32::consts::FRAC_PI_2 - my_angle;
            for i in 0..total_tiles {
                let pos = get_tile_position(i, total_players, center, scale, rotation);
                if is_hit(cursor_position, pos, 12.0 * scale) {
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::BoardClicked(i)),
                    );
                }
            }
        }
        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = frame.center();
        let total_players = self.game.players.len();
        let ring_size = self.game.board.ring_size;
        let total_tiles = ring_size + total_players * 4;
        let scale = bounds.width.min(bounds.height) / 850.0;

        // FIX: Brett dreht sich jetzt immer passend zu deiner EIGENEN Farbe!
        let my_angle = (self.my_idx as f32 * 16.0 / ring_size as f32) * std::f32::consts::TAU;
        let rotation = std::f32::consts::FRAC_PI_2 - my_angle;
        let board_radius = 340.0 * scale;
        let line_width = 3.0 * scale;

        let shadow = canvas::Path::circle(
            Point::new(center.x + 10.0 * scale, center.y + 10.0 * scale),
            board_radius,
        );
        frame.fill(&shadow, IcedColor::from_rgba(0.0, 0.0, 0.0, 0.5));
        let bg = canvas::Path::circle(center, board_radius);
        frame.fill(&bg, IcedColor::from_rgb(0.85, 0.70, 0.55));
        frame.stroke(
            &bg,
            canvas::Stroke::default()
                .with_width(line_width)
                .with_color(IcedColor::from_rgb(0.5, 0.3, 0.1)),
        );

        for offset in 1..total_players {
            let opponent_idx = (self.my_idx + offset) % total_players;
            let card_count = self.game.players[opponent_idx].cards.len();
            let cw = 30.0 * scale;
            let ch = 45.0 * scale;
            let card_orbit_radius = board_radius + (ch / 2.0) + (15.0 * scale);
            let angle_step = std::f32::consts::TAU / (total_players as f32);
            let card_angle = std::f32::consts::FRAC_PI_2 - (offset as f32 * angle_step);
            let center_card_pos = Point::new(
                center.x + card_orbit_radius * card_angle.cos(),
                center.y + card_orbit_radius * card_angle.sin(),
            );
            let is_horizontal = card_angle.cos().abs() < 0.5;
            for c in 0..card_count {
                let spread = 15.0 * scale;
                let total_w = (card_count as f32 - 1.0) * spread;
                let pos = if !is_horizontal {
                    Point::new(
                        center_card_pos.x,
                        center_card_pos.y - total_w / 2.0 + (c as f32 * spread),
                    )
                } else {
                    Point::new(
                        center_card_pos.x - total_w / 2.0 + (c as f32 * spread),
                        center_card_pos.y,
                    )
                };
                let rect = if !is_horizontal {
                    iced::Rectangle::new(
                        Point::new(pos.x - ch / 2.0, pos.y - cw / 2.0),
                        iced::Size::new(ch, cw),
                    )
                } else {
                    iced::Rectangle::new(
                        Point::new(pos.x - cw / 2.0, pos.y - ch / 2.0),
                        iced::Size::new(cw, ch),
                    )
                };
                let back = canvas::Path::rectangle(rect.position(), rect.size());
                frame.fill(&back, IcedColor::from_rgb(0.2, 0.3, 0.7));
                frame.stroke(
                    &back,
                    canvas::Stroke::default()
                        .with_color(IcedColor::WHITE)
                        .with_width(2.0 * scale),
                );
            }
        }

        let track_stroke = canvas::Stroke::default()
            .with_width(line_width)
            .with_color(IcedColor::from_rgba(0.4, 0.2, 0.1, 0.3));
        for i in 0..ring_size {
            let p1 = get_tile_position(i, total_players, center, scale, rotation);
            let p2 = get_tile_position((i + 1) % ring_size, total_players, center, scale, rotation);
            frame.stroke(
                &canvas::Path::new(|p| {
                    p.move_to(p1);
                    p.line_to(p2);
                }),
                track_stroke.clone(),
            );
        }
        for p_idx in 0..total_players {
            let start_idx = self.game.board.start_field(p_idx);
            let house_start_idx = ring_size + p_idx * 4;
            let p_start = get_tile_position(start_idx, total_players, center, scale, rotation);
            let p_house =
                get_tile_position(house_start_idx, total_players, center, scale, rotation);
            frame.stroke(
                &canvas::Path::new(|p| {
                    p.move_to(p_start);
                    p.line_to(p_house);
                }),
                track_stroke.clone(),
            );
            for k in 0..3 {
                let h1 =
                    get_tile_position(house_start_idx + k, total_players, center, scale, rotation);
                let h2 = get_tile_position(
                    house_start_idx + k + 1,
                    total_players,
                    center,
                    scale,
                    rotation,
                );
                frame.stroke(
                    &canvas::Path::new(|p| {
                        p.move_to(h1);
                        p.line_to(h2);
                    }),
                    track_stroke.clone(),
                );
            }
        }

        let board_state = self.game.board_state();
        let my_color_iced = match self.game.players[self.my_idx].color {
            GameColor::Red => IcedColor::from_rgb(0.8, 0.2, 0.2),
            GameColor::Green => IcedColor::from_rgb(0.2, 0.8, 0.2),
            GameColor::Blue => IcedColor::from_rgb(0.2, 0.2, 0.8),
            GameColor::Yellow => IcedColor::from_rgb(0.8, 0.8, 0.2),
            GameColor::Purple => IcedColor::from_rgb(0.5, 0.0, 0.5),
            GameColor::Orange => IcedColor::from_rgb(1.0, 0.65, 0.0),
        };

        for i in 0..total_tiles {
            let pos = get_tile_position(i, total_players, center, scale, rotation);
            for p in 0..total_players {
                if i == self.game.board.start_field(p) {
                    let c = match self.game.players[p].color {
                        GameColor::Red => IcedColor::from_rgb(0.8, 0.2, 0.2),
                        GameColor::Green => IcedColor::from_rgb(0.2, 0.8, 0.2),
                        GameColor::Blue => IcedColor::from_rgb(0.2, 0.2, 0.8),
                        GameColor::Yellow => IcedColor::from_rgb(0.8, 0.8, 0.2),
                        GameColor::Purple => IcedColor::from_rgb(0.5, 0.0, 0.5),
                        GameColor::Orange => IcedColor::from_rgb(1.0, 0.65, 0.0),
                    };
                    frame.fill(
                        &canvas::Path::circle(pos, 13.0 * scale),
                        IcedColor::from_rgba(c.r, c.g, c.b, 0.3),
                    );
                }
            }

            let is_animating_target = self
                .animation
                .as_ref()
                .map_or(false, |a| a.to == i && a.progress < 1.0);
            match board_state.get(i).and_then(|t| t.as_ref()) {
                Some(piece) => {
                    if !is_animating_target {
                        let c = match self.game.players[piece.owner].color {
                            GameColor::Red => IcedColor::from_rgb(0.8, 0.2, 0.2),
                            GameColor::Green => IcedColor::from_rgb(0.2, 0.8, 0.2),
                            GameColor::Blue => IcedColor::from_rgb(0.2, 0.2, 0.8),
                            GameColor::Yellow => IcedColor::from_rgb(0.8, 0.8, 0.2),
                            GameColor::Purple => IcedColor::from_rgb(0.5, 0.0, 0.5),
                            GameColor::Orange => IcedColor::from_rgb(1.0, 0.65, 0.0),
                        };
                        draw_marble(&mut frame, pos, c, scale);
                    } else {
                        frame.fill(
                            &canvas::Path::circle(
                                Point::new(pos.x + 1.0 * scale, pos.y + 1.0 * scale),
                                7.0 * scale,
                            ),
                            IcedColor::from_rgba(0.0, 0.0, 0.0, 0.2),
                        );
                        frame.fill(
                            &canvas::Path::circle(pos, 7.0 * scale),
                            IcedColor::from_rgb(0.4, 0.3, 0.2),
                        );
                    }
                }
                None => {
                    frame.fill(
                        &canvas::Path::circle(
                            Point::new(pos.x + 1.0 * scale, pos.y + 1.0 * scale),
                            7.0 * scale,
                        ),
                        IcedColor::from_rgba(0.0, 0.0, 0.0, 0.2),
                    );
                    frame.fill(
                        &canvas::Path::circle(pos, 7.0 * scale),
                        IcedColor::from_rgb(0.4, 0.3, 0.2),
                    );
                }
            };

            if self.highlights.contains(&i) {
                frame.fill(
                    &canvas::Path::circle(pos, 6.0 * scale),
                    IcedColor::from_rgba(my_color_iced.r, my_color_iced.g, my_color_iced.b, 0.4),
                );
                frame.stroke(
                    &canvas::Path::circle(pos, 11.0 * scale),
                    canvas::Stroke::default()
                        .with_color(my_color_iced)
                        .with_width(3.0 * scale),
                );
            }
        }

        for p_idx in 0..total_players {
            let player = &self.game.players[p_idx];
            let count = player.pieces_to_place;
            if count > 0 {
                let p_color = match player.color {
                    GameColor::Red => IcedColor::from_rgb(0.8, 0.2, 0.2),
                    GameColor::Green => IcedColor::from_rgb(0.2, 0.8, 0.2),
                    GameColor::Blue => IcedColor::from_rgb(0.2, 0.2, 0.8),
                    GameColor::Yellow => IcedColor::from_rgb(0.8, 0.8, 0.2),
                    GameColor::Purple => IcedColor::from_rgb(0.5, 0.0, 0.5),
                    GameColor::Orange => IcedColor::from_rgb(1.0, 0.65, 0.0),
                };
                let start_angle = (self.game.board.start_field(p_idx) as f32)
                    * (std::f32::consts::TAU / (ring_size as f32))
                    + rotation;
                for k in 0..count {
                    let offset_angle = start_angle + 0.12 - (k as f32 * 0.08);
                    let wait_pos = Point::new(
                        center.x + 295.0 * scale * offset_angle.cos(),
                        center.y + 295.0 * scale * offset_angle.sin(),
                    );
                    frame.fill(
                        &canvas::Path::circle(wait_pos, 6.0 * scale),
                        IcedColor::from_rgba(0.0, 0.0, 0.0, 0.1),
                    );
                    draw_marble(&mut frame, wait_pos, p_color, scale);
                }
            }
        }

        if let Some(anim) = &self.animation {
            let p1 = if let Some(p_idx) = anim.from_zwinger_of_player {
                let count = self.game.players[p_idx].pieces_to_place;
                let offset_angle = ((self.game.board.start_field(p_idx) as f32)
                    * (std::f32::consts::TAU / (ring_size as f32))
                    + rotation)
                    + 0.12
                    - (count as f32 * 0.08);
                Point::new(
                    center.x + 295.0 * scale * offset_angle.cos(),
                    center.y + 295.0 * scale * offset_angle.sin(),
                )
            } else {
                get_tile_position(anim.from, total_players, center, scale, rotation)
            };
            let p2 = get_tile_position(anim.to, total_players, center, scale, rotation);
            let mut x = p1.x + (p2.x - p1.x) * anim.progress;
            let mut y = p1.y + (p2.y - p1.y) * anim.progress;
            let hop = (1.0 - (2.0 * anim.progress - 1.0).powi(2)) * 80.0 * scale;
            y -= hop;
            frame.fill(
                &canvas::Path::circle(
                    Point::new(x, p1.y + (p2.y - p1.y) * anim.progress + 5.0 * scale),
                    10.0 * scale,
                ),
                IcedColor::from_rgba(0.0, 0.0, 0.0, 0.3 * (1.0 - (hop / (80.0 * scale)) * 0.7)),
            );
            draw_marble(&mut frame, Point::new(x, y), anim.color, scale);
        }

        vec![frame.into_geometry()]
    }
}

struct HandView<'a> {
    game: &'a Game,
    selected_card: Option<Card>,
    my_idx: usize,
}
impl<'a> HandView<'a> {
    fn get_layout(
        &self,
        bounds: iced::Rectangle,
        cursor_position: Point,
    ) -> Vec<(usize, Card, iced::Rectangle, bool)> {
        // FIX: Wir holen uns jetzt DEINE Handkarten, egal wer am Zug ist!
        let cards = &self.game.players[self.my_idx].cards;

        let count = cards.len();
        if count == 0 {
            return Vec::new();
        }
        let (card_w, card_h, gap, scale) = (60.0, 90.0, 15.0, 1.0);
        let start_x =
            (bounds.width / 2.0) - ((count as f32 * card_w) + ((count as f32 - 1.0) * gap)) / 2.0;
        let base_y = (bounds.height / 2.0) - (card_h / 2.0) + (10.0 * scale);

        cards
            .iter()
            .enumerate()
            .map(|(i, &card)| {
                let x = start_x + (i as f32 * (card_w + gap));
                let mut y = base_y;
                let is_hovered =
                    iced::Rectangle::new(Point::new(x, y), iced::Size::new(card_w, card_h))
                        .contains(cursor_position);
                let is_selected = Some(card) == self.selected_card;
                if is_hovered || is_selected {
                    y -= 15.0 * scale;
                }
                (
                    i,
                    card,
                    iced::Rectangle::new(Point::new(x, y), iced::Size::new(card_w, card_h)),
                    is_hovered,
                )
            })
            .collect()
    }
}

impl<'a> canvas::Program<Message> for HandView<'a> {
    type State = ();
    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        if let canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            if let Some(p) = cursor.position_in(bounds) {
                for (_idx, card, rect, _hovered) in self.get_layout(bounds, p).into_iter().rev() {
                    if rect.contains(p) {
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::CardSelected(card)),
                        );
                    }
                }
            }
        }
        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        for (_i, card, rect, is_hovered) in self.get_layout(
            bounds,
            cursor.position_in(bounds).unwrap_or(Point::new(-1.0, -1.0)),
        ) {
            let is_selected = Some(card) == self.selected_card;
            if is_hovered || is_selected {
                frame.fill(
                    &canvas::Path::rectangle(Point::new(rect.x + 3.0, rect.y + 10.0), rect.size()),
                    IcedColor::from_rgba(0.0, 0.0, 0.0, 0.2),
                );
            } else {
                frame.fill(
                    &canvas::Path::rectangle(Point::new(rect.x + 1.0, rect.y + 1.0), rect.size()),
                    IcedColor::from_rgba(0.0, 0.0, 0.0, 0.1),
                );
            }
            frame.fill(
                &canvas::Path::rectangle(rect.position(), rect.size()),
                if card == Card::Joker {
                    IcedColor::from_rgb(1.0, 0.95, 0.95)
                } else {
                    IcedColor::WHITE
                },
            );
            frame.stroke(
                &canvas::Path::rectangle(rect.position(), rect.size()),
                canvas::Stroke::default()
                    .with_color(if is_selected {
                        IcedColor::from_rgb(0.0, 0.5, 1.0)
                    } else if is_hovered {
                        IcedColor::from_rgb(0.3, 0.3, 0.3)
                    } else {
                        IcedColor::from_rgb(0.7, 0.7, 0.7)
                    })
                    .with_width(if is_selected { 3.0 } else { 1.0 }),
            );

            let label = match card {
                Card::Ace => "A",
                Card::King => "K",
                Card::Queen => "Q",
                Card::Jack => "J",
                Card::Joker => "JOK",
                _ => {
                    if card.value() == 10 {
                        "10"
                    } else {
                        match card {
                            Card::Two => "2",
                            Card::Three => "3",
                            Card::Four => "4",
                            Card::Five => "5",
                            Card::Six => "6",
                            Card::Seven => "7",
                            Card::Eight => "8",
                            Card::Nine => "9",
                            _ => "?",
                        }
                    }
                }
            };
            let text_color = if card == Card::Joker {
                IcedColor::from_rgb(0.8, 0.0, 0.0)
            } else {
                IcedColor::BLACK
            };
            frame.fill_text(canvas::Text {
                content: label.to_string(),
                position: Point::new(rect.x + 5.0, rect.y + 5.0),
                color: text_color,
                size: 12.0.into(),
                ..Default::default()
            });
            draw_card_art(&mut frame, card, rect, text_color);
        }
        vec![frame.into_geometry()]
    }
}

fn draw_card_art(frame: &mut canvas::Frame, card: Card, rect: iced::Rectangle, color: IcedColor) {
    let center = rect.center();
    let w = rect.width;
    
    // Erweiterte Farbpalette für mehr Details
    let haut = IcedColor::from_rgb(0.98, 0.88, 0.75);
    let gold = IcedColor::from_rgb(1.0, 0.8, 0.0);
    let blond = IcedColor::from_rgb(0.95, 0.85, 0.3);
    let rot = IcedColor::from_rgb(0.85, 0.1, 0.1);
    let braun = IcedColor::from_rgb(0.3, 0.15, 0.05); // Für Haare
    let blau = IcedColor::from_rgb(0.2, 0.4, 0.8);   // Für den Buben-Hut
    let schwarz = IcedColor::BLACK;

    match card {
        Card::King => {
            let r = w * 0.26;
            // Haare / Basis
            frame.fill(&canvas::Path::circle(center, r + 2.0), blond);
            
            // Gesicht
            frame.fill(&canvas::Path::circle(center, r), haut);
            frame.stroke(
                &canvas::Path::circle(center, r),
                canvas::Stroke::default().with_width(1.5).with_color(schwarz),
            );
            
            // Krone
            frame.fill(
                &canvas::Path::new(|p| {
                    let wc = r * 2.2;
                    p.move_to(Point::new(center.x - wc / 2.0, center.y - r * 0.6));
                    p.line_to(Point::new(center.x - wc / 2.0, center.y - r * 0.6 - 10.0));
                    p.line_to(Point::new(center.x - wc / 4.0, center.y - r * 0.6 - 5.0));
                    p.line_to(Point::new(center.x, center.y - r * 0.6 - 18.0));
                    p.line_to(Point::new(center.x + wc / 4.0, center.y - r * 0.6 - 5.0));
                    p.line_to(Point::new(center.x + wc / 2.0, center.y - r * 0.6 - 10.0));
                    p.line_to(Point::new(center.x + wc / 2.0, center.y - r * 0.6));
                    p.close();
                }),
                gold,
            );
            
            draw_eyes(frame, center, 2.0);
            
            // Bart und Mund
            frame.fill(&canvas::Path::circle(Point::new(center.x, center.y + 8.0), 5.0), blond);
            frame.stroke(
                &canvas::Path::new(|p| {
                    p.move_to(Point::new(center.x - 3.0, center.y + 6.0));
                    p.line_to(Point::new(center.x + 3.0, center.y + 6.0));
                }),
                canvas::Stroke::default().with_width(1.0).with_color(schwarz),
            );
        }
        Card::Queen => {
            let r = w * 0.24;
            
            // Braune Haare im Hintergrund
            frame.fill(&canvas::Path::circle(center, r + 2.0), braun);
            
            // Gesicht
            frame.fill(&canvas::Path::circle(center, r), haut);
            frame.stroke(
                &canvas::Path::circle(center, r),
                canvas::Stroke::default().with_width(1.5).with_color(schwarz),
            );
            
            // Krone
            frame.fill(
                &canvas::Path::new(|p| {
                    p.move_to(Point::new(center.x - r * 0.8, center.y - r * 0.7));
                    p.line_to(Point::new(center.x, center.y - r * 0.7 - 12.0));
                    p.line_to(Point::new(center.x + r * 0.8, center.y - r * 0.7));
                    p.close();
                }),
                gold,
            );
            
            draw_eyes(frame, center, 1.8);
            
            // Roter Kussmund
            frame.fill(
                &canvas::Path::circle(Point::new(center.x, center.y + 5.0), 2.0),
                rot,
            );
        }
        Card::Jack => {
            let r = w * 0.23;
            
            // Gesicht
            frame.fill(&canvas::Path::circle(center, r), haut);
            frame.stroke(
                &canvas::Path::circle(center, r),
                canvas::Stroke::default().with_width(1.5).with_color(schwarz),
            );
            
            // Blauer Buben-Hut
            frame.fill(
                &canvas::Path::new(|p| {
                    p.move_to(Point::new(center.x - r - 2.0, center.y - r * 0.2));
                    p.line_to(Point::new(center.x + r + 2.0, center.y - r * 0.2));
                    p.line_to(Point::new(center.x, center.y - r * 1.4));
                    p.close();
                }),
                blau,
            );
            
            draw_eyes(frame, center, 2.0);
            
            // Freches Grinsen
            frame.stroke(
                &canvas::Path::new(|p| {
                    p.move_to(Point::new(center.x - 4.0, center.y + 5.0));
                    p.line_to(Point::new(center.x + 3.0, center.y + 4.0));
                }),
                canvas::Stroke::default().with_width(1.0).with_color(schwarz),
            );
        }
        Card::Joker => {
            let r = w * 0.22;
            
            // Gesicht (Weiß)
            frame.fill(&canvas::Path::circle(center, r), IcedColor::WHITE);
            frame.stroke(
                &canvas::Path::circle(center, r),
                canvas::Stroke::default().with_width(1.5).with_color(schwarz),
            );
            
            // Roter Narrenhut
            frame.fill(
                &canvas::Path::new(|p| {
                    p.move_to(Point::new(center.x - r, center.y - r * 0.2));
                    p.line_to(Point::new(center.x - r * 1.5, center.y - r * 1.5));
                    p.line_to(Point::new(center.x, center.y - r * 0.8));
                    p.line_to(Point::new(center.x + r * 1.5, center.y - r * 1.5));
                    p.line_to(Point::new(center.x + r, center.y - r * 0.2));
                    p.close();
                }),
                rot,
            );
            // Goldene Glöckchen am Hut
            frame.fill(&canvas::Path::circle(Point::new(center.x - r * 1.5, center.y - r * 1.5), 3.0), gold);
            frame.fill(&canvas::Path::circle(Point::new(center.x + r * 1.5, center.y - r * 1.5), 3.0), gold);

            draw_eyes(frame, center, 2.5);
            
            // Rote Clown-Nase
            frame.fill(
                &canvas::Path::circle(Point::new(center.x, center.y + 1.0), 3.5),
                rot,
            );
            
            // Breites rotes Joker-Lächeln
            frame.stroke(
                &canvas::Path::new(|p| {
                    p.arc(canvas::path::Arc {
                        center: Point::new(center.x, center.y + 1.0),
                        radius: 8.0,
                        start_angle: iced::Radians(0.5),
                        end_angle: iced::Radians(2.64), // Etwas kleiner als PI
                    });
                }),
                canvas::Stroke::default().with_width(2.0).with_color(rot),
            );
        }
        Card::Four => {
            frame.stroke(
                &canvas::Path::new(|p| {
                    let sz = 15.0;
                    p.move_to(Point::new(center.x + sz, center.y));
                    p.line_to(Point::new(center.x - sz + 5.0, center.y));
                    p.move_to(Point::new(center.x - sz + 10.0, center.y - 8.0));
                    p.line_to(Point::new(center.x - sz, center.y));
                    p.line_to(Point::new(center.x - sz + 10.0, center.y + 8.0));
                }),
                canvas::Stroke::default().with_color(color).with_width(4.0),
            );
        }
        Card::Seven => {
            frame.stroke(
                &canvas::Path::new(|p| {
                    p.move_to(Point::new(center.x - 10.0, center.y + 15.0));
                    p.line_to(Point::new(center.x + 10.0, center.y - 15.0));
                    p.move_to(Point::new(center.x + 10.0, center.y + 15.0));
                    p.line_to(Point::new(center.x - 10.0, center.y - 15.0));
                }),
                canvas::Stroke::default().with_color(color).with_width(3.0),
            );
        }
        Card::Ace => {
            frame.fill_text(canvas::Text {
                content: "A".to_string(),
                position: center,
                color,
                size: 40.0.into(),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Center,
                ..Default::default()
            });
        }
        _ => {
            let label = match card {
                Card::Ten => "10",
                Card::Nine => "9",
                Card::Eight => "8",
                Card::Six => "6",
                Card::Five => "5",
                Card::Three => "3",
                Card::Two => "2",
                _ => "",
            };
            frame.fill_text(canvas::Text {
                content: label.to_string(),
                position: center,
                color,
                size: 32.0.into(),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Center,
                ..Default::default()
            });
        }
    }
}

fn draw_eyes(frame: &mut canvas::Frame, center: Point, sz: f32) {
    frame.fill(
        &canvas::Path::circle(Point::new(center.x - 5.0, center.y - 3.0), sz),
        IcedColor::BLACK,
    );
    frame.fill(
        &canvas::Path::circle(Point::new(center.x + 5.0, center.y - 3.0), sz),
        IcedColor::BLACK,
    );
}
fn draw_marble(frame: &mut canvas::Frame, center: Point, color: IcedColor, scale: f32) {
    let radius = 10.0 * scale;
    frame.fill(
        &canvas::Path::circle(
            Point::new(center.x + 2.0 * scale, center.y + 2.0 * scale),
            radius,
        ),
        IcedColor::from_rgba(0.0, 0.0, 0.0, 0.3),
    );
    frame.fill(&canvas::Path::circle(center, radius), color);
    frame.stroke(
        &canvas::Path::circle(center, radius),
        canvas::Stroke::default()
            .with_color(IcedColor::from_rgba(0.0, 0.0, 0.0, 0.2))
            .with_width(1.0 * scale),
    );
    frame.fill(
        &canvas::Path::circle(
            Point::new(center.x - radius * 0.3, center.y - radius * 0.3),
            radius * 0.4,
        ),
        IcedColor::from_rgba(1.0, 1.0, 1.0, 0.4),
    );
}
