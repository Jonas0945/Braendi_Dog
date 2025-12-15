

use super::piece::*;
use super::action::*;
use super::color::*;
use super::deck::*;
use super::card::*;
use super::player::*;

const CARDS_PER_ROUND: [u8;4] = [5,4,3,2];

pub struct Game {
    board: [Option<Piece>; 80],
    history: Vec<Action>,
    round: u8,

    deck: Deck,
    discard: Vec<Card>,

    red: Player,
    green: Player,
    blue: Player,
    yellow: Player,

    current_player_color: Color,
    swapping_phase: bool,
    swap_buffer: Vec<(Player, Card)>
}


pub trait DogGame {
    // Creates new instance with an empty board and initialized deck and players
    fn new() -> Self;

    // Returns the current state of the board
    fn board_state(&self) -> &[Option<Piece>; 80];

    // Returns the current player
    fn current_player(&self) -> &Player;

    // Matches and applies the action of playing the given card for the current player
    fn action(&mut self, action: Action) -> Result<(), &'static str>;

    // Undoes the last action
    fn undo(&mut self) -> Result<(), &'static str>;

    // Returns the current state of the board
    fn board(&self) -> &[Option<Piece>; 80];

    // Gives players new cards and lets theem swap one card
    fn new_round(&mut self);
    
    // Is called by new_round() and swaps two cards in between team members
    fn swap_cards(&mut self)-> &mut Self; 

    // Checks if there is yet a winning team
    fn is_winner(&self) -> bool;
}

impl DogGame for Game {
    fn new() -> Self {
        Self {
            board: [None; 80],
            history: Vec::new(),
            round: 0,

            deck: Deck::new(),
            discard: Vec::new(),

            red: Player::new(Color::Red),
            green: Player::new(Color::Green),
            blue: Player::new(Color::Blue),
            yellow: Player::new(Color::Yellow),

            current_player_color: Color::Red,
            swapping_phase: true,
            swap_buffer: Vec::new(),
        }
    }

    fn current_player(&self) -> &Player {
        match self.current_player_color {
            Color::Red => &self.red,
            Color::Green => &self.green,
            Color::Blue => &self.blue,
            Color::Yellow => &self.yellow,
        }
    }
    
    fn board_state(&self) -> &[Option<Piece>; 80] {
        &self.board
    }

    fn action(&mut self,  action: Action) -> Result<(), &'static str> {
        match action.action{
            ActionKind::Place => todo!(),
            ActionKind::Move(_, _) => todo!(),
            ActionKind::Switch(_, _) => todo!(),
            ActionKind::Swap(card_index) => {
                let playercolor = action.player.color;
                let swapping_player;
                match playercolor {
                    Color::Red => swapping_player = &self.red   ,
                    Color::Green => swapping_player = &self.green,
                    Color::Blue => swapping_player = &self.blue,
                    Color::Yellow => swapping_player = &self.yellow,
                }
                if swapping_player.swapped_cards_count == self.round{
                    if self.swapping_phase{
                        if self.swap_buffer.iter().any(|(p, _)| p == swapping_player){
                            return Err("Es darf pro Spieler nur eine Karte getauscht werden")
                        }
                        if card_index >= swapping_player.cards.len() {
                            return Err("Ungültiger Kartenindex für den Tausch")
                        }

                        self.swap_buffer.push((swapping_player.clone(), swapping_player.cards.get(card_index).unwrap().clone()));
                        
                        match playercolor {
                            Color::Red => {self.red.cards.remove(card_index); self.red.swapped_cards_count +=1;},
                            Color::Green => {self.green.cards.remove(card_index); self.green.swapped_cards_count +=1;},
                            Color::Blue => {self.blue.cards.remove(card_index); self.blue.swapped_cards_count +=1;},
                            Color::Yellow => {self.yellow.cards.remove(card_index); self.yellow.swapped_cards_count +=1;},
}
                        if self.swap_buffer.len()==4 {
                                for (p, c) in self.swap_buffer.drain(..){
                                    match p.teammate() {
                                        Color::Red => self.red.cards.push(c),
                                        Color::Green => self.green.cards.push(c),
                                        Color::Blue => self.blue.cards.push(c),
                                        Color::Yellow => self.yellow.cards.push(c),
                                    }
                                }  
                            self.swapping_phase = false;
                            
                            return Ok(())
                        }
                    }else {
                        return  Err("In dieser Phase des Spiels dürfen keine Karten getauscht werden");
                    }
                } else  {return  Err("Dieser Spieler darf keine Karte tauschen") };
            Ok(())},
        }
    }
    
    fn undo(&mut self) -> Result<(), &'static str> {
        todo!()
    }
    
    fn board(&self) -> &[Option<Piece>; 80] {
        todo!()
    }
    
    fn new_round(&mut self) {
        let current_round = (self.round % 4) as usize;
        let cards_to_deal = CARDS_PER_ROUND[current_round];
        if self.deck.len() <= (cards_to_deal as usize * 4 ){
            self.deck.replenish(&mut self.discard);
        }

        for _ in 0..cards_to_deal {
            self.red.cards.push(self.deck.draw().unwrap());
            self.green.cards.push(self.deck.draw().unwrap());
            self.blue.cards.push(self.deck.draw().unwrap());
            self.yellow.cards.push(self.deck.draw().unwrap());
        }

        
        self.round += 1;
    }
    
    fn swap_cards(&mut self)-> &mut Self {
        todo!()
    }
    
    fn is_winner(&self) -> bool {
        todo!()
    }
}

// Game Logik: 
// 4 Spieler je 5-2 Karten
// Tauschen 1 Karte 
// Spielt Karten aus
// Musst alle Karten ausspielen
// Wenn Legen nicht möglich, alle Karten ablegen


   /* let mut game =Game::new();
    game.new_round();s
    let a1=Action{player: game.red.clone(),card:Card::Ace, action: ActionKind::Swap(0)};
    game.red.cards.push(Card::Ace);
    game.action(a1);
    game.swap_buffer.push((game.blue.clone(),Card::Five));
        game.swap_buffer.push((game.yellow.clone(),Card::Joker));

            game.swap_buffer.push((game.green.clone(), Card::Four));

    assert!(game.blue.cards[0]==Card::Ace);*/
  #[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_successful() {
        let mut game = Game::new();
        
        // new_round() aufrufen, um Karten zu verteilen (5 pro Spieler in Runde 0)
        game.new_round();
        
        // Round zurücksetzen, damit Swapping in "Runde 0" funktioniert (swapped_cards_count == 0 == round)
        game.round = 0;
        
        // Stelle sicher, dass Swapping-Phase aktiv ist (sollte von new() kommen)
        assert!(game.swapping_phase);
        
        // Swap für roten Spieler (verwende die erste Karte aus der Hand)
        let red_card = game.red.cards[0].clone();  // Echte Karte aus der Hand
        let a1 = Action {
            player: game.red.clone(),
            card: red_card,  // Verwende die echte Karte
            action: ActionKind::Swap(0),
        };
        game.action(a1).unwrap();
        
        // Swap für grünen Spieler
        let green_card = game.green.cards[0].clone();
        let a2 = Action {
            player: game.green.clone(),
            card: green_card,
            action: ActionKind::Swap(0),
        };
        game.action(a2).unwrap();
        
        // Swap für blauen Spieler
        let blue_card = game.blue.cards[0].clone();
        let a3 = Action {
            player: game.blue.clone(),
            card: blue_card,
            action: ActionKind::Swap(0),
        };
        game.action(a3).unwrap();
        
        // Swap für gelben Spieler – triggert Verteilung
        let yellow_card = game.yellow.cards[0].clone();
        let a4 = Action {
            player: game.yellow.clone(),
            card: yellow_card,
            action: ActionKind::Swap(0),
        };
        game.action(a4).unwrap();
        
        // Prüfe Verteilung an Teammates (verwende die echten Karten)
        assert!(game.blue.cards.contains(&red_card));     // Red -> Blue
        assert!(game.yellow.cards.contains(&green_card)); // Green -> Yellow
        assert!(game.red.cards.contains(&blue_card));     // Blue -> Red
        assert!(game.green.cards.contains(&yellow_card)); // Yellow -> Green
        
        // Buffer geleert, Phase beendet
        assert_eq!(game.swap_buffer.len(), 0);
        assert!(!game.swapping_phase);
    }
}
