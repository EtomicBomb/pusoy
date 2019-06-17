use serde::{Serialize, Deserialize};

use std::fmt;
use std::str::FromStr;

use self::Rank::*;
use self::Suit::*;


pub const ALL_RANKS: [Rank; 13] = [Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace, Two];

pub const ALL_SUITS: [Suit; 4] = [Clubs, Spades, Hearts, Diamonds];

pub const THREE_OF_CLUBS: Card = Card {
    rank: Three,
    suit: Clubs,
};


pub fn entire_deck() -> Vec<Card> {
    let mut cards = Vec::with_capacity(52);

    for &rank in ALL_RANKS.iter() {
        for &suit in ALL_SUITS.iter() {
            cards.push(Card { rank, suit });
        }
    }

    cards
}


// note: ord impl compares rank first, then suit
// this is how cards are ranked in pusoy
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub fn numeric_value(self) -> usize {
        // value between 0 and 51
        (4*self.rank as usize) + self.suit as usize
    }

}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self) // lets just use the Display implementation
    }
}

impl FromStr for Card {
    type Err = ();

    fn from_str(s: &str) -> Result<Card, ()> {
        let first_char = s.chars().nth(0).unwrap().to_string();
        let second_char = s.chars().nth(1).unwrap().to_string();

        let rank = first_char.parse()?;
        let suit = second_char.parse()?;

        Ok(Card { rank, suit })
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum Rank {
    Three = 0,
    Four = 1,
    Five = 2,
    Six = 3,
    Seven = 4,
    Eight = 5,
    Nine = 6,
    Ten = 7,
    Jack = 8,
    Queen = 9,
    King = 10,
    Ace = 11,
    Two = 12,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Three => "3",
            Four => "4",
            Five => "5",
            Six => "6",
            Seven => "7",
            Eight => "8",
            Nine => "9",
            Ten => "T",
            Jack => "J",
            Queen => "Q",
            King => "K",
            Ace => "A",
            Two => "2",
        })
    }
}

impl fmt::Debug for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self) // lets just use the Display implementation
    }
}

impl FromStr for Rank {
    type Err = ();

    fn from_str(s: &str) -> Result<Rank, ()> {
        Ok(match s.trim() {
            "3" => Three,
            "4" => Four,
            "5" => Five,
            "6" => Six,
            "7" => Seven,
            "8" => Eight,
            "9" => Nine,
            "T" => Ten,
            "J" => Jack,
            "Q" => Queen,
            "K" => King,
            "A" => Ace,
            "2" => Two,
            _ => return Err(()),
        })
    }
}


#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum Suit {
    Clubs = 0,
    Spades = 1,
    Hearts = 2,
    Diamonds = 3,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Clubs => "♣",
            Spades => "♠",
            Hearts => "♥",
            Diamonds => "♦",
        })
    }
}

impl fmt::Debug for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self) // lets just use the Display implementation
    }
}

impl FromStr for Suit {
    type Err = ();

    fn from_str(s: &str) -> Result<Suit, ()> {
        Ok(match s.trim() {
            "♣" | "C" => Clubs,
            "♠" | "S" => Spades,
            "♥" | "H" => Hearts,
            "♦" | "D" => Diamonds,
            _ => return Err(()),
        })
    }
}
