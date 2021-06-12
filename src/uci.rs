pub use crate::board::*;
pub use crate::engine::*;
use std::io::{self, BufRead, Write};

pub fn play_game_uci() {
    let mut buffer = String::new();
    let mut board =
        board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let log = std::fs::File::create("C:\\Users\\Mitch\\Desktop\\log.txt")
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
                board = board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                    .unwrap();
            } else if command[1] == "startpos" {
                if command.len() >= 3 && command[2] == "moves" {
                    let mov = command.len() - 1; // only play last move
                    let start_pos = &command[mov][0..2];
                    let end_pos = &command[mov][2..4];
                    let start_pair = algebraic_pairs_to_board_position(start_pos).unwrap();
                    let end_pair = algebraic_pairs_to_board_position(end_pos).unwrap();
                    board.board[end_pair.0][end_pair.1] = board.board[start_pair.0][start_pair.1];
                    board.board[start_pair.0][start_pair.1] = EMPTY;
                }
            }
        } else if command[0] == "go" {
            board.to_move = PieceColor::Black;
            let evaluation = alpha_beta_search(&board, 5, i32::MIN, i32::MAX, PieceColor::Black);
            let b = evaluation.0.unwrap();
            let best_move = b.last_move.unwrap();
            let start = best_move.0;
            let end = best_move.1;
            let best_move_alg =
                board_position_to_algebraic_pair(start) + &board_position_to_algebraic_pair(end);
            let command = "bestmove ".to_string() + &best_move_alg.to_string() + &"\n".to_string();
            board = b;
            send_to_gui(command, &log);
        } else {
            log_error("Unrecognized command ".to_string() + &buffer, &log);
        }
    }
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
