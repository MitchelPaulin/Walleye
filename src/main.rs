extern crate clap;
use clap::{App, Arg};
use std::{time::Instant, cmp::max};
mod board;
mod engine;
mod evaluation;
mod move_generation;
mod search;
mod time_control;
mod uci;
mod utils;
mod zobrist;

/*
    A custom memory allocator with better performance
    characteristics than rusts default.
    During testing this resulted in a ~20% speed up in move generation.
    If you are having trouble compiling the engine for your target system
    you can try removing the two lines below.
    https://github.com/microsoft/mimalloc
*/
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
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
                .help("Set the depth the engine should search to, only used for profiling")
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
    const DEFAULT_DEPTH: &str = "6";
    let depth_str = matches.value_of("depth").unwrap_or(DEFAULT_DEPTH);
    let depth = match depth_str.parse::<u8>() {
        Ok(d) => d,
        Err(_) => {
            println!("Invalid depth provided");
            return;
        }
    };

    if depth >= search::MAX_DEPTH {
        println!("Can not have depth greater than {}", search::MAX_DEPTH - 1);
        return;
    }

    let fen = matches.value_of("fen").unwrap_or(board::DEFAULT_FEN_STRING);
    let board = match board::BoardState::from_fen(fen) {
        Ok(b) => b,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    if matches.is_present("test bench") {
        let mut moves_states = [0; search::MAX_DEPTH as usize];
        let start = Instant::now();
        let zobrist_hasher = zobrist::ZobristHasher::create_zobrist_hasher();
        move_generation::generate_moves_test(
            &board,
            0,
            depth as usize,
            &mut moves_states,
            true,
            &zobrist_hasher,
        );
        let time_to_run = Instant::now().duration_since(start);
        let nodes: u32 = moves_states.iter().sum();
        println!(
            "Searched to a depth of {} and evaluated {} nodes in {:?} for a total speed of {} nps",
            depth,
            nodes,
            time_to_run,
            nodes / max(time_to_run.as_secs() as u32, 1)
        );
        return;
    }

    if matches.is_present("play self") {
        let simple_print = matches.is_present("simple print");
        let max_moves = 100;
        let time_per_move_ms = 1000;
        engine::play_game_against_self(&board, max_moves, time_per_move_ms, simple_print);
        return;
    }

    uci::play_game_uci();
}
