use iced::widget::{canvas, column, container, text, row, button};
use iced::{executor, Application, Command, Element, Length, Point, Renderer, Settings, Theme, Color as IcedColor};
use iced::mouse;

use braendi_dog::game::{Game, DogGame, Color, Card, GameVariant}; 

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

impl Application for DogApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let game = Game::new(GameVariant::TwoVsTwo);
        
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

        let status = text(&self.status_message).size(20);

        column![
            status,
            container(board).width(Length::Fill).height(Length::Fill).center_x().center_y(),
            hand
        ]
        .padding(20)
        .into()
    }
}

impl DogApp {
    fn view_hand(&self) -> Element<Message> {
        let current_player = self.game.current_player();
        
        let cards_iter = current_player.cards.iter().map(|card| {
            button(text(format!("{:?}", card)))
                .on_press(Message::CardSelected(*card))
                .padding(10)
                .into()
        });

        let cards_vec: Vec<Element<Message>> = cards_iter.collect();

        let cards_row = row(cards_vec).spacing(10);

        container(cards_row).center_x().into()
    }
}


struct BoardView<'a> {
    game: &'a Game,
}

fn rotate_point(x: f32, y: f32, quarters: usize) -> (f32, f32) {
    match quarters % 4 {
        0 => (x, y),             // 0 Grad
        1 => (-y, x),            // 90 Grad
        2 => (-x, -y),           // 180 Grad
        3 => (y, -x),            // 270 Grad
        _ => (x, y),
    }
}

fn get_board_coordinates(index: usize) -> Point {
    let scale = 35.0; // Abstand zwischen den Punkten (Skalierung)
    let offset_x = 220.0; // Wie weit das Brett vom Zentrum entfernt beginnt
    
    if index < 64 {
        let local_index = index % 16;
        let player_sector = index / 16; // 0, 1, 2 oder 3

        let (local_x, local_y) = match local_index {
            0 => (offset_x, 0.0), 
            1..=7 => (offset_x, -(local_index as f32) * scale),
            8 => (offset_x - 10.0, -8.0 * scale - 10.0),
            9 => (offset_x - 40.0, -8.0 * scale - 30.0),
            10 => (offset_x - 80.0, -8.0 * scale - 40.0),
            11..=15 => (offset_x - 80.0 - ((local_index - 10) as f32 * scale), -8.0 * scale - 40.0),
            _ => (0.0, 0.0),
        };

        let (rot_x, rot_y) = rotate_point(local_x, local_y, player_sector);
        
        return Point::new(rot_x, rot_y);
    } 
    
    else {
        let house_idx = index - 64;
        let player_sector = house_idx / 4;
        let step_in_house = house_idx % 4;

        let start_x = offset_x - scale; 
        let local_x = start_x - (step_in_house as f32 * scale);
        let local_y = 0.0; 

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

        let board_bg = canvas::Path::rectangle(
            Point::new(center.x - 350.0, center.y - 350.0),
            iced::Size::new(700.0, 700.0)
        );
        frame.fill(&board_bg, IcedColor::from_rgb8(222, 184, 135)); 

        for i in 0..80 { 
            let pos_rel = get_board_coordinates(i);
            let pos_abs = Point::new(center.x + pos_rel.x, center.y + pos_rel.y);

            let board_state = self.game.board_state();
            
            let color = match board_state.get(i).and_then(|t| t.as_ref()) {
                Some(piece) => match piece.owner {
                    0 => IcedColor::from_rgb(0.8, 0.0, 0.0), //ROt
                    1 => IcedColor::from_rgb(0.0, 0.6, 0.0), // Grün
                    2 => IcedColor::from_rgb(0.0, 0.0, 0.8), // Blau
                    3 => IcedColor::from_rgb(0.9, 0.8, 0.0), // Gelb
                    _ => IcedColor::BLACK,
                },
                None => {
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
            let hole_outline = canvas::Path::circle(pos_abs, 10.0);
            frame.fill(&hole_outline, IcedColor::from_rgba(0.0, 0.0, 0.0, 0.2));

            let circle = canvas::Path::circle(pos_abs, 8.0);
            frame.fill(&circle, color);
            
           
        }
        vec![frame.into_geometry()]
    }
}