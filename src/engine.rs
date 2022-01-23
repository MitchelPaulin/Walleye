pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};
pub use crate::evaluation::*;
pub use crate::move_generation::*;
pub use crate::search::{Search, KILLER_MOVE_PLY_SIZE, MAX_DEPTH};
pub use crate::uci::send_to_gui;
pub use crate::utils::out_of_time;
use crate::zobrist::{ZobristHasher, ZobristKey};
use std::cmp::{max, min, Reverse};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const MATE_SCORE: i32 = 100000;
const POS_INF: i32 = 9999999;
const NEG_INF: i32 = -POS_INF;
/*
    We want killer moves to be ordered behind all "good" captures, but still ahead of other moves
    For our purposes a good capture is capturing any with a piece of lower value

    Ex: capturing a pawn with a queen is a "bad" capture
        capturing a queen with a pawn is a "good" capture

    For this reason we give killer moves a -1, or ranked slightly below equal captures
*/
const KILLER_MOVE_SCORE: i32 = -1;

type BoardSender = std::sync::mpsc::Sender<BoardState>;
pub type DrawTable = HashMap<ZobristKey, u8>;

/*
    Capture extension, only search captures from here on to
    find a "quite" position
*/
fn quiesce(
    board: &BoardState,
    mut alpha: i32,
    beta: i32,
    search_info: &mut Search,
    zobrist_hasher: &ZobristHasher,
) -> i32 {
    search_info.node_searched();
    let stand_pat = get_evaluation(board);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let mut moves = generate_moves(board, MoveGenerationMode::CapturesOnly, zobrist_hasher);
    moves.sort_unstable_by_key(|k| Reverse(k.order_heuristic));
    for mov in moves {
        let score = -quiesce(&mov, -beta, -alpha, search_info, zobrist_hasher);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

fn remove_board_from_draw_table(board: &BoardState, draw_table: &mut DrawTable) {
    if let Some(&val) = draw_table.get(&board.zobrist_key) {
        draw_table.insert(board.zobrist_key, if val > 0 { val - 1 } else { 0 });
    }
}

/*
    Run a standard alpha beta search to try and find the best move
    Orders moves by piece value to attempt to improve search efficiency
*/
fn alpha_beta_search(
    start: Instant,
    time_to_move_ms: u128,
    board: &BoardState,
    mut depth: u8,
    ply_from_root: i32,
    mut alpha: i32,
    mut beta: i32,
    search_info: &mut Search,
    allow_null: bool,
    zobrist_hasher: &ZobristHasher,
    draw_table: &mut DrawTable,
) -> i32 {
    // we are out of time, exit the search
    if out_of_time(start, time_to_move_ms) {
        remove_board_from_draw_table(board, draw_table);
        return NEG_INF;
    }

    search_info.node_searched();

    // check for three fold repetition
    if let Some(&val) = draw_table.get(&board.zobrist_key) {
        if val == 2 {
            return 0; // this position has been seen twice before, so its a draw, return an eval of 0
        } else {
            draw_table.insert(board.zobrist_key, val + 1);
        }
    } else {
        draw_table.insert(board.zobrist_key, 1);
    }

    if depth == 0 {
        // need to resolve check before we enter quiesce
        if is_check(board, board.to_move) {
            depth += 1;
        } else {
            remove_board_from_draw_table(board, draw_table);
            return quiesce(board, alpha, beta, search_info, zobrist_hasher);
        }
    }

    // Skip this position if a mating sequence has already been found earlier in
    // the search, which would be shorter than any mate we could find from here.
    alpha = max(alpha, -MATE_SCORE + ply_from_root);
    beta = min(beta, MATE_SCORE - ply_from_root);
    if alpha >= beta {
        remove_board_from_draw_table(board, draw_table);
        return alpha;
    }

    // Null move pruning https://www.chessprogramming.org/Null_Move_Pruning
    // With R = 2
    if allow_null && depth >= 3 && !is_check(board, board.to_move) {
        // allow this player to go again
        let mut b = board.clone();
        b.to_move = board.to_move.opposite();
        let eval = -alpha_beta_search(
            start,
            time_to_move_ms,
            &b,
            depth - 3,
            ply_from_root + 10, //hack for now but passing in a large ply ensures we don't overwrite the pv
            -beta,
            -beta + 1,
            search_info,
            false,
            zobrist_hasher,
            draw_table,
        );

        if eval >= beta {
            // null move prune
            remove_board_from_draw_table(board, draw_table);
            return beta;
        }
    }

    let mut moves = generate_moves(board, MoveGenerationMode::AllMoves, zobrist_hasher);
    if moves.is_empty() {
        if is_check(board, board.to_move) {
            // checkmate
            remove_board_from_draw_table(board, draw_table);
            let mate_score = MATE_SCORE - ply_from_root;
            return -mate_score;
        }
        // stalemate
        remove_board_from_draw_table(board, draw_table);
        return 0;
    }

    // rank killer moves and pv moves
    for mov in &mut moves {
        if mov.last_move == search_info.pv_moves[ply_from_root as usize] {
            // consider principle variation moves before anything else
            mov.order_heuristic = POS_INF;
        } else {
            for i in 0..KILLER_MOVE_PLY_SIZE {
                if mov.last_move == search_info.killer_moves[ply_from_root as usize][i] {
                    mov.order_heuristic = KILLER_MOVE_SCORE;
                    break;
                }
            }
        }
    }

    moves.sort_unstable_by_key(|k| Reverse(k.order_heuristic));
    search_info.insert_into_cur_line(ply_from_root, &moves[0]);
    if moves[0].order_heuristic != POS_INF {
        search_info.set_principle_variation();
    }

    // do a full search with what we think is the best move
    // which should be the first move in the array
    let mut best_score = -alpha_beta_search(
        start,
        time_to_move_ms,
        &moves[0],
        depth - 1,
        ply_from_root + 1,
        -beta,
        -alpha,
        search_info,
        true,
        zobrist_hasher,
        draw_table,
    );

    if best_score > alpha {
        if best_score >= beta {
            remove_board_from_draw_table(board, draw_table);
            return best_score;
        }
        search_info.set_principle_variation();
        alpha = best_score;
    }

    // https://en.wikipedia.org/wiki/Principal_variation_search
    // try out all remaining moves with a reduced window
    for mov in moves.iter().skip(1) {
        search_info.insert_into_cur_line(ply_from_root, mov);
        // zero window search
        let mut score = -alpha_beta_search(
            start,
            time_to_move_ms,
            mov,
            depth - 1,
            ply_from_root + 1,
            -alpha - 1,
            -alpha,
            search_info,
            true,
            zobrist_hasher,
            draw_table,
        );

        if score > alpha && score < beta {
            // got a result outside our window, need to redo full search
            score = -alpha_beta_search(
                start,
                time_to_move_ms,
                mov,
                depth - 1,
                ply_from_root + 1,
                -beta,
                -alpha,
                search_info,
                true,
                zobrist_hasher,
                draw_table,
            );

            if score > alpha {
                alpha = score;
            }
        }

        if score > best_score {
            if score >= beta {
                if mov.order_heuristic == i32::MIN {
                    search_info.insert_killer_move(ply_from_root, mov);
                }
                remove_board_from_draw_table(board, draw_table);
                return score;
            }
            search_info.set_principle_variation();
            best_score = score;
        }
    }

    remove_board_from_draw_table(board, draw_table);

    best_score
}

/*
    Interface to the alpha_beta function, works very similarly but returns a board state at the end
    and also operates with a channel to send the best board state found so far
*/
pub fn get_best_move(
    board: &BoardState,
    draw_table: &mut DrawTable,
    start: Instant,
    time_to_move_ms: u128,
    tx: &BoardSender,
) {
    let mut cur_depth = 1;
    let ply_from_root = 0;
    let mut best_move: Option<BoardState> = None;

    let mut search_info = Search::new_search();
    let zobrist_hasher = ZobristHasher::create_zobrist_hasher();

    let mut moves = generate_moves(board, MoveGenerationMode::AllMoves, &zobrist_hasher);

    while cur_depth < MAX_DEPTH {
        let mut alpha = NEG_INF;
        let beta = POS_INF;
        search_info.reset_search();
        moves.sort_unstable_by_key(|k| Reverse(k.order_heuristic));
        for mov in &moves {
            // make an effort to exit once we are out of time
            if out_of_time(start, time_to_move_ms) {
                // if we have not found a move to send back, send back the best move as determined by the order_heuristic
                // this can happen on very short time control situations
                if best_move.is_none() {
                    tx.send(moves[0].clone()).unwrap();
                }
                return;
            }

            let evaluation = -alpha_beta_search(
                start,
                time_to_move_ms,
                mov,
                cur_depth - 1,
                ply_from_root + 1,
                -beta,
                -alpha,
                &mut search_info,
                true,
                &zobrist_hasher,
                draw_table,
            );

            search_info.insert_into_cur_line(ply_from_root, mov);

            if evaluation > alpha && !out_of_time(start, time_to_move_ms) {
                //alpha raised, remember this line as the pv
                alpha = evaluation;
                best_move = Some(mov.clone());
                tx.send(mov.clone()).unwrap();
                search_info.set_principle_variation();
                send_search_info(&search_info, cur_depth, evaluation, start);
            }
        }
        moves = generate_moves(board, MoveGenerationMode::AllMoves, &zobrist_hasher);
        if let Some(b) = &best_move {
            for mov in &mut moves {
                if mov.last_move == b.last_move {
                    mov.order_heuristic = POS_INF;
                    break;
                }
            }
        }
        cur_depth += 1;
    }
}

/*
    Send information about the current search status to the GUI
*/
fn send_search_info(search_info: &Search, depth: u8, eval: i32, start: Instant) {
    let mut ponder_move = "".to_string();
    for mov in &search_info.pv_moves {
        if let Some(m) = mov {
            ponder_move = format!("{} {}{}", ponder_move, m.0.to_string(), m.1.to_string())
        } else {
            break;
        }
    }

    let mate_window = 15;
    if eval >= MATE_SCORE - mate_window {
        // this player is threatening checkmate
        send_to_gui(&format!(
            "info pv{} depth {} nodes {} score mate {} time {}",
            ponder_move,
            depth,
            search_info.nodes_searched,
            (MATE_SCORE - eval + 1) / 2,
            Instant::now().duration_since(start).as_millis()
        ));
    } else if eval <= -MATE_SCORE + mate_window {
        // this player is getting matted
        send_to_gui(&format!(
            "info pv{} depth {} nodes {} score mate {} time {}",
            ponder_move,
            depth,
            search_info.nodes_searched,
            (MATE_SCORE + eval) / -2,
            Instant::now().duration_since(start).as_millis()
        ));
    } else {
        send_to_gui(&format!(
            "info pv{} depth {} nodes {} score cp {} time {}",
            ponder_move,
            depth,
            search_info.nodes_searched,
            eval,
            Instant::now().duration_since(start).as_millis()
        ));
    }
}

/*
    Play a game in the terminal where the engine plays against itself
*/
pub fn play_game_against_self(
    b: &BoardState,
    max_moves: u8,
    time_to_move_ms: u128,
    simple_print: bool,
) {
    let show_board = |simple_print: bool, b: &BoardState| {
        if simple_print {
            b.simple_print_board()
        } else {
            b.pretty_print_board()
        }
    };

    let mut board = b.clone();
    let draw_table: DrawTable = HashMap::new();
    show_board(simple_print, &board);
    for _ in 0..max_moves {
        let (tx, rx) = mpsc::channel();
        let start = Instant::now();
        let clone = board.clone();
        let mut draw_clone = draw_table.clone();
        thread::spawn(move || get_best_move(&clone, &mut draw_clone, start, time_to_move_ms, &tx));
        while !out_of_time(start, time_to_move_ms) {
            if let Ok(b) = rx.try_recv() {
                board = b;
            } else {
                thread::sleep(Duration::from_millis(1));
            }
        }
        show_board(simple_print, &board);
    }
}
