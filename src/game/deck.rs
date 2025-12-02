use rand::seq::SliceRandom;
use rand::rng;

use crate::card::Card;

#[derive(Clone, PartialEq, Eq, Debug)]
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

}