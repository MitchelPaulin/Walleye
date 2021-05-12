use colored::*;

const COLOR_MASK: u8 = 0b10000000;
const WHITE: u8 = 0b10000000;
const BLACK: u8 = 0b00000000;

const PIECE_MASK: u8 = 0b00000111;
const PAWN: u8 = 0b00000001;
const KNIGHT: u8 = 0b00000010;
const BISHOP: u8 = 0b00000011;
const ROOK: u8 = 0b00000100;
const QUEEN: u8 = 0b00000110;
const KING: u8 = 0b00000111;

const EMPTY: u8 = 0;
const SENTINEL: u8 = 0b11111111;

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

fn is_empty(square: u8) -> bool {
    square == EMPTY
}

fn is_outside_board(square: u8) -> bool {
    square == SENTINEL
}

pub struct Board {
    board: [[u8; 10]; 12],
    to_move: u8,
}

impl Board {
    pub fn print_board(&self) {
        for i in 2..10 {
            for j in 2..10 {
                let mut piece = " ";
                if self.board[i][j] == WHITE | PAWN {
                    piece = "♙";
                } else if self.board[i][j] == WHITE | KNIGHT {
                    piece = "♘";
                } else if self.board[i][j] == WHITE | BISHOP {
                    piece = "♗";
                } else if self.board[i][j] == WHITE | ROOK {
                    piece = "♖";
                } else if self.board[i][j] == WHITE | QUEEN {
                    piece = "♕";
                } else if self.board[i][j] == WHITE | KING {
                    piece = "♔";
                } else if self.board[i][j] == BLACK | PAWN {
                    piece = "♟︎";
                } else if self.board[i][j] == BLACK | KNIGHT {
                    piece = "♞";
                } else if self.board[i][j] == BLACK | BISHOP {
                    piece = "♝";
                } else if self.board[i][j] == BLACK | ROOK {
                    piece = "♜";
                } else if self.board[i][j] == BLACK | QUEEN {
                    piece = "♛";
                } else if self.board[i][j] == BLACK | KING {
                    piece = "♚";
                }
                if (i + j) % 2 == 0 {
                    print!("{}", piece.on_red());
                    print!("{}", " ".on_red());
                } else {
                    print!("{} ", piece);
                }
            }
            println!();
        }
    }
}

pub fn new_board() -> Board {
    let mut b = [[SENTINEL; 10]; 12];

    // White pieces
    b[2][2] = WHITE | ROOK;
    b[2][3] = WHITE | KNIGHT;
    b[2][4] = WHITE | BISHOP;
    b[2][5] = WHITE | KING;
    b[2][6] = WHITE | QUEEN;
    b[2][7] = WHITE | BISHOP;
    b[2][8] = WHITE | KNIGHT;
    b[2][9] = WHITE | ROOK;
    for i in 2..10 {
        b[3][i] = WHITE | PAWN;
    }

    // No mans land
    for i in 4..8 {
        for j in 2..10 {
            b[i][j] = EMPTY;
        }
    }

    // Black pieces
    b[9][2] = BLACK | ROOK;
    b[9][3] = BLACK | KNIGHT;
    b[9][4] = BLACK | BISHOP;
    b[9][5] = BLACK | KING;
    b[9][6] = BLACK | QUEEN;
    b[9][7] = BLACK | BISHOP;
    b[9][8] = BLACK | KNIGHT;
    b[9][9] = BLACK | ROOK;

    for i in 2..10 {
        b[8][i] = BLACK | PAWN;
    }

    return Board {
        board: b,
        to_move: WHITE,
    };
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
