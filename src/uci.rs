pub use crate::board::*;
pub use crate::engine::*;
use std::io::{self, BufRead, Write};

pub fn play_game_uci() {
    let mut buffer;
    let mut board = board_from_fen(DEFAULT_FEN_STRING).unwrap();
    let log = std::fs::File::create("/home/mitch/Desktop/log.txt")
        .expect("Could not create log file");
    buffer = read_from_gui(&log);
    if buffer != "uci\n" {
        log_error("Expected uci protocol but got ".to_string() + &buffer, &log);
        return;
    }
    send_to_gui("id name ChessEngine\n".to_string(), &log);
    send_to_gui("id author Mitchel Paulin\n".to_string(), &log);
    send_to_gui("uciok\n".to_string(), &log);

    while true {
        buffer = read_from_gui(&log);
        let command: Vec<&str> = buffer.split(' ').collect();
        if command[0] == "quit\n" {
            break;
        } else if command[0] == "isready\n" {
            send_to_gui("readyok\n".to_string(), &log);
        } else if command[0] == "ucinewgame\n" {
            // ignore
        } else if command[0] == "position" {
            if command[1] == "startpos\n" {
                board = board_from_fen(DEFAULT_FEN_STRING).unwrap();
            } else if command.len() >= 3 && command[2] == "moves" {
                let mov = command.len() - 1; // only play last move, the rest has been recorded in the board state
                let start_pair = algebraic_pairs_to_board_position(&command[mov][0..2]).unwrap();
                let end_pair = algebraic_pairs_to_board_position(&command[mov][2..4]).unwrap();
                board.board[end_pair.0][end_pair.1] = board.board[start_pair.0][start_pair.1];
                board.board[start_pair.0][start_pair.1] = EMPTY;

                //deal with castling 
                if &command[mov][0..4] == "e1g1" && board.board[end_pair.0][end_pair.1] == WHITE | KING {
                    board.board[BOARD_END - 1][BOARD_END - 1] = EMPTY;
                    board.board[BOARD_END - 1][BOARD_END - 3] = WHITE | ROOK;
                } else if &command[mov][0..4] == "e1c1" && board.board[end_pair.0][end_pair.1] == WHITE | KING {
                    board.board[BOARD_END - 1][BOARD_START] = EMPTY;
                    board.board[BOARD_END - 1][BOARD_START + 3] = WHITE | ROOK;
                } else if &command[mov][0..4] == "e8g8" && board.board[end_pair.0][end_pair.1] == BLACK| KING {
                    board.board[BOARD_START][BOARD_END - 1] = EMPTY;
                    board.board[BOARD_START][BOARD_END - 3] = BLACK | ROOK;
                } else if &command[mov][0..4] == "e8c8" && board.board[end_pair.0][end_pair.1] == BLACK| KING {
                    board.board[BOARD_START][BOARD_START] = EMPTY;
                    board.board[BOARD_START][BOARD_START + 3] = BLACK | ROOK;
                }

                board.swap_color();
                log_info(board.simple_board(), &log);
            }
        } else if command[0] == "go" {
            let evaluation = alpha_beta_search(&board, 5, i32::MIN, i32::MAX, board.to_move);
            let next_board = evaluation.0.unwrap();
            let best_move = next_board.last_move.clone().unwrap();
            board = next_board;
            send_to_gui(format!("bestmove {}\n", best_move), &log);
            log_info(board.simple_board(), &log);
        } else {
            log_error("Unrecognized command ".to_string() + &buffer, &log);
        }
    }
}

fn log_info(message: String, mut log: &std::fs::File) {
    log.write_all(("<INFO> ".to_string() + &message).as_bytes())
        .expect("write failed");
}

fn log_error(message: String, mut log: &std::fs::File) {
    log.write_all(("<ERROR> ".to_string() + &message).as_bytes())
        .expect("write failed");
}

fn send_to_gui(message: String, mut log: &std::fs::File) {
    print!("{}", message);
    log.write_all(("ENGINE >> ".to_string() + &message).as_bytes())
        .expect("write failed");
}

fn read_from_gui(mut log: &std::fs::File) -> String {
    let stdin = io::stdin();
    let mut buffer = String::new();
    stdin.lock().read_line(&mut buffer).unwrap();
    log.write_all(("ENGINE << ".to_string() + &buffer).as_bytes())
        .expect("write failed");
    return buffer;
}
