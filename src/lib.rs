pub mod game;
pub mod ui;
//pub mod net;

//pub mod bin;
pub use game::game::{Game, DogGame};
pub use game::color::Color;
pub use game::piece::Piece;
pub use game::action::{Action, ActionKind};
pub use ui::render;

//Aktionen, die SPieler versuchen können
pub enum ClientNachricht {
    beitritt,
    make_play,
    quit
}
//informiert Client um Gui zu aktualisieren
pub enum ServerNachrich{
    gamestate,
    error,
    spieler_beigetreten
}