pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};
pub use crate::move_generation::*;
use std::cmp;
use std::cmp::Reverse;

const MATE_SCORE : i32 = 100000;
const POS_INF : i32 = 9999999;
const NEG_INF : i32 = -POS_INF;

/*
    Evaluation function based on https://www.chessprogramming.org/Simplified_Evaluation_Function
*/

static PAWN_WEIGHTS: [[i32; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [50, 50, 50, 50, 50, 50, 50, 50],
    [10, 10, 20, 30, 30, 20, 10, 10],
    [5, 5, 10, 25, 25, 10, 5, 5],
    [0, 0, 0, 20, 20, 0, 0, 0],
    [5, -5, -10, 0, 0, -10, -5, 5],
    [5, 10, 10, -20, -20, 10, 10, 5],
    [0, 0, 0, 0, 0, 0, 0, 0],
];

static KNIGHT_WEIGHTS: [[i32; 8]; 8] = [
    [-50, -40, -30, -30, -30, -30, -40, -50],
    [-40, -20, 0, 0, 0, 0, -20, -40],
    [-30, 0, 10, 15, 15, 10, 0, -30],
    [-30, 5, 15, 20, 20, 15, 5, -30],
    [-30, 0, 15, 20, 20, 15, 0, -30],
    [-30, 5, 10, 15, 15, 10, 5, -30],
    [-40, -20, 0, 5, 5, 0, -20, -40],
    [-50, -40, -30, -30, -30, -30, -40, -50],
];

static BISHOP_WEIGHTS: [[i32; 8]; 8] = [
    [-20, -10, -10, -10, -10, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 10, 10, 5, 0, -10],
    [-10, 5, 5, 10, 10, 5, 5, -10],
    [-10, 0, 10, 10, 10, 10, 0, -10],
    [-10, 10, 10, 10, 10, 10, 10, -10],
    [-10, 5, 0, 0, 0, 0, 5, -10],
    [-20, -10, -10, -10, -10, -10, -10, -20],
];

static ROOK_WEIGHTS: [[i32; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 10, 10, 10, 10, 10, 10, 5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [0, 0, 0, 5, 5, 0, 0, 0],
];

static QUEEN_WEIGHTS: [[i32; 8]; 8] = [
    [-20, -10, -10, -5, -5, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 5, 5, 5, 0, -10],
    [-5, 0, 5, 5, 5, 5, 0, -5],
    [0, 0, 5, 5, 5, 5, 0, -5],
    [-10, 5, 5, 5, 5, 5, 0, -10],
    [-10, 0, 5, 0, 0, 0, 0, -10],
    [-20, -10, -10, -5, -5, -10, -10, -20],
];

static KING_WEIGHTS: [[i32; 8]; 8] = [
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-20, -30, -30, -40, -40, -30, -30, -20],
    [-10, -20, -20, -20, -20, -20, -20, -10],
    [20, 20, 0, 0, 0, 0, 20, 20],
    [20, 30, 10, 0, 0, 10, 30, 20],
];

static KING_LATE_GAME: [[i32; 8]; 8] = [
    [-50, -40, -30, -20, -20, -30, -40, -50],
    [-30, -20, -10, 0, 0, -10, -20, -30],
    [-30, -10, 20, 30, 30, 20, -10, -30],
    [-30, -10, 30, 40, 40, 30, -10, -30],
    [-30, -10, 30, 40, 40, 30, -10, -30],
    [-30, -10, 20, 30, 30, 20, -10, -30],
    [-30, -30, 0, 0, 0, 0, -30, -30],
    [-50, -30, -30, -30, -30, -30, -30, -50],
];

fn get_pos_evaluation(row: usize, col: usize, board: &BoardState, color: PieceColor) -> i32 {
    if let Square::Full(piece) = board.board[row][col] {
        let col = col - BOARD_START;
        let row = match color {
            PieceColor::White => row - BOARD_START,
            _ => 9 - row,
        };

        match piece.kind {
            Pawn => PAWN_WEIGHTS[row][col],
            Rook => ROOK_WEIGHTS[row][col],
            Bishop => BISHOP_WEIGHTS[row][col],
            Knight => KNIGHT_WEIGHTS[row][col],
            King => {
                if is_end_game(board) {
                    KING_LATE_GAME[row][col]
                } else {
                    KING_WEIGHTS[row][col]
                }
            }
            Queen => QUEEN_WEIGHTS[row][col],
        }
    } else {
        panic!("Could not recognize piece")
    }
}

fn is_end_game(board: &BoardState) -> bool {
    board.white_total_piece_value + board.black_total_piece_value < King.value() * 2 + 1600
}

/*
    Return how good a position is from the perspective of whose turn it is
*/
pub fn get_evaluation(board: &BoardState) -> i32 {
    let mut evaluation = match board.to_move {
        White => board.white_total_piece_value - board.black_total_piece_value,
        _ => board.black_total_piece_value - board.white_total_piece_value,
    };

    for row in BOARD_START..BOARD_END {
        for col in BOARD_START..BOARD_END {
            let square = board.board[row][col];
            if let Square::Full(Piece { color, .. }) = square {
                if color == board.to_move {
                    evaluation += get_pos_evaluation(row, col, board, color);
                } else {
                    evaluation -= get_pos_evaluation(row, col, board, color);
                }
            }
        }
    }
    evaluation
}

/*
    Run a standard alpha beta search to try and find the best move searching up to 'depth'
    Orders moves by piece value to attempt to improve search efficiency
*/
fn alpha_beta_search(board: &BoardState, depth: u8, ply_from_root: i32, mut alpha: i32, mut beta: i32) -> i32 {
    if depth == 0 {
        return get_evaluation(board);
    }

    let mut moves = generate_moves(board);
    if board.to_move == White {
        moves.sort_by_key(|b| Reverse(piece_value_differential(b)))
    } else {
        moves.sort_by_key(|a| piece_value_differential(a));
    }

    alpha = cmp::max(alpha, -MATE_SCORE + ply_from_root);
    beta = cmp::min(beta, MATE_SCORE - ply_from_root);
    if alpha >= beta {
        return alpha;
    }

    if moves.is_empty() {
        if is_check(board, board.to_move) {
            let mate_score = MATE_SCORE - ply_from_root;
            return -mate_score;
        }
        return 0;
    }

    for mov in moves {
        let evaluation = -alpha_beta_search(&mov, depth - 1, ply_from_root + 1, -beta, -alpha);
        if evaluation >= beta {
            return beta;
        }
        alpha = cmp::max(alpha, evaluation);
    }

    alpha
}

/*
    Interface to the alpha_beta function, works very similarly but returns a board state at the end
*/
pub fn get_best_move(board: &BoardState, depth: u8) -> Option<BoardState> {
    if depth == 0 {
        return Some(board.clone());
    }

    let mut alpha = NEG_INF;
    let beta = POS_INF;

    let mut moves = generate_moves(board);
    if board.to_move == White {
        moves.sort_by_key(|b| Reverse(piece_value_differential(b)))
    } else {
        moves.sort_by_key(|a| piece_value_differential(a));
    }

    if moves.is_empty() {
        if is_check(board, board.to_move) {
            return None; //checkmate
        }
        return None; // stalemate
    }

    let mut best_move = None;
    for mov in moves {
        let evaluation = -alpha_beta_search(&mov, depth - 1, 1, -beta, -alpha);
        if evaluation >= beta {
            return Some(mov);
        }
        if evaluation > alpha {
            alpha = evaluation;
            best_move = Some(mov);
        }
    }

    best_move
}

fn piece_value_differential(board: &BoardState) -> i32 {
    board.white_total_piece_value - board.black_total_piece_value
}

/*
    Play a game in the terminal where the engine plays against itself
*/
pub fn play_game_against_self(b: &BoardState, depth: u8, max_moves: u8, simple_print: bool) {
    let mut board = b.clone();

    let show_board = |simple_print: bool, b: &BoardState| {
        if simple_print {
            b.simple_print_board()
        } else {
            b.pretty_print_board()
        }
    };

    show_board(simple_print, &board);
    while board.full_move_clock < max_moves {
        let res = get_best_move(&board, depth);
        board = match res {
            Some(b) => b,
            _ => break,
        };
        show_board(simple_print, &board);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn position_evaluation() {
        let b = BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
        assert_eq!(get_evaluation(&b), 0);
    }

    #[test]
    fn position_evaluation2() {
        let b = BoardState::from_fen("rnbqkbnr/1ppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
        assert_eq!(get_evaluation(&b), 105);
    }
}
