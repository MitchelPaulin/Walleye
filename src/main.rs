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
        .get_matches();

    let fen = matches.value_of("fen").unwrap_or(DEFAULT_FEN_STRING);

    let b = board::board_from_fen(fen);
    let mut board = match b {
        Ok(b) => b,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let mut best_move;
    let mut next_board = board;
    while board.full_move_clock < 200 {
        if board.to_move == board::PieceColor::White {
            best_move = i32::MIN;
        } else {
            best_move = i32::MAX;
        }

        let moves = move_generation::generate_moves(&board);
        if moves.len() == 0 {
            break;
        }
        for mov in moves {

            let maximizer = match board.to_move {
                board::PieceColor::White => board::PieceColor::Black,
                _ => board::PieceColor::White
            };

            let res = engine::alpha_beta_search(&mov, 4, i32::MIN, i32::MAX, maximizer);
            if board.to_move == board::PieceColor::White && best_move < res {
                best_move = res;
                next_board = mov;
            } else if board.to_move == board::PieceColor::Black && res < best_move {
                best_move = res;
                next_board = mov;
            }
        }
        next_board.pretty_print_board();
        board = next_board;
    }
    board.pretty_print_board();
}
