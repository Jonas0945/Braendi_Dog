// src/lib.rs
pub mod ai;
pub mod game;
pub mod net;
pub mod ui;
pub use net::*;
//pub mod bin;
pub use game::action::{Action, ActionKind};
pub use game::card::Card;
pub use game::color::Color;
pub use game::game::GameVariant;
pub use game::game::{DogGame, Game};
pub use game::piece::Piece;
use serde::{Deserialize, Serialize};
pub use ui::render;

//Aktionen, die SPieler versuchen können
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientNachricht {
    beitritt,
    make_play,
    quit,
}
//informiert Client um Gui zu aktualisieren
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "data")]
pub enum ServerNachrich{
    Fehler(String),
    State(Game),
    Welcome(usize), 
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BeginGameMesage {
    ErstelleSpiel {
        variant: GameVariant,
        player_name: String,
    },

    SpielBeitreten {
        player_name: String,
    },
}
