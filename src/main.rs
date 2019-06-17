#![allow(dead_code)]


use rand::thread_rng;
use rand::seq::SliceRandom;

mod bot;
use crate::bot::{Player, HumanPlayer, MachinePlayer, AlternateMachinePlayer};

mod card;
use crate::card::{entire_deck};

mod play;

mod game;
use game::GameState;
use crate::game::SafeGameInterface;

mod util;

const OUTPUT_FILENAME: &'static str = "record.txt";


fn main() {

    let mut deck = entire_deck();
    deck.shuffle(&mut thread_rng());
    let mut game = GameState::new(4, deck);


    // generate the objects representing all of the players
    let players: Vec<Box<Player>> = vec![
        Box::new(HumanPlayer),
        Box::new(AlternateMachinePlayer),
        Box::new(MachinePlayer),
        Box::new(MachinePlayer),
    ];


    loop {
        let current_player_index = game.current_player;
        let current_player = &players[current_player_index];

        let interface = SafeGameInterface::from_game(&game);
        let play = current_player.choose_play(&interface);

        // report on this play
        if play.is_pass() {
            println!("player #{} passed", current_player_index);
        } else {
            println!("player #{} played {:?}", current_player_index, play.cards());
        }

        game.play(play);

        match game.winning_player() {
            Some(winning_player_id) => {
                println!("player #{} won!", winning_player_id);
                // want to record this
                break;
            }
            None => {}, // game is still in progress
        }
    }

}
