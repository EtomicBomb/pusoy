use ordered_float::OrderedFloat;

use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::f64::INFINITY;
use std::io;

use crate::card::{Card, Rank, THREE_OF_CLUBS};
use crate::game::SafeGameInterface;
use crate::play::finder::Finder;
use crate::play::{Play, PlayKind};

pub const N_PARAMETERS: usize = 10;

// the thing that decides which move to make.
// we are going to calculate all of the possible positions that we could get
// then, we are going to choose the one that generates the most favorable game position

pub trait Player {
    fn choose_play(&self, game: &SafeGameInterface) -> Play;
}

pub struct HumanPlayer;

impl Player for HumanPlayer {
    fn choose_play(&self, game: &SafeGameInterface) -> Play {
        loop {
            let mut your_hand = game.my_hand().to_vec();
            your_hand.sort();
            println!("your turn - {:?}", your_hand);
            let mut cards_string = String::new();
            io::stdin().read_line(&mut cards_string).unwrap();

            let cards: Vec<Card> = cards_string
                .split_whitespace()
                .map(|c| c.parse().unwrap())
                .collect();

            // try to play these cards
            match game.can_play(cards) {
                Ok(play) => {
                    // it worked
                    break play;
                }
                Err(e) => {
                    eprintln!("invalid turn: {:?}", e);
                    // we're gonna have to prompt the user again
                }
            }
        }
    }
}

pub struct MachinePlayer {
    constants: [f64; N_PARAMETERS],
}

impl MachinePlayer {
    pub fn new(constants: [f64; N_PARAMETERS]) -> MachinePlayer {
        MachinePlayer { constants }
    }
}

impl Player for MachinePlayer {
    fn choose_play(&self, game: &SafeGameInterface) -> Play {
        let hand = game.my_hand().to_vec();
        let n_cards = hand.len();
        let available_plays = Finder::new(hand).all_plays();

        // we can play any of these plays, or pass
        let state = State::new(&self.constants, game);
        let depth = min(6, n_cards);

        let mut first_plays_with_average_cost: HashMap<Play, (f64, f64)> =
            HashMap::with_capacity(available_plays.len());

        search(depth, available_plays, state, &mut |state: State| {
            // we want to
            let first_play = state.our_plays_so_far.first().unwrap().clone();

            let entry = first_plays_with_average_cost
                .entry(first_play)
                .or_insert((0.0, 0.0));
            entry.0 += state.total_cost;
            entry.1 += 1.0;
        });

        // find the best one that doesn't involve passing
        let best = first_plays_with_average_cost
            .iter()
            .min_by_key(|(_play, (total, count))| OrderedFloat(total / count))
            .unwrap()
            .0
            .cards()
            .to_vec();

        match game.can_play(best) {
            Ok(play) => play,
            Err(_) => {
                // we're gonna have to pass here
                match game.can_play(vec![]) {
                    Ok(pass) => pass,
                    Err(e) => unreachable!("{:?}", e),
                }
            }
        }
    }
}

fn search(
    card_depth: usize,
    available_plays: Vec<Play>,
    current_state: State,
    f: &mut impl FnMut(State),
) {
    // f gets called on all of the leaf notes from our search
    if card_depth == 0 {
        f(current_state)
    } else {
        for play in available_plays.iter() {
            let n_cards = play.cards().len();
            if card_depth < n_cards {
                continue;
            }

            // we want to construct a map with all of the plays available, excluding the ones that use the cards we just spent
            let mut plays_available_to_child = available_plays.clone();
            let cards_we_just_played: HashSet<&Card> = play.cards().iter().collect();
            plays_available_to_child.retain(|p| {
                // make sure that p doesn't overlap with any of the cards we just played
                p.cards().iter().all(|c| !cards_we_just_played.contains(c))
            });

            let child_state = current_state.next_state(play);

            search(
                card_depth - n_cards,
                plays_available_to_child,
                child_state,
                f,
            );
        }
    }
}

// describes the state of the game after a move has been played
#[derive(Clone)]
struct State<'a> {
    // this is the play that on our turn, we are looking to play on top of.
    // None if it is the first turn
    constants: &'a [f64; N_PARAMETERS],
    status: Status,
    total_cost: f64,
    our_plays_so_far: Vec<Play>,
    game_interface: &'a SafeGameInterface<'a>,
}

#[derive(Clone)]
enum Status {
    FirstTurnOfGame,
    FirstAnalysis(Play), // previous term
    Rest(Play),          // four terms before
}

impl<'a> State<'a> {
    fn new(
        constants: &'a [f64; N_PARAMETERS],
        game_interface: &'a SafeGameInterface<'a>,
    ) -> State<'a> {
        let status = match game_interface.get_play_on_table() {
            Some(play) => Status::FirstAnalysis(play.clone()),
            None => Status::FirstTurnOfGame,
        };

        State {
            constants,
            status,
            total_cost: 0.0,
            our_plays_so_far: vec![],
            game_interface,
        }
    }

    #[inline]
    fn next_state(&self, play: &Play) -> State {
        let mut new_state = self.clone();

        new_state.total_cost += match new_state.status {
            Status::FirstTurnOfGame => {
                if play.cards().contains(&THREE_OF_CLUBS) {
                    0.0 // we literally won't be able to pass
                } else {
                    INFINITY
                }
            }
            Status::FirstAnalysis(ref before) => {
                // we are trying to play directly on these cards
                if new_state
                    .game_interface
                    .can_play(play.cards().to_vec())
                    .is_ok()
                {
                    0.0
                } else {
                    // how many turns do we think it will take
                    // TODO: include numbers from research!
                    first_analysis_cost(before, play, self.constants)
                }
            }
            Status::Rest(ref four_turns_before) => {
                // TODO: include numbers from the research!
                cost(&four_turns_before, play, self.constants)
            }
        };

        // change the status going forward
        new_state.status = Status::Rest(play.clone());

        new_state.our_plays_so_far.push(play.clone());
        new_state
    }
}

fn first_analysis_cost(play1: &Play, play2: &Play, constants: &[f64; N_PARAMETERS]) -> f64 {
    assert!(!play1.is_pass() && !play2.is_pass());

    let play1_rank = play1.ranking_card().unwrap();
    let play2_rank = play2.ranking_card().unwrap();

    match play2.kind() {
        PlayKind::Pass => unimplemented!(),

        PlayKind::Single | PlayKind::Pair => {
            // how much higher is play2 than play1
            if would_get_control(play1) {
                constants[5]
            } else {
                // we'd better hope that it goes around and doesn't get higher than play2
                let gap = play2_rank.numeric_value() as isize - play1_rank.numeric_value() as isize;

                if gap > 20 {
                    // i guess we can say we are pretty good
                    constants[6]
                } else {
                    // just a big number, we will probably hand control over to someone else
                    // and hope that eventually we will get control bac
                    constants[7]
                }
            }
        }
        PlayKind::Strait | PlayKind::Flush => {
            // this is very dependant on what cards the other players have

            constants[8]
        }
        PlayKind::FullHouse => constants[9],
        PlayKind::FourOfAKind => 0.0, // so rare, doesn't even matter
        PlayKind::StraitFlush => 0.0,
    }
}

pub fn cost(play1: &Play, play2: &Play, constants: &[f64]) -> f64 {
    // returns an estimate of the number of times we would neet to pass to play `play2` on top of `play1`
    // we therefore want to keep the total cost for our game as low as possible

    // another good quantity to calculate would be the number of cards the other players are expected to play
    // versus the number of cards that we play

    // what other factors influence this model?
    // control
    // the cards that have already been played
    // the length of the hand thats on the top of the deck

    // problems with this model:
    // we ignore the goal of reducing the total number of cards we have
    // IDEA: instead of having depth be a cap on the number of plays, we should have depth be a cap on the number of cards played
    // then, the total cost of a sequence would be higher if it takes longer to shed cards

    // right now, we are just going to use a 'heuristic' approach

    // TODO: use numbers from research

    assert!(!play1.is_pass() && !play2.is_pass());

    let play1_rank = play1.ranking_card().unwrap();
    let play2_rank = play2.ranking_card().unwrap();

    match play2.kind() {
        PlayKind::Pass => unimplemented!(),

        PlayKind::Single | PlayKind::Pair => {
            // how much higher is play2 than play1
            if would_get_control(play1) {
                constants[0]
            } else {
                // we'd better hope that it goes around and doesn't get higher than play2
                let gap = play2_rank.numeric_value() as isize - play1_rank.numeric_value() as isize;

                if gap > 20 {
                    // i guess we can say we are pretty good
                    constants[1]
                } else {
                    // just a big number, we will probably hand control over to someone else
                    // and hope that eventually we will get control bac
                    constants[2]
                }
            }
        }
        PlayKind::Strait | PlayKind::Flush => {
            // this is very dependant on what cards the other players have

            constants[3]
        }
        PlayKind::FullHouse => constants[4],
        PlayKind::FourOfAKind => 0.0, // so rare, doesn't even matter
        PlayKind::StraitFlush => 0.0,
    }
}

fn would_get_control(play: &Play) -> bool {
    // figure out how high this play is relative to other plays

    // this is a terrible implementation

    // TODO: use data!
    match play.kind() {
        PlayKind::Pass => unimplemented!(),
        PlayKind::Single | PlayKind::Pair => {
            let rank = play.ranking_card().unwrap().rank;

            rank == Rank::Two || rank == Rank::Ace
        }
        PlayKind::Strait => false,
        PlayKind::Flush => false,
        PlayKind::FullHouse => true, // this is dummy stuff
        PlayKind::FourOfAKind => true,
        PlayKind::StraitFlush => true,
    }
}
