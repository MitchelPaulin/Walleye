#![allow(dead_code)]
pub use crate::board::*;
pub use crate::move_generation::*;
use std::cmp;

/*
    Evaluation function based on https://www.chessprogramming.org/Simplified_Evaluation_Function
*/

static PIECE_VALUES: [i32; 7] = [0, 100, 320, 330, 500, 900, 20000];

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
    let piece = board.board[row][col] & PIECE_MASK;
    let mut _row = row - BOARD_START;
    let _col = col - BOARD_START;
    if color == PieceColor::Black {
        _row = BOARD_END - BOARD_START - 1 - _row;
    }

    match piece {
        PAWN => PAWN_WEIGHTS[_row][_col],
        ROOK => ROOK_WEIGHTS[_row][_col],
        BISHOP => BISHOP_WEIGHTS[_row][_col],
        KNIGHT => KNIGHT_WEIGHTS[_row][_col],
        KING => KING_WEIGHTS[_row][_col],
        QUEEN => QUEEN_WEIGHTS[_row][_col],
        _ => panic!("Could not recognize piece"),
    }
}

/*
    Return a number to represent how good a certain position is

    White will attempt to "maximize" this score while black will attempt to "minimize" it
*/
pub fn get_evaluation(board: &BoardState) -> i32 {
    let mut evaluation = 0;
    for row in BOARD_START..BOARD_END {
        for col in BOARD_START..BOARD_END {
            let square = board.board[row][col];
            if is_empty(square) {
                continue;
            }

            if get_color(square) == Some(PieceColor::White) {
                evaluation += PIECE_VALUES[(square & PIECE_MASK) as usize];
            } else {
                evaluation += PIECE_VALUES[(square & PIECE_MASK) as usize] * -1;
            }
            evaluation += get_pos_evaluation(row, col, board, PieceColor::Black) * -1;
            evaluation += get_pos_evaluation(row, col, board, PieceColor::White);
        }
    }
    return evaluation;
}

pub fn alpha_beta_search(
    board: &BoardState,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: PieceColor,
) -> i32 {
    if depth == 0 {
        return get_evaluation(board);
    }

    let states = generate_moves(board);
    if states.len() == 0 {
        return get_evaluation(board);
    }

    if maximizing_player == PieceColor::White {
        let mut val = i32::MIN;
        for board in states {
            val = cmp::max(
                val,
                alpha_beta_search(&board, depth - 1, alpha, beta, PieceColor::Black),
            );
            if val >= beta {
                break;
            }
            alpha = cmp::max(alpha, val);
        }
        return val;
    } else {
        let mut val = i32::MAX;
        for board in states {
            val = cmp::min(
                val,
                alpha_beta_search(&board, depth - 1, alpha, beta, PieceColor::White),
            );
            if val <= alpha {
                break;
            }
            beta = cmp::min(beta, val);
        }
        return val;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn right_values() {
        assert_eq!(PIECE_VALUES[PAWN as usize], 100);
        assert_eq!(PIECE_VALUES[KNIGHT as usize], 320);
        assert_eq!(PIECE_VALUES[BISHOP as usize], 330);
        assert_eq!(PIECE_VALUES[ROOK as usize], 500);
        assert_eq!(PIECE_VALUES[QUEEN as usize], 900);
        assert_eq!(PIECE_VALUES[KING as usize], 20000);
    }
}
