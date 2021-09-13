pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};

/*
    Evaluation function based on https://www.chessprogramming.org/Simplified_Evaluation_Function
*/

const PAWN_WEIGHTS: [[i32; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [50, 50, 50, 50, 50, 50, 50, 50],
    [10, 10, 20, 30, 30, 20, 10, 10],
    [5, 5, 10, 25, 25, 10, 5, 5],
    [0, 0, 0, 20, 20, 0, 0, 0],
    [5, -5, -10, 0, 0, -10, -5, 5],
    [5, 10, 10, -20, -20, 10, 10, 5],
    [0, 0, 0, 0, 0, 0, 0, 0],
];

const KNIGHT_WEIGHTS: [[i32; 8]; 8] = [
    [-50, -40, -30, -30, -30, -30, -40, -50],
    [-40, -20, 0, 0, 0, 0, -20, -40],
    [-30, 0, 10, 15, 15, 10, 0, -30],
    [-30, 5, 15, 20, 20, 15, 5, -30],
    [-30, 0, 15, 20, 20, 15, 0, -30],
    [-30, 5, 10, 15, 15, 10, 5, -30],
    [-40, -20, 0, 5, 5, 0, -20, -40],
    [-50, -40, -30, -30, -30, -30, -40, -50],
];

const BISHOP_WEIGHTS: [[i32; 8]; 8] = [
    [-20, -10, -10, -10, -10, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 10, 10, 5, 0, -10],
    [-10, 5, 5, 10, 10, 5, 5, -10],
    [-10, 0, 10, 10, 10, 10, 0, -10],
    [-10, 10, 10, 10, 10, 10, 10, -10],
    [-10, 5, 0, 0, 0, 0, 5, -10],
    [-20, -10, -10, -10, -10, -10, -10, -20],
];

const ROOK_WEIGHTS: [[i32; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 10, 10, 10, 10, 10, 10, 5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [0, 0, 0, 5, 5, 0, 0, 0],
];

const QUEEN_WEIGHTS: [[i32; 8]; 8] = [
    [-20, -10, -10, -5, -5, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 5, 5, 5, 0, -10],
    [-5, 0, 5, 5, 5, 5, 0, -5],
    [0, 0, 5, 5, 5, 5, 0, -5],
    [-10, 5, 5, 5, 5, 5, 0, -10],
    [-10, 0, 5, 0, 0, 0, 0, -10],
    [-20, -10, -10, -5, -5, -10, -10, -20],
];

const KING_WEIGHTS: [[i32; 8]; 8] = [
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-20, -30, -30, -40, -40, -30, -30, -20],
    [-10, -20, -20, -20, -20, -20, -20, -10],
    [20, 20, 0, 0, 0, 0, 20, 20],
    [20, 30, 10, 0, 0, 10, 30, 20],
];

const KING_LATE_GAME: [[i32; 8]; 8] = [
    [-50, -40, -30, -20, -20, -30, -40, -50],
    [-30, -20, -10, 0, 0, -10, -20, -30],
    [-30, -10, 20, 30, 30, 20, -10, -30],
    [-30, -10, 30, 40, 40, 30, -10, -30],
    [-30, -10, 30, 40, 40, 30, -10, -30],
    [-30, -10, 20, 30, 30, 20, -10, -30],
    [-30, -30, 0, 0, 0, 0, -30, -30],
    [-50, -30, -30, -30, -30, -30, -30, -50],
];

// approximated at 2 kings, 2 bishops and 10 pawns
const END_GAME_MATERIAL_VALUE: i32 = 41660;

/*
    Return how good a position is from the perspective of the current player
*/
pub fn get_evaluation(board: &BoardState) -> i32 {
    let mut evaluation = 0;
    let mut total_piece_value = 0;
    for row in BOARD_START..BOARD_END {
        for col in BOARD_START..BOARD_END {
            let square = board.board[row][col];
            if let Square::Full(Piece { color, kind }) = square {
                total_piece_value += kind.value();
                if color == board.to_move {
                    evaluation += get_pos_evaluation(row, col, color, kind) + kind.value();
                } else {
                    evaluation -= get_pos_evaluation(row, col, color, kind) + kind.value();
                }
            }
        }
    }

    // an approximation of when the end game has started
    let is_end_game = total_piece_value <= END_GAME_MATERIAL_VALUE;
    let white_king_eval = get_pos_evaluation_king_white(&board, is_end_game);
    let black_king_eval = get_pos_evaluation_king_black(&board, is_end_game);
    evaluation += match board.to_move {
        White => white_king_eval - black_king_eval,
        Black => black_king_eval - white_king_eval,
    };

    evaluation
}

/*
    Get the pos evaluation for every piece excluding the King
*/
fn get_pos_evaluation(row: usize, col: usize, color: PieceColor, kind: PieceKind) -> i32 {
    let col = col - BOARD_START;
    let row = match color {
        White => row - BOARD_START,
        Black => 9 - row,
    };

    match kind {
        Pawn => PAWN_WEIGHTS[row][col],
        Rook => ROOK_WEIGHTS[row][col],
        Bishop => BISHOP_WEIGHTS[row][col],
        Knight => KNIGHT_WEIGHTS[row][col],
        Queen => QUEEN_WEIGHTS[row][col],
        _ => 0,
    }
}

/*
    Get the pos evaluation for the kings
    We make this a separate function in order to avoid looping over the board more than once
*/

fn get_pos_evaluation_king_white(board: &BoardState, is_end_game: bool) -> i32 {
    if is_end_game {
        return KING_LATE_GAME[board.white_king_location.0 - BOARD_START]
            [board.white_king_location.1 - BOARD_START];
    }
    KING_WEIGHTS[board.white_king_location.0 - BOARD_START]
        [board.white_king_location.1 - BOARD_START]
}

fn get_pos_evaluation_king_black(board: &BoardState, is_end_game: bool) -> i32 {
    if is_end_game {
        return KING_LATE_GAME[9 - board.black_king_location.0]
            [board.black_king_location.1 - BOARD_START];
    }
    KING_WEIGHTS[9 - board.black_king_location.0][board.black_king_location.1 - BOARD_START]
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
