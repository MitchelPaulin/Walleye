pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};
pub use crate::configs::*;
pub use crate::engine::*;
pub use crate::move_generation::*;
pub use crate::time_control::*;
pub use crate::utils::*;
use log::{error, info};
use std::io::{self, BufRead};
use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const WHITE_KING_SIDE_CASTLE_STRING: &str = "e1g1";
const WHITE_QUEEN_SIDE_CASTLE_STRING: &str = "e1c1";
const BLACK_KING_SIDE_CASTLE_STRING: &str = "e8g8";
const BLACK_QUEEN_SIDE_CASTLE_STRING: &str = "e8c8";

pub fn play_game_uci() {
    let mut board = BoardState::from_fen(DEFAULT_FEN_STRING).unwrap();
    let buffer = read_from_gui();
    if buffer != "uci" {
        error!("Expected uci protocol but got {}", buffer);
        return;
    }

    send_to_gui(format!("id name {} {}", ENGINE_NAME, VERSION));
    send_to_gui(format!("id author {}", AUTHOR));
    send_to_gui("option name DebugLogLevel type combo default None var Info var None".to_string());
    send_to_gui("uciok".to_string());

    loop {
        let buffer = read_from_gui();
        let commands: Vec<&str> = buffer.split(' ').collect();

        match commands[0] {
            "isready" => send_to_gui("readyok".to_string()),
            "ucinewgame" => (), // we don't keep any internal state really so no need to reset anything here
            "position" => {
                board = play_out_position(commands);
                info!("{}", board.simple_board());
            }
            "go" => {
                board = find_and_play_best_move(&commands, &mut board);
            }
            "setoption" => {
                if commands.contains(&"DebugLogLevel") && commands.contains(&"Info") {
                    // set up logging
                    let log_name = format!("walleye_{}.log", process::id());
                    if simple_logging::log_to_file(log_name, log::LevelFilter::Info).is_err() {
                        panic!("Something went wrong when trying to set up logs");
                    };
                }
            }
            "quit" => process::exit(1),
            _ => error!("Unrecognized command: {}", buffer),
        };
    }
}

/*
    Finds an plays the best move and sends it to UCI
    Returns the new board state with the best move played
*/
fn find_and_play_best_move(commands: &[&str], board: &mut BoardState) -> BoardState {
    let start = Instant::now();
    let time_to_move = parse_go_command(&commands).calculate_time_slice(board.to_move);
    let mut best_move = None;

    let (tx, rx) = mpsc::channel();
    let clone = board.clone();
    thread::spawn(move || get_best_move(&clone, time_to_move, tx));
    // keep looking until we are out of time
    // also add a guard to ensure we at least get a move from the search thread
    while Instant::now().duration_since(start).as_millis() < time_to_move || best_move.is_none() {
        if let Ok(b) = rx.try_recv() {
            best_move = Some(b);
        } else {
            thread::sleep(Duration::from_millis(1));
        }
    }
    let board = best_move.unwrap();
    send_best_move_to_gui(&board);
    info!("{}", board.simple_board());
    board
}

// parse the go command and get relevant info about the current game time
fn parse_go_command(commands: &[&str]) -> GameTime {
    let mut gt = GameTime {
        wtime: 0,
        btime: 0,
        winc: 0,
        binc: 0,
        movestogo: None,
    };

    let mut i = 0;
    while i + 1 < commands.len() {
        match commands[i] {
            "wtime" => {
                gt.wtime = commands[i + 1].parse().unwrap();
                i += 1;
            }
            "btime" => {
                gt.btime = commands[i + 1].parse().unwrap();
                i += 1;
            }
            "binc" => {
                gt.binc = commands[i + 1].parse().unwrap();
                i += 1;
            }
            "winc" => {
                gt.winc = commands[i + 1].parse().unwrap();
                i += 1;
            }
            "movestogo" => {
                gt.movestogo = Some(commands[i + 1].parse().unwrap());
                i += 1;
            }
            _ => (),
        }
        i += 1;
    }

    gt
}

fn play_out_position(commands: Vec<&str>) -> BoardState {
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
                error!("{}", err);
                panic!("Got bad fen string, cant continue");
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

    if let Some(start_index) = moves_start_index {
        for mov in commands.iter().skip(start_index + 1) {
            make_move(&mut board, *mov);
        }
    }

    board
}

fn make_move(board: &mut BoardState, player_move: &str) {
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
                error!("Could not recognize piece value, default to queen");
                Queen
            }
        };
        board.board[end_pair.0][end_pair.1] = Piece {
            color: board.to_move,
            kind,
        }
        .into();
    }

    // deal with castling, here we also make sure the right king is on the target square to
    // distinguish between castling and normal moves
    if player_move == WHITE_KING_SIDE_CASTLE_STRING
        && board.board[end_pair.0][end_pair.1] == Piece::king(White)
    {
        board.board[BOARD_END - 1][BOARD_END - 1] = Square::Empty;
        board.board[BOARD_END - 1][BOARD_END - 3] = Piece::rook(White).into();
    } else if player_move == WHITE_QUEEN_SIDE_CASTLE_STRING
        && board.board[end_pair.0][end_pair.1] == Piece::king(White)
    {
        board.board[BOARD_END - 1][BOARD_START] = Square::Empty;
        board.board[BOARD_END - 1][BOARD_START + 3] = Piece::rook(White).into();
    } else if player_move == BLACK_KING_SIDE_CASTLE_STRING
        && board.board[end_pair.0][end_pair.1] == Piece::king(Black)
    {
        board.board[BOARD_START][BOARD_END - 1] = Square::Empty;
        board.board[BOARD_START][BOARD_END - 3] = Piece::rook(Black).into();
    } else if player_move == BLACK_QUEEN_SIDE_CASTLE_STRING
        && board.board[end_pair.0][end_pair.1] == Piece::king(Black)
    {
        board.board[BOARD_START][BOARD_START] = Square::Empty;
        board.board[BOARD_START][BOARD_START + 3] = Piece::rook(Black).into();
    }
    board.swap_color();
}

fn send_best_move_to_gui(board: &BoardState) {
    let best_move = board.last_move.unwrap();
    if let Some(pawn_promotion) = board.pawn_promotion {
        send_to_gui(format!(
            "bestmove {}{}{}",
            best_move.0,
            best_move.1,
            pawn_promotion.kind.alg()
        ));
    } else {
        send_to_gui(format!("bestmove {}{}", best_move.0, best_move.1));
    }
}

pub fn send_to_gui(message: String) {
    println!("{}", message);
    info!("ENGINE >> {}", message);
}

pub fn read_from_gui() -> String {
    let stdin = io::stdin();
    let mut buffer = String::new();
    stdin.lock().read_line(&mut buffer).unwrap();
    buffer = clean_input(buffer);
    info!("ENGINE << {}", buffer);
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_parse_go_command_no_inc() {
        let buffer = "go wtime 12345 btime 300000 movestogo 40";
        let commands: Vec<&str> = buffer.split(' ').collect();
        let res = parse_go_command(&commands);
        assert_eq!(res.winc, 0);
        assert_eq!(res.binc, 0);
        assert_eq!(res.wtime, 12345);
        assert_eq!(res.btime, 300000);
        assert_eq!(res.movestogo, Some(40));
    }

    #[test]
    fn can_parse_go_command() {
        let buffer = "go wtime 300000 btime 300000 winc 1 binc 2 movestogo 40";
        let commands: Vec<&str> = buffer.split(' ').collect();
        let res = parse_go_command(&commands);
        assert_eq!(res.winc, 1);
        assert_eq!(res.binc, 2);
        assert_eq!(res.wtime, 300000);
        assert_eq!(res.btime, 300000);
        assert_eq!(res.movestogo, Some(40));
    }

    #[test]
    fn can_parse_go_command_no_moves_to_go() {
        let buffer = "go wtime 300000 btime 300000 winc 1 binc 2";
        let commands: Vec<&str> = buffer.split(' ').collect();
        let res = parse_go_command(&commands);
        assert_eq!(res.winc, 1);
        assert_eq!(res.binc, 2);
        assert_eq!(res.wtime, 300000);
        assert_eq!(res.btime, 300000);
        assert_eq!(res.movestogo, None);
    }
}
