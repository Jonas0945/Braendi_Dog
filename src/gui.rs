
use iced::widget::{canvas, column, container, text, row, button};
use iced::{executor, Application, Command, Element, Length, Point, Renderer, Settings, Theme, Color as IcedColor};
use iced::mouse;

use braendi_dog::game::{Game, DogGame, Color}; 
use braendi_dog::GameVariant;
use braendi_dog::Card;
pub fn launch() -> iced::Result {
    DogApp::run(Settings::default())
}

struct DogApp {
    game: Game,
    selected_card: Option<Card>,
    status_message: String,
}

#[derive(Debug, Clone)]
enum Message {
    CardSelected(Card),
    BoardClicked(u8),
    ActionFinished(Result<(), &'static str>),
}

fn iced_color_from_player(color: Color) -> IcedColor {
    match color {
        Color::Red => IcedColor::from_rgb(0.8, 0.0, 0.0),
        Color::Green => IcedColor::from_rgb(0.0, 0.6, 0.0),
        Color::Blue => IcedColor::from_rgb(0.0, 0.0, 0.8),
        Color::Yellow => IcedColor::from_rgb(0.9, 0.8, 0.0),
        _ => IcedColor::BLACK,
    }
}

impl Application for DogApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut game = Game::new(GameVariant::TwoVsTwo);
        game.new_round();

        (
            DogApp {
                game,
                selected_card: None,
                status_message: String::from("Willkommen bei Brändi Dog!"),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Brändi Dog - Rust GUI")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CardSelected(card) => {
                self.selected_card = Some(card);
                self.status_message = format!("Karte gewählt: {:?}", card);
            }
            Message::BoardClicked(tile_index) => {
                println!("Board was clicked");
                if let Some(card) = self.selected_card {
                    self.status_message = format!("Feld {} geklickt. Implementiere Logik...", tile_index);
                    // TODO: Hier später self.game.action(...) aufrufen für Move/Place
                } else {
                    self.status_message = String::from("Bitte erst eine Karte wählen!");
                }
            }
            Message::ActionFinished(res) => {
                if let Err(e) = res {
                    self.status_message = format!("Fehler: {}", e);
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
    let board = canvas(BoardView { game: &self.game })
        .width(Length::Fill)
        .height(Length::Fill);

    let hand = self.view_hand();
    let status = self.view_game_status();

    row![
        column![
            container(board)
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
            hand
        ]
        .width(Length::FillPortion(3)),

        status
    ]
    .spacing(20)
    .padding(20)
    .height(Length::Fill)
    .into()
}

}

impl DogApp {
    fn view_game_status(&self) -> Element<Message> {
    use iced::widget::{column, container, text};

    let game = &self.game;

    let header = column![
        text("Spielstatus").size(22),
        text(format!("Runde: {}", game.round - 1)),
        text(format!(
            "Tauschphase: {}",
            if game.trading_phase { "JA" } else { "NEIN" }
        )),
        text(format!("Aktiver Spieler: {}", game.current_player_index + 1)),
    ]
    .spacing(6);

    let players = game.players.iter().enumerate().map(|(i, p)| {
        let is_active = i == game.current_player_index;

        let title = text(format!(
            "{} Spieler {}",
            if is_active { "Aktiv" } else { "" },
            i + 1
        ))
        .size(18)
        .style(iced_color_from_player(p.color));

        column![
            title,
            text(format!("Farbe: {:?}", p.color)),
            text(format!("Steine zu setzen: {}", p.pieces_to_place)),
            text(format!("Steine im Haus: {}", p.pieces_in_house)),
            text(format!("Karten: {}", p.cards.len())),
        ]
        .spacing(4)
        .padding(6)
        .into()
    });

    container(
        column![
            header,
            text("Spieler").size(20),
            column(players).spacing(12),
        ]
        .spacing(14)
    )
    .width(Length::Fixed(280.0))
    .padding(12)
    .into()
}

    fn view_hand(&self) -> Element<Message> {

    let current_player = self.game.current_player();

    let cards: Vec<Element<Message>> = current_player.cards.iter().map(|card| {

        button(text(format!("{:?}", card)))
            .on_press(Message::CardSelected(*card))
            .padding(10)
            .into()
    }).collect();

    container(row(cards).spacing(10))
        .center_x()
        .into()
}

}


struct BoardView<'a> {
    game: &'a Game,
}

// Hilfsfunktion: Rotiert einen Punkt (x, y) um 90 Grad * n um den Nullpunkt (0,0)
fn rotate_point(x: f32, y: f32, quarters: usize) -> (f32, f32) {
    match quarters % 4 {
        0 => (x, y),             // 0 Grad
        1 => (-y, x),            // 90 Grad
        2 => (-x, -y),           // 180 Grad
        3 => (y, -x),            // 270 Grad
        _ => (x, y),
    }
}

// Hilfsfunktion: Berechnet die Position für einen Index auf dem Brändi-Brett
fn get_board_coordinates(index: usize) -> Point {
    let scale = 35.0; // Abstand zwischen den Punkten (Skalierung)
    let offset_x = 220.0; // Wie weit das Brett vom Zentrum entfernt beginnt
    
    // --- 1. DER RING (0 bis 63) ---
    if index < 64 {
        // Wir berechnen nur die Positionen für den ERSTEN Spieler (Index 0-15)
        // und rotieren das Ergebnis dann für die anderen.
        let local_index = index % 16;
        let player_sector = index / 16; // 0, 1, 2 oder 3

        // Definiere die Form eines Viertels (Rechte Seite des Bretts)
        // Das ist eine Annäherung an die geschwungene Form auf dem Foto.
        let (local_x, local_y) = match local_index {
            // Startfeld (Index 0 beim Spieler) ist meist "unten rechts" im Segment
            0 => (offset_x, 0.0),
            // Die Gerade nach oben
            1..=7 => (offset_x, -(local_index as f32) * scale),
            // Die Kurve um die Ecke (einfache Annäherung)
            8 => (offset_x - 10.0, -8.0 * scale - 10.0),
            9 => (offset_x - 40.0, -8.0 * scale - 30.0),
            10 => (offset_x - 80.0, -8.0 * scale - 40.0),
            // Die Gerade nach links (oben)
            11..=15 => (offset_x - 80.0 - ((local_index - 10) as f32 * scale), -8.0 * scale - 40.0),
            _ => (0.0, 0.0),
        };

        // Jetzt rotieren wir den Punkt passend zum Spieler
        let (rot_x, rot_y) = rotate_point(local_x, local_y, player_sector);
        
        // Verschieben in die Mitte des Fensters (wird im draw Aufruf addiert)
        return Point::new(rot_x, rot_y);
    } 
    
    // --- 2. DIE HÄUSER (64 bis 79) ---
    else {
        // Index 64-67 (Spieler 0), 68-71 (Spieler 1), etc.
        let house_idx = index - 64;
        let player_sector = house_idx / 4;
        let step_in_house = house_idx % 4; // 0 bis 3 (Schritte ins Haus)

        // Das Haus beginnt beim Startfeld (offset_x, 0) und geht nach INNEN (links)
        // Schritt 1 ist bei offset_x - scale, etc.
        let start_x = offset_x - scale; // Erster Schritt im Haus
        let local_x = start_x - (step_in_house as f32 * scale);
        let local_y = 0.0; // Auf der Mittellinie bleiben

        let (rot_x, rot_y) = rotate_point(local_x, local_y, player_sector);
        return Point::new(rot_x, rot_y);
    }
}

impl<'a> canvas::Program<Message> for BoardView<'a> {
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
        let center = frame.center();

        // 1. Hintergrund "Holz" zeichnen (optional, hellbraun)
        let board_bg = canvas::Path::rectangle(
            Point::new(center.x - 350.0, center.y - 350.0),
            iced::Size::new(700.0, 700.0)
        );
        frame.fill(&board_bg, IcedColor::from_rgb8(222, 184, 135)); // "Burlywood" Farbe

        // 2. Alle Felder zeichnen
        for i in 0..80 { 
            let pos_rel = get_board_coordinates(i);
            let pos_abs = Point::new(center.x + pos_rel.x, center.y + pos_rel.y);

            // Farbe des Feldes bestimmen (Ist ein Stück drauf?)
            let board_state = self.game.board_state();
            
            let color = match board_state.get(i).and_then(|t| t.as_ref()) {
                Some(piece) => match piece.owner {
                    0 => IcedColor::from_rgb(0.8, 0.0, 0.0), // Rot (Kräftiger)
                    1 => IcedColor::from_rgb(0.0, 0.6, 0.0), // Grün
                    2 => IcedColor::from_rgb(0.0, 0.0, 0.8), // Blau
                    3 => IcedColor::from_rgb(0.9, 0.8, 0.0), // Gelb
                    _ => IcedColor::BLACK,
                },
                None => {
                    // Startfelder markieren (Indizes 0, 16, 32, 48)
                    if i % 16 == 0 && i < 64 {
                         IcedColor::from_rgb(0.5, 0.5, 0.5) // Dunkleres Grau für Start
                    } else if i >= 64 {
                        IcedColor::from_rgb(0.9, 0.9, 0.9) // Helles Grau für Haus
                    } else {
                        IcedColor::from_rgb(0.7, 0.7, 0.7) // Standard Grau
                    }
                }
            };

            // Zeichnen:
            // Kleiner schwarzer Rand um jedes Loch, damit es wie ein Brett aussieht
            let hole_outline = canvas::Path::circle(pos_abs, 10.0);
            frame.fill(&hole_outline, IcedColor::from_rgba(0.0, 0.0, 0.0, 0.2));

            // Die Murmel / Das Loch selbst
            let circle = canvas::Path::circle(pos_abs, 8.0);
            frame.fill(&circle, color);
            
            // Text-Debug (optional): Index Nummer auf dem Feld anzeigen
            frame.fill_text(canvas::Text {
                 content: i.to_string(),
                 position: pos_abs,
                 size: 10.0.into(),
                 color: IcedColor::BLACK,
                 horizontal_alignment: iced::alignment::Horizontal::Center,
                 vertical_alignment: iced::alignment::Vertical::Center,
                 ..Default::default()
             });
        }
        vec![frame.into_geometry()]
    }
}