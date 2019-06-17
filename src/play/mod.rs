use serde::{Deserialize, Serialize};

pub mod finder;
use finder::Finder;

use crate::card::Card;
use std::cmp::Ordering;



#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Deserialize, Serialize)]
pub enum PlayKind {
    Pass,
    Single,
    Pair,

    Strait,
    Flush,
    FullHouse,
    FourOfAKind,
    StraitFlush,
}

impl PlayKind {
    fn len(self) -> usize {
        match self {
            PlayKind::Pass => 0,
            PlayKind::Single => 1,
            PlayKind::Pair => 2,
            PlayKind::Strait => 5,
            PlayKind::Flush => 5,
            PlayKind::FullHouse => 5,
            PlayKind::FourOfAKind => 5,
            PlayKind::StraitFlush => 5,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Deserialize, Serialize)]
pub struct Play {
    cards: Vec<Card>,
    kind: PlayKind,
    ranking_card: Option<Card>,
}

impl Ord for Play {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_pass() { unimplemented!() }

        self.kind.cmp(&other.kind)
            .then_with(|| self.ranking_card.unwrap().cmp(&other.ranking_card.unwrap()))
    }
}

impl PartialOrd for Play {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}


impl Play {
    pub fn new(kind: PlayKind, ranking_card: Card, cards: Vec<Card>) -> Play {
        Play {
            cards,
            kind,
            ranking_card: Some(ranking_card),
        }
    }

    pub fn is_pass(&self) -> bool {
        self.kind == PlayKind::Pass
    }

    pub fn len_eq(&self, other: &Self) -> bool {
        self.kind.len() == other.kind.len()
    }

    pub fn can_play_on(&self, other: &Self) -> bool {
        if self.is_pass() { return true }

        if self.kind.len() != other.kind.len() {
            false
        } else if self.kind != other.kind {
            self.kind > other.kind
        } else {
            self.ranking_card.unwrap() > other.ranking_card.unwrap()
        }
    }


    pub fn infer_from_cards(cards: Vec<Card>) -> Option<Play> {
        let finder = Finder::new(cards);
        finder.infer()
    }

    pub fn kind(&self) -> PlayKind {
        self.kind
    }

    pub fn ranking_card(&self) -> Option<Card> {
        self.ranking_card
    }

    pub fn into_cards(self) -> Vec<Card> {
        self.cards
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn replace_kind(&mut self, kind: PlayKind) {
        self.kind = kind;
    }
}

