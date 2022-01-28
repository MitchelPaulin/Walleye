pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};
use crate::draw_table::DrawTable;
pub use crate::engine::*;
pub use crate::move_generation::*;
pub use crate::time_control::*;
use crate::transposition_table::{self, TranspositionTable};
pub use crate::utils::*;
use crate::zobrist::ZobristHasher;
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

    send_to_gui(&format!(
        "id name {} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    ));
    send_to_gui(&format!("id author {}", env!("CARGO_PKG_AUTHORS")));
    send_to_gui("option name DebugLogLevel type combo default None var Info var None");
    send_to_gui("uciok");

    let zobrist_hasher = ZobristHasher::create_zobrist_hasher();
    let mut draw_table = DrawTable::new();
    let mut tt_table = TranspositionTable::new();
    loop {
        let buffer = read_from_gui();
        let start = Instant::now();
        let commands: Vec<&str> = buffer.split(' ').collect();

        match commands[0] {
            "isready" => send_to_gui("readyok"),
            "ucinewgame" => (), // we don't keep any internal state really so no need to reset anything here
            "position" => {
                draw_table.clear();
                board = play_out_position(&commands, &zobrist_hasher, &mut draw_table);
                info!("{}", board.simple_board());
            }
            "go" => {
                board = find_and_play_best_move(
                    &commands,
                    &mut board,
                    start,
                    &mut draw_table,
                    &mut &mut tt_table,
                );
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
fn find_and_play_best_move(
    commands: &[&str],
    board: &mut BoardState,
    start: Instant,
    draw_table: &mut DrawTable,
    transposition_table: &mut TranspositionTable,
) -> BoardState {
    let time_to_move_ms = parse_go_command(commands).calculate_time_slice(board.to_move);
    let mut best_move = None;

    let (tx, rx) = mpsc::channel();
    let clone = board.clone();
    let mut draw_clone = draw_table.clone();
    let mut tt_clone = transposition_table.clone();
    thread::spawn(move || {
        get_best_move(
            &clone,
            &mut draw_clone,
            &mut tt_clone,
            start,
            time_to_move_ms,
            &tx,
        )
    });
    // keep looking until we are out of time
    // also add a guard to ensure we at least get a move from the search thread
    while !out_of_time(start, time_to_move_ms) || best_move.is_none() {
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

/*
    From the provided fen string set up the board state
*/
fn play_out_position(
    commands: &[&str],
    zobrist_hasher: &ZobristHasher,
    draw_table: &mut DrawTable,
) -> BoardState {
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

    draw_table.table.insert(board.zobrist_key, 1);

    if let Some(start_index) = moves_start_index {
        for mov in commands.iter().skip(start_index + 1) {
            make_move(&mut board, *mov, zobrist_hasher);
            draw_table.add_board_to_draw_table(&board);
        }
    }

    board
}

/*
    Play the opponents move on the board
*/
fn make_move(board: &mut BoardState, player_move: &str, zobrist_hasher: &ZobristHasher) {
    let start_pair: Point = (player_move[0..2]).parse().unwrap();
    let end_pair: Point = (player_move[2..4]).parse().unwrap();
    board.unset_pawn_double_move(zobrist_hasher);

    if let Square::Full(piece) = board.board[start_pair.0][start_pair.1] {
        // update king location
        if piece.kind == King {
            if piece.color == White {
                board.white_king_location = end_pair;
                board.take_away_castling_rights(CastlingType::WhiteQueenSide, zobrist_hasher);
                board.take_away_castling_rights(CastlingType::WhiteKingSide, zobrist_hasher);
            } else {
                board.black_king_location = end_pair;
                board.take_away_castling_rights(CastlingType::BlackQueenSide, zobrist_hasher);
                board.take_away_castling_rights(CastlingType::BlackKingSide, zobrist_hasher);
            }
        } else if piece.kind == Pawn {
            if (start_pair.0 as i8 - end_pair.0 as i8).abs() == 2 {
                // pawn made a double move, record space behind pawn for en passant
                let target = match piece.color {
                    White => Point(start_pair.0 - 1, start_pair.1),
                    Black => Point(start_pair.0 + 1, start_pair.1),
                };
                board.zobrist_key ^= zobrist_hasher.get_val_for_en_passant(target.1);
                board.pawn_double_move = Some(target);
            }
            // check for en passant captures
            // if a pawn moves diagonally and no capture is made, it must be an en passant capture
            if start_pair.1 != end_pair.1 && board.board[end_pair.0][end_pair.1] == Square::Empty {
                board.board[start_pair.0][end_pair.1] = Square::Empty;
                board.zobrist_key ^= zobrist_hasher.get_val_for_piece(
                    Piece::pawn(board.to_move.opposite()),
                    Point(start_pair.0, end_pair.1),
                );
            }
        }
    } else {
        panic!("UCI Error: Trying to move a piece that does not exist");
    }

    //deal with castling privileges related to the movement/capture of rooks
    if player_move.contains("a8") {
        board.take_away_castling_rights(CastlingType::BlackQueenSide, zobrist_hasher);
    }
    if player_move.contains("h8") {
        board.take_away_castling_rights(CastlingType::BlackKingSide, zobrist_hasher);
    }
    if player_move.contains("a1") {
        board.take_away_castling_rights(CastlingType::WhiteQueenSide, zobrist_hasher);
    }
    if player_move.contains("h1") {
        board.take_away_castling_rights(CastlingType::WhiteKingSide, zobrist_hasher);
    }

    //move piece
    board.move_piece(start_pair, end_pair, zobrist_hasher);

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
        let promotion_piece = Piece {
            color: board.to_move,
            kind,
        };
        board.zobrist_key ^= zobrist_hasher.get_val_for_piece(Piece::pawn(board.to_move), end_pair)
            ^ zobrist_hasher.get_val_for_piece(promotion_piece, end_pair);
        board.board[end_pair.0][end_pair.1] = promotion_piece.into();
    }

    // deal with castling, here we also make sure the right king is on the target square to
    // distinguish between castling and normal moves
    if player_move == WHITE_KING_SIDE_CASTLE_STRING
        && board.board[end_pair.0][end_pair.1] == Piece::king(White)
    {
        board.move_piece(
            Point(BOARD_END - 1, BOARD_END - 1),
            Point(BOARD_END - 1, BOARD_END - 3),
            zobrist_hasher,
        );
    } else if player_move == WHITE_QUEEN_SIDE_CASTLE_STRING
        && board.board[end_pair.0][end_pair.1] == Piece::king(White)
    {
        board.move_piece(
            Point(BOARD_END - 1, BOARD_START),
            Point(BOARD_END - 1, BOARD_START + 3),
            zobrist_hasher,
        );
    } else if player_move == BLACK_KING_SIDE_CASTLE_STRING
        && board.board[end_pair.0][end_pair.1] == Piece::king(Black)
    {
        board.move_piece(
            Point(BOARD_START, BOARD_END - 1),
            Point(BOARD_START, BOARD_END - 3),
            zobrist_hasher,
        );
    } else if player_move == BLACK_QUEEN_SIDE_CASTLE_STRING
        && board.board[end_pair.0][end_pair.1] == Piece::king(Black)
    {
        board.move_piece(
            Point(BOARD_START, BOARD_START),
            Point(BOARD_START, BOARD_START + 3),
            zobrist_hasher,
        );
    }

    board.swap_color(zobrist_hasher);
}

fn send_best_move_to_gui(board: &BoardState) {
    let best_move = board.last_move.unwrap();
    if let Some(pawn_promotion) = board.pawn_promotion {
        send_to_gui(&format!(
            "bestmove {}{}{}",
            best_move.0,
            best_move.1,
            pawn_promotion.kind.alg()
        ));
    } else {
        send_to_gui(&format!("bestmove {}{}", best_move.0, best_move.1));
    }
}

pub fn send_to_gui(message: &str) {
    println!("{}", message);
    info!("ENGINE >> {}", message);
}

pub fn read_from_gui() -> String {
    let stdin = io::stdin();
    let mut buffer = String::new();
    stdin.lock().read_line(&mut buffer).unwrap();
    buffer = clean_input(&buffer);
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

    #[test]
    fn en_passant_capture_parsed_correctly_black() {
        let mut board = BoardState::from_fen("8/1k6/8/8/7p/8/1K4P1/8 w - - 0 1").unwrap();
        let zobrist_hasher = ZobristHasher::create_zobrist_hasher();
        make_move(&mut board, "g2g4", &zobrist_hasher);
        make_move(&mut board, "h4g3", &zobrist_hasher);
        assert_eq!(board.board[7][8], Square::from(Piece::pawn(Black)));

        let mut pawn_count = 0;
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                if let Square::Full(Piece { kind, .. }) = board.board[i][j] {
                    if kind == Pawn {
                        pawn_count += 1;
                    }
                }
            }
        }
        assert_eq!(pawn_count, 1);
    }

    #[test]
    fn en_passant_capture_parsed_correctly_white() {
        let mut board = BoardState::from_fen("8/1k4p1/8/5P2/8/8/1K6/8 b - - 0 1").unwrap();
        let zobrist_hasher = ZobristHasher::create_zobrist_hasher();
        make_move(&mut board, "g7g5", &zobrist_hasher);
        make_move(&mut board, "f5g6", &zobrist_hasher);
        assert_eq!(board.board[4][8], Square::from(Piece::pawn(White)));

        let mut pawn_count = 0;
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                if let Square::Full(Piece { kind, .. }) = board.board[i][j] {
                    if kind == Pawn {
                        pawn_count += 1;
                    }
                }
            }
        }
        assert_eq!(pawn_count, 1);
    }

    #[test]
    fn full_game_played_white_wins() {
        let zobrist_hasher = ZobristHasher::create_zobrist_hasher();
        let mut draw_table: DrawTable = DrawTable::new();
        let commands: Vec<&str> = "position startpos moves g1f3 g8f6 d2d4 d7d5 e2e3 e7e6 f1d3 b8c6 b1c3 f8e7 e1g1 e8g8 a2a3 h7h6 b2b4 a7a6 c1b2 e7d6 a1c1 b7b5 h2h3 c8b7 f1e1 f8e8 g2g3 d8d7 e3e4 e6e5 c3d5 f6d5 e4d5 c6d4 f3d4 e5d4 d1h5 d6e7 b2d4 d7d5 h5d5 b7d5 c2c4 b5c4 d3c4 d5c4 c1c4 e7d6 e1e8 a8e8 c4c6 e8e1 g1g2 e1d1 d4e3 d1a1 c6a6 d6b4 a3a4 h6h5 a6a8 g8h7 a8a7 h7g6 a7c7 a1a4 c7c4 g6f6 e3d2 b4d2 c4a4 d2c3 g2f3 f6e6 f3e4 f7f5 e4e3 e6f7 e3f4 c3e1 f2f3 g7g6 a4a7 f7e6 f4g5 e1g3 a7a6 e6e5 g5g6 e5d4 a6e6 h5h4 g6f5 d4c3 e6e8 g3f2 e8d8 c3c4 f5g4 f2e1 f3f4 c4b3 f4f5 e1c3 g4g5 c3a5 d8e8 a5d2 g5h4 d2c3 h4g5 b3c4 f5f6 c3b2 f6f7 b2a3 g5g6 c4d5 h3h4 d5c4 h4h5 a3d6 h5h6 d6f8 e8f8 c4d5 f8d8 d5e5 f7f8q e5e4 f8f2 e4e5 f2f5".split(' ').collect();
        let board = play_out_position(&commands, &zobrist_hasher, &mut draw_table);
        let end_board = BoardState::from_fen("3R4/8/6KP/4kQ2/8/8/8/8 b - - 4 66").unwrap();

        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                assert_eq!(board.board[i][j], end_board.board[i][j]);
            }
        }
        assert_eq!(
            board.white_queen_side_castle,
            end_board.white_queen_side_castle
        );
        assert_eq!(
            board.white_king_side_castle,
            end_board.white_king_side_castle
        );
        assert_eq!(
            board.black_king_side_castle,
            end_board.black_king_side_castle
        );
        assert_eq!(
            board.black_queen_side_castle,
            end_board.black_queen_side_castle
        );
    }

    #[test]
    fn zobrist_hash_full_game_played_white_wins() {
        let zobrist_hasher = ZobristHasher::create_zobrist_hasher();
        let mut draw_table: DrawTable = DrawTable::new();
        let commands: Vec<&str> = "position startpos moves g1f3 g8f6 d2d4 d7d5 e2e3 e7e6 f1d3 b8c6 b1c3 f8e7 e1g1 e8g8 a2a3 h7h6 b2b4 a7a6 c1b2 e7d6 a1c1 b7b5 h2h3 c8b7 f1e1 f8e8 g2g3 d8d7 e3e4 e6e5 c3d5 f6d5 e4d5 c6d4 f3d4 e5d4 d1h5 d6e7 b2d4 d7d5 h5d5 b7d5 c2c4 b5c4 d3c4 d5c4 c1c4 e7d6 e1e8 a8e8 c4c6 e8e1 g1g2 e1d1 d4e3 d1a1 c6a6 d6b4 a3a4 h6h5 a6a8 g8h7 a8a7 h7g6 a7c7 a1a4 c7c4 g6f6 e3d2 b4d2 c4a4 d2c3 g2f3 f6e6 f3e4 f7f5 e4e3 e6f7 e3f4 c3e1 f2f3 g7g6 a4a7 f7e6 f4g5 e1g3 a7a6 e6e5 g5g6 e5d4 a6e6 h5h4 g6f5 d4c3 e6e8 g3f2 e8d8 c3c4 f5g4 f2e1 f3f4 c4b3 f4f5 e1c3 g4g5 c3a5 d8e8 a5d2 g5h4 d2c3 h4g5 b3c4 f5f6 c3b2 f6f7 b2a3 g5g6 c4d5 h3h4 d5c4 h4h5 a3d6 h5h6 d6f8 e8f8 c4d5 f8d8 d5e5 f7f8q e5e4 f8f2 e4e5 f2f5".split(' ').collect();
        let board = play_out_position(&commands, &zobrist_hasher, &mut draw_table);
        let end_board = BoardState::from_fen("3R4/8/6KP/4kQ2/8/8/8/8 b - - 4 66").unwrap();

        assert_eq!(board.zobrist_key, end_board.zobrist_key);
    }

    #[test]
    fn zobrist_hash_full_game_played_white_wins_2() {
        let zobrist_hasher = ZobristHasher::create_zobrist_hasher();
        let mut draw_table: DrawTable = DrawTable::new();
        // this game contains en-passant, castling and pawn promotion
        let commands: Vec<&str> = "position startpos moves e2e4 d7d5 e4e5 f7f5 e5f6 b8c6 f6g7 c8e6 g7h8q d8d6 d2d3 e8c8 d1h5 c6a5 h8g8 e6d7 g8f8 a5c6 h5g4 h7h6 g4a4 c6d4 a4a7 h6h5 a7a8".split(' ').collect();
        let board = play_out_position(&commands, &zobrist_hasher, &mut draw_table);
        let end_board =
            BoardState::from_fen("Q1kr1Q2/1ppbp3/3q4/3p3p/3n4/3P4/PPP2PPP/RNB1KBNR b KQ - 1 13")
                .unwrap();

        assert_eq!(board.zobrist_key, end_board.zobrist_key);
    }

    #[test]
    fn game_is_draw_three_fold_repetition() {
        let zobrist_hasher = ZobristHasher::create_zobrist_hasher();
        let mut draw_table: DrawTable = DrawTable::new();
        // this game contains en-passant, castling and pawn promotion
        let commands: Vec<&str> = "position fen 8/8/k7/p7/P7/K7/8/8 w - - 0 1 moves a3b3 a6b6 b3c4 b6c6 c4d4 c6d6 d4c4 d6c6 c4d4 c6d6 d4c4 d6c6".split(' ').collect();
        let board = play_out_position(&commands, &zobrist_hasher, &mut draw_table);
        assert_eq!(*draw_table.table.get(&board.zobrist_key).unwrap(), 3);
    }
}
