pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};

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
                    evaluation += get_pos_evaluation(row, col, board, color) + kind.value();
                } else {
                    evaluation -= get_pos_evaluation(row, col, board, color) + kind.value();
                }
            }
        }
    }

    // an approximation of when the end game has started
    let is_end_game = total_piece_value <= King.value() * 2 + Bishop.value() * 2 + Pawn.value() * 5;


    if board.to_move == White {
        evaluation += get_pos_evaluation_king(&board, White, is_end_game)
            - get_pos_evaluation_king(&board, Black, is_end_game);
    } else {
        evaluation += get_pos_evaluation_king(&board, Black, is_end_game)
            - get_pos_evaluation_king(&board, White, is_end_game);
    }

    evaluation
}

/*
    Get the pos evaluation for every piece excluding the King
*/
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
            Queen => QUEEN_WEIGHTS[row][col],
            _ => 0,
        }
    } else {
        panic!("Could not recognize piece")
    }
}

/*
    Get the pos evaluation for the kings
    We make this a separate function in order to avoid looping over the board more than once
*/
fn get_pos_evaluation_king(board: &BoardState, color: PieceColor, is_end_game: bool) -> i32 {
    if color == White {
        if is_end_game {
            return KING_LATE_GAME[board.white_king_location.0 - BOARD_START]
                [board.white_king_location.1 - BOARD_START];
        }
        KING_WEIGHTS[board.white_king_location.0 - BOARD_START]
            [board.white_king_location.1 - BOARD_START]
    } else {
        if is_end_game {
            return KING_LATE_GAME[9 - board.black_king_location.0]
                [board.black_king_location.1 - BOARD_START];
        }
        KING_WEIGHTS[9 - board.black_king_location.0][board.black_king_location.1 - BOARD_START]
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
