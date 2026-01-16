use iced::widget::{canvas, column, container, row, button, pick_list, text, text_input};
use iced::{executor, mouse, Application, Command, Element, Length, Point, Renderer, Settings, Theme, Color as IcedColor};

// Alles aus der Lib holen
use braendi_dog::{
    Game, DogGame, GameVariant, Card, Action, ActionKind, Color as GameColor
};

pub fn launch() -> iced::Result {
    DogApp::run(Settings::default())
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
        let label = match self {
            GameVariantKind::TwoVsTwo => "2 vs 2",
            GameVariantKind::ThreeVsThree => "3 vs 3",
            GameVariantKind::TwoVsTwoVsTwo => "2 vs 2 vs 2",
            GameVariantKind::FreeForAll => "Free For All",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone)]
enum GameAction {
    Place, Move, Remove, Trade, Grab, Interchange,
}

#[derive(Debug, Clone)]
enum PendingAction {
    Move { from: Option<usize> },
    Interchange { from: Option<usize> },
}

enum Screen { Start, Game }

struct DogApp {
    game: Option<Game>,
    screen: Screen,
    
    selected_variant: Option<GameVariantKind>,
    ffa_players_input: String,
    
    selected_card: Option<Card>,
    pending_action: Option<PendingAction>,
    msg: String, 
}

#[derive(Debug, Clone)]
enum Message {
    VariantSelected(GameVariantKind),
    FreeForAllPlayersChanged(String),
    StartGame,
    
    CardSelected(Card),
    BoardClicked(usize), 
    GameActionBtn(GameAction),
    CancelPendingAction,
}

impl Application for DogApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            DogApp {
                game: None,
                screen: Screen::Start,
                selected_variant: None,
                ffa_players_input: String::new(),
                selected_card: None,
                pending_action: None,
                msg: String::from("Willkommen! Wähle einen Spielmodus."),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Brändi Dog")
    }

    fn update(&mut self, message: Message) -> Command<Message> {

        match message {
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
            Message::StartGame => {
                if let Some(variant) = self.build_game_variant() {
                    let mut game = Game::new(variant);
                    game.new_round(); 
                    
                    self.game = Some(game);
                    self.screen = Screen::Game;
                    self.msg = "Spiel gestartet! Wähle eine Karte.".to_string();
                }
            }

            Message::CardSelected(card) => {
                self.selected_card = Some(card);
                self.msg = format!("Karte {:?} gewählt. Wähle Aktion rechts.", card);
                self.pending_action = None; 
            }

            Message::CancelPendingAction => {
                self.pending_action = None;
                self.msg = "Abgebrochen.".to_string();
            }

            Message::GameActionBtn(action_type) => {
                self.handle_btn_click(action_type);
            }

            Message::BoardClicked(tile_index) => {
                self.handle_board_click(tile_index);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Start => self.render_start(),
            Screen::Game => self.render_game(),
        }
    }
}

impl DogApp {
    fn build_game_variant(&self) -> Option<GameVariant> {
        match self.selected_variant? {
            GameVariantKind::TwoVsTwo => Some(GameVariant::TwoVsTwo),
            GameVariantKind::ThreeVsThree => Some(GameVariant::ThreeVsThree),
            GameVariantKind::TwoVsTwoVsTwo => Some(GameVariant::TwoVsTwoVsTwo),
            GameVariantKind::FreeForAll => {
                let players: usize = self.ffa_players_input.parse().ok()?;
                if players < 2 || players > 6 { return None; }
                Some(GameVariant::FreeForAll(players))
            }
        }
    }

    fn render_start(&self) -> Element<'_, Message> {
        let dropdown = pick_list(
            &GameVariantKind::ALL[..],
            self.selected_variant,
            Message::VariantSelected
        ).placeholder("Spielmodus wählen...");

        let mut content = column![
            text("Brändi Dog - Setup").size(30),
            dropdown
        ].spacing(20).padding(20);

        if self.selected_variant == Some(GameVariantKind::FreeForAll) {
            content = content.push(
                text_input("Anzahl Spieler (2-6)", &self.ffa_players_input)
                    .on_input(Message::FreeForAllPlayersChanged)
                    .padding(10)
            );
        }

        let can_start = self.build_game_variant().is_some();
        
        content.push(
            button("Spiel Starten")
            .padding(15)
            .on_press_maybe(can_start.then_some(Message::StartGame))
        ).into()
    }

    fn render_game(&self) -> Element<'_, Message> {
        let game = self.game.as_ref().unwrap(); 
        
        let board = canvas(BoardView { game })
            .width(Length::Fill)
            .height(Length::Fill);

        let hand = self.make_hand_view(game);
        let sidebar = self.make_sidebar(game);

        let main_area = row![
            container(board).width(Length::Fill).height(Length::Fill).style(iced::theme::Container::Box),
            container(sidebar).width(Length::Fixed(250.0)).padding(10)
        ].spacing(10).height(Length::Fill);

        column![
            main_area,
            container(hand).padding(0).height(Length::Fixed(140.0)) 
        ].into()
    }

    fn make_hand_view<'a>(&self, game: &'a Game) -> Element<'a, Message> {
        let hand_logic = HandView {
            game,
            selected_card: self.selected_card,
        };

        container(
            canvas(hand_logic).width(Length::Fill).height(Length::Fill) 
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &Theme| container::Appearance {
            // Tisch-Farbe (grau)
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.92, 0.92, 0.92))),
            ..Default::default()
        })
        .into()
    }

    fn make_sidebar(&self, game: &Game) -> Element<'_, Message> {
        let info = column![
            text(format!("Runde: {}", game.round)),
            text(format!("Am Zug: {:?} (P{})", game.current_player().color, game.current_player_index)),
            text(&self.msg).size(14),
        ].spacing(5);

        let btns = column![
            text("Aktionen:").size(16),
            button("Legen (Place)").on_press(Message::GameActionBtn(GameAction::Place)).width(Length::Fill),
            button("Ziehen (Move)").on_press(Message::GameActionBtn(GameAction::Move)).width(Length::Fill),
            button("Tauschen (Interchange)").on_press(Message::GameActionBtn(GameAction::Interchange)).width(Length::Fill),
            button("Abwerfen (Remove)").on_press(Message::GameActionBtn(GameAction::Remove)).width(Length::Fill),
            button("Handel (Trade)").on_press(Message::GameActionBtn(GameAction::Trade)).width(Length::Fill),
            button("Klauen (Grab)").on_press(Message::GameActionBtn(GameAction::Grab)).width(Length::Fill),
        ].spacing(10);
        
        let mut col = column![info, btns].spacing(30);

        if self.pending_action.is_some() {
            col = col.push(
                button("Abbrechen").style(iced::theme::Button::Destructive).on_press(Message::CancelPendingAction)
            );
        }

        col.into()
    }


    fn handle_btn_click(&mut self, action_type: GameAction) {
        if self.selected_card.is_none() {
            self.msg = "Erst Karte wählen!".into();
            return;
        }
        
        let card = self.selected_card.unwrap();

        let (current_color, current_idx) = if let Some(g) = &self.game {
            (g.current_player().color, g.current_player_index)
        } else {
            return; 
        };

        match action_type {
            GameAction::Move => {
                self.pending_action = Some(PendingAction::Move { from: None });
                self.msg = "Wähle Figur (Start).".into();
            }
            GameAction::Interchange => {
                self.pending_action = Some(PendingAction::Interchange { from: None });
                self.msg = "Wähle erste Figur zum Tauschen.".into();
            }
            
            // Direct Actions
            GameAction::Place => {
                let act = Action {
                    player: current_color,
                    card,
                    action: ActionKind::Place { target_player: current_idx },
                };
                self.do_action(card, act);
            }
            GameAction::Remove => {
                let act = Action {
                    player: current_color,
                    card,
                    action: ActionKind::Remove,
                };
                self.do_action(card, act);
            }
            GameAction::Trade => {
                let act = Action {
                    player: current_color,
                    card,
                    action: ActionKind::Trade,
                };
                self.do_action(card, act);
            }
             _ => { self.msg = "TODO: Implementieren".into(); }
        }
    }

    fn handle_board_click(&mut self, tile_idx: usize) {
        if self.selected_card.is_none() {
            self.msg = format!("Feld {} geklickt. Wähle erst Karte!", tile_idx);
            return;
        }
        
        let card = self.selected_card.unwrap();
        
        let current_color = if let Some(g) = &self.game {
            g.current_player().color
        } else {
            return;
        };

        if let Some(pending) = self.pending_action.clone() {
             match pending {
                PendingAction::Move { from } => {
                    if let Some(start_idx) = from {
                        // Zweiter Klick -> Ausführen
                        let act = Action {
                            player: current_color,
                            card,
                            action: ActionKind::Move { from: start_idx, to: tile_idx },
                        };
                        self.do_action(card, act);
                    } else {
                        // Erster Klick -> Merken
                        self.pending_action = Some(PendingAction::Move { from: Some(tile_idx) });
                        self.msg = format!("Start: {}. Wähle Ziel!", tile_idx);
                    }
                }
                PendingAction::Interchange { from } => {
                    match from {
                         None => {
                            self.pending_action = Some(PendingAction::Interchange { from: Some(tile_idx) });
                            self.msg = format!("Figur 1: {}. Wähle Figur 2.", tile_idx);
                        }
                        Some(first_idx) => {
                            let act = Action {
                                player: current_color,
                                card,
                                action: ActionKind::Interchange { a: first_idx, b: tile_idx },
                            };
                            self.do_action(card, act);
                        }
                    }
                }
             }
        } else {
            self.msg = "Wähle rechts erst eine Aktion (z.B. Move).".to_string();
        }
    }

    fn do_action(&mut self, card: Card, action: Action) {
        if let Some(game) = self.game.as_mut() {
            match game.action(card, action) {
                Ok(_) => {
                    self.msg = "Zug erfolgreich!".into();
                    // Aufräumen
                    self.selected_card = None;
                    self.pending_action = None;
                    // self.clicked_tile = None; 
                }
                Err(e) => {
                    self.msg = format!("Fehler: {}", e);
                    // Bei Move Fehler -> Reset state
                    if let Some(PendingAction::Move { from: Some(_) }) = self.pending_action {
                        self.pending_action = None; 
                    }
                }
            }
        }
    }
}


fn draw_card_art(frame: &mut canvas::Frame, card: Card, rect: iced::Rectangle, color: IcedColor) {
    let center = rect.center();
    let w = rect.width;
    
    // Farben lokal, ist einfacher
    let haut = IcedColor::from_rgb(0.98, 0.88, 0.75); 
    let gold = IcedColor::from_rgb(1.0, 0.8, 0.0);    
    let blond = IcedColor::from_rgb(0.95, 0.85, 0.3); 
    let bart_blond = IcedColor::from_rgb(0.85, 0.75, 0.2); 
    
    let suit_color = color;
    let rot = IcedColor::from_rgb(0.85, 0.1, 0.1); 

    match card {
        
        Card::King => {
            let r = w * 0.26; // radius
            // Haare
            let hair = canvas::Path::new(|p| p.arc(canvas::path::Arc { center, radius: r + 2.0, start_angle: iced::Radians(0.0), end_angle: iced::Radians(6.28) }));
            frame.fill(&hair, blond);
            // Kopf
            let head = canvas::Path::circle(center, r);
            frame.fill(&head, haut);
            frame.stroke(&head, canvas::Stroke::default().with_width(1.5));
            // Bart
            let beard = canvas::Path::new(|p| {
                p.move_to(Point::new(center.x - r + 2.0, center.y));
                p.quadratic_curve_to(Point::new(center.x, center.y + r + 15.0), Point::new(center.x + r - 2.0, center.y));
                p.close();
            });
            frame.fill(&beard, bart_blond);
            // Schnurrbart
            let mustache = canvas::Path::new(|p| {
                p.move_to(Point::new(center.x - 10.0, center.y + 10.0));
                p.quadratic_curve_to(Point::new(center.x, center.y + 5.0), Point::new(center.x + 10.0, center.y + 10.0));
                p.line_to(Point::new(center.x, center.y + 8.0));
                p.close();
            });
            frame.fill(&mustache, IcedColor::BLACK);
            // Krone
            let crown = canvas::Path::new(|p| {
                let y = center.y - r * 0.6;
                let wc = r * 2.2;
                p.move_to(Point::new(center.x - wc / 2.0, y));
                p.line_to(Point::new(center.x - wc / 2.0, y - 10.0));
                p.line_to(Point::new(center.x - wc / 4.0, y - 5.0));
                p.line_to(Point::new(center.x, y - 18.0));
                p.line_to(Point::new(center.x + wc / 4.0, y - 5.0));
                p.line_to(Point::new(center.x + wc / 2.0, y - 10.0));
                p.line_to(Point::new(center.x + wc / 2.0, y));
                p.close();
            });
            frame.fill(&crown, gold);
            frame.stroke(&crown, canvas::Stroke::default().with_color(IcedColor::BLACK).with_width(1.5));
            
            draw_eyes(frame, center, 2.0);
        }

        Card::Queen => {
            let r = w * 0.24;
            // Haare 
            let hair_bg = canvas::Path::new(|p| {
                let top = center.y - r - 5.0;
                let bot = center.y + r + 15.0;
                let wd = r * 2.8;
                p.move_to(Point::new(center.x, top));
                p.quadratic_curve_to(Point::new(center.x - wd/1.5, center.y), Point::new(center.x - wd/2.0, bot));
                p.line_to(Point::new(center.x + wd/2.0, bot));
                p.quadratic_curve_to(Point::new(center.x + wd/1.5, center.y), Point::new(center.x, top));
            });
            frame.fill(&hair_bg, blond);
            frame.stroke(&hair_bg, canvas::Stroke::default().with_color(IcedColor::from_rgba(0.0,0.0,0.0,0.2)).with_width(1.0));
            // Kopf
            let head = canvas::Path::circle(center, r);
            frame.fill(&head, haut);
            frame.stroke(&head, canvas::Stroke::default().with_width(1.5));
            // Diadem
            let tiara = canvas::Path::new(|p| {
                let y = center.y - r * 0.7;
                p.move_to(Point::new(center.x - r * 0.8, y));
                p.line_to(Point::new(center.x, y - 12.0));
                p.line_to(Point::new(center.x + r * 0.8, y));
                p.close();
            });
            frame.fill(&tiara, gold);
            frame.stroke(&tiara, canvas::Stroke::default().with_color(IcedColor::BLACK).with_width(1.0));
            draw_eyes(frame, center, 1.8);
            // Lippen
            let lips = canvas::Path::new(|p| {
                 p.move_to(Point::new(center.x - 4.0, center.y + 10.0));
                 p.quadratic_curve_to(Point::new(center.x, center.y + 13.0), Point::new(center.x + 4.0, center.y + 10.0));
            });
            frame.stroke(&lips, canvas::Stroke::default().with_color(IcedColor::from_rgb(0.8, 0.2, 0.2)).with_width(2.0));
        }

        Card::Jack => {
            let r = w * 0.23;
            // Haare
            let hair = canvas::Path::new(|p| {
                p.move_to(Point::new(center.x, center.y - r));
                p.quadratic_curve_to(Point::new(center.x - r - 8.0, center.y), Point::new(center.x - r, center.y + r));
                p.line_to(Point::new(center.x + r, center.y + r));
                p.quadratic_curve_to(Point::new(center.x + r + 8.0, center.y), Point::new(center.x, center.y - r));
            });
            frame.fill(&hair, blond);
            frame.stroke(&hair, canvas::Stroke::default().with_color(IcedColor::from_rgba(0.0,0.0,0.0,0.3)).with_width(1.0));
            // Kopf
            let head = canvas::Path::circle(center, r);
            frame.fill(&head, haut);
            frame.stroke(&head, canvas::Stroke::default().with_width(1.5));
            // Hut (Barett)
            let hat = canvas::Path::new(|p| {
                let y = center.y - r * 0.6;
                let hw = r * 2.6;
                p.move_to(Point::new(center.x - hw / 2.0, y));
                p.quadratic_curve_to(Point::new(center.x, y - 5.0), Point::new(center.x + hw / 2.0, y));
                p.quadratic_curve_to(Point::new(center.x + hw / 2.0 + 5.0, y - 10.0), Point::new(center.x, y - 15.0));
                p.quadratic_curve_to(Point::new(center.x - hw / 2.0 - 5.0, y - 10.0), Point::new(center.x - hw / 2.0, y));
                p.close();
            });
            frame.fill(&hat, suit_color);
            // Feder
            let feather = canvas::Path::new(|p| {
                let y = center.y - r * 0.6 - 10.0;
                p.move_to(Point::new(center.x + 10.0, y));
                p.quadratic_curve_to(Point::new(center.x + 25.0, y - 20.0), Point::new(center.x + 15.0, y - 25.0));
                p.quadratic_curve_to(Point::new(center.x + 15.0, y - 10.0), Point::new(center.x + 10.0, y));
            });
            frame.fill(&feather, IcedColor::WHITE);
            frame.stroke(&feather, canvas::Stroke::default().with_color(IcedColor::BLACK).with_width(1.0));
            
            draw_eyes(frame, center, 2.0);
            
            // Mund
            let smile = canvas::Path::new(|p| {
                p.move_to(Point::new(center.x - 5.0, center.y + 8.0));
                p.quadratic_curve_to(Point::new(center.x, center.y + 10.0), Point::new(center.x + 5.0, center.y + 8.0));
            });
            frame.stroke(&smile, canvas::Stroke::default().with_color(IcedColor::BLACK).with_width(1.5));
        }

        Card::Joker => {
            let r = w * 0.22;
            let head = canvas::Path::circle(center, r);
            frame.fill(&head, IcedColor::WHITE); 
            frame.stroke(&head, canvas::Stroke::default().with_width(1.5));
            // Kappe
            let cap = canvas::Path::new(|p| {
                let y = center.y - r * 0.6;
                p.move_to(Point::new(center.x - r, y));
                p.line_to(Point::new(center.x + r, y));
                // rechts
                p.quadratic_curve_to(Point::new(center.x + r + 15.0, y - 10.0), Point::new(center.x + r + 5.0, y + 10.0));
                p.quadratic_curve_to(Point::new(center.x + 10.0, y - 25.0), Point::new(center.x, y));
                // links
                p.quadratic_curve_to(Point::new(center.x - 10.0, y - 25.0), Point::new(center.x - r - 5.0, y + 10.0));
                p.quadratic_curve_to(Point::new(center.x - r - 15.0, y - 10.0), Point::new(center.x - r, y));
            });
            frame.fill(&cap, rot);
            frame.stroke(&cap, canvas::Stroke::default().with_color(IcedColor::BLACK).with_width(1.0));
            // Schellen
            let y = center.y - r * 0.6;
            let bell_l = canvas::Path::circle(Point::new(center.x - r - 5.0, y + 10.0), 3.0);
            let bell_r = canvas::Path::circle(Point::new(center.x + r + 5.0, y + 10.0), 3.0);
            frame.fill(&bell_l, gold);
            frame.fill(&bell_r, gold);
            
            draw_eyes(frame, center, 2.5);
            // Nase
            let nose = canvas::Path::circle(Point::new(center.x, center.y + 2.0), 4.5);
            frame.fill(&nose, rot);
            
            // Mund 
            let smile = canvas::Path::new(|p| {
                p.move_to(Point::new(center.x - 8.0, center.y + 7.0));
                p.quadratic_curve_to(
                    Point::new(center.x, center.y + 11.0), 
                    Point::new(center.x + 8.0, center.y + 7.0)
                );
            });
            frame.stroke(&smile, canvas::Stroke::default().with_color(IcedColor::BLACK).with_width(2.0));
        }

        // Specials
        Card::Four => {
            let arrow = canvas::Path::new(|p| {
                let sz = 15.0;
                p.move_to(Point::new(center.x + sz, center.y)); 
                p.line_to(Point::new(center.x - sz + 5.0, center.y));
                p.move_to(Point::new(center.x - sz + 10.0, center.y - 8.0));
                p.line_to(Point::new(center.x - sz, center.y));
                p.line_to(Point::new(center.x - sz + 10.0, center.y + 8.0));
            });
            frame.stroke(&arrow, canvas::Stroke::default().with_color(color).with_width(4.0));
        }

        Card::Seven => {
            let scissors = canvas::Path::new(|p| {
                p.move_to(Point::new(center.x - 10.0, center.y + 15.0));
                p.line_to(Point::new(center.x + 10.0, center.y - 15.0));
                p.move_to(Point::new(center.x + 10.0, center.y + 15.0));
                p.line_to(Point::new(center.x - 10.0, center.y - 15.0));
            });
            frame.stroke(&scissors, canvas::Stroke::default().with_color(color).with_width(3.0));
            
            let h_l = canvas::Path::circle(Point::new(center.x - 10.0, center.y + 18.0), 4.0);
            let h_r = canvas::Path::circle(Point::new(center.x + 10.0, center.y + 18.0), 4.0);
            frame.stroke(&h_l, canvas::Stroke::default().with_color(color).with_width(2.0));
            frame.stroke(&h_r, canvas::Stroke::default().with_color(color).with_width(2.0));
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

        // Rest = Zahlen
        _ => {
            // Quick hack für labels
            let label = match card {
                Card::Ten => "10",
                Card::Nine => "9",
                Card::Eight => "8",
                Card::Six => "6",
                Card::Five => "5",
                Card::Three => "3",
                Card::Two => "2",
                _ => ""
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

// Augen helper
fn draw_eyes(frame: &mut canvas::Frame, center: Point, sz: f32) {
    let l = canvas::Path::circle(Point::new(center.x - 5.0, center.y - 3.0), sz);
    let r = canvas::Path::circle(Point::new(center.x + 5.0, center.y - 3.0), sz);
    frame.fill(&l, IcedColor::BLACK);
    frame.fill(&r, IcedColor::BLACK);
}


struct BoardView<'a> {
    game: &'a Game,
}

fn get_tile_position(index: usize, total_players: usize, center: Point) -> Point {
    // 250 radius passt gut auf Screen
    let r_ring = 250.0;
    let ring_size = total_players * 16;
    
    if index < ring_size {
        // Außen
        let angle = (index as f32 / ring_size as f32) * std::f32::consts::TAU;
        Point::new(
            center.x + r_ring * angle.cos(),
            center.y + r_ring * angle.sin(),
        )
    } else {
        // Häuser (Spirale nach innen)
        let house_global_index = index - ring_size;
        let player_idx = house_global_index / 4;
        let step = house_global_index % 4;

        let start_idx = player_idx * 16;
        let angle = (start_idx as f32 / ring_size as f32) * std::f32::consts::TAU;

        let r_current = r_ring - 30.0 - (step as f32 * 35.0);
        
        Point::new(
            center.x + r_current * angle.cos(),
            center.y + r_current * angle.sin(),
        )
    }
}

// Hitbox check
fn is_hit(cursor: Point, center: Point, radius: f32) -> bool {
    let dx = cursor.x - center.x;
    let dy = cursor.y - center.y;
    (dx*dx + dy*dy).sqrt() < radius
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
        
        let cursor_position = if let Some(p) = cursor.position_in(bounds) { p } else { return (canvas::event::Status::Ignored, None); };

        if let canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            // Mitte relativ
            let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
            let total_players = self.game.players.len();
            let total_tiles = total_players * 16 + total_players * 4;

            for i in 0..total_tiles {
                let pos = get_tile_position(i, total_players, center);
                if is_hit(cursor_position, pos, 12.0) { // 12px toleranz
                    return (canvas::event::Status::Captured, Some(Message::BoardClicked(i)));
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
        let total_tiles = total_players * 16 + total_players * 4;

        // Background circle
        let bg = canvas::Path::circle(center, 300.0);
        frame.fill(&bg, IcedColor::from_rgb(0.9, 0.85, 0.7)); 

        let board_state = self.game.board_state();

        for i in 0..total_tiles {
            let pos = get_tile_position(i, total_players, center);
            
            // Player Farben
            let color = match board_state.get(i).and_then(|t| t.as_ref()) {
                Some(piece) => match self.game.players[piece.owner].color {
                    GameColor::Red => IcedColor::from_rgb(0.8, 0.2, 0.2),
                    GameColor::Green => IcedColor::from_rgb(0.2, 0.8, 0.2),
                    GameColor::Blue => IcedColor::from_rgb(0.2, 0.2, 0.8),
                    GameColor::Yellow => IcedColor::from_rgb(0.8, 0.8, 0.2),
                    GameColor::Purple => IcedColor::from_rgb(0.5, 0.0, 0.5),
                    GameColor::Orange => IcedColor::from_rgb(1.0, 0.65, 0.0),
                },
                None => {
                    // Empty tile colors
                    if i < total_players*16 && i % 16 == 0 {
                         IcedColor::from_rgb(0.5, 0.5, 0.5) 
                    } else if i >= total_players*16 {
                         IcedColor::WHITE 
                    } else {
                         IcedColor::from_rgb(0.8, 0.8, 0.8) 
                    }
                }
            };

            let circle = canvas::Path::circle(pos, 10.0);
            frame.fill(&circle, color);
            frame.stroke(&circle, canvas::Stroke::default().with_width(1.0).with_color(IcedColor::BLACK));
        }

        vec![frame.into_geometry()]
    }
}


struct HandView<'a> {
    game: &'a Game,
    selected_card: Option<Card>,
}

impl<'a> HandView<'a> {
    // Copy paste von get_card_layout logik für update und draw
    fn get_layout(&self, bounds: iced::Rectangle, cursor_position: Point) -> Vec<(usize, Card, iced::Rectangle, bool)> {
        let cards = &self.game.current_player().cards;
        let count = cards.len();
        if count == 0 { return Vec::new(); }

        let card_w = 60.0;
        let card_h = 90.0;
        let gap = 15.0;
        
        let total_w = (count as f32 * card_w) + ((count as f32 - 1.0) * gap);
        let start_x = (bounds.width / 2.0) - (total_w / 2.0);
        let base_y = (bounds.height / 2.0) - (card_h / 2.0) + 10.0;

        cards.iter().enumerate().map(|(i, &card)| {
            let x = start_x + (i as f32 * (card_w + gap));
            let mut y = base_y;

            // Hit check
            let base_rect = iced::Rectangle::new(Point::new(x, y), iced::Size::new(card_w, card_h));
            let is_hovered = base_rect.contains(cursor_position);
            let is_selected = Some(card) == self.selected_card;
            
            // Pop up effekt
            if is_hovered || is_selected {
                y -= 15.0; 
            }

            let rect = iced::Rectangle::new(Point::new(x, y), iced::Size::new(card_w, card_h));
            (i, card, rect, is_hovered)
        }).collect()
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
        let cursor_position = if let Some(p) = cursor.position_in(bounds) { p } else { return (canvas::event::Status::Ignored, None); };

        if let canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            let layout = self.get_layout(bounds, cursor_position);
            // Reverse loop damit wir die oberste karte treffen
            for (_idx, card, rect, _hovered) in layout.into_iter().rev() {
                if rect.contains(cursor_position) {
                    return (canvas::event::Status::Captured, Some(Message::CardSelected(card)));
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
        
        let cursor_pos = cursor.position_in(bounds).unwrap_or(Point::new(-1.0, -1.0));
        let layout = self.get_layout(bounds, cursor_pos);

        for (_i, card, rect, is_hovered) in layout {
            let is_selected = Some(card) == self.selected_card;

            let bg = canvas::Path::rectangle(rect.position(), rect.size());
            
            // Schatten
            if is_hovered || is_selected {
                let shadow = canvas::Path::rectangle(Point::new(rect.x + 3.0, rect.y + 10.0), rect.size());
                frame.fill(&shadow, IcedColor::from_rgba(0.0, 0.0, 0.0, 0.2));
            } else {
                let shadow = canvas::Path::rectangle(Point::new(rect.x + 1.0, rect.y + 1.0), rect.size());
                frame.fill(&shadow, IcedColor::from_rgba(0.0, 0.0, 0.0, 0.1));
            }

            let color = if card == Card::Joker { IcedColor::from_rgb(1.0, 0.95, 0.95) } else { IcedColor::WHITE };
            frame.fill(&bg, color);

            // Rand
            let border_c = if is_selected { 
                IcedColor::from_rgb(0.0, 0.5, 1.0) 
            } else if is_hovered {
                IcedColor::from_rgb(0.3, 0.3, 0.3) 
            } else {
                IcedColor::from_rgb(0.7, 0.7, 0.7) 
            };
            
            let width = if is_selected { 3.0 } else { 1.0 };
            
            frame.stroke(&bg, canvas::Stroke::default().with_color(border_c).with_width(width));

            // Eck Text
            let label = match card {
                Card::Ace => "A",
                Card::King => "K",
                Card::Queen => "Q",
                Card::Jack => "J",
                Card::Joker => "JOK",
                _ => { 
                    if card.value() == 10 { "10" } else { 
                        match card {
                            Card::Two => "2", Card::Three => "3", Card::Four => "4", 
                            Card::Five => "5", Card::Six => "6", Card::Seven => "7", 
                            Card::Eight => "8", Card::Nine => "9", _ => "?" 
                        }
                    }
                }
            };
            
            let txt_col = if card == Card::Joker { IcedColor::from_rgb(0.8, 0.0, 0.0) } else { IcedColor::BLACK };

            frame.fill_text(canvas::Text {
                content: label.to_string(),
                position: Point::new(rect.x + 5.0, rect.y + 5.0),
                color: txt_col,
                size: 12.0.into(),
                ..Default::default()
            });

            // Grafik malen
            draw_card_art(&mut frame, card, rect, txt_col);
        }

        vec![frame.into_geometry()]
    }
}
