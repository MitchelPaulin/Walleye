pub use crate::board::*;

const KNIGHT_CORDS: [(i8, i8); 8] = [
    (1, 2),
    (1, -2),
    (2, 1),
    (2, -1),
    (-1, 2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
];

/*
    Generate pseudo-legal moves for a knight
*/
pub fn knight_moves(row: i8, col: i8, piece: u8, board: &BoardState, moves: &mut Vec<(usize, usize)>) {
    for mods in KNIGHT_CORDS.iter() {
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
pub fn pawn_moves(row: i8, col: i8, piece: u8, board: &BoardState, moves: &mut Vec<(usize, usize)>) {
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
pub fn king_moves(row: i8, col: i8, piece: u8, board: &BoardState, moves: &mut Vec<(usize, usize)>) {
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
pub fn rook_moves(row: i8, col: i8, piece: u8, board: &BoardState, moves: &mut Vec<(usize, usize)>) {
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
pub fn bishop_moves(row: i8, col: i8, piece: u8, board: &BoardState, moves: &mut Vec<(usize, usize)>) {
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
pub fn queen_moves(row: i8, col: i8, piece: u8, board: &BoardState, moves: &mut Vec<(usize, usize)>) {
    rook_moves(row, col, piece, board, moves);
    bishop_moves(row, col, piece, board, moves);
}

/*
    Generate pseudo-legal moves for a piece
*/
pub fn get_moves(row: i8, col: i8, piece: u8, board: &BoardState, moves: &mut Vec<(usize, usize)>) {
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
pub fn is_check(board: &BoardState, color: u8) -> bool {
    let king_location;
    let attacking_color;
    if color == WHITE {
        king_location = board.white_king_location;
        attacking_color = BLACK;
    } else {
        king_location = board.black_king_location;
        attacking_color = WHITE;
    }

    // Check from knight

    for mods in KNIGHT_CORDS.iter() {
        let _row = (king_location.0 as i8 + mods.0) as usize;
        let _col = (king_location.1 as i8 + mods.1) as usize;
        let square = board.board[_row][_col];

        if square == KNIGHT | attacking_color {
            return true;
        }
    }
    // Check from pawn
    let _row;
    if color == WHITE {
        _row = (king_location.0 as i8 - 1) as usize;
    } else {
        _row = (king_location.0 as i8 + 1) as usize;
    }

    if board.board[_row][(king_location.1 as i8 - 1) as usize] == attacking_color | PAWN
        || board.board[_row][(king_location.1 as i8 + 1) as usize] == attacking_color | PAWN
    {
        return true;
    }

    // Check from rook or queen
    let mods = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    for m in mods.iter() {
        let mut multiplier = 1;
        let mut _row = king_location.0 as i8 + m.0;
        let mut _col = king_location.1 as i8 + m.1;
        let mut square = board.board[_row as usize][_col as usize];
        while is_empty(square) {
            multiplier += 1;
            _row = king_location.0 as i8 + m.0 * multiplier;
            _col = king_location.1 as i8 + m.1 * multiplier;
            square = board.board[_row as usize][_col as usize];
        }

        if square == attacking_color | ROOK || square == attacking_color | QUEEN {
            return true;
        }
    }

    // Check from bishop or queen
    let mods = [1, -1];
    for i in mods.iter() {
        for j in mods.iter() {
            let mut multiplier = 1;
            let mut _row = king_location.0 as i8 + i;
            let mut _col = king_location.1 as i8 + j;
            let mut square = board.board[_row as usize][_col as usize];
            while is_empty(square) {
                multiplier += 1;
                _row = king_location.0 as i8 + i * multiplier;
                _col = king_location.1 as i8 + j * multiplier;
                square = board.board[_row as usize][_col as usize];
            }

            if square == attacking_color | BISHOP || square == attacking_color | QUEEN {
                return true;
            }
        }
    }

    return false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_sanity_test() {
        let b = board_from_fen("8/8/8/8/3K4/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, WHITE));
    }

    #[test]
    fn knight_checks() {
        let mut b = board_from_fen("8/8/4n3/8/3K4/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, WHITE));

        b = board_from_fen("8/8/8/8/8/8/1RK5/nRB5 w - - 0 1").unwrap();
        assert!(is_check(&b, WHITE));

        b = board_from_fen("8/8/8/8/3k4/5N2/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, BLACK));

        b = board_from_fen("8/8/8/8/3k4/5n2/8/7N w - - 0 1").unwrap();
        assert!(!is_check(&b, BLACK));

        b = board_from_fen("8/8/2N5/8/3k4/5n2/8/7N w - - 0 1").unwrap();
        assert!(is_check(&b, BLACK));
    }

    #[test]
    fn pawn_checks() {
        let mut b = board_from_fen("8/8/8/4k3/3P4/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, BLACK));

        b = board_from_fen("8/8/8/4k3/5P2/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, BLACK));

        b = board_from_fen("8/8/8/4k3/4P3/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, BLACK));

        b = board_from_fen("8/8/3PPP2/4k3/8/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, BLACK));

        b = board_from_fen("8/8/8/8/8/5p2/6K1/8 w - - 0 1").unwrap();
        assert!(is_check(&b, WHITE));

        b = board_from_fen("8/8/8/8/8/7p/6K1/8 w - - 0 1").unwrap();
        assert!(is_check(&b, WHITE));

        b = board_from_fen("8/8/8/8/8/6p1/6K1/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, WHITE));

        b = board_from_fen("8/8/8/8/8/6K1/5ppp/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, WHITE));
    }

    #[test]
    fn rook_checks() {
        let mut b = board_from_fen("8/8/8/R3k3/8/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, BLACK));

        b = board_from_fen("8/8/8/R1r1k3/8/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, BLACK));

        b = board_from_fen("8/8/8/R1r1k3/8/8/8/4R3 w - - 0 1").unwrap();
        assert!(is_check(&b, BLACK));

        b = board_from_fen("4R3/8/8/R1r5/8/8/8/4k3 w - - 0 1").unwrap();
        assert!(is_check(&b, BLACK));

        b = board_from_fen("8/8/8/R1r5/8/8/7R/4k3 w - - 0 1").unwrap();
        assert!(!is_check(&b, BLACK));

        b = board_from_fen("4R3/8/8/8/8/3r4/R3K2R/2r1Rr2 w - - 0 1").unwrap();
        assert!(!is_check(&b, WHITE));

        b = board_from_fen("4R3/8/8/8/4K3/3r4/R6R/2r1rr2 w - - 0 1").unwrap();
        assert!(is_check(&b, WHITE));

        b = board_from_fen("4R3/8/8/8/4K2r/3r4/R6R/2r2r2 w - - 0 1").unwrap();
        assert!(is_check(&b, WHITE));

        b = board_from_fen("4r3/8/8/4B3/r2QKP1r/3rR3/R6R/2r1rr2 w - - 0 1").unwrap();
        assert!(!is_check(&b, WHITE));
    }

    #[test]
    fn bishop_checks() {
        let mut b = board_from_fen("8/8/8/1B6/8/8/8/5k2 w - - 0 1").unwrap();
        assert!(is_check(&b, BLACK));

        b = board_from_fen("8/8/2B1B3/1B3B2/1B1k1B2/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, BLACK));

        b = board_from_fen("8/8/8/8/5k2/8/8/2B5 w - - 0 1").unwrap();
        assert!(is_check(&b, BLACK));

        b = board_from_fen("8/8/8/8/5k2/4n3/8/2B5 w - - 0 1").unwrap();
        assert!(!is_check(&b, BLACK));

        b = board_from_fen("8/8/8/8/3K4/8/8/6b1 w - - 0 1").unwrap();
        assert!(is_check(&b, WHITE));

        b = board_from_fen("8/8/8/8/3K4/4r3/8/6b1 w - - 0 1").unwrap();
        assert!(!is_check(&b, WHITE));

        b = board_from_fen("8/8/8/8/3K4/4r3/8/b5b1 w - - 0 1").unwrap();
        assert!(is_check(&b, WHITE));

        b = board_from_fen("8/8/8/8/3K4/2P1r3/8/b5b1 w - - 0 1").unwrap();
        assert!(!is_check(&b, WHITE));
    }

    #[test]
    fn queen_checks() {
        let mut b = board_from_fen("8/8/8/8/3k1Q2/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, BLACK));

        b = board_from_fen("8/8/2k5/8/8/8/6Q1/8 w - - 0 1").unwrap();
        assert!(is_check(&b, BLACK));

        b = board_from_fen("8/8/2K5/8/3q4/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, WHITE));

        b = board_from_fen("8/8/1K6/2Q5/3q4/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, WHITE));

        b = board_from_fen("8/5Q2/1K6/8/3q4/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, WHITE));

        b = board_from_fen("8/5Q2/1K6/1P6/8/8/1q6/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, WHITE));

        b = board_from_fen("8/2P2Q2/1K6/8/8/8/1q6/8 w - - 0 1").unwrap();
        assert!(is_check(&b, WHITE));
    }

    // Knight tests

    #[test]
    fn knight_moves_empty_board() {
        let b = board_from_fen("8/8/8/8/3N4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        knight_moves(6, 5, WHITE | KNIGHT, &b, &mut ret);
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn knight_moves_corner() {
        let b = board_from_fen("N7/8/8/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        knight_moves(2, 2, BLACK | KNIGHT, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }
    #[test]
    fn knight_moves_with_other_pieces_with_capture() {
        let b = board_from_fen("8/8/5n2/3NQ3/2K2P2/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        knight_moves(5, 5, WHITE | KNIGHT, &b, &mut ret);
        assert_eq!(ret.len(), 7);
    }

    // Pawn tests - white pawn

    #[test]
    fn white_pawn_double_push() {
        let b = board_from_fen("8/8/8/8/8/8/P7/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        pawn_moves(8, 2, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn white_pawn_has_moved() {
        let b = board_from_fen("8/8/8/8/8/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        pawn_moves(7, 5, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn white_pawn_cant_move_black_piece_block() {
        let b = board_from_fen("8/8/8/8/3r4/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        pawn_moves(7, 5, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_pawn_cant_move_white_piece_block() {
        let b = board_from_fen("8/8/8/8/3K4/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        pawn_moves(7, 5, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_pawn_with_two_captures_and_start() {
        let b = board_from_fen("8/8/8/8/8/n1q5/1P6/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        pawn_moves(8, 3, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn white_pawn_with_one_capture() {
        let b = board_from_fen("8/8/Q1b5/1P6/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        pawn_moves(5, 3, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn white_pawn_double_push_piece_in_front() {
        let b = board_from_fen("8/8/8/8/8/b7/P7/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        pawn_moves(8, 2, WHITE | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    // Pawn tests - black pawn

    #[test]
    fn black_pawn_double_push() {
        let b = board_from_fen("8/p7/8/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        pawn_moves(3, 2, BLACK | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn black_pawn_has_moved() {
        let b = board_from_fen("8/8/8/3p4/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        pawn_moves(5, 5, BLACK | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn black_pawn_cant_move_white_piece_block() {
        let b = board_from_fen("8/3p4/3R4/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        pawn_moves(3, 5, BLACK | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn black_pawn_with_two_captures_and_start() {
        let b = board_from_fen("8/3p4/2R1R3/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        pawn_moves(3, 5, BLACK | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn black_pawn_with_one_capture() {
        let b = board_from_fen("8/3p4/3qR3/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        pawn_moves(3, 5, BLACK | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 1);
    }

    // Piece test - king

    #[test]
    fn king_empty_board_center() {
        let b = board_from_fen("8/8/8/8/3K4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        king_moves(5, 6, WHITE | KING, &b, &mut ret);
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn king_start_pos() {
        let b = board_from_fen("8/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        king_moves(9, 6, WHITE | KING, &b, &mut ret);
        assert_eq!(ret.len(), 5);
    }

    #[test]
    fn king_start_pos_other_pieces() {
        let b = board_from_fen("8/8/8/8/8/8/3Pn3/3QKB2 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        king_moves(9, 6, WHITE | KING, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn king_black_other_pieces() {
        let b = board_from_fen("8/8/8/8/8/3Pn3/3QkB2/3R1q2 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        king_moves(8, 6, BLACK | KING, &b, &mut ret);
        assert_eq!(ret.len(), 6);
    }

    // Piece test - rook
    #[test]
    fn rook_center_of_empty_board() {
        let b = board_from_fen("8/8/8/8/3R4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        rook_moves(6, 5, WHITE | ROOK, &b, &mut ret);
        assert_eq!(ret.len(), 14);
    }

    #[test]
    fn rook_center_of_board() {
        let b = board_from_fen("8/8/8/3q4/2kRp3/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        rook_moves(6, 5, WHITE | ROOK, &b, &mut ret);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn rook_center_of_board_with_white_pieces() {
        let b = board_from_fen("7p/3N4/8/4n3/2kR4/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        rook_moves(6, 5, WHITE | ROOK, &b, &mut ret);
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn rook_corner() {
        let b = board_from_fen("7p/3N4/8/4n3/2kR4/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        rook_moves(9, 9, WHITE | ROOK, &b, &mut ret);
        assert_eq!(ret.len(), 14);
    }
    #[test]
    fn black_rook_center_of_board_with_white_pieces() {
        let b = board_from_fen("7p/3N4/8/4n3/2kR4/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        rook_moves(6, 5, BLACK | ROOK, &b, &mut ret);
        assert_eq!(ret.len(), 7);
    }

    // Piece test - bishop
    #[test]
    fn black_bishop_center_empty_board() {
        let b = board_from_fen("8/8/8/3b4/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        bishop_moves(5, 5, BLACK | BISHOP, &b, &mut ret);
        assert_eq!(ret.len(), 13);
    }

    #[test]
    fn black_bishop_center_with_captures() {
        let b = board_from_fen("6P1/8/8/3b4/8/1R6/8/3Q4 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        bishop_moves(5, 5, BLACK | BISHOP, &b, &mut ret);
        assert_eq!(ret.len(), 12);
    }

    #[test]
    fn black_bishop_center_with_captures_and_black_pieces() {
        let b = board_from_fen("6P1/8/2Q5/3b4/2k1n3/1R6/8/b2Q4 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        bishop_moves(5, 5, BLACK | BISHOP, &b, &mut ret);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn white_bishop_center_with_captures_and_white_pieces() {
        let b = board_from_fen("8/8/8/4r3/5B2/8/3Q4/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        bishop_moves(6, 7, WHITE | BISHOP, &b, &mut ret);
        assert_eq!(ret.len(), 6);
    }

    // Piece test - queen

    #[test]
    fn white_queen_empty_board() {
        let b = board_from_fen("8/8/8/8/3Q4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        queen_moves(6, 5, WHITE | QUEEN, &b, &mut ret);
        assert_eq!(ret.len(), 27);
    }

    #[test]
    fn white_queen_cant_move() {
        let b = board_from_fen("8/8/8/2NBR3/2PQR3/2RRR3/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        queen_moves(6, 5, WHITE | QUEEN, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_queen_with_other_piece() {
        let b = board_from_fen("8/6r1/8/8/3Q4/5N2/8/6P1 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        queen_moves(6, 5, WHITE | QUEEN, &b, &mut ret);
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
