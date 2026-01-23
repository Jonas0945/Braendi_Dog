pub mod game;
pub mod ui;
//pub mod net;

//pub mod bin;
pub use game::game::{Game, DogGame};
pub use game::color::Color;
pub use game::piece::Piece;
pub use game::action::{Action, ActionKind};
use serde::{Deserialize, Serialize};
pub use ui::render;

use crate::game::GameVariant;

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

#[derive(Serialize, Deserialize)]
pub enum BeginGameMesage {
    ErstelleSpiel {variant: GameVariant, player_name: String},

    SpielBeitreten {player_name: String},
}