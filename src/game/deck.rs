use rand::seq::SliceRandom;
use rand::rng;
use serde::{Serialize, Deserialize};

use super::card::Card;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::new();
        for _ in 0..2 {
            cards.extend(Self::one_deck());
        }

        let mut deck = Deck { cards };
        deck.shuffle();
        deck
    }

    fn one_deck() -> Vec<Card> {
        use Card::*;

        let ranks = [
            Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten,
            Jack, Queen, King,
        ];

        let mut deck = Vec::new();
        for r in ranks {
            for _ in 0..4 {
                deck.push(r);
            }
        }
        deck.push(Card::Joker);
        deck.push(Card::Joker);

        deck
    }

    pub fn draw(& mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn replenish(&mut self, discard: &mut Vec<Card>) {
        self.cards.append(discard);
        discard.clear();
        self.shuffle();
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut rng());
    }

    pub fn from_cards(cards: Vec<Card>) -> Self {
        Deck { cards }
    }
}

#[cfg(test)]
mod tests {
    use super::Deck;
    use crate::game::card::Card;

    #[test]
    fn test_one_deck_creation() {
        use Card::*;

        let deck = Deck::one_deck();

        assert_eq!(deck.len(), 54);

        let ranks = [
            Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten,
            Jack, Queen, King,
        ];

        for r in ranks {
            assert_eq!(deck.iter().filter(|&&c| c == r).count(), 4);
        }

        assert_eq!(deck.iter().filter(|&&c| c == Joker).count(), 2)
    }

    #[test]
    fn test_complete_deck_creation() {
        let deck = Deck::new();
        assert_eq!(deck.len(), 108); // 2 decks of 54 cards each
    }

    #[test]
    fn test_draw() {
        let mut deck = Deck::from_cards(vec![Card::Ace, Card::King]);

        assert_eq!(deck.draw(), Some(Card::King));
        assert_eq!(deck.draw(), Some(Card::Ace));
        assert_eq!(deck.draw(), None);
    }

    #[test]
    fn test_replenish() {
        let mut deck = Deck::from_cards(vec![]);
        let mut discard = vec![Card::Ace, Card::Two];

        deck.replenish(&mut discard);

        assert_eq!(deck.len(), 2);
        assert!(discard.is_empty());
    }

    #[test]
    fn test_shuffle_changes_order() {
        let mut deck = Deck::new();

        let before = deck.clone();
        deck.shuffle();
        let after = deck.clone();

        assert_ne!(before, after, "Shuffle should probably change order sometimes");
    }

}