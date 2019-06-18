#![allow(dead_code)]

mod bot;

mod card;

mod play;

mod game;

mod util;

mod train;
use train::{DEFAULT_PARAMETERS, training_step};

fn main() {
    let mut current_parameters = DEFAULT_PARAMETERS;

    for _ in 0.. {
        let results = training_step(current_parameters);

        println!("{:?}", results);

        current_parameters = results.0;
    }
}

