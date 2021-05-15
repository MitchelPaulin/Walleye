use colored::*;

pub const COLOR_MASK: u8 = 0b10000000;
pub const WHITE: u8 = 0b10000000;
pub const BLACK: u8 = 0b00000000;

const PIECE_MASK: u8 = 0b00000111;
pub const PAWN: u8 = 0b00000001;
pub const KNIGHT: u8 = 0b00000010;
pub const BISHOP: u8 = 0b00000011;
pub const ROOK: u8 = 0b00000100;
pub const QUEEN: u8 = 0b00000110;
pub const KING: u8 = 0b00000111;

pub const EMPTY: u8 = 0;
pub const SENTINEL: u8 = 0b11111111;

pub const BOARD_START : usize = 2;
pub const BOARD_END : usize = 10;

fn is_white(square: u8) -> bool {
    square & COLOR_MASK == WHITE
}

fn is_black(square: u8) -> bool {
    !is_white(square)
}

fn is_pawn(square: u8) -> bool {
    square & PIECE_MASK == PAWN
}

fn is_knight(square: u8) -> bool {
    square & PIECE_MASK == KNIGHT
}

fn is_bishop(square: u8) -> bool {
    square & PIECE_MASK == BISHOP
}

fn is_rook(square: u8) -> bool {
    square & PIECE_MASK == ROOK
}

fn is_queen(square: u8) -> bool {
    square & PIECE_MASK == QUEEN
}

fn is_king(square: u8) -> bool {
    square & PIECE_MASK == KING
}

pub fn is_empty(square: u8) -> bool {
    square == EMPTY
}

fn is_outside_board(square: u8) -> bool {
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
        _ => " "
    }
}

pub struct Board {
    pub board: [[u8; 10]; 12],
    pub to_move: u8,
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
    }
}
