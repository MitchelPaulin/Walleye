pub use crate::board::*;
pub use crate::engine::*;
pub use crate::move_generation::*;
use std::io::{self, BufRead, Write};

pub fn play_game_uci(search_depth: u8) {
    let mut board = board_from_fen(DEFAULT_FEN_STRING).unwrap();
    let log = std::fs::File::create("log.txt").expect("Could not create log file");
    let buffer = read_from_gui(&log);
    if buffer != "uci\n" {
        log_error("Expected uci protocol but got ".to_string() + &buffer, &log);
        return;
    }
    send_to_gui("id name Walleye\n".to_string(), &log);
    send_to_gui("id author Mitchel Paulin\n".to_string(), &log);
    send_to_gui("uciok\n".to_string(), &log);

    loop {
        let buffer = read_from_gui(&log);
        let command: Vec<&str> = buffer.split(' ').collect();
        if command[0] == "quit\n" {
            break;
        } else if command[0] == "isready\n" {
            send_to_gui("readyok\n".to_string(), &log);
        } else if command[0] == "ucinewgame\n" {
            let buffer = read_from_gui(&log);
            board = match setup_new_game(buffer, &log) {
                Some(b) => b,
                _ => {
                    break;
                }
            }
        } else if command[0] == "position" && command.contains(&"moves") {
            // only play last move, the rest has been recorded in the board state
            let player_move = command.last().unwrap();
            handle_player_move(&mut board, player_move, &log);
        } else if command[0] == "go" {
            board = find_best_move(&board, search_depth, &log);
        } else {
            log_error(format!("Unrecognized command: {}", buffer), &log);
        }
    }
}

fn handle_player_move(board: &mut BoardState, player_move: &&str, log: &std::fs::File) {
    let start_pair = algebraic_pairs_to_board_position(&player_move[0..2]).unwrap();
    let end_pair = algebraic_pairs_to_board_position(&player_move[0..2][2..4]).unwrap();
    let target_square = board.board[end_pair.0][end_pair.1];
    if !is_empty(target_square) {
        if is_white(target_square) {
            board.white_total_piece_value -= PIECE_VALUES[(target_square & PIECE_MASK) as usize];
        } else {
            board.black_total_piece_value -= PIECE_VALUES[(target_square & PIECE_MASK) as usize];
        }
    }

    board.board[end_pair.0][end_pair.1] = board.board[start_pair.0][start_pair.1];
    board.board[start_pair.0][start_pair.1] = EMPTY;
    //deal with pawn promotions, check for 6 because of new line character
    if player_move.len() == 6 {
        let piece = match player_move.chars().nth(4).unwrap() {
            'q' => QUEEN,
            'n' => KNIGHT,
            'b' => BISHOP,
            'r' => ROOK,
            _ => {
                log_error(
                    "Could not recognize piece value, default to queen".to_string(),
                    &log,
                );
                QUEEN
            }
        };
        board.board[end_pair.0][end_pair.1] = piece | board.to_move.as_mask();
    }

    //deal with castling
    if &player_move[0..4] == WHITE_KING_SIDE_ALG
        && board.board[end_pair.0][end_pair.1] == WHITE | KING
    {
        board.board[BOARD_END - 1][BOARD_END - 1] = EMPTY;
        board.board[BOARD_END - 1][BOARD_END - 3] = WHITE | ROOK;
    } else if &player_move[0..4] == WHITE_QUEEN_SIDE_ALG
        && board.board[end_pair.0][end_pair.1] == WHITE | KING
    {
        board.board[BOARD_END - 1][BOARD_START] = EMPTY;
        board.board[BOARD_END - 1][BOARD_START + 3] = WHITE | ROOK;
    } else if &player_move[0..4] == BLACK_KING_SIDE_CASTLE_ALG
        && board.board[end_pair.0][end_pair.1] == BLACK | KING
    {
        board.board[BOARD_START][BOARD_END - 1] = EMPTY;
        board.board[BOARD_START][BOARD_END - 3] = BLACK | ROOK;
    } else if &player_move[0..4] == BLACK_QUEEN_SIDE_CASTLE_ALG
        && board.board[end_pair.0][end_pair.1] == BLACK | KING
    {
        board.board[BOARD_START][BOARD_START] = EMPTY;
        board.board[BOARD_START][BOARD_START + 3] = BLACK | ROOK;
    }

    board.swap_color();
    log_info(board.simple_board(), &log);
}

fn find_best_move(board: &BoardState, search_depth: u8, log: &std::fs::File) -> BoardState {
    let evaluation = alpha_beta_search(&board, search_depth, i32::MIN, i32::MAX, board.to_move);
    let next_board = evaluation.0.unwrap();
    let best_move = next_board.last_move.clone().unwrap();
    send_to_gui(format!("bestmove {}\n", best_move), &log);
    log_info(board.simple_board(), &log);
    next_board
}

fn setup_new_game(buffer: String, log: &std::fs::File) -> Option<BoardState> {
    let command: Vec<&str> = buffer.split(' ').collect();
    if command[1] == "startpos\n" {
        return Some(board_from_fen(DEFAULT_FEN_STRING).unwrap());
    } else if command[1] == "fen" {
        let mut fen = "".to_string();
        for i in 2..7 {
            fen += &(command[i].to_string() + &" ".to_string());
        }
        fen += &command[7].to_string();
        match board_from_fen(&fen) {
            Ok(b) => return Some(b),
            Err(err) => {
                log_error(format!("{} : {}", err, fen), &log);
                return None;
            }
        }
    }
    return None;
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
    buffer
}
