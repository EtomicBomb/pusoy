use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

mod card;

fn main() {
    println!("Hello, world!");
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    fn rank_first_cmp(&self, other: &Card) -> Ordering {
        self.rank.cmp(&other.rank)
            .then_with(|| self.suit.cmp(&other.suit))
    }

    fn suit_first_cmp(&self, other: &Card) -> Ordering {
        self.suit.cmp(&other.suit)
            .then_with(|| self.rank.cmp(&other.rank))
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum Rank {
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
    Two,
}

impl FromStr for Rank {
    type Err = ();

    fn from_str(s: &str) -> Result<Rank, ()> {
        use crate::Rank::*;

        Ok(match s.trim() {
            "3" => Three,
            "4" => Four,
            "5" => Five,
            "6" => Six,
            "7" => Seven,
            "8" => Eight,
            "9" => Nine,
            "10" | "T" => Ten,
            "J" => Jack,
            "Q" => Queen,
            "K" => King,
            "A" => Ace,
            "2" => Two,
            _ => return Err(()),
        })
    }
}

impl fmt::Debug for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::Rank::*;

        f.write_str(match self {
                Three => " 3",
                Four => " 4",
                Five => " 5",
                Six => " 6",
                Seven => " 7",
                Eight => " 8",
                Nine => " 9",
                Ten => "T",
                Jack => " J",
                Queen => " Q",
                King => " K",
                Ace => " A",
                Two => " 2",
        })
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum Suit {
    Clubs,
    Spades,
    Hearts,
    Diamonds,
}

impl fmt::Debug for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::Suit::*;

        f.write_str(match self {
                Suit::Clubs => "♣",
                Suit::Spades => "♠",
                Suit::Hearts => "♥",
                Suit::Diamonds => "♦",
        })
    }
}

impl FromStr for Suit {
    type Err = ();

    fn from_str(s: &str) -> Result<Suit, ()> {
        use crate::Suit::*;

        Ok(match s.trim() {
            "♣" | "C" => Clubs,
            "♠" | "S" => Spades,
            "♥" | "H" => Hearts,
            "♦" | "D" => Diamonds,
            _ => return Err(()),
        })
    }
}
