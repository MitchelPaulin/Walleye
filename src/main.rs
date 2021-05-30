extern crate clap;
use clap::{App, Arg};
mod board;
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
        .get_matches();

    let fen = matches.value_of("fen").unwrap_or(DEFAULT_FEN_STRING);

    let b = move_generation::board_from_fen(fen);
    match b {
        Ok(b) => b.print_board(),
        Err(err) => println!("{}", err),
    }
}
