use crate::play::{Play, PlayKind};

use crate::card::Card;
use std::collections::HashSet;
use crate::util::{counter, concat};

fn rank_blocks(cards: &[Card]) -> [Vec<Card>; 13] {
    let mut blocks: [Vec<Card>; 13] = Default::default();

    for &card in cards {
        blocks[card.rank as usize].push(card);
    }

    blocks
}

fn flushes(cards: &[Card]) -> Vec<Play> {
    // collect all of the cards
    let mut suit_blocks: [Vec<Card>; 4] = Default::default();

    for &card in cards {
        suit_blocks[card.suit as usize].push(card);
    }

    // now, we just need to compute all of the flushes
    let mut chunks = Vec::new();

    for block in suit_blocks.iter() {
        if block.len() < 5 {
            continue;
        }

        chunks.extend(
            permute(block, 5)
                .into_iter()
                .map(|cs| Play::new(PlayKind::Flush, max_card(&cs), cs)),
        );
    }

    chunks
}

// i will probably refactor this to be optimized for the types of queries we are giving finder
// currently, we want to support the operations
//      all_plays
//      infer_from_cards - we only want to return the type of hand corresponding to the number of cards we give

#[derive(Clone, Debug)]
pub struct Finder {
    pub cards: Vec<Card>,
    pub rank_blocks: [Vec<Card>; 13],
    flushes: Vec<Play>, // we store the flushes, because a `suit_blocks` data structure would be useless for anything else
}

impl Finder {
    pub fn new(mut cards: Vec<Card>) -> Finder {
        cards.sort();

        let rank_blocks = rank_blocks(&cards);
        let flushes = flushes(&cards);

        Finder {
            cards,
            rank_blocks,
            flushes,
        }
    }

    pub fn all_plays(&self) -> Vec<Play> {
        let mut plays = Vec::new();

        // singles
        plays.append(&mut self.singles());

        // pairs
        plays.append(&mut self.pairs());

        // five card hands
        plays.append(&mut self.strait_flushes());
        plays.append(&mut self.four_of_a_kinds());
        plays.append(&mut self.full_houses());
        plays.append(&mut self.flushes());
        plays.append(&mut self.straits());

        plays
    }

    pub fn infer(&self) -> Option<Play> {
        Some(match self.cards.len() {
            0 => Play { kind: PlayKind::Pass, cards: vec![], ranking_card: None },
            1 => Play::new(PlayKind::Single, self.cards[0], self.cards.clone()),
            2 => Play::new(PlayKind::Pair, self.cards[1], if self.cards[0].rank == self.cards[1].rank { self.cards.clone() } else { return None }, ),
            5 => self.max_five_of_a_kind()?,
            _ => return None,
        })
    }

    pub fn max_five_of_a_kind(&self) -> Option<Play> {
        let strait_flushes = self.strait_flushes();
        if !strait_flushes.is_empty() {
            return strait_flushes.iter().max().cloned();
        }

        let four_of_a_kinds = self.four_of_a_kinds();
        if !four_of_a_kinds.is_empty() {
            return four_of_a_kinds.iter().max().cloned();
        }
        
        let full_houses = self.full_houses();
        if !full_houses.is_empty() {
            return full_houses.iter().max().cloned();
        }

        let flushes = self.flushes();
        if !flushes.is_empty() {
            return flushes.iter().max().cloned();
        }

        let straits = self.straits();
        if !straits.is_empty() {
            return straits.iter().max().cloned();
        }

        None
    }

    pub fn singles(&self) -> Vec<Play> {
        self.cards
            .iter()
            .map(|&c| Play::new(PlayKind::Single, c, vec![c]))
            .collect()
    }


    pub fn n_of_a_kinds(&self, n: usize) -> Vec<Vec<Card>> {
        let mut chunks = Vec::new();

        for block in self.rank_blocks.iter() {
            if block.len() < n {
                continue;
            } // this block is useless to us
            chunks.append(&mut permute(block, n));
        }

        chunks
    }

    pub fn strait_flushes(&self) -> Vec<Play> {
        let mut strait_flushes = Vec::new();

        for mut strait in self.straits() {
            strait.replace_kind(PlayKind::StraitFlush);
            if are_flush(strait.cards()) {
                strait_flushes.push(strait);
            }
        }

        strait_flushes
    }

    pub fn flushes(&self) -> Vec<Play> {
        self.flushes.clone()
    }

    pub fn four_of_a_kinds(&self) -> Vec<Play> {
        // in pusoy, the four of a kind is played with a trash card

        let mut four_of_a_kinds = Vec::new();

        for four_of_a_kind in self.n_of_a_kinds(4) {
            for card in self.cards.iter() {
                if !four_of_a_kind.contains(card) {
                    let mut collection = four_of_a_kind.clone();
                    collection.push(*card);
                    let play = Play::new(PlayKind::FourOfAKind, four_of_a_kind[3], collection);
                    four_of_a_kinds.push(play);
                }
            }
        }

        four_of_a_kinds
    }

    pub fn pairs(&self) -> Vec<Play> {
        self.n_of_a_kinds(2)
            .into_iter()
            .map(|cs| Play::new(PlayKind::Pair, cs[1], cs))
            .collect()
    }

    pub fn full_houses(&self) -> Vec<Play> {
        let mut full_houses = Vec::new();
        let pairs = self.n_of_a_kinds(2);

        for three_of_a_kind in self.n_of_a_kinds(3) {
            for pair in pairs.iter() {
                if !do_overlap(&three_of_a_kind, pair) {
                    let collection = concat(three_of_a_kind.clone(), pair.clone());
                    let play = Play::new(PlayKind::FullHouse, three_of_a_kind[2], collection);
                    full_houses.push(play);
                }
            }
        }

        full_houses
    }

    pub fn straits(&self) -> Vec<Play> {
        let mut straits = Vec::new();

        let blocks_starting_at = |i: usize| (i..i + 5).map(|i| &self.rank_blocks[i % 13]);

        for start_index in 0..13 {
            strait_from_blocks(blocks_starting_at(start_index), &mut straits);
        }

        straits
    }
}

fn strait_from_blocks<'a>(
    blocks: impl Clone + Iterator<Item = &'a Vec<Card>> + 'a,
    straits: &mut Vec<Play>,
) {
    let base: Vec<usize> = blocks.clone().map(|b| b.len()).collect();

    let f = |x: &[usize]| {
        let entry: Vec<Card> = blocks
            .clone()
            .zip(x.iter())
            .map(|(block, &i)| block[i])
            .collect();

        let play = Play::new(PlayKind::Strait, max_card(&entry), entry);
        straits.push(play);
    };

    counter(&base, f);
}

pub fn max_card(cards: &[Card]) -> Card {
    *cards.iter().max_by(|a, b| a.cmp(b)).unwrap()
}


fn do_overlap(cards: &[Card], other: &[Card]) -> bool {
    // check if other and cards contain any of the same cards

    let other_set: HashSet<Card> = other.iter().cloned().collect();

    for card in cards {
        if other_set.contains(card) {
            return true;
        }
    }

    false
}

fn are_flush(cards: &[Card]) -> bool {
    let first_suit = cards[0].suit;

    cards.iter().skip(1).all(|c| c.suit == first_suit)
}

fn permute<T: Clone>(list: &[T], n: usize) -> Vec<Vec<T>> {
    assert!(list.len() >= n);
    let mut ret = Vec::new();

    if list.len() == n {
        ret.push(list.to_vec());
    } else if n == 1 {
        ret.extend(list.iter().map(|i| vec![i.clone()]));
    } else {
        for i in 0..=list.len() - n {
            let results = permute(&list[i + 1..], n - 1);

            for mut r in results {
                r.insert(0, list[i].clone());
                //r.push(list[i].clone());
                ret.push(r);
            }
        }
    }

    ret
}
