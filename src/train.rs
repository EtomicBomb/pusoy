use rand::{thread_rng, Rng};
use abc::{Candidate};
use rand::seq::SliceRandom;

use std::thread;
use std::sync::{Arc, Mutex};

use crate::bot::N_PARAMETERS;
use crate::bot::{Player, MachinePlayer, cost};
use crate::game::{SafeGameInterface, GameState};
use crate::play::Play;
use crate::card::entire_deck;

const FITNESS_GAMES_TO_CHECK: usize = 5;
const GAMES_PER_CPU: usize = 1;

pub const DEFAULT_PARAMETERS: [f64; N_PARAMETERS] = [0.0, 1.0, 10.0, 5.0, 0.5, 0.0, 0.0, 3.0, 0.0, 0.0];
const BEE_COUNT: usize = 10;
const N_ABC_ROUNDS: usize = 10;


/// Performs one training step, attempting to improve the ability of the parameters
/// Collects experimental data from real matches, and then improves the cost function
pub fn training_step(current_parameters: [f64; N_PARAMETERS]) -> ([f64; N_PARAMETERS], f64) {
    // how many times does it win against our base model?

    // we are trying to get experimental data, in order to get an approximation function
    //let mut experimental_data = Vec::new();

    let cpu_count = num_cpus::get();
    

    let mut threads = Vec::new();
    let experimental_data = Arc::new(Mutex::new(Vec::new()));

    for _ in 0..GAMES_PER_CPU {
        for _ in 0..cpu_count {
            let local_parameters = current_parameters.clone();
            let local_data = Arc::clone(&experimental_data);

            let handle = thread::spawn(move || {
                let mut to_add = get_data_from_one_game(local_parameters);

                println!("computed game");
                local_data.lock().unwrap().append(&mut to_add);
            });

            threads.push(handle);
        }
    }

    for handle in threads {
        handle.join().unwrap();
    }
    let experimental_data = experimental_data.lock().unwrap().clone();


    // lets do some actual optimization
    let solution_finder = SolutionFinder { experimental_data };

    let optimizer = abc::HiveBuilder::new(solution_finder, BEE_COUNT).build().unwrap();

    let solution_candidate = optimizer.run_for_rounds(N_ABC_ROUNDS).unwrap();
    let loss = solution_candidate.fitness.recip();
    let solution_vec = solution_candidate.solution;

    let mut solution_array = [0.0; N_PARAMETERS];
    for (i, elem) in solution_vec.into_iter().enumerate() {
        solution_array[i] = elem;
    }

    (solution_array, loss)
}


struct SolutionFinder {
    experimental_data: Vec<(Play, Play, usize)>,
}

impl abc::Context for SolutionFinder {
    type Solution = Vec<f64>;

    fn make(&self) -> Self::Solution {
        let mut vec = Vec::with_capacity(N_PARAMETERS);

        for _ in 0..N_PARAMETERS {
            let n = thread_rng().gen_range(-10.0, 10.0);
            vec.push(n);
        }

        vec
    }

    fn evaluate_fitness(&self, solution: &Self::Solution) -> f64 {
        // figure out mean squared error
        let mut total_error = 0.0;
        let len = self.experimental_data.len() as f64;

        for &(ref play1, ref play2, actual_cost) in self.experimental_data.iter() {
            let estimated_cost = cost(play1, play2, solution);

            let diff = actual_cost as f64 - estimated_cost;
            total_error += diff*diff;
        }

        let mean_squared_error = total_error / len;

        // since this is being maximized, lets use a decreasing function
        mean_squared_error.recip()
    }

    fn explore(&self, field: &[Candidate<Self::Solution>], index: usize) -> Self::Solution {
        let mut rng = thread_rng();
        let mut to_modify = field[index].solution.clone();

        // lets add random stuff to this at a random index
        let index_to_modify = rng.gen_range(0, N_PARAMETERS);
        to_modify[index_to_modify] += rng.gen_range(-2.0, 2.0);
        to_modify
    }
}


fn get_data_from_one_game(current_parameters: [f64; N_PARAMETERS]) -> Vec<(Play, Play, usize)> {
    let mut data_to_add = Vec::new();

    let players: Vec<Box<Player>> = vec![
        Box::new(MachinePlayer::new(current_parameters)),
        Box::new(MachinePlayer::new(current_parameters)),
        Box::new(MachinePlayer::new(current_parameters)),
        Box::new(MachinePlayer::new(current_parameters)),
    ];

    let play_by_play = play_by_play(players);

    for (i, play1) in play_by_play.iter().enumerate() {
        if play1.is_pass() { continue }
        // figure out how many rounds between not passing
        for d_round in 1.. {
            let j = i + d_round*4;
            if j >= play_by_play.len() { break }

            let play2 = &play_by_play[j];

            if !play2.is_pass() {
                data_to_add.push((play1.clone(), play2.clone(), d_round));
            }
        }
    }

    data_to_add
}

fn play_by_play(players: Vec<Box<Player>>) -> Vec<Play> {
    let mut deck = entire_deck();
    deck.shuffle(&mut thread_rng());

    let mut game = GameState::new(4, deck);

    loop {
        let current_player_index = game.current_player;
        let current_player = &players[current_player_index];

        let interface = SafeGameInterface::from_game(&game);
        let play = current_player.choose_play(&interface);

        game.play(play);

        match game.winning_player() {
            Some(_winning_player_id) => {
                // want to record this
                break game.get_record().to_vec();
            }
            None => {}, // game is still in progress
        }
    }

}
