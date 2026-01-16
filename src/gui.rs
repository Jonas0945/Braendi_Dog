use std::fmt::Debug;
use std::convert::TryInto;

use iced::Length;
use iced::widget::{container, row};
use iced::{
    Element, Sandbox, Settings,
    widget::{button, column, pick_list, text, text_input},
};

use braendi_dog::Action;
use braendi_dog::ActionKind;
use braendi_dog::Card;
use braendi_dog::game::DogGame;
use braendi_dog::game::Game;
use braendi_dog::game::board::Point;
use braendi_dog::{GameVariant, game::board::SEGMENT_LENGTH};

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

#[derive(Debug, Clone)]
enum GameAction {
    Place,
    Move,
    Remove,
    Trade,
    Grab,
    Interchange,
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

enum Screen {
    Start,
    Game,
}

struct DogApp {
    game: Option<Game>,
    screen: Screen,
    selected_variant: Option<GameVariantKind>,
    selected_card: Option<Card>,
    clicked_tile: Option<Point>,
    ffa_players_input: String,
    status_message: String,
    pending_action: Option<PendingAction>,
}

#[derive(Debug, Clone)]
enum PendingAction {
    Move { from: Option<Point> },
    Interchange { from: Option<Point> },
}

#[derive(Debug, Clone)]
enum Message {
    CardSelected(Card),
    CancelPendingAction,
    TileClicked(Point),
    VariantSelected(GameVariantKind),
    FreeForAllPlayersChanged(String),
    StartGame,
    GameAction(GameAction),
}

impl Sandbox for DogApp {
    type Message = Message;

    fn new() -> Self {
        Self {
            game: None,
            screen: Screen::Start,
            selected_variant: None,
            ffa_players_input: String::new(),
            selected_card: None,
            clicked_tile: None,
            status_message: String::from("Willkommen bei Brändi Dog!"),
            pending_action: None,
        }
    }

    fn title(&self) -> String {
        String::from("Dog Game")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::CardSelected(card) => {
                self.selected_card = Some(card);
                self.status_message = format!("Karte gewählt: {:?}", card);
            }
            Message::TileClicked(point) => match self.pending_action.take() {
                Some(PendingAction::Move { from: None }) => {
                    self.pending_action = Some(PendingAction::Move { from: Some(point) });
                    self.status_message = format!("Move FROM selected: {}", point);
                }
                Some(PendingAction::Move { from: Some(from) }) => {
                    self.execute_move(from, point);
                }
                Some(PendingAction::Interchange { from: None }) => {
                    self.pending_action = Some(PendingAction::Interchange { from: Some(point) });
                    self.status_message = format!("Interchange FIRST selected: {}", point);
                }
                Some(PendingAction::Interchange { from: Some(from) }) => {
                    self.execute_interchange(from, point);
                }
                None => {
                    self.clicked_tile = Some(point);
                    self.status_message = format!("Tile clicked: {}", point);
                }
            },
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
                }
            }
            Message::GameAction(action) => {
                self.status_message = format!("Action selected: {:?}", action);
                self.try_apply_action(action);
            }
            Message::CancelPendingAction => {
                self.pending_action = None;
                self.clicked_tile = None;
                self.status_message = "Action cancelled".to_string();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Start => self.start_view(),
            Screen::Game => self.game_view(),
        }
    }
}

impl DogApp {
    fn reset_selection(&mut self) {
        self.selected_card = None;
        self.clicked_tile = None;
        self.pending_action = None;
    }

    fn execute_move(&mut self, from: Point, to: Point) {
        let game = match self.game.as_mut() {
            Some(g) => g,
            None => return,
        };
        let card = match self.selected_card {
            Some(c) => c,
            None => return,
        };
        let player = game.current_player().color;
        let act = Action {
            player,
            card,
            action: ActionKind::Move { from, to },
        };
        match game.action(card, act) {
            Ok(_) => {
                self.status_message =
                    format!("{} moved {:?} from {} to {}", player, card, from, to);
                self.reset_selection();
            }
            Err(e) => self.status_message = format!("Move failed: {:?}", e),
        }
    }

    fn execute_interchange(&mut self, a: Point, b: Point) {
        let game = match self.game.as_mut() {
            Some(g) => g,
            None => return,
        };
        let card = match self.selected_card {
            Some(c) => c,
            None => return,
        };
        let player = game.current_player().color;
        let act = Action {
            player,
            card,
            action: ActionKind::Interchange { a, b },
        };
        match game.action(card, act) {
            Ok(_) => {
                self.status_message =
                    format!("{} interchanged {:?} between {} and {}", player, card, a, b);
                self.reset_selection();
            }
            Err(e) => self.status_message = format!("Interchange failed: {:?}", e),
        }
    }

    fn try_apply_action(&mut self, action: GameAction) {
        let game = match self.game.as_mut() {
            Some(g) => g,
            None => {
                self.status_message = "Game not started".into();
                return;
            }
        };
        let card = match self.selected_card {
            Some(c) => c,
            None => {
                self.status_message = "Select a card first".into();
                return;
            }
        };
        let cur_player = game.current_player();
        let idx = game.current_player_index;
        let cur_player_color = cur_player.color;

        match action {
            GameAction::Place => {
                let act = Action {
                    player: cur_player_color,
                    card,
                    action: ActionKind::Place { target_player: idx },
                };
                match game.action(card, act) {
                    Ok(_) => {
                        self.status_message =
                            format!("{} placed {:?} successfully", cur_player_color, card);
                        self.reset_selection();
                    }
                    Err(e) => self.status_message = format!("Action failed: {:?}", e),
                }
            }
            GameAction::Remove => {
                let act = Action {
                    player: cur_player_color,
                    card,
                    action: ActionKind::Remove,
                };
                match game.action(card, act) {
                    Ok(_) => {
                        self.status_message =
                            format!("{} Removed {:?} successfully", cur_player_color, card);
                        self.reset_selection();
                    }
                    Err(e) => self.status_message = format!("Action failed: {:?}", e),
                }
            }
            GameAction::Trade => {
                let act = Action {
                    player: cur_player_color,
                    card,
                    action: ActionKind::Trade,
                };
                match game.action(card, act) {
                    Ok(_) => {
                        self.status_message =
                            format!("{} Traded {:?} successfully", cur_player_color, card);
                        self.reset_selection();
                    }
                    Err(e) => self.status_message = format!("Action failed: {:?}", e),
                }
            }
            GameAction::Move => {
                self.pending_action = Some(PendingAction::Move { from: None });
                self.status_message = format!("Move selected. Click FROM tile. Card: {:?}", card);
            }
            GameAction::Interchange => {
                self.pending_action = Some(PendingAction::Interchange { from: None });
                self.status_message =
                    format!("Interchange selected. Click FIRST tile. Card: {:?}", card);
            }
            _ => {
                self.status_message = "Not implemented yet".into();
            }
        }
    }

    fn view_board(&self, game: &Game) -> Element<'_, Message> {
        let num_players = game.players.len();
        let mut player_rows: Vec<Element<Message>> = Vec::new();

        for player_index in 0..num_players {
            let start = game.board.start_field(player_index);
            let house = game.board.house_by_player(player_index);

            let track_tiles: Vec<Element<Message>> = (0..SEGMENT_LENGTH)
                .map(|i| {
                    let idx = (start + i) % game.board.tiles.len();
                    let tile_text = match &game.board.tiles[idx] {
                        Some(piece) => match game.players[piece.owner].color {
                            braendi_dog::game::Color::Red => "R",
                            braendi_dog::game::Color::Green => "G",
                            braendi_dog::game::Color::Blue => "B",
                            braendi_dog::game::Color::Yellow => "Y",
                            braendi_dog::game::Color::Purple => "P",
                            braendi_dog::game::Color::Orange => "O",
                        }
                        .to_string(),
                        None => ".".to_string(),
                    };
                    container(button(text(tile_text)).on_press(Message::TileClicked(idx)))
                        .width(Length::FillPortion(1))
                        .height(Length::Fill)
                        .padding(2)
                        .into()
                })
                .collect();

            let house_tiles: Vec<Element<Message>> = house
                .iter()
                .copied()
                .map(|idx| {
                    let tile_text = match &game.board.tiles[idx] {
                        Some(piece) => match game.players[piece.owner].color {
                            braendi_dog::game::Color::Red => "R",
                            braendi_dog::game::Color::Green => "G",
                            braendi_dog::game::Color::Blue => "B",
                            braendi_dog::game::Color::Yellow => "Y",
                            braendi_dog::game::Color::Purple => "P",
                            braendi_dog::game::Color::Orange => "O",
                        }
                        .to_string(),
                        None => ".".to_string(),
                    };
                    container(button(text(tile_text)).on_press(Message::TileClicked(idx)))
                        .width(Length::FillPortion(1))
                        .height(Length::Fill)
                        .padding(2)
                        .into()
                })
                .collect();

            let label = container(text(format!("{:?}", game.players[player_index].color)))
                .width(Length::FillPortion(2))
                .height(Length::Fill)
                .center_x()
                .center_y();

            let num_house_tiles: u16 = house.len().try_into().unwrap();
            let player_row = row![
                label,
                row(track_tiles)
                    .spacing(2)
                    .width(Length::FillPortion(SEGMENT_LENGTH.try_into().unwrap()))
                    .height(Length::Fill),
                text("|").width(Length::FillPortion(1)).height(Length::Fill),
                row(house_tiles)
                    .spacing(2)
                    .width(Length::FillPortion(num_house_tiles))
                    .height(Length::Fill)
            ]
            .spacing(5)
            .width(Length::Fill)
            .height(Length::FillPortion(1));

            player_rows.push(player_row.into());
        }

        column(player_rows).spacing(5).width(Length::Fill).height(Length::Fill).into()
    }

    fn view_hand(&self, game: &Game) -> Element<'_, Message> {
        let current_player = game.current_player();
        let cards: Vec<Element<Message>> = current_player
            .cards
            .iter()
            .map(|card| button(text(format!("{:?}", card)))
                .on_press(Message::CardSelected(*card))
                .padding(10)
                .into())
            .collect();
        container(row(cards).spacing(10)).center_x().into()
    }

    fn start_view(&self) -> Element<'_, Message> {
        let dropdown = pick_list(GameVariantKind::ALL, self.selected_variant, Message::VariantSelected)
            .placeholder("Select game variant");
        let mut content = column![text("Choose game variant"), dropdown].spacing(12);
        if self.selected_variant == Some(GameVariantKind::FreeForAll) {
            content = content.push(
                text_input("Number of players", &self.ffa_players_input)
                    .on_input(Message::FreeForAllPlayersChanged),
            );
        }
        let can_start = self.build_game_variant().is_some();
        content.push(button("Start Game").on_press_maybe(can_start.then_some(Message::StartGame)))
            .padding(20)
            .into()
    }

    fn game_view(&self) -> Element<'_, Message> {
        let game = self.game.as_ref().expect("game must exist");
        let pending_action_text = match &self.pending_action {
            Some(PendingAction::Move { from: None }) => "Move: click FROM tile",
            Some(PendingAction::Move { from: Some(_) }) => "Move: click TO tile",
            Some(PendingAction::Interchange { from: None }) => "Interchange: click FIRST tile",
            Some(PendingAction::Interchange { from: Some(_) }) => "Interchange: click SECOND tile",
            None => "No action in progress",
        };
        let selected_card_text = match self.selected_card {
            Some(card) => format!("Selected Card: {:?}", card),
            None => "Selected Card: None".to_string(),
        };
        let clicked_tile_text = match self.clicked_tile {
            Some(tile) => format!("Clicked Tile: {}", tile),
            None => "Clicked Tile: None".to_string(),
        };
        let last_clicked_tile_text = match self.clicked_tile {
            Some(tile) => format!("Last Clicked Tile: {}", tile),
            None => "Last Clicked Tile: None".to_string(),
        };
        let status_info = column![
            text(format!("Variant: {:?}", game.game_variant)),
            text(format!("Round: {}", game.round)),
            text(format!("Current player index: {}", game.current_player_index)),
            text(format!("Trading phase: {}", game.trading_phase)),
            text(&selected_card_text),
            text(&clicked_tile_text),
            text(&last_clicked_tile_text),
            text(&self.status_message),
            text(&pending_action_text),
        ].spacing(5).padding(10);

        let buttons_enabled = self.pending_action.is_none();
        let mut action_buttons = column![
            button("Place").on_press_maybe(buttons_enabled.then_some(Message::GameAction(GameAction::Place))),
            button("Move").on_press_maybe(buttons_enabled.then_some(Message::GameAction(GameAction::Move))),
            button("Remove").on_press_maybe(buttons_enabled.then_some(Message::GameAction(GameAction::Remove))),
            button("Trade").on_press_maybe(buttons_enabled.then_some(Message::GameAction(GameAction::Trade))),
            button("Grab").on_press_maybe(buttons_enabled.then_some(Message::GameAction(GameAction::Grab))),
            button("Interchange").on_press_maybe(buttons_enabled.then_some(Message::GameAction(GameAction::Interchange))),
        ].spacing(5);

        if self.pending_action.is_some() {
            action_buttons = action_buttons.push(button("Cancel action").on_press(Message::CancelPendingAction));
        }

        let action_buttons: Element<Message> = action_buttons.into();
        let hand = self.view_hand(game);
        let hand_and_actions = row![hand, action_buttons].spacing(20).padding(10);
        let board = self.view_board(game);
        let board_container = container(board).height(Length::FillPortion(3)).width(Length::Fill).padding(5);
        let left_column = column![board_container, hand_and_actions]
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .spacing(5)
            .padding(5);
        let right_column = status_info.width(Length::FillPortion(1)).height(Length::Fill).padding(10);

        row![left_column, right_column].spacing(10).padding(10).height(Length::Fill).into()
    }

    fn build_game_variant(&self) -> Option<GameVariant> {
        match self.selected_variant? {
            GameVariantKind::TwoVsTwo => Some(GameVariant::TwoVsTwo),
            GameVariantKind::ThreeVsThree => Some(GameVariant::ThreeVsThree),
            GameVariantKind::TwoVsTwoVsTwo => Some(GameVariant::TwoVsTwoVsTwo),
            GameVariantKind::FreeForAll => {
                let players: usize = self.ffa_players_input.parse().ok()?;
                if players < 2 { return None; }
                Some(GameVariant::FreeForAll(players))
            }
        }
    }
}
