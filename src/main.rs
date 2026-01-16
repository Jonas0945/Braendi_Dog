// src/main.rs

// Wir binden das GUI-Modul ein (das wir gleich erstellen)
pub mod gui;

fn main() -> iced::Result {
    // Startet die GUI aus der Datei gui.rs
    gui::launch()
}
