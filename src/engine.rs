pub use crate::board::*;

/*
    Generate pseudo-legal moves for a knight
*/
pub fn knight_moves(row: i8, col: i8, piece: u8, board: &Board, moves: &mut Vec<(usize, usize)>) {
    let cords = [
        (1, 2),
        (1, -2),
        (2, 1),
        (2, -1),
        (-1, 2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
    ];
    for mods in cords.iter() {
        let _row = (row + mods.0) as usize;
        let _col = (col + mods.1) as usize;
        let square = board.board[_row][_col];

        if is_outside_board(board.board[_row][_col]) {
            continue;
        }

        if is_empty(square) || square & COLOR_MASK != piece & COLOR_MASK {
            moves.push((_row, _col));
        }
    }
}

/*
    Generate pseudo-legal moves for a pawn
*/
pub fn pawn_moves(row: i8, col: i8, piece: u8, board: &Board, moves: &mut Vec<(usize, usize)>) {
    // TODO en passant

    // white pawns move up board
    if is_white(piece) {
        // check capture
        let left_cap = board.board[(row - 1) as usize][(col - 1) as usize];
        let right_cap = board.board[(row - 1) as usize][(col + 1) as usize];
        if !is_outside_board(left_cap) && is_black(left_cap) {
            moves.push(((row - 1) as usize, (col - 1) as usize));
        }
        if !is_outside_board(right_cap) && is_black(right_cap) {
            moves.push(((row - 1) as usize, (col + 1) as usize));
        }

        // check a normal push
        if is_empty(board.board[(row - 1) as usize][col as usize]) {
            moves.push(((row - 1) as usize, col as usize));
            // check double push
            if row == 8 && is_empty(board.board[(row - 2) as usize][col as usize]) {
                moves.push(((row - 2) as usize, col as usize));
            }
        }
    } else {
        // check capture
        let left_cap = board.board[(row + 1) as usize][(col + 1) as usize];
        let right_cap = board.board[(row + 1) as usize][(col - 1) as usize];
        if !is_outside_board(left_cap) && is_white(left_cap) {
            moves.push(((row + 1) as usize, (col + 1) as usize));
        }
        if !is_outside_board(right_cap) && is_white(right_cap) {
            moves.push(((row + 1) as usize, (col - 1) as usize));
        }

        // check a normal push
        if is_empty(board.board[(row + 1) as usize][col as usize]) {
            moves.push(((row + 1) as usize, col as usize));
            // check double push
            if row == 3 && is_empty(board.board[(row + 2) as usize][col as usize]) {
                moves.push(((row + 2) as usize, col as usize));
            }
        }
    }
}

/*
    Generate pseudo-legal moves for a king
*/
pub fn king_moves(row: i8, col: i8, piece: u8, board: &Board, moves: &mut Vec<(usize, usize)>) {
    for i in -1..2 {
        for j in -1..2 {
            let _row = (row + i) as usize;
            let _col = (col + j) as usize;

            if is_outside_board(board.board[_row][_col]) {
                continue;
            }

            if is_empty(board.board[_row][_col])
                || board.board[_row][_col] & COLOR_MASK != piece & COLOR_MASK
            {
                moves.push((_row, _col));
            }
        }
    }
}

/*
    Generate pseudo-legal moves for a rook
*/
pub fn rook_moves(row: i8, col: i8, piece: u8, board: &Board, moves: &mut Vec<(usize, usize)>) {
    let mods = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    for m in mods.iter() {
        let mut multiplier = 1;
        let mut _row = row + m.0;
        let mut _col = col + m.1;
        let mut square = board.board[_row as usize][_col as usize];
        while is_empty(square) {
            moves.push((_row as usize, _col as usize));
            multiplier += 1;
            _row = row + m.0 * multiplier;
            _col = col + m.1 * multiplier;
            square = board.board[_row as usize][_col as usize];
        }

        if !is_outside_board(square) && piece & COLOR_MASK != square & COLOR_MASK {
            moves.push((_row as usize, _col as usize));
        }
    }
}

/*
    Generate pseudo-legal moves for a bishop
*/
pub fn bishop_moves(row: i8, col: i8, piece: u8, board: &Board, moves: &mut Vec<(usize, usize)>) {
    let mods = [1, -1];
    for i in mods.iter() {
        for j in mods.iter() {
            let mut multiplier = 1;
            let mut _row = row + i;
            let mut _col = col + j;
            let mut square = board.board[_row as usize][_col as usize];
            while is_empty(square) {
                moves.push((_row as usize, _col as usize));
                multiplier += 1;
                _row = row + i * multiplier;
                _col = col + j * multiplier;
                square = board.board[_row as usize][_col as usize];
            }

            if !is_outside_board(square) && piece & COLOR_MASK != square & COLOR_MASK {
                moves.push((_row as usize, _col as usize));
            }
        }
    }
}

/*
    Generate pseudo-legal moves for a queen
*/
pub fn queen_moves(row: i8, col: i8, piece: u8, board: &Board, moves: &mut Vec<(usize, usize)>) {
    rook_moves(row, col, piece, board, moves);
    bishop_moves(row, col, piece, board, moves);
}

/*
    Generate pseudo-legal moves for a piece
*/
pub fn get_moves(row: i8, col: i8, piece: u8, board: &Board, moves: &mut Vec<(usize, usize)>) {
    match piece & PIECE_MASK {
        PAWN => pawn_moves(row, col, piece, board, moves),
        ROOK => rook_moves(row, col, piece, board, moves),
        BISHOP => bishop_moves(row, col, piece, board, moves),
        KNIGHT => knight_moves(row, col, piece, board, moves),
        KING => king_moves(row, col, piece, board, moves),
        QUEEN => queen_moves(row, col, piece, board, moves),
        _ => panic!("Unrecognized piece"),
    }
}

/*
    Determine if the current position is check
*/
pub fn is_check(board: &Board, color: u8) -> bool {
    return true;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Knight tests

    #[test]
    fn knight_moves_empty_board() {
        let b = board_from_fen("8/8/8/8/3N4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 6;
        let col = 5;
        knight_moves(row, col, WHITE | KNIGHT, &b, &mut ret);
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn knight_moves_corner() {
        let b = board_from_fen("N7/8/8/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 2;
        let col = 2;
        knight_moves(row, col, BLACK | KNIGHT, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }
    #[test]
    fn knight_moves_with_other_pieces_with_capture() {
        let b = board_from_fen("8/8/5n2/3NQ3/2K2P2/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 5;
        let col = 5;
        knight_moves(row, col, WHITE | KNIGHT, &b, &mut ret);
        assert_eq!(ret.len(), 7);
    }

    // Pawn tests - white pawn

    #[test]
    fn white_pawn_double_push() {
        let b = board_from_fen("8/8/8/8/8/8/P7/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 8;
        let col = 2;
        pawn_moves(row, col, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn white_pawn_has_moved() {
        let b = board_from_fen("8/8/8/8/8/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 7;
        let col = 5;
        pawn_moves(row, col, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn white_pawn_cant_move_black_piece_block() {
        let b = board_from_fen("8/8/8/8/3r4/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 7;
        let col = 5;
        pawn_moves(row, col, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_pawn_cant_move_white_piece_block() {
        let b = board_from_fen("8/8/8/8/3K4/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 7;
        let col = 5;
        pawn_moves(row, col, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_pawn_with_two_captures_and_start() {
        let b = board_from_fen("8/8/8/8/8/n1q5/1P6/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 8;
        let col = 3;
        pawn_moves(row, col, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn white_pawn_with_one_capture() {
        let b = board_from_fen("8/8/Q1b5/1P6/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 5;
        let col = 3;
        pawn_moves(row, col, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn white_pawn_double_push_piece_in_front() {
        let b = board_from_fen("8/8/8/8/8/b7/P7/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 8;
        let col = 2;
        pawn_moves(row, col, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    // Pawn tests - black pawn

    #[test]
    fn black_pawn_double_push() {
        let b = board_from_fen("8/p7/8/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 3;
        let col = 2;
        pawn_moves(row, col, BLACK | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn black_pawn_has_moved() {
        let b = board_from_fen("8/8/8/3p4/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 5;
        let col = 5;
        pawn_moves(row, col, BLACK | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn black_pawn_cant_move_white_piece_block() {
        let b = board_from_fen("8/3p4/3R4/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 3;
        let col = 5;
        pawn_moves(row, col, BLACK | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn black_pawn_with_two_captures_and_start() {
        let b = board_from_fen("8/3p4/2R1R3/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 3;
        let col = 5;
        pawn_moves(row, col, BLACK | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn black_pawn_with_one_capture() {
        let b = board_from_fen("8/3p4/3qR3/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 3;
        let col = 5;
        pawn_moves(row, col, BLACK | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 1);
    }

    // Piece test - king

    #[test]
    fn king_empty_board_center() {
        let b = board_from_fen("8/8/8/8/3K4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 5;
        let col = 6;
        king_moves(row, col, WHITE | KING, &b, &mut ret);
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn king_start_pos() {
        let b = board_from_fen("8/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 9;
        let col = 6;
        king_moves(row, col, WHITE | KING, &b, &mut ret);
        assert_eq!(ret.len(), 5);
    }

    #[test]
    fn king_start_pos_other_pieces() {
        let b = board_from_fen("8/8/8/8/8/8/3Pn3/3QKB2 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 9;
        let col = 6;
        king_moves(row, col, WHITE | KING, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn king_black_other_pieces() {
        let b = board_from_fen("8/8/8/8/8/3Pn3/3QkB2/3R1q2 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 8;
        let col = 6;
        king_moves(row, col, BLACK | KING, &b, &mut ret);
        assert_eq!(ret.len(), 6);
    }

    // Piece test - rook
    #[test]
    fn rook_center_of_empty_board() {
        let b = board_from_fen("8/8/8/8/3R4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 6;
        let col = 5;
        rook_moves(row, col, WHITE | ROOK, &b, &mut ret);
        assert_eq!(ret.len(), 14);
    }

    #[test]
    fn rook_center_of_board() {
        let b = board_from_fen("8/8/8/3q4/2kRp3/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 6;
        let col = 5;
        rook_moves(row, col, WHITE | ROOK, &b, &mut ret);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn rook_center_of_board_with_white_pieces() {
        let b = board_from_fen("7p/3N4/8/4n3/2kR4/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 6;
        let col = 5;
        rook_moves(row, col, WHITE | ROOK, &b, &mut ret);
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn rook_corner() {
        let b = board_from_fen("7p/3N4/8/4n3/2kR4/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 9;
        let col = 9;
        rook_moves(row, col, WHITE | ROOK, &b, &mut ret);
        assert_eq!(ret.len(), 14);
    }
    #[test]
    fn black_rook_center_of_board_with_white_pieces() {
        let b = board_from_fen("7p/3N4/8/4n3/2kR4/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 6;
        let col = 5;
        rook_moves(row, col, BLACK | ROOK, &b, &mut ret);
        assert_eq!(ret.len(), 7);
    }

    // Piece test - bishop
    #[test]
    fn black_bishop_center_empty_board() {
        let b = board_from_fen("8/8/8/3b4/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 5;
        let col = 5;
        bishop_moves(row, col, BLACK | BISHOP, &b, &mut ret);
        assert_eq!(ret.len(), 13);
    }

    #[test]
    fn black_bishop_center_with_captures() {
        let b = board_from_fen("6P1/8/8/3b4/8/1R6/8/3Q4 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 5;
        let col = 5;
        bishop_moves(row, col, BLACK | BISHOP, &b, &mut ret);
        assert_eq!(ret.len(), 12);
    }

    #[test]
    fn black_bishop_center_with_captures_and_black_pieces() {
        let b = board_from_fen("6P1/8/2Q5/3b4/2k1n3/1R6/8/b2Q4 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 5;
        let col = 5;
        bishop_moves(row, col, BLACK | BISHOP, &b, &mut ret);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn white_bishop_center_with_captures_and_white_pieces() {
        let b = board_from_fen("8/8/8/4r3/5B2/8/3Q4/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 6;
        let col = 7;
        bishop_moves(row, col, WHITE | BISHOP, &b, &mut ret);
        assert_eq!(ret.len(), 6);
    }

    // Piece test - queen

    #[test]
    fn white_queen_empty_board() {
        let b = board_from_fen("8/8/8/8/3Q4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 6;
        let col = 5;
        queen_moves(row, col, WHITE | QUEEN, &b, &mut ret);
        assert_eq!(ret.len(), 27);
    }

    #[test]
    fn white_queen_cant_move() {
        let b = board_from_fen("8/8/8/2NBR3/2PQR3/2RRR3/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 6;
        let col = 5;
        queen_moves(row, col, WHITE | QUEEN, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_queen_whit_other_piece() {
        let b = board_from_fen("8/6r1/8/8/3Q4/5N2/8/6P1 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 6;
        let col = 5;
        queen_moves(row, col, WHITE | QUEEN, &b, &mut ret);
        assert_eq!(ret.len(), 25);
    }

    // Perft tests - move generation. Table of values taken from https://www.chessprogramming.org/Perft_Results
    #[test]
    fn perft_test_depth_one() {
        let mut moves: Vec<(usize, usize)> = vec![];
        let b = board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                if is_white(b.board[i][j]) {
                    get_moves(i as i8, j as i8, b.board[i][j], &b, &mut moves);
                }
            }
        }
        assert_eq!(moves.len(), 20);
    }
}
