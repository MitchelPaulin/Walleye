pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};
pub use crate::engine::*;
pub use crate::move_generation::*;
use std::io::{self, BufRead, Write};

pub fn play_game_uci(search_depth: u8) {
    let mut board = BoardState::from_fen(DEFAULT_FEN_STRING).unwrap();
    let log = std::fs::File::create("log.txt").expect("Could not create log file");
    let buffer = read_from_gui(&log);
    if buffer != "uci" {
        log_error("Expected uci protocol but got ".to_string() + &buffer, &log);
        return;
    }
    send_to_gui("id name Walleye\n".to_string(), &log);
    send_to_gui("id author Mitchel Paulin\n".to_string(), &log);
    send_to_gui("uciok\n".to_string(), &log);

    loop {
        let buffer = read_from_gui(&log);
        let command: Vec<&str> = buffer.split(' ').collect();

        if command[0] == "quit" {
            break;
        } else if command[0] == "isready" {
            send_to_gui("readyok\n".to_string(), &log);
        } else if command[0] == "ucinewgame" {
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
            log_info(player_move.to_string(), &log);
            handle_player_move(&mut board, player_move, &log);
        } else if command[0] == "go" {
            board = find_best_move(&board, search_depth, &log);
        } else {
            log_error(format!("Unrecognized command: {}", buffer), &log);
        }
    }
}

fn handle_player_move(board: &mut BoardState, player_move: &&str, log: &std::fs::File) {
    let start_pair: Point = (&player_move[0..2]).parse().unwrap();
    let end_pair: Point = (&player_move[2..4]).parse().unwrap();

    board.board[end_pair.0][end_pair.1] = board.board[start_pair.0][start_pair.1];
    board.board[start_pair.0][start_pair.1] = Square::Empty;
    //deal with pawn promotions, check for 6 because of new line character
    if player_move.len() == 6 {
        let kind = match player_move.chars().nth(4).unwrap() {
            'q' => Queen,
            'n' => Knight,
            'b' => Bishop,
            'r' => Rook,
            _ => {
                log_error(
                    "Could not recognize piece value, default to queen".to_string(),
                    &log,
                );
                Queen
            }
        };
        board.board[end_pair.0][end_pair.1] = Piece {
            color: board.to_move,
            kind,
        }
        .into();
    }

    //deal with castling
    if &player_move[0..4] == WHITE_KING_SIDE_CASTLE_ALG
        && board.board[end_pair.0][end_pair.1] == Piece::king(White)
    {
        board.board[BOARD_END - 1][BOARD_END - 1] = Square::Empty;
        board.board[BOARD_END - 1][BOARD_END - 3] = Piece::rook(White).into();
    } else if &player_move[0..4] == WHITE_QUEEN_SIDE_CASTLE_ALG
        && board.board[end_pair.0][end_pair.1] == Piece::king(White)
    {
        board.board[BOARD_END - 1][BOARD_START] = Square::Empty;
        board.board[BOARD_END - 1][BOARD_START + 3] = Piece::rook(White).into();
    } else if &player_move[0..4] == BLACK_KING_SIDE_CASTLE_ALG
        && board.board[end_pair.0][end_pair.1] == Piece::king(Black)
    {
        board.board[BOARD_START][BOARD_END - 1] = Square::Empty;
        board.board[BOARD_START][BOARD_END - 3] = Piece::rook(Black).into();
    } else if &player_move[0..4] == BLACK_QUEEN_SIDE_CASTLE_ALG
        && board.board[end_pair.0][end_pair.1] == Piece::king(Black)
    {
        board.board[BOARD_START][BOARD_START] = Square::Empty;
        board.board[BOARD_START][BOARD_START + 3] = Piece::rook(Black).into();
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
    log_info(format!("pos eval: {}", get_evaluation(board)), &log);
    next_board
}

fn setup_new_game(buffer: String, log: &std::fs::File) -> Option<BoardState> {
    let command: Vec<&str> = buffer.split(' ').collect();
    if command[1] == "startpos" {
        return Some(BoardState::from_fen(DEFAULT_FEN_STRING).unwrap());
    } else if command[1] == "fen" {
        let mut fen = "".to_string();
        for i in 2..7 {
            fen += &format!("{} ", command[i]);
        }
        fen += command[7];
        match BoardState::from_fen(&fen) {
            Ok(b) => return Some(b),
            Err(err) => {
                log_error(format!("{} : {}", err, fen), &log);
                return None;
            }
        }
    }
    None
}

fn log_info(message: String, mut log: &std::fs::File) {
    log.write_all(format!("<INFO> {}\n", message).as_bytes())
        .expect("write failed");
}

fn log_error(message: String, mut log: &std::fs::File) {
    log.write_all(format!("<ERROR> {}\n", message).as_bytes())
        .expect("write failed");
}

fn send_to_gui(message: String, mut log: &std::fs::File) {
    print!("{}", message);
    log.write_all(format!("ENGINE >> {}", message).as_bytes())
        .expect("write failed");
}

fn read_from_gui(mut log: &std::fs::File) -> String {
    let stdin = io::stdin();
    let mut buffer = String::new();
    stdin.lock().read_line(&mut buffer).unwrap();
    buffer = clean_input(buffer);
    log.write_all(format!("ENGINE << {}\n", buffer).as_bytes())
        .expect("write failed");
    buffer
}

fn clean_input(buffer: String) -> String {
    let mut cleaned = String::new();
    let mut prev_char = ' ';
    for c in buffer.chars() {
        if !c.is_whitespace() {
            cleaned.push(c);
        } else if c.is_whitespace() && !prev_char.is_whitespace() {
            cleaned.push(' ');
        }
        prev_char = c;
    }
    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_string() {
        assert_eq!(clean_input("   debug     on  \n".to_string()), "debug on");
        assert_eq!(
            clean_input("\t  debug \t  \t\ton\t  \n".to_string()),
            "debug on"
        );
    }
}
