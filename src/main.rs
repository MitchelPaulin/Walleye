extern crate clap;
use clap::{App, Arg};
mod board;
mod engine;
mod move_generation;
mod uci;
mod utils;

// During testing I found a depth of 7 to perform best on the optimized build, recommend depth 4 on debug build
const DEFAULT_DEPTH: &str = "7";

fn main() {
    let matches = App::new("Chess Engine")
        .version("0.1")
        .author("Mitchel P. <mitchel0022@gmail.com>")
        .about("Plays Chess (Sometimes well)")
        .arg(
            Arg::with_name("fen")
                .short("f")
                .long("fen")
                .value_name("FEN STRING")
                .help("Load a board state from a fen string")
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
            Arg::with_name("simple print")
                .short("S")
                .long("simple-print")
                .help("Does not use unicode or background coloring in the output, useful on windows OS"),
        )
        .get_matches();

    let depth_str = matches.value_of("depth").unwrap_or(DEFAULT_DEPTH);
    let depth = match depth_str.parse::<u8>() {
        Ok(d) => d,
        Err(_) => {
            println!("Invalid depth provided");
            return;
        }
    };

    let fen = matches.value_of("fen").unwrap_or(board::DEFAULT_FEN_STRING);
    let board = match board::board_from_fen(fen) {
        Ok(b) => b,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let simple_print = matches.is_present("simple print");
    if matches.is_present("play self") {
        engine::play_game_against_self(&board, depth, 50, simple_print);
        return;
    }

    uci::play_game_uci();
}
