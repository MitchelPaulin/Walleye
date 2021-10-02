extern crate clap;
use clap::{App, Arg};
use std::time::Instant;
mod board;
pub mod configs;
mod engine;
mod evaluation;
mod move_generation;
pub mod search;
mod time_control;
mod uci;
mod utils;

fn main() {
    let matches = App::new(configs::ENGINE_NAME)
        .version(configs::VERSION)
        .author(configs::AUTHOR)
        .about("Plays Chess - Sometimes well")
        .arg(
            Arg::with_name("fen")
                .short("f")
                .long("fen")
                .value_name("FEN STRING")
                .help("Load a board state from a fen string, defaults to the start of a new game")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("depth")
                .short("d")
                .long("depth")
                .value_name("DEPTH")
                .help("Set the depth the engine should search to")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("play self")
                .short("P")
                .long("play-self")
                .help("Play a game against itself in the terminal"),
        )
        .arg(
            Arg::with_name("test bench")
                .short("T")
                .long("test-bench")
                .help(
                "Evaluates <FEN STRING> to benchmark move generation - incompatible with play self",
            ),
        )
        .arg(
            Arg::with_name("simple print")
                .short("S")
                .long("simple-print")
                .help("Does not use unicode or background coloring in the output"),
        )
        .get_matches();

    let depth_str = matches.value_of("depth").unwrap_or(configs::DEFAULT_DEPTH);
    let depth = match depth_str.parse::<u8>() {
        Ok(d) => d,
        Err(_) => {
            println!("Invalid depth provided");
            return;
        }
    };

    let fen = matches.value_of("fen").unwrap_or(board::DEFAULT_FEN_STRING);
    let board = match board::BoardState::from_fen(fen) {
        Ok(b) => b,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    if matches.is_present("test bench") {
        let mut moves_states = [0; 15];
        let start = Instant::now();
        move_generation::generate_moves_test(&board, 0, depth as usize, &mut moves_states, true);
        let time_to_run = Instant::now().duration_since(start).as_secs() as u32;
        let nodes: u32 = moves_states.iter().sum();
        println!(
            "Searched to a depth of {} and evaluated {} nodes in {}s for a total speed of {} nps",
            depth,
            nodes,
            time_to_run,
            nodes / time_to_run
        );
        return;
    }

    if matches.is_present("play self") {
        let simple_print = matches.is_present("simple print");
        let max_moves = 100;
        engine::play_game_against_self(&board, max_moves, depth, simple_print);
        return;
    }

    uci::play_game_uci();
}
