extern crate clap;
use clap::{App, Arg};
mod board;
mod engine;
mod move_generation;

// Board position for the start of a new game
const DEFAULT_FEN_STRING: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

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
        .get_matches();

    let depth_str = matches.value_of("depth").unwrap_or("4");
    let depth = match depth_str.parse::<u8>() {
        Ok(d) => d,
        Err(_) => {
            println!("Invalid depth provided");
            return;
        }
    };

    let fen = matches.value_of("fen").unwrap_or(DEFAULT_FEN_STRING);
    let board = match board::board_from_fen(fen) {
        Ok(b) => b,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    engine::play_game_against_self(&board, depth, 100);
}
