use crate::play::Play;

use crate::card::{Card, THREE_OF_CLUBS};


// MOTIVATION: if HumanPlayer or MachinePlayer had access to the regular GameState object,
// they could call .hands and other info that would just be cheating. This struct only gives
// access to data that isn't cheating

pub struct SafeGameInterface<'a> {
    inner: &'a GameState,
}

impl<'a> SafeGameInterface<'a> {
    pub fn from_game(game: &GameState) -> SafeGameInterface {
        SafeGameInterface {
            inner: game,
        }
    }

    pub fn can_play(&self, cards: Vec<Card>) -> Result<Play, GameError> {
        self.inner.can_play(cards)
    }

    pub fn my_hand(&self) -> &[Card] {
        self.inner.my_hand()
    }

    pub fn get_play_on_table(&self) -> Option<&Play> {
        self.inner.get_play_on_table()
    }
}


#[derive(Debug)]
pub struct GameState {
    pub hands: Vec<Vec<Card>>,
    pub current_player: usize,
    cards_down: Option<Play>,
    turn_index: usize, // need to store because on first turn, must play a hand with three of clubs
    last_player_to_not_pass: usize,
    n_players: usize,
    winning_player: Option<usize>,
    record: Vec<Play>, // all of the plays that happened in the game in order, for analysis
}

impl GameState {
    pub fn new(n_players: usize, cards: Vec<Card>) -> GameState {
        let hands = deal(&cards, n_players);

        // figure out who has the three of clubs
        let mut player_who_starts = None;
        for (i, hand) in hands.iter().enumerate() {
            if hand.contains(&THREE_OF_CLUBS) {
                player_who_starts = Some(i);
                break;
            }
        }
        let player_who_starts = player_who_starts.expect("Supplied deck didn't contain the 3♣");

        GameState {
            hands,
            current_player: player_who_starts,
            cards_down: None,
            turn_index: 0,
            last_player_to_not_pass: player_who_starts,
            n_players,
            winning_player: None,
            record: Vec::new(),
        }
    }

    pub fn can_play(&self, cards: Vec<Card>) -> Result<Play, GameError> {
        let play = Play::infer_from_cards(cards).ok_or(GameError::PlayDoesntExist)?;

        // make sure the move we are trying to do is legal
        if self.is_first_turn() {
            if  !play.cards().contains(&THREE_OF_CLUBS) {
                // the only requirement on the first move is that they play the three of clubs somehow
                return Err(GameError::IsntPlayingThreeOfClubs);
            }

        } else if self.have_control() {
            // if we have control, we can pretty much do anything except passing
            if play.is_pass() {
                return Err(GameError::CannotPass);
            }

        } else if !play.is_pass() {
            // here, we have our standard conditions, where we are not passing, and we don't have control

            // since we don't have control, we have to make sure they are making a valid play in the context
            // of the cards that they are trying to play on.
            let cards_down = self.cards_down.as_ref().unwrap();

            // this is the problem
            if !play.len_eq(cards_down) {
                return Err(GameError::WrongLength);
            }

            if !play.can_play_on(cards_down) {
                return Err(GameError::TooLow);
            }
        } // we don't have to list out the condition where we don't have control are passing, because this is always legal

        // make sure we have all the cards in that play
        for card in play.cards() {
            if !self.my_hand().contains(card) {
                return Err(GameError::DontHaveCard);
            }
        }

        Ok(play)
    }

    pub fn play(&mut self, play: Play) {
        // assumes that play is_legal

        subtract_cards(&mut self.hands[self.current_player], play.cards()).unwrap();

        self.record.push(play.clone());

        if self.hands[self.current_player].is_empty() {
            self.winning_player = Some(self.current_player);
        }

        if !play.is_pass() {
            self.last_player_to_not_pass = self.current_player;
            self.cards_down = Some(play); // if we are passing, the card that the next person has to play on doesn't change
        }

        self.turn_index += 1;
        self.current_player = (self.current_player+1)%self.n_players;
    }

    pub fn have_control(&self) -> bool {
        self.last_player_to_not_pass == self.current_player
    }

    pub fn get_record(&self) -> &[Play] {
        &self.record
    }

    pub fn winning_player(&self) -> Option<usize> {
        self.winning_player
    }

    pub fn get_play_on_table(&self) -> Option<&Play> {
        self.cards_down.as_ref()
    }

    pub fn my_hand(&self) -> &[Card] {
        &self.hands[self.current_player]
    }

    pub fn cards_down(&self) -> Option<&[Card]> {
        self.cards_down.as_ref().map(move |x| x.cards())
    }

    pub fn is_first_turn(&self) -> bool {
        self.turn_index == 0
    }
}

#[derive(Debug)]
pub enum GameError {
    DontHaveCard,
    IsntPlayingThreeOfClubs,
    TooLow,
    WrongLength,
    CannotPass,
    PlayDoesntExist,
}

fn subtract_cards(cards: &mut Vec<Card>, to_remove: &[Card]) -> Option<()> {
    for card_to_remove in to_remove {
        // just the guts of cards.remove_item
        let pos = cards.iter().position(|x| x == card_to_remove).unwrap();
        cards.remove(pos);
    }

    Some(())
}

pub fn deal(cards: &[Card], n_groups: usize) -> Vec<Vec<Card>> {
    // should shuffle these cards before calling this function

    assert_eq!(cards.len() % n_groups, 0, "Cannot deal out the cards evenly");

    let cards_per_group = cards.len() / n_groups;

    let mut groups = vec![Vec::with_capacity(cards_per_group); n_groups];

    for (i, &card) in cards.iter().enumerate() {
        let group_to_add = i % n_groups;

        groups[group_to_add].push(card);
    }

    groups
}