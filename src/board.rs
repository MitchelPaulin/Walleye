use colored::*;

/*
    Example Piece: 0b11000101
    1st bit: Color 1 = White, 0 = Black
    2nd bit: Whether this piece has moved yet, 0=has not moved, 1=has moved
    3-5 bit: Unused
    6-8 bit: Piece identifier
*/

pub const MOVED_MASK: u8 = 0b01000000;

pub const COLOR_MASK: u8 = 0b10000000;
pub const WHITE: u8 = 0b10000000;
pub const BLACK: u8 = 0b00000000;

pub const PIECE_MASK: u8 = 0b00000111;
pub const PAWN: u8 = 0b00000001;
pub const KNIGHT: u8 = 0b00000010;
pub const BISHOP: u8 = 0b00000011;
pub const ROOK: u8 = 0b00000100;
pub const QUEEN: u8 = 0b00000110;
pub const KING: u8 = 0b00000111;

pub const EMPTY: u8 = 0;
pub const SENTINEL: u8 = 0b11111111;

pub const BOARD_START: usize = 2;
pub const BOARD_END: usize = 10;

pub fn has_moved(square: u8) -> bool {
    square & MOVED_MASK != 0
}

pub fn is_white(square: u8) -> bool {
    !is_empty(square) && square & COLOR_MASK == WHITE
}

pub fn is_black(square: u8) -> bool {
    !is_empty(square) && square & COLOR_MASK == BLACK
}

pub fn is_pawn(square: u8) -> bool {
    square & PIECE_MASK == PAWN
}

pub fn is_knight(square: u8) -> bool {
    square & PIECE_MASK == KNIGHT
}

pub fn is_bishop(square: u8) -> bool {
    square & PIECE_MASK == BISHOP
}

pub fn is_rook(square: u8) -> bool {
    square & PIECE_MASK == ROOK
}

pub fn is_queen(square: u8) -> bool {
    square & PIECE_MASK == QUEEN
}

pub fn is_king(square: u8) -> bool {
    square & PIECE_MASK == KING
}

pub fn is_empty(square: u8) -> bool {
    square == EMPTY
}

pub fn is_outside_board(square: u8) -> bool {
    square == SENTINEL
}

fn get_piece_character(piece: u8) -> &'static str {
    match piece & PIECE_MASK {
        PAWN => "♟︎",
        KNIGHT => "♞",
        BISHOP => "♝",
        ROOK => "♜",
        QUEEN => "♛",
        KING => "♚",
        _ => " ",
    }
}

pub struct Board {
    pub board: [[u8; 12]; 12],
    pub to_move: u8,
    pub white_king_location: (usize, usize),
    pub black_king_location: (usize, usize),
}

impl Board {
    pub fn print_board(&self) {
        println!("a b c d e f g h");
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                let piece = format!("{} ", get_piece_character(self.board[i][j]));
                if (i + j) % 2 != 0 {
                    if is_white(self.board[i][j]) {
                        print!("{}", piece.white().on_truecolor(158, 93, 30));
                    } else {
                        print!("{}", piece.black().on_truecolor(158, 93, 30));
                    }
                } else {
                    if is_white(self.board[i][j]) {
                        print!("{}", piece.white().on_truecolor(205, 170, 125));
                    } else {
                        print!("{}", piece.black().on_truecolor(205, 170, 125));
                    }
                }
            }
            println!(" {}", 10 - i);
        }
    }
}

/*
    Parse the standard fen string notation (en.wikipedia.org/wiki/Forsyth–Edwards_Notation) and return a board state
*/
pub fn board_from_fen(fen: &str) -> Result<Board, &str> {
    let mut board = [[SENTINEL; 12]; 12];
    let fen_config: Vec<&str> = fen.split(' ').collect();
    if fen_config.len() != 6 {
        return Err("Could not parse fen string: Invalid fen string");
    }

    let to_move = if fen_config[1] == "w" { WHITE } else { BLACK };
    let castling_privileges = fen_config[2];
    let en_passant = fen_config[3];
    let halfmove_clock = fen_config[4];
    let fullmove_clock = fen_config[5];

    let mut white_king_location = (0, 0);
    let mut black_king_location = (0, 0);

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
                    board[row][col] = EMPTY;
                    col += 1;
                    square_skip_count -= 1;
                }
            } else {
                match get_piece_from_fen_string_char(square) {
                    Some(piece) => board[row][col] = piece,
                    None => return Err("Could not parse fen string: Invalid character found"),
                }

                if is_king(board[row][col]) {
                    if is_white(board[row][col]) {
                        white_king_location = (row, col);
                    } else {
                        black_king_location = (row, col);
                    }
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
        board: board,
        to_move: to_move,
        white_king_location: white_king_location,
        black_king_location: black_king_location,
    })
}

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pieces_recognized() {
        assert!(is_white(WHITE | BISHOP));
        assert!(is_white(WHITE | ROOK));
        assert!(is_white(WHITE | KING));
        assert!(is_white(WHITE | PAWN));

        assert!(is_black(BLACK | BISHOP));
        assert!(is_black(BLACK | ROOK));
        assert!(is_black(BLACK | KING));
        assert!(is_black(BLACK | PAWN));

        assert!(is_pawn(WHITE | PAWN));
        assert!(is_pawn(BLACK | PAWN));
        assert!(!is_pawn(BLACK | ROOK));

        assert!(is_knight(WHITE | KNIGHT));
        assert!(is_knight(BLACK | KNIGHT));
        assert!(!is_knight(WHITE | QUEEN));

        assert!(is_bishop(WHITE | BISHOP));
        assert!(is_bishop(BLACK | BISHOP));
        assert!(!is_bishop(WHITE | ROOK));

        assert!(is_queen(WHITE | QUEEN));
        assert!(is_queen(BLACK | QUEEN));
        assert!(!is_queen(WHITE | PAWN));

        assert!(is_king(WHITE | KING));
        assert!(is_king(BLACK | KING));
        assert!(!is_king(WHITE | QUEEN));

        assert!(is_empty(EMPTY));
        assert!(!is_empty(WHITE | KING));

        assert!(is_outside_board(SENTINEL));
        assert!(!is_outside_board(EMPTY));
        assert!(!is_outside_board(WHITE | KING));

        assert!(has_moved(WHITE | PAWN | MOVED_MASK));
        assert!(!has_moved(WHITE | PAWN));
    }

    // fen string tests

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
    fn correct_king_location() {
        let b = board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(b.black_king_location, (2, 6));
        assert_eq!(b.white_king_location, (9, 6));
    }

    #[test]
    fn correct_king_location_two() {
        let b = board_from_fen("6rk/1b4np/5pp1/1p6/8/1P3NP1/1B3P1P/5RK1 w KQkq - 0 1").unwrap();
        assert_eq!(b.black_king_location, (2, 9));
        assert_eq!(b.white_king_location, (9, 8));
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
}
