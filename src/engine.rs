pub use crate::board::*;

fn get_piece_from_fen_string_char(piece: char) -> Option<u8> {
    match piece {
        'r' => Some(BLACK | ROOK),
        'n' => Some(BLACK | KNIGHT),
        'b' => Some(BLACK | BISHOP),
        'q' => Some(BLACK | QUEEN),
        'k' => Some(BLACK | KING),
        'p' => Some(BLACK | PAWN),
        'R' => Some(WHITE | ROOK),
        'N' => Some(WHITE | KNIGHT),
        'B' => Some(WHITE | BISHOP),
        'Q' => Some(WHITE | QUEEN),
        'K' => Some(WHITE | KING),
        'P' => Some(WHITE | PAWN),
        _ => None,
    }
}

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
        let space = board.board[_row][_col];
        if is_empty(space) || space & COLOR_MASK != piece & COLOR_MASK {
            moves.push((_row, _col));
        }
    }
}

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
        }

        // check a double push
        if row == 8
            && is_empty(board.board[(row - 1) as usize][col as usize])
            && is_empty(board.board[(row - 2) as usize][col as usize])
        {
            moves.push(((row - 2) as usize, col as usize));
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
        }

        // check a double push
        if row == 3
            && is_empty(board.board[(row + 1) as usize][col as usize])
            && is_empty(board.board[(row + 2) as usize][col as usize])
        {
            moves.push(((row + 2) as usize, col as usize));
        }
    }
}

/*
    Parse the standard fen string notation en.wikipedia.org/wiki/Forsyth–Edwards_Notation
*/
pub fn board_from_fen(fen: &str) -> Result<Board, &str> {
    let mut b = [[SENTINEL; 10]; 12];
    let fen_config: Vec<&str> = fen.split(' ').collect();
    if fen_config.len() != 6 {
        return Err("Could not parse fen string: Invalid fen string");
    }

    let to_move = if fen_config[1] == "w" { WHITE } else { BLACK };
    let castling_privileges = fen_config[2];
    let en_passant = fen_config[3];
    let halfmove_clock = fen_config[4];
    let fullmove_clock = fen_config[5];

    let fen_rows: Vec<&str> = fen_config[0].split('/').collect();

    if fen_rows.len() != 8 {
        return Err("Could not parse fen string: Invalid number of rows provided, 8 expected");
    }

    let mut row: usize = BOARD_START;
    let mut col: usize = BOARD_START;
    for fen_row in fen_rows {
        for square in fen_row.chars() {
            if square.is_digit(10) {
                let mut square_skip_count = square.to_digit(10).unwrap() as usize;
                if square_skip_count + col > BOARD_END {
                    return Err("Could not parse fen string: Index out of bounds");
                }
                while square_skip_count > 0 {
                    b[row][col] = EMPTY;
                    col += 1;
                    square_skip_count -= 1;
                }
            } else {
                match get_piece_from_fen_string_char(square) {
                    Some(piece) => b[row][col] = piece,
                    None => return Err("Could not parse fen string: Invalid character found"),
                }
                col += 1;
            }
        }
        if col != BOARD_END {
            return Err("Could not parse fen string: Complete row was not specified");
        }
        row += 1;
        col = BOARD_START;
    }
    Ok(Board {
        board: b,
        to_move: to_move,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_board() {
        let b = board_from_fen("8/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                assert_eq!(b.board[i][j], EMPTY);
            }
        }
    }

    #[test]
    fn starting_pos() {
        let b = board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(b.board[2][2], BLACK | ROOK);
        assert_eq!(b.board[2][3], BLACK | KNIGHT);
        assert_eq!(b.board[2][4], BLACK | BISHOP);
        assert_eq!(b.board[2][5], BLACK | QUEEN);
        assert_eq!(b.board[2][6], BLACK | KING);
        assert_eq!(b.board[2][7], BLACK | BISHOP);
        assert_eq!(b.board[2][8], BLACK | KNIGHT);
        assert_eq!(b.board[2][9], BLACK | ROOK);

        for i in BOARD_START..BOARD_END {
            assert_eq!(b.board[3][i], BLACK | PAWN);
        }

        for i in 4..8 {
            for j in BOARD_START..BOARD_END {
                assert_eq!(b.board[i][j], EMPTY);
            }
        }

        assert_eq!(b.board[9][2], WHITE | ROOK);
        assert_eq!(b.board[9][3], WHITE | KNIGHT);
        assert_eq!(b.board[9][4], WHITE | BISHOP);
        assert_eq!(b.board[9][5], WHITE | QUEEN);
        assert_eq!(b.board[9][6], WHITE | KING);
        assert_eq!(b.board[9][7], WHITE | BISHOP);
        assert_eq!(b.board[9][8], WHITE | KNIGHT);
        assert_eq!(b.board[9][9], WHITE | ROOK);

        for i in BOARD_START..BOARD_END {
            assert_eq!(b.board[8][i], WHITE | PAWN);
        }
    }

    #[test]
    fn correct_starting_player() {
        let mut b =
            board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(b.to_move, WHITE);
        b = board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();
        assert_eq!(b.to_move, BLACK);
    }

    #[test]
    fn random_pos() {
        let b = board_from_fen("4R1B1/1kp5/1B1Q4/1P5p/1p2p1pK/8/3pP3/4N1b1 w - - 0 1").unwrap();
        assert_eq!(b.board[2][6], WHITE | ROOK);
        assert_eq!(b.board[2][8], WHITE | BISHOP);
        assert_eq!(b.board[3][3], BLACK | KING);
        assert_eq!(b.board[3][4], BLACK | PAWN);
        assert_eq!(b.board[4][3], WHITE | BISHOP);
        assert_eq!(b.board[4][5], WHITE | QUEEN);
        assert_eq!(b.board[5][3], WHITE | PAWN);
        assert_eq!(b.board[5][9], BLACK | PAWN);
        assert_eq!(b.board[6][3], BLACK | PAWN);
        assert_eq!(b.board[6][6], BLACK | PAWN);
        assert_eq!(b.board[6][8], BLACK | PAWN);
        assert_eq!(b.board[6][9], WHITE | KING);
        assert_eq!(b.board[8][5], BLACK | PAWN);
        assert_eq!(b.board[8][6], WHITE | PAWN);
        assert_eq!(b.board[9][6], WHITE | KNIGHT);
        assert_eq!(b.board[9][8], BLACK | BISHOP);
    }

    #[test]
    #[should_panic]
    fn bad_fen_string() {
        board_from_fen("this isn't a fen string").unwrap();
    }

    #[test]
    #[should_panic]
    fn bad_fen_string_bad_char() {
        board_from_fen("rnbqkbnH/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    }

    #[test]
    #[should_panic]
    fn bad_fen_string_too_many_chars() {
        board_from_fen("rnbqkbnrrrrr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    }

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
        knight_moves(row, col, WHITE | KNIGHT, &b, &mut ret);
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
    #[test]
    fn black_pawn_with_one_capture() {
        let b = board_from_fen("8/3p4/3qR3/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<(usize, usize)> = vec![];
        let row = 3;
        let col = 5;
        pawn_moves(row, col, BLACK | PAWN, &b, &mut ret);
        assert_eq!(ret.len(), 1);
    }
}
