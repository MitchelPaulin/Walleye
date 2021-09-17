pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};
pub use crate::configs::*;
pub use crate::engine::*;
pub use crate::move_generation::*;
pub use crate::utils::*;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::process;

const WHITE_KING_SIDE_CASTLE_STRING: &str = "e1g1";
const WHITE_QUEEN_SIDE_CASTLE_STRING: &str = "e1c1";
const BLACK_KING_SIDE_CASTLE_STRING: &str = "e8g8";
const BLACK_QUEEN_SIDE_CASTLE_STRING: &str = "e8c8";

pub fn play_game_uci(search_depth: u8) {
    let mut board = BoardState::from_fen(DEFAULT_FEN_STRING).unwrap();
    let log = File::create(format!("walleye_log_{}.txt", process::id()))
        .expect("Could not create log file");
    let buffer = read_from_gui(&log);
    if buffer != "uci" {
        log_error("Expected uci protocol but got ".to_string() + &buffer, &log);
        return;
    }

    send_to_gui(format!("id name {} {}\n", ENGINE_NAME, VERSION), &log);
    send_to_gui(format!("id author {}\n", AUTHOR), &log);
    send_to_gui("uciok\n".to_string(), &log);

    loop {
        let buffer = read_from_gui(&log);
        let commands: Vec<&str> = buffer.split(' ').collect();

        if commands[0] == "quit" {
            break;
        } else if commands[0] == "isready" {
            send_to_gui("readyok\n".to_string(), &log);
        } else if commands[0] == "ucinewgame" {
            // we don't keep any internal state really so no need to reset anything here
        } else if commands[0] == "position" {
            board = play_out_position(commands, &log);
            log_info(board.simple_board(), &log);
        } else if commands[0] == "go" {
            board = find_best_move(&board, search_depth, &log);
            log_info(board.simple_board(), &log);
        } else {
            log_error(format!("Unrecognized command: {}", buffer), &log);
        }
    }
}

fn play_out_position(commands: Vec<&str>, log: &File) -> BoardState {
    let mut board;
    if commands[1] == "fen" {
        let mut fen = "".to_string();
        for c in commands.iter().take(7).skip(2) {
            fen += &format!("{} ", c);
        }
        fen += commands[7];

        board = match BoardState::from_fen(&fen) {
            Ok(board) => board,
            Err(err) => {
                log_error(err.to_string(), &log);
                panic!("Got bad fen string, cant continue")
            }
        };
    } else {
        board = BoardState::from_fen(DEFAULT_FEN_STRING).unwrap();
    }

    let mut moves_start_index = None;
    for (i, command) in commands.iter().enumerate() {
        if command == &"moves" {
            moves_start_index = Some(i);
            break;
        }
    }

    if moves_start_index.is_some() {
        let first_move_index = moves_start_index.unwrap() + 1;
        for mov in commands.iter().skip(first_move_index) {
            make_move(&mut board, *mov, &log);
        }
    }
    board
}

fn make_move(board: &mut BoardState, player_move: &str, log: &File) {
    let start_pair: Point = (player_move[0..2]).parse().unwrap();
    let end_pair: Point = (player_move[2..4]).parse().unwrap();

    // update king location
    if let Square::Full(Piece { kind, color }) = board.board[start_pair.0][start_pair.1] {
        if kind == King {
            if color == White {
                board.white_king_location = end_pair;
                board.white_king_side_castle = false;
                board.white_queen_side_castle = false;
            } else {
                board.black_king_location = end_pair;
                board.black_king_side_castle = false;
                board.black_queen_side_castle = false;
            }
        }
    }

    //deal with castling privileges related to the movement/capture of rooks
    if player_move.contains("a8") {
        board.black_queen_side_castle = false;
    } 
    
    if player_move.contains("h8") {
        board.black_king_side_castle = false;
    } 
    
    if player_move.contains("a1") {
        board.white_queen_side_castle = false;
    } 
    
    if player_move.contains("h1") {
        board.white_king_side_castle = false;
    }

    //move piece
    board.board[end_pair.0][end_pair.1] = board.board[start_pair.0][start_pair.1];
    board.board[start_pair.0][start_pair.1] = Square::Empty;

    //deal with any pawn promotions
    if player_move.len() == 5 {
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
    if &player_move[0..4] == WHITE_KING_SIDE_CASTLE_STRING
        && board.board[end_pair.0][end_pair.1] == Piece::king(White)
    {
        board.board[BOARD_END - 1][BOARD_END - 1] = Square::Empty;
        board.board[BOARD_END - 1][BOARD_END - 3] = Piece::rook(White).into();
    } else if &player_move[0..4] == WHITE_QUEEN_SIDE_CASTLE_STRING
        && board.board[end_pair.0][end_pair.1] == Piece::king(White)
    {
        board.board[BOARD_END - 1][BOARD_START] = Square::Empty;
        board.board[BOARD_END - 1][BOARD_START + 3] = Piece::rook(White).into();
    } else if &player_move[0..4] == BLACK_KING_SIDE_CASTLE_STRING
        && board.board[end_pair.0][end_pair.1] == Piece::king(Black)
    {
        board.board[BOARD_START][BOARD_END - 1] = Square::Empty;
        board.board[BOARD_START][BOARD_END - 3] = Piece::rook(Black).into();
    } else if &player_move[0..4] == BLACK_QUEEN_SIDE_CASTLE_STRING
        && board.board[end_pair.0][end_pair.1] == Piece::king(Black)
    {
        board.board[BOARD_START][BOARD_START] = Square::Empty;
        board.board[BOARD_START][BOARD_START + 3] = Piece::rook(Black).into();
    }
    board.swap_color();
    board.order_heuristic = i32::MIN;
}

fn find_best_move(board: &BoardState, search_depth: u8, log: &File) -> BoardState {
    let next_board = get_best_move(&board, search_depth).unwrap();
    let best_move = next_board.last_move.unwrap();
    if next_board.pawn_promotion.is_some() {
        send_to_gui(
            format!(
                "bestmove {}{}{}\n",
                best_move.0,
                best_move.1,
                next_board.pawn_promotion.unwrap().kind.alg()
            ),
            &log,
        );
    } else {
        send_to_gui(format!("bestmove {}{}\n", best_move.0, best_move.1), &log);
    }
    next_board
}

fn log_info(message: String, mut log: &File) {
    log.write_all(format!("<INFO> {}\n", message).as_bytes())
        .expect("write failed");
}

fn log_error(message: String, mut log: &File) {
    log.write_all(format!("<ERROR> {}\n", message).as_bytes())
        .expect("write failed");
}

fn send_to_gui(message: String, mut log: &File) {
    print!("{}", message);
    log.write_all(format!("ENGINE >> {}", message).as_bytes())
        .expect("write failed");
}

fn read_from_gui(mut log: &File) -> String {
    let stdin = io::stdin();
    let mut buffer = String::new();
    stdin.lock().read_line(&mut buffer).unwrap();
    buffer = clean_input(buffer);
    log.write_all(format!("ENGINE << {}\n", buffer).as_bytes())
        .expect("write failed");
    buffer
}
