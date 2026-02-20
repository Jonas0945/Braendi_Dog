use iced::widget::{Button, Container, Row, Text};
use iced::widget::{button, canvas, column, container, pick_list, row, text, text_input};
use iced::{
    event, executor, mouse, window, Application, Color as IcedColor, Command, Element, Length,
    Point, Renderer, Settings, Size, Subscription, Theme,
};

use braendi_dog::{Action, ActionKind, Card, Color as GameColor, DogGame, Game, GameVariant};

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
    Place,
    Move,
    Remove,
    Trade,
    Grab,
    Interchange,
    TradeGrab,
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
    Game,
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

    selected_opponent: Option<usize>,
    selected_opponent_card: Option<usize>,
}

#[derive(Debug, Clone)]
enum Message {
    WindowResized(Size),
    VariantSelected(GameVariantKind),
    FreeForAllPlayersChanged(String),
    StartGame,

    CardSelected(Card),
    BoardClicked(usize),
    GameActionBtn(GameAction),
    CancelPendingAction,

    OpponentSelected(usize),
    OpponentCardSelected(usize),
    OpponentCardBack,
    None,
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
                window_size: Size::new(1024.0, 768.0),
                selected_variant: None,
                ffa_players_input: String::new(),
                selected_card: None,
                pending_action: None,
                msg: String::from("Willkommen! Wähle einen Spielmodus."),

                selected_opponent: None,
                selected_opponent_card: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Brändi Dog")
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(|event| {
            if let iced::Event::Window(_, window::Event::Resized { width, height }) = event {
                Message::WindowResized(Size::new(width as f32, height as f32))
            } else {
                Message::None
            }
        })
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::None => {}
            Message::WindowResized(size) => {
                self.window_size = size;
            }
            Message::OpponentSelected(idx) => {
                self.selected_opponent = Some(idx);
                self.selected_opponent_card = None;
                if let Some(game) = &self.game {
                    self.msg = format!(
                        "Gegner {:?} gewählt. Wähle Karte zum Klauen.",
                        game.players[idx].color
                    );
                }
            }

            Message::OpponentCardSelected(card_idx) => {
                self.selected_opponent_card = Some(card_idx);

                match self.pending_action {
                    Some(PendingAction::Grab) => {
                        self.execute_grab(false);
                    }
                    Some(PendingAction::TradeGrab) => {
                        self.execute_grab(true);
                    }
                    _ => {
                        self.msg = format!("Karte {} von Gegner gewählt.", card_idx + 1);
                    }
                }
            }

            Message::OpponentCardBack => {
                self.selected_opponent = None;
                self.selected_opponent_card = None;
                self.msg = "Wähle einen Gegner.".to_string();
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
    fn execute_grab(&mut self, trade: bool) {
        let Some(card) = self.selected_card else {
            self.msg = "Keine Karte gewählt!".into();
            return;
        };

        let (Some(target_player_idx), Some(target_card)) =
            (self.selected_opponent, self.selected_opponent_card)
        else {
            return;
        };

        let game = self.game.as_mut().unwrap();
        let current_color = game.current_player().color;
        let target_color = game.players[target_player_idx].color;

        let action = if trade {
            Action {
                player: current_color,
                card: None,
                action: ActionKind::TradeGrab { target_card },
            }
        } else {
            Action {
                player: current_color,
                card: Some(card),
                action: ActionKind::Grab {
                    target_player: target_color,
                    target_card,
                },
            }
        };

        self.do_action(card, action);
    }

    fn get_possible_moves(&self) -> Vec<usize> {
        let (
            Some(game),
            Some(PendingAction::Move {
                from: Some(from_idx),
            }),
            Some(card),
        ) = (&self.game, &self.pending_action, self.selected_card)
        else {
            return vec![];
        };

        let mut distances: Vec<i8> = card
            .possible_distances()
            .into_iter()
            .map(|x| x as i8)
            .collect();

        if matches!(card, Card::Joker | Card::Four) {
            distances.push(-4);
        }

        let mut targets = Vec::new();
        let board_len = game.board.tiles.len();

        for dist in distances {
            let backward = dist < 0;
            let abs_dist = dist.abs() as u8;

            if !game.can_piece_move_distance(*from_idx, abs_dist, backward) {
                continue;
            }

            for to_idx in 0..board_len {
                let ok = if backward {
                    game.board
                        .distance_between(to_idx, *from_idx, game.current_player_index)
                        == Some(abs_dist)
                } else {
                    game.board
                        .distance_between(*from_idx, to_idx, game.current_player_index)
                        == Some(abs_dist)
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
        // 1. Der "gemalte" Titel
        let title = text("Brändi Dog")
            .size(80)
            .style(IcedColor::from_rgb(0.9, 0.75, 0.2)) // Goldenes Gelb
            .horizontal_alignment(iced::alignment::Horizontal::Center);

        // Subtitel für etwas mehr Flair
        let subtitle = text("Willkommen am Spieltisch")
            .size(22)
            .style(IcedColor::from_rgb(0.7, 0.7, 0.7)) // Leichtes Grau
            .horizontal_alignment(iced::alignment::Horizontal::Center);

        // NEU: Aufforderungs-Text
        let instruction = text("Bitte Spielmodus wählen:")
            .size(18)
            .style(IcedColor::WHITE)
            .horizontal_alignment(iced::alignment::Horizontal::Center);

        // 2. Das Dropdown-Menü
        let dropdown = pick_list(
            &GameVariantKind::ALL[..],
            self.selected_variant,
            Message::VariantSelected,
        )
        .placeholder("Spielmodus wählen...")
        .width(Length::Fixed(300.0))
        .padding(15); 

        // 3. Spalte für die Steuerelemente (Jetzt mit dem Instruction-Text)
        let mut controls = column![instruction, dropdown]
            .spacing(10) // Kleinerer Abstand zwischen Text und Dropdown, damit sie zusammengehören
            .align_items(iced::Alignment::Center);

        // Wenn FreeForAll gewählt ist, zeige das Textfeld
        if self.selected_variant == Some(GameVariantKind::FreeForAll) {
            let ffa_input = text_input("Anzahl Spieler (2-6)", &self.ffa_players_input)
                .on_input(Message::FreeForAllPlayersChanged)
                .padding(15)
                .width(Length::Fixed(300.0));
            controls = controls.push(iced::widget::Space::with_height(Length::Fixed(10.0)));
            controls = controls.push(ffa_input);
        }

        let can_start = self.build_game_variant().is_some();

        let start_btn = button(
            text("Spiel Starten")
                .size(24)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
        )
        .padding([15, 50])
        .on_press_maybe(can_start.then_some(Message::StartGame));

        controls = controls.push(iced::widget::Space::with_height(Length::Fixed(30.0)));
        controls = controls.push(start_btn);

        let menu_card = container(
            column![
                title,
                subtitle,
                iced::widget::Space::with_height(Length::Fixed(50.0)), 
                controls,
            ]
            .align_items(iced::Alignment::Center) 
        )
        .padding(60)
        .style(|_: &Theme| container::Appearance {
            background: Some(iced::Background::Color(IcedColor::from_rgba(0.0, 0.0, 0.0, 0.6))),
            border: iced::Border {
                radius: 20.0.into(),
                width: 3.0,
                color: IcedColor::from_rgb(0.6, 0.4, 0.2), 
            },
            ..Default::default()
        });

        container(menu_card)
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

    fn render_game(&self) -> Element<'_, Message> {
        let game = self.game.as_ref().unwrap();

        let highlights = self.get_possible_moves();
        let board = canvas(BoardView {
            game,
            highlights: highlights.clone(),
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let hand = self.make_hand_view(game);
        let sidebar = self.make_sidebar(game);

        let main_area = row![
            container(board)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(iced::theme::Container::Transparent),
            
            container(sidebar)
                .width(Length::Fixed(250.0))
                .padding(10)
                .style(|_: &Theme| container::Appearance {
                    background: Some(iced::Background::Color(IcedColor::from_rgba(0.0, 0.0, 0.0, 0.3))),
                    text_color: Some(IcedColor::WHITE),
                    ..Default::default()
                })
        ]
        .spacing(0)
        .height(Length::Fill);

        let grab_bar = self.build_grab_bar();

        container(
            column![
                grab_bar.unwrap_or_else(|| container(text("")).padding(0).into()),
                main_area,
                container(hand).padding(0).height(Length::Fixed(180.0)) 
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &Theme| container::Appearance {
            background: Some(iced::Background::Color(IcedColor::from_rgb(0.1, 0.3, 0.2))),
            ..Default::default()
        })
        .into()
    }

    fn build_grab_bar(&self) -> Option<Element<'_, Message>> {
        if let Some(pending) = &self.pending_action {
            match pending {
                PendingAction::Grab => {
                    if self.selected_opponent.is_none() {
                        let mut row_buttons: Row<'_, Message> = Row::new().spacing(10);
                        for (idx, player) in self.game.as_ref().unwrap().players.iter().enumerate()
                        {
                            if idx == self.game.as_ref().unwrap().current_player_index {
                                continue;
                            }
                            row_buttons = row_buttons.push(
                                Button::new(Text::new(format!("{:?}", player.color)).size(14))
                                    .on_press(Message::OpponentSelected(idx)),
                            );
                        }
                        Some(Container::new(row_buttons).padding(10).into())
                    } else {
                        let opponent_idx = self.selected_opponent.unwrap();
                        let opponent_cards =
                            &self.game.as_ref().unwrap().players[opponent_idx].cards;

                        let mut row_buttons: Row<'_, Message> = Row::new().spacing(10);
                        for (idx, _) in opponent_cards.iter().enumerate() {
                            row_buttons = row_buttons.push(
                                Button::new(Text::new(format!("{}", idx + 1)).size(14))
                                    .on_press(Message::OpponentCardSelected(idx)),
                            );
                        }

                        row_buttons = row_buttons.push(
                            Button::new(Text::new("Zurück").size(14))
                                .style(iced::theme::Button::Destructive)
                                .on_press(Message::OpponentCardBack),
                        );

                        Some(Container::new(row_buttons).padding(10).into())
                    }
                }

                PendingAction::TradeGrab => {
                    let opponent_idx = self.selected_opponent.unwrap();
                    let opponent_cards = &self.game.as_ref().unwrap().players[opponent_idx].cards;

                    let mut row_buttons: Row<'_, Message> = Row::new().spacing(10);
                    for (idx, _) in opponent_cards.iter().enumerate() {
                        row_buttons = row_buttons.push(
                            Button::new(Text::new(format!("{}", idx + 1)).size(14))
                                .on_press(Message::OpponentCardSelected(idx)),
                        );
                    }

                    Some(Container::new(row_buttons).padding(10).into())
                }

                _ => None,
            }
        } else {
            None
        }
    }

    fn make_hand_view<'a>(&self, game: &'a Game) -> Element<'a, Message> {
        let hand_logic = HandView {
            game,
            selected_card: self.selected_card,
        };

        container(canvas(hand_logic).width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Transparent)
            .into()
    }
    
    fn debug_view(&self) -> Element<'_, Message> {
        let font_size = 12;
        let game_debug = if let Some(game) = &self.game {
            column![
                text("GAME").size(font_size).style(IcedColor::WHITE),
                text(format!("round: {}", game.round)).size(font_size).style(IcedColor::from_rgb(0.8,0.8,0.8)),
                text(format!("trading_phase: {}", game.trading_phase)).size(font_size).style(IcedColor::from_rgb(0.8,0.8,0.8)),
                text(format!(
                    "current_player: {}",
                    game.current_player_index
                )).size(font_size).style(IcedColor::from_rgb(0.8,0.8,0.8)),
            ]
            .spacing(2)
        } else {
            column![text("GAME: none").size(font_size)].spacing(2)
        };

        column![
            text("DEBUG").size(14).style(IcedColor::WHITE),
            text(format!("sel_card: {:?}", self.selected_card)).size(font_size).style(IcedColor::from_rgb(0.8,0.8,0.8)),
            text(format!("pending: {:?}", self.pending_action)).size(font_size).style(IcedColor::from_rgb(0.8,0.8,0.8)),
            text("----------------").size(font_size).style(IcedColor::from_rgb(0.5,0.5,0.5)),
            game_debug,
        ]
        .spacing(4)
        .into()
    }

    fn make_sidebar(&self, game: &Game) -> Element<'_, Message> {
        let font_std = 16;
        let info = column![
            text(format!("Runde: {}", game.round)).size(18).style(IcedColor::WHITE),
            text(format!(
                "Am Zug: {:?} (P{})",
                game.current_player().color,
                game.current_player_index
            )).size(18).style(IcedColor::WHITE),
            text(&self.msg).size(14).style(IcedColor::from_rgb(0.9, 0.9, 0.9)),
        ]
        .spacing(10);

        let player = game.current_player();
        let hand = &player.cards;
        let piece_on_board = (player.pieces_to_place + player.pieces_in_house) < 4;
        let can_move =
            piece_on_board && hand.iter().any(|c| !matches!(c, Card::Jack | Card::Seven));
        let can_interchange =
            piece_on_board && hand.iter().any(|c| matches!(c, Card::Jack | Card::Joker));
        let has_place_card = hand
            .iter()
            .any(|c| matches!(c, Card::Ace | Card::King | Card::Joker));
        let has_grab_card = hand.iter().any(|c| matches!(c, Card::Two))
            && matches!(self.selected_variant, Some(GameVariantKind::FreeForAll));

        let mut btns = column![].spacing(10);

        if !game.trading_phase {
            if can_move {
                btns = btns.push(
                    button(text("Ziehen (Move)").size(font_std))
                        .on_press(Message::GameActionBtn(GameAction::Move))
                        .width(Length::Fill),
                );
            }

            if has_place_card {
                btns = btns.push(
                    button(text("Legen (Place)").size(font_std))
                        .on_press(Message::GameActionBtn(GameAction::Place))
                        .width(Length::Fill),
                );
            }

            if can_interchange {
                btns = btns.push(
                    button(text("Tauschen (Interchange)").size(font_std))
                        .on_press(Message::GameActionBtn(GameAction::Interchange))
                        .width(Length::Fill),
                );
            }

            btns = btns.push(
                button(text("Abwerfen (Remove)").size(font_std))
                    .on_press(Message::GameActionBtn(GameAction::Remove))
                    .width(Length::Fill),
            );

            if has_grab_card {
                btns = btns.push(
                    button(text("Klauen (Grab)").size(font_std))
                        .on_press(Message::GameActionBtn(GameAction::Grab))
                        .width(Length::Fill),
                );
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

        column![
            info,
            btns,
            container(self.debug_view())
                .padding(10)
                .style(|_: &Theme| container::Appearance {
                    background: Some(iced::Background::Color(IcedColor::from_rgba(0.0, 0.0, 0.0, 0.3))),
                    text_color: Some(IcedColor::from_rgb(0.7, 0.7, 0.7)),
                    border: iced::Border {
                        radius: 5.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(30)
        .into()
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
            GameAction::Grab => {
                self.pending_action = Some(PendingAction::Grab);
                self.selected_opponent = None;
                self.selected_opponent_card = None;
                self.msg = "Wähle einen Gegner zum Klauen.".into();
            }

            GameAction::TradeGrab => {
                let game = self.game.as_ref().unwrap();
                let prev_idx = if game.current_player_index == 0 {
                    game.players.len() - 1
                } else {
                    game.current_player_index - 1
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
                        target_player: current_idx,
                    },
                };
                self.do_action(card, act);
            }
            GameAction::Remove => {
                let act = Action {
                    player: current_color,
                    card: Some(card),
                    action: ActionKind::Remove,
                };
                self.do_action(card, act);
            }
            GameAction::Trade => {
                let act = Action {
                    player: current_color,
                    card: Some(card),
                    action: ActionKind::Trade,
                };
                self.do_action(card, act);
            }
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
                PendingAction::Grab | PendingAction::TradeGrab => {
                    self.msg = "Wähle einen Gegner und eine Karte oben.".into();
                }
                PendingAction::Move { from } => {
                    if let Some(start_idx) = from {
                        let act = Action {
                            player: current_color,
                            card: Some(card),
                            action: ActionKind::Move {
                                from: start_idx,
                                to: tile_idx,
                            },
                        };
                        self.do_action(card, act);
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
                        self.do_action(card, act);
                    }
                },
            }
        } else {
            self.msg = "Wähle rechts erst eine Aktion (z.B. Move).".to_string();
        }
    }

    fn do_action(&mut self, card: Card, mut action: Action) {
        if matches!(action.action, ActionKind::TradeGrab { .. }) {
            action.card = None;
        } else {
            action.card = Some(card);
        }

        if let Some(game) = self.game.as_mut() {
            match game.action(action.card, action) {
                Ok(_) => self.msg = "Zug erfolgreich!".into(),
                Err(e) => self.msg = format!("Fehler: {}", e),
            }

            self.selected_card = None;
            self.pending_action = None;
            self.selected_opponent = None;
            self.selected_opponent_card = None;
        }
    }
}


struct BoardView<'a> {
    game: &'a Game,
    highlights: Vec<usize>,
}


fn get_tile_position(index: usize, total_players: usize, center: Point, scale: f32, rotation_angle: f32) -> Point {
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
            let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
            let total_players = self.game.players.len();
            let total_tiles = total_players * 16 + total_players * 4;
            let ring_size = self.game.board.ring_size;

            let min_dim = bounds.width.min(bounds.height);
            let scale = min_dim / 850.0; 
            
            let current_p_idx = self.game.current_player_index;
            let current_p_angle = (current_p_idx as f32 * 16.0 / ring_size as f32) * std::f32::consts::TAU;
            let rotation = std::f32::consts::FRAC_PI_2 - current_p_angle;

            for i in 0..total_tiles {
                let pos = get_tile_position(i, total_players, center, scale, rotation);
                if is_hit(cursor_position, pos, 12.0 * scale) { 
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
        let ring_size = self.game.board.ring_size;
        let total_tiles = ring_size + total_players * 4;

        let min_dim = bounds.width.min(bounds.height);
        let scale = min_dim / 850.0; 

        let current_p_idx = self.game.current_player_index;
        let current_p_angle = (current_p_idx as f32 * 16.0 / ring_size as f32) * std::f32::consts::TAU;
        let rotation = std::f32::consts::FRAC_PI_2 - current_p_angle;

        let board_radius = 340.0 * scale;
        let line_width = 3.0 * scale;
        
        let shadow = canvas::Path::circle(Point::new(center.x + 10.0*scale, center.y + 10.0*scale), board_radius);
        frame.fill(&shadow, IcedColor::from_rgba(0.0, 0.0, 0.0, 0.5));

        let bg = canvas::Path::circle(center, board_radius);
        frame.fill(&bg, IcedColor::from_rgb(0.85, 0.70, 0.55)); // Holz
        frame.stroke(&bg, canvas::Stroke::default().with_width(line_width).with_color(IcedColor::from_rgb(0.5, 0.3, 0.1)));

    
        
        for offset in 1..total_players {
            let opponent_idx = (current_p_idx + offset) % total_players;
            let card_count = self.game.players[opponent_idx].cards.len();
            
            let cw = 30.0 * scale;
            let ch = 45.0 * scale;
            
            let card_orbit_radius = board_radius + (ch / 2.0) + (15.0 * scale); 

         
            let angle_step = std::f32::consts::TAU / (total_players as f32);
            let card_angle = std::f32::consts::FRAC_PI_2 - (offset as f32 * angle_step);

            let center_card_pos = Point::new(
                center.x + card_orbit_radius * card_angle.cos(),
                center.y + card_orbit_radius * card_angle.sin() 
            );

         
            let is_horizontal = card_angle.cos().abs() < 0.5;

            for c in 0..card_count {
                let spread = 15.0 * scale;
                let total_w = (card_count as f32 - 1.0) * spread;
                
                let pos = if !is_horizontal {
                    let start_y = center_card_pos.y - total_w / 2.0;
                    Point::new(center_card_pos.x, start_y + (c as f32 * spread))
                } else {
                    let start_x = center_card_pos.x - total_w / 2.0;
                    Point::new(start_x + (c as f32 * spread), center_card_pos.y)
                };

                let rect = if !is_horizontal {
                    iced::Rectangle::new(Point::new(pos.x - ch/2.0, pos.y - cw/2.0), iced::Size::new(ch, cw))
                } else {
                    iced::Rectangle::new(Point::new(pos.x - cw/2.0, pos.y - ch/2.0), iced::Size::new(cw, ch))
                };

                let back = canvas::Path::rectangle(rect.position(), rect.size());
                frame.fill(&back, IcedColor::from_rgb(0.2, 0.3, 0.7)); 
                frame.stroke(&back, canvas::Stroke::default().with_color(IcedColor::WHITE).with_width(2.0 * scale));
            }
        }

        let track_stroke = canvas::Stroke::default()
            .with_width(line_width)
            .with_color(IcedColor::from_rgba(0.4, 0.2, 0.1, 0.3));

        for i in 0..ring_size {
            let p1 = get_tile_position(i, total_players, center, scale, rotation);
            let p2 = get_tile_position((i + 1) % ring_size, total_players, center, scale, rotation);
            let path = canvas::Path::new(|p| {
                p.move_to(p1);
                p.line_to(p2);
            });
            frame.stroke(&path, track_stroke.clone());
        }

        for p_idx in 0..total_players {
            let start_idx = self.game.board.start_field(p_idx);
            let house_start_idx = ring_size + p_idx * 4;
            
            let p_start = get_tile_position(start_idx, total_players, center, scale, rotation);
            let p_house = get_tile_position(house_start_idx, total_players, center, scale, rotation);
            
            let path_entry = canvas::Path::new(|p| {
                p.move_to(p_start);
                p.line_to(p_house);
            });
            frame.stroke(&path_entry, track_stroke.clone());

            for k in 0..3 {
                let h1 = get_tile_position(house_start_idx + k, total_players, center, scale, rotation);
                let h2 = get_tile_position(house_start_idx + k + 1, total_players, center, scale, rotation);
                let path_house = canvas::Path::new(|p| {
                    p.move_to(h1);
                    p.line_to(h2);
                });
                frame.stroke(&path_house, track_stroke.clone());
            }
        }

        let board_state = self.game.board_state();
        let current_color_enum = self.game.current_player().color;
        let current_color_iced = match current_color_enum {
            GameColor::Red => IcedColor::from_rgb(0.8, 0.2, 0.2),
            GameColor::Green => IcedColor::from_rgb(0.2, 0.8, 0.2),
            GameColor::Blue => IcedColor::from_rgb(0.2, 0.2, 0.8),
            GameColor::Yellow => IcedColor::from_rgb(0.8, 0.8, 0.2),
            GameColor::Purple => IcedColor::from_rgb(0.5, 0.0, 0.5),
            GameColor::Orange => IcedColor::from_rgb(1.0, 0.65, 0.0),
        };

        // 4. Felder und Figuren
        for i in 0..total_tiles {
            let pos = get_tile_position(i, total_players, center, scale, rotation);
            
            let mut is_start = false;
            let mut start_mark_color = IcedColor::TRANSPARENT;
            
            for p in 0..total_players {
                if i == self.game.board.start_field(p) {
                    is_start = true;
                    start_mark_color = match self.game.players[p].color {
                        GameColor::Red => IcedColor::from_rgb(0.8, 0.2, 0.2),
                        GameColor::Green => IcedColor::from_rgb(0.2, 0.8, 0.2),
                        GameColor::Blue => IcedColor::from_rgb(0.2, 0.2, 0.8),
                        GameColor::Yellow => IcedColor::from_rgb(0.8, 0.8, 0.2),
                        GameColor::Purple => IcedColor::from_rgb(0.5, 0.0, 0.5),
                        GameColor::Orange => IcedColor::from_rgb(1.0, 0.65, 0.0),
                    };
                }
            }

            if is_start {
                let marker = canvas::Path::circle(pos, 13.0 * scale);
                frame.fill(&marker, IcedColor::from_rgba(start_mark_color.r, start_mark_color.g, start_mark_color.b, 0.3));
            }

            match board_state.get(i).and_then(|t| t.as_ref()) {
                Some(piece) => {
                    let color = match self.game.players[piece.owner].color {
                        GameColor::Red => IcedColor::from_rgb(0.8, 0.2, 0.2),
                        GameColor::Green => IcedColor::from_rgb(0.2, 0.8, 0.2),
                        GameColor::Blue => IcedColor::from_rgb(0.2, 0.2, 0.8),
                        GameColor::Yellow => IcedColor::from_rgb(0.8, 0.8, 0.2),
                        GameColor::Purple => IcedColor::from_rgb(0.5, 0.0, 0.5),
                        GameColor::Orange => IcedColor::from_rgb(1.0, 0.65, 0.0),
                    };
                    draw_marble(&mut frame, pos, color, scale);
                },
                None => {
                    let shadow = canvas::Path::circle(Point::new(pos.x + 1.0*scale, pos.y + 1.0*scale), 7.0 * scale);
                    frame.fill(&shadow, IcedColor::from_rgba(0.0,0.0,0.0,0.2));
                    
                    let hole = canvas::Path::circle(pos, 7.0 * scale);
                    frame.fill(&hole, IcedColor::from_rgb(0.4, 0.3, 0.2));
                }
            };

            if self.highlights.contains(&i) {
                let ghost_fill = IcedColor::from_rgba(current_color_iced.r, current_color_iced.g, current_color_iced.b, 0.4);
                let ghost = canvas::Path::circle(pos, 6.0 * scale);
                frame.fill(&ghost, ghost_fill);

                let ring = canvas::Path::circle(pos, 11.0 * scale);
                frame.stroke(&ring, canvas::Stroke::default().with_color(current_color_iced).with_width(3.0 * scale));
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

                let start_idx = self.game.board.start_field(p_idx);
                let angle_step = std::f32::consts::TAU / (ring_size as f32);
                let start_angle = (start_idx as f32) * angle_step + rotation;

                let wait_radius = 295.0 * scale;
                let marble_spacing = 0.08; 
                let center_offset = 0.12; 

                for k in 0..count {
                    let offset_angle = start_angle + center_offset - (k as f32 * marble_spacing);
                    
                    let wait_pos = Point::new(
                        center.x + wait_radius * offset_angle.cos(),
                        center.y + wait_radius * offset_angle.sin(),
                    );

                    let hole = canvas::Path::circle(wait_pos, 6.0 * scale);
                    frame.fill(&hole, IcedColor::from_rgba(0.0, 0.0, 0.0, 0.1));

                    draw_marble(&mut frame, wait_pos, p_color, scale);
                }
            }
        }

        vec![frame.into_geometry()]
    }
}


struct HandView<'a> {
    game: &'a Game,
    selected_card: Option<Card>,
}

impl<'a> HandView<'a> {
    fn get_layout(&self, bounds: iced::Rectangle, cursor_position: Point) -> Vec<(usize, Card, iced::Rectangle, bool)> {
        let cards = &self.game.current_player().cards;
        let count = cards.len();
        if count == 0 { return Vec::new(); }

        let scale = 1.0; 
        
        let card_w = 60.0 * scale;
        let card_h = 90.0 * scale;
        let gap = 15.0 * scale;
        
        let total_w = (count as f32 * card_w) + ((count as f32 - 1.0) * gap);
        let start_x = (bounds.width / 2.0) - (total_w / 2.0); // Zentriert
        let base_y = (bounds.height / 2.0) - (card_h / 2.0) + (10.0 * scale);

        cards.iter().enumerate().map(|(i, &card)| {
            let x = start_x + (i as f32 * (card_w + gap));
            let mut y = base_y;

            let base_rect = iced::Rectangle::new(Point::new(x, y), iced::Size::new(card_w, card_h));
            let is_hovered = base_rect.contains(cursor_position);
            let is_selected = Some(card) == self.selected_card;
            
            if is_hovered || is_selected {
                y -= 15.0 * scale; 
            }

            let final_rect = iced::Rectangle::new(Point::new(x, y), iced::Size::new(card_w, card_h));

            (i, card, final_rect, is_hovered)
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
        
        let scale = 1.0;

        for (_i, card, rect, is_hovered) in layout {
            let is_selected = Some(card) == self.selected_card;

            let bg_path = canvas::Path::rectangle(rect.position(), rect.size());
            
            if is_hovered || is_selected {
                let shadow_rect = canvas::Path::rectangle(Point::new(rect.x + 3.0*scale, rect.y + 10.0*scale), rect.size());
                frame.fill(&shadow_rect, IcedColor::from_rgba(0.0, 0.0, 0.0, 0.2));
            } else {
                let shadow_rect = canvas::Path::rectangle(Point::new(rect.x + 1.0*scale, rect.y + 1.0*scale), rect.size());
                frame.fill(&shadow_rect, IcedColor::from_rgba(0.0, 0.0, 0.0, 0.1));
            }

            let bg_color = if card == Card::Joker { IcedColor::from_rgb(1.0, 0.95, 0.95) } else { IcedColor::WHITE };
            frame.fill(&bg_path, bg_color);

            let border_color = if is_selected { 
                IcedColor::from_rgb(0.0, 0.5, 1.0) 
            } else if is_hovered {
                IcedColor::from_rgb(0.3, 0.3, 0.3) 
            } else {
                IcedColor::from_rgb(0.7, 0.7, 0.7) 
            };
            let border_width = if is_selected { 3.0 * scale } else { 1.0 * scale };
            
            frame.stroke(&bg_path, canvas::Stroke::default().with_color(border_color).with_width(border_width));

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
            
            let text_color = if card == Card::Joker { IcedColor::from_rgb(0.8, 0.0, 0.0) } else { IcedColor::BLACK };

            frame.fill_text(canvas::Text {
                content: label.to_string(),
                position: Point::new(rect.x + 5.0*scale, rect.y + 5.0*scale),
                color: text_color,
                size: (12.0*scale).into(),
                ..Default::default()
            });

            draw_card_art(&mut frame, card, rect, text_color);
        }

        vec![frame.into_geometry()]
    }
}

// --- Drawing Stuff ---

fn draw_card_art(frame: &mut canvas::Frame, card: Card, rect: iced::Rectangle, color: IcedColor) {
    let center = rect.center();
    let w = rect.width;
    
    let haut = IcedColor::from_rgb(0.98, 0.88, 0.75); 
    let gold = IcedColor::from_rgb(1.0, 0.8, 0.0);    
    let blond = IcedColor::from_rgb(0.95, 0.85, 0.3); 
    let bart_blond = IcedColor::from_rgb(0.85, 0.75, 0.2); 
    
    let suit_color = color;
    let rot = IcedColor::from_rgb(0.85, 0.1, 0.1); 

    match card {
        Card::King => {
            let r = w * 0.26;
            let hair = canvas::Path::new(|p| p.arc(canvas::path::Arc { center, radius: r + 2.0, start_angle: iced::Radians(0.0), end_angle: iced::Radians(6.28) }));
            frame.fill(&hair, blond);
            let head = canvas::Path::circle(center, r);
            frame.fill(&head, haut);
            frame.stroke(&head, canvas::Stroke::default().with_width(1.5));
            let beard = canvas::Path::new(|p| {
                p.move_to(Point::new(center.x - r + 2.0, center.y));
                p.quadratic_curve_to(Point::new(center.x, center.y + r + 15.0), Point::new(center.x + r - 2.0, center.y));
                p.close();
            });
            frame.fill(&beard, bart_blond);
            let mustache = canvas::Path::new(|p| {
                p.move_to(Point::new(center.x - 10.0, center.y + 10.0));
                p.quadratic_curve_to(Point::new(center.x, center.y + 5.0), Point::new(center.x + 10.0, center.y + 10.0));
                p.line_to(Point::new(center.x, center.y + 8.0));
                p.close();
            });
            frame.fill(&mustache, IcedColor::BLACK);
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
            let head = canvas::Path::circle(center, r);
            frame.fill(&head, haut);
            frame.stroke(&head, canvas::Stroke::default().with_width(1.5));
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
            let lips = canvas::Path::new(|p| {
                 p.move_to(Point::new(center.x - 4.0, center.y + 10.0));
                 p.quadratic_curve_to(Point::new(center.x, center.y + 13.0), Point::new(center.x + 4.0, center.y + 10.0));
            });
            frame.stroke(&lips, canvas::Stroke::default().with_color(IcedColor::from_rgb(0.8, 0.2, 0.2)).with_width(2.0));
        }
        Card::Jack => {
            let r = w * 0.23;
            let hair = canvas::Path::new(|p| {
                p.move_to(Point::new(center.x, center.y - r));
                p.quadratic_curve_to(Point::new(center.x - r - 8.0, center.y), Point::new(center.x - r, center.y + r));
                p.line_to(Point::new(center.x + r, center.y + r));
                p.quadratic_curve_to(Point::new(center.x + r + 8.0, center.y), Point::new(center.x, center.y - r));
            });
            frame.fill(&hair, blond);
            frame.stroke(&hair, canvas::Stroke::default().with_color(IcedColor::from_rgba(0.0,0.0,0.0,0.3)).with_width(1.0));
            let head = canvas::Path::circle(center, r);
            frame.fill(&head, haut);
            frame.stroke(&head, canvas::Stroke::default().with_width(1.5));
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
            let feather = canvas::Path::new(|p| {
                let y = center.y - r * 0.6 - 10.0;
                p.move_to(Point::new(center.x + 10.0, y));
                p.quadratic_curve_to(Point::new(center.x + 25.0, y - 20.0), Point::new(center.x + 15.0, y - 25.0));
                p.quadratic_curve_to(Point::new(center.x + 15.0, y - 10.0), Point::new(center.x + 10.0, y));
            });
            frame.fill(&feather, IcedColor::WHITE);
            frame.stroke(&feather, canvas::Stroke::default().with_color(IcedColor::BLACK).with_width(1.0));
            draw_eyes(frame, center, 2.0);
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
            let cap = canvas::Path::new(|p| {
                let y = center.y - r * 0.6;
                p.move_to(Point::new(center.x - r, y));
                p.line_to(Point::new(center.x + r, y));
                p.quadratic_curve_to(Point::new(center.x + r + 15.0, y - 10.0), Point::new(center.x + r + 5.0, y + 10.0));
                p.quadratic_curve_to(Point::new(center.x + 10.0, y - 25.0), Point::new(center.x, y));
                p.quadratic_curve_to(Point::new(center.x - 10.0, y - 25.0), Point::new(center.x - r - 5.0, y + 10.0));
                p.quadratic_curve_to(Point::new(center.x - r - 15.0, y - 10.0), Point::new(center.x - r, y));
            });
            frame.fill(&cap, rot);
            frame.stroke(&cap, canvas::Stroke::default().with_color(IcedColor::BLACK).with_width(1.0));
            let y = center.y - r * 0.6;
            let bell_l = canvas::Path::circle(Point::new(center.x - r - 5.0, y + 10.0), 3.0);
            let bell_r = canvas::Path::circle(Point::new(center.x + r + 5.0, y + 10.0), 3.0);
            frame.fill(&bell_l, gold);
            frame.fill(&bell_r, gold);
            draw_eyes(frame, center, 2.5);
            let nose = canvas::Path::circle(Point::new(center.x, center.y + 2.0), 4.5);
            frame.fill(&nose, rot);
            let smile = canvas::Path::new(|p| {
                p.move_to(Point::new(center.x - 8.0, center.y + 7.0));
                p.quadratic_curve_to(
                    Point::new(center.x, center.y + 11.0), 
                    Point::new(center.x + 8.0, center.y + 7.0)
                );
            });
            frame.stroke(&smile, canvas::Stroke::default().with_color(IcedColor::BLACK).with_width(2.0));
        }
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
            let handle_l = canvas::Path::circle(Point::new(center.x - 10.0, center.y + 18.0), 4.0);
            let handle_r = canvas::Path::circle(Point::new(center.x + 10.0, center.y + 18.0), 4.0);
            frame.stroke(&handle_l, canvas::Stroke::default().with_color(color).with_width(2.0));
            frame.stroke(&handle_r, canvas::Stroke::default().with_color(color).with_width(2.0));
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

fn draw_eyes(frame: &mut canvas::Frame, center: Point, sz: f32) {
    let l = canvas::Path::circle(Point::new(center.x - 5.0, center.y - 3.0), sz);
    let r = canvas::Path::circle(Point::new(center.x + 5.0, center.y - 3.0), sz);
    frame.fill(&l, IcedColor::BLACK);
    frame.fill(&r, IcedColor::BLACK);
}

fn draw_marble(frame: &mut canvas::Frame, center: Point, color: IcedColor, scale: f32) {
    let radius = 10.0 * scale;
    let shadow = canvas::Path::circle(Point::new(center.x + 2.0*scale, center.y + 2.0*scale), radius);
    frame.fill(&shadow, IcedColor::from_rgba(0.0, 0.0, 0.0, 0.3));
    let body = canvas::Path::circle(center, radius);
    frame.fill(&body, color);
    frame.stroke(&body, canvas::Stroke::default().with_color(IcedColor::from_rgba(0.0,0.0,0.0,0.2)).with_width(1.0 * scale));
    let shine_pos = Point::new(center.x - radius * 0.3, center.y - radius * 0.3);
    let shine = canvas::Path::circle(shine_pos, radius * 0.4);
    frame.fill(&shine, IcedColor::from_rgba(1.0, 1.0, 1.0, 0.4));
    let spot_pos = Point::new(center.x - radius * 0.4, center.y - radius * 0.4);
    let spot = canvas::Path::circle(spot_pos, radius * 0.15);
    frame.fill(&spot, IcedColor::from_rgba(1.0, 1.0, 1.0, 0.8));
}
