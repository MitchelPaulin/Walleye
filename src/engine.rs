pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};
use crate::configs;
pub use crate::evaluation::*;
pub use crate::move_generation::*;
pub use crate::uci::send_to_gui;
use std::cmp::{max, min, Reverse};
use std::time::Instant;

const MATE_SCORE: i32 = 100000;
const POS_INF: i32 = 9999999;
const NEG_INF: i32 = -POS_INF;
/*
    We want killer moves to be ordered behind all captures, but still ahead of other moves
    so pick a very negative value but still larger than i32::Min
*/
const KILLER_MOVE_SCORE: i32 = i32::MIN + 1;
const KILLER_MOVE_PLY_SIZE: usize = 2;
type KillerMoveArray =
    [[Option<(Point, Point)>; KILLER_MOVE_PLY_SIZE]; configs::MAX_DEPTH as usize];

type BoardSender = std::sync::mpsc::Sender<BoardState>;
type MoveArray = [Option<(Point, Point)>; configs::MAX_DEPTH as usize];

/*
    Capture extension, only search captures from here on to
    find a "quite" position
*/
fn quiesce(
    board: &BoardState,
    mut alpha: i32,
    beta: i32,
    depth: u32,
    nodes_searched: &mut u32,
) -> i32 {
    *nodes_searched += 1;
    let stand_pat = get_evaluation(board);
    if depth == 0 {
        return stand_pat;
    }
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let mut moves = generate_moves(board, MoveGenerationMode::CapturesOnly);
    moves.sort_unstable_by_key(|k| Reverse(k.order_heuristic));
    for mov in moves {
        let score = -quiesce(&mov, -beta, -alpha, depth - 1, nodes_searched);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

/*
    Run a standard alpha beta search to try and find the best move
    Orders moves by piece value to attempt to improve search efficiency
*/
fn alpha_beta_search(
    board: &BoardState,
    depth: u8,
    ply_from_root: i32,
    mut alpha: i32,
    mut beta: i32,
    killer_moves: &mut KillerMoveArray,
    pv_moves: &mut MoveArray,
    cur_line: &mut MoveArray,
    nodes_searched: &mut u32,
) -> i32 {
    *nodes_searched += 1;

    if depth == 0 {
        // look 10 captures into the future
        return quiesce(board, alpha, beta, 10, nodes_searched);
    }

    // Skip this position if a mating sequence has already been found earlier in
    // the search, which would be shorter than any mate we could find from here.
    alpha = max(alpha, -MATE_SCORE + ply_from_root);
    beta = min(beta, MATE_SCORE - ply_from_root);
    if alpha >= beta {
        return alpha;
    }

    let mut moves = generate_moves(board, MoveGenerationMode::AllMoves);
    if moves.is_empty() {
        if is_check(board, board.to_move) {
            // checkmate
            let mate_score = MATE_SCORE - ply_from_root;
            return -mate_score;
        }
        //stalemate
        return 0;
    }

    // rank killer moves and pv moves
    for mov in &mut moves {
        if mov.last_move == pv_moves[ply_from_root as usize] {
            // consider principle variation moves before anything else
            mov.order_heuristic = POS_INF;
        } else {
            for i in 0..KILLER_MOVE_PLY_SIZE {
                if mov.last_move == killer_moves[ply_from_root as usize][i] {
                    // if this move has a higher heuristic value already, we don't want to overwrite it
                    mov.order_heuristic = max(KILLER_MOVE_SCORE, mov.order_heuristic);
                }
            }
        }
    }

    moves.sort_unstable_by_key(|k| Reverse(k.order_heuristic));
    for mov in moves {
        let evaluation = -alpha_beta_search(
            &mov,
            depth - 1,
            ply_from_root + 1,
            -beta,
            -alpha,
            killer_moves,
            pv_moves,
            cur_line,
            nodes_searched,
        );

        cur_line[ply_from_root as usize] = mov.last_move;

        if evaluation >= beta {
            // beta cutoff, store the move for the "killer move" heuristic
            insert_killer_move(killer_moves, ply_from_root, &mov);
            return beta;
        }

        if evaluation > alpha {
            //alpha raised, remember this line as the pv
            pv_moves.clone_from_slice(cur_line);
            alpha = evaluation;
        }
    }

    alpha
}

/*
    Insert a killer move into the killer move array
    Shifts all killer moves down by one and puts the next killer move at the front
*/
fn insert_killer_move(killer_moves: &mut KillerMoveArray, ply_from_root: i32, mov: &BoardState) {
    let ply = ply_from_root as usize;
    for i in 0..(KILLER_MOVE_PLY_SIZE - 1) {
        killer_moves[ply][i + 1] = killer_moves[ply][i];
    }
    killer_moves[ply][0] = mov.last_move;
}

/*
    Interface to the alpha_beta function, works very similarly but returns a board state at the end
    and also operates with a channel to send the best board state found so far
*/
pub fn get_best_move(board: &BoardState, time_to_move: u128, tx: BoardSender) {
    let mut cur_depth = 1;
    let ply_from_root = 0;
    let start = Instant::now();
    let mut best_move: Option<BoardState> = None;

    // moves are accessed via [ply][slot]
    let mut killer_moves = [[None; KILLER_MOVE_PLY_SIZE]; configs::MAX_DEPTH as usize];
    let mut pv_moves = [None; configs::MAX_DEPTH as usize];
    let mut current_line = [None; configs::MAX_DEPTH as usize];

    let mut moves = generate_moves(board, MoveGenerationMode::AllMoves);

    while cur_depth < configs::MAX_DEPTH {
        let mut alpha = NEG_INF;
        let beta = POS_INF;
        let mut nodes_searched = 0;
        moves.sort_unstable_by_key(|k| Reverse(k.order_heuristic));
        for mov in &moves {
            // make an effort to exit once we are out of time
            if Instant::now().duration_since(start).as_millis() > time_to_move {
                // if we have not found a move to send back, send back the best move as determined by the order_heuristic
                // this can happen on very short time control situations
                if best_move.is_none() {
                    tx.send(moves[0]).unwrap();
                }
                return;
            }

            let evaluation = -alpha_beta_search(
                &mov,
                cur_depth - 1,
                ply_from_root + 1,
                -beta,
                -alpha,
                &mut killer_moves,
                &mut pv_moves,
                &mut current_line,
                &mut nodes_searched,
            );

            current_line[ply_from_root as usize] = mov.last_move;

            if evaluation > alpha {
                //alpha raised, remember this line as the pv
                alpha = evaluation;
                best_move = Some(*mov);
                tx.send(*mov).unwrap();
                pv_moves.clone_from_slice(&current_line);
                send_search_info(&pv_moves, cur_depth, nodes_searched, evaluation, start);
            }
        }
        moves = generate_moves(board, MoveGenerationMode::AllMoves);
        if let Some(b) = best_move {
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
fn send_search_info(
    pv_moves: &MoveArray,
    depth: u8,
    nodes_searched: u32,
    eval: i32,
    start: Instant,
) {
    let mut ponder_move = "".to_string();
    for mov in pv_moves {
        if let Some(m) = mov {
            ponder_move = format!("{} {}{}", ponder_move, m.0.to_string(), m.1.to_string())
        } else {
            break;
        }
    }
    send_to_gui(format!(
        "info pv{} depth {} nodes {} score cp {} time {}",
        ponder_move,
        depth,
        nodes_searched,
        eval,
        Instant::now().duration_since(start).as_millis()
    ));
}

/*
    A single threaded move generation function which will search to the desired depth
*/
pub fn get_best_move_synchronous(board: &BoardState, depth: u8) -> Option<BoardState> {
    let mut alpha = NEG_INF;
    let beta = POS_INF;
    let ply_from_root = 0;
    let mut best_move: Option<BoardState> = None;
    let mut nodes_searched = 0;

    // moves are accessed via [ply][slot]
    let mut killer_moves = [[None; KILLER_MOVE_PLY_SIZE]; configs::MAX_DEPTH as usize];
    let mut pv_moves = [None; configs::MAX_DEPTH as usize];
    let mut current_line = [None; configs::MAX_DEPTH as usize];

    let mut moves = generate_moves(board, MoveGenerationMode::AllMoves);
    moves.sort_unstable_by_key(|k| Reverse(k.order_heuristic));

    for mov in moves {
        let evaluation = -alpha_beta_search(
            &mov,
            depth - 1,
            ply_from_root + 1,
            -beta,
            -alpha,
            &mut killer_moves,
            &mut pv_moves,
            &mut current_line,
            &mut nodes_searched,
        );

        if evaluation > alpha {
            alpha = evaluation;
            best_move = Some(mov);
        }
    }

    best_move
}

/*
    Play a game in the terminal where the engine plays against itself
*/
pub fn play_game_against_self(b: &BoardState, max_moves: u8, depth: u8, simple_print: bool) {
    let mut board = b.clone();

    let show_board = |simple_print: bool, b: &BoardState| {
        if simple_print {
            b.simple_print_board()
        } else {
            b.pretty_print_board()
        }
    };

    show_board(simple_print, &board);
    for _ in 0..max_moves {
        let res = get_best_move_synchronous(&board, depth);
        board = match res {
            Some(b) => b,
            _ => break,
        };
        show_board(simple_print, &board);
    }
}
