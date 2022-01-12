use crate::engine::*;
use crate::utils::*;
use crate::zobrist::ZobristHasher;
use colored::*;
use std::fmt;
use std::str::FromStr;

// Board position for the start of a new game
pub const DEFAULT_FEN_STRING: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Square {
    // This square is empty.
    Empty,

    // A piece is on this square
    Full(Piece),

    // A non-board square; the board data structure contains squares not present on the
    // actual board in order to make move calculation easier, and all such squares have
    // this variant.
    Boundary,
}

impl Square {
    // Check if this square is empty or contains a piece of the given color (used in move
    // generation)
    pub fn is_empty_or_color(self, color: PieceColor) -> bool {
        match self {
            Square::Full(Piece {
                color: square_color,
                ..
            }) => color == square_color,
            Square::Empty => true,
            _ => false,
        }
    }

    // Check if this square is empty
    pub fn is_empty(self) -> bool {
        self == Square::Empty
    }

    // Check if a square is a certain color, return false if empty
    pub fn is_color(self, color: PieceColor) -> bool {
        match self {
            Square::Full(Piece {
                color: square_color,
                ..
            }) => color == square_color,
            _ => false,
        }
    }

    // Get the "fancy" character to represent the content of this square
    fn fancy_char(self) -> &'static str {
        match self {
            Square::Full(piece) => piece.fancy_char(),
            _ => " ",
        }
    }

    // Get the "simple" character to represent this content of this square (capitalized based on
    // the piece's color)
    fn simple_char(self) -> &'static str {
        match self {
            Square::Full(piece) => piece.simple_char(),
            _ => ".",
        }
    }
}

impl From<Piece> for Square {
    // Given a piece, generate a square containing that piece
    fn from(piece: Piece) -> Self {
        Square::Full(piece)
    }
}

impl PartialEq<Piece> for Square {
    fn eq(&self, other: &Piece) -> bool {
        match self {
            Square::Full(piece) => piece == other,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Piece {
    pub color: PieceColor,
    pub kind: PieceKind,
}

impl Piece {
    pub fn index(self) -> usize {
        self.kind.index()
    }

    pub const fn pawn(color: PieceColor) -> Self {
        Self { kind: Pawn, color }
    }

    pub const fn knight(color: PieceColor) -> Self {
        Self {
            kind: Knight,
            color,
        }
    }

    pub const fn bishop(color: PieceColor) -> Self {
        Self {
            kind: Bishop,
            color,
        }
    }

    pub const fn rook(color: PieceColor) -> Self {
        Self { kind: Rook, color }
    }

    pub const fn queen(color: PieceColor) -> Self {
        Self { kind: Queen, color }
    }

    pub const fn king(color: PieceColor) -> Self {
        Self { kind: King, color }
    }

    // Get the "fancy" character for this piece
    fn fancy_char(self) -> &'static str {
        match self.kind {
            Pawn => "♟︎",
            Knight => "♞",
            Bishop => "♝",
            Rook => "♜",
            Queen => "♛",
            King => "♚",
        }
    }

    // Get the "simple" character to represent this piece (capitalized based on the piece's color)
    fn simple_char(self) -> &'static str {
        match (self.color, self.kind) {
            (White, Pawn) => "P",
            (White, Knight) => "N",
            (White, Bishop) => "B",
            (White, Rook) => "R",
            (White, Queen) => "Q",
            (White, King) => "K",
            (Black, Pawn) => "p",
            (Black, Knight) => "n",
            (Black, Bishop) => "b",
            (Black, Rook) => "r",
            (Black, Queen) => "q",
            (Black, King) => "k",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PieceColor {
    Black,
    White,
}

impl PieceColor {
    // Get the opposite color
    pub fn opposite(self) -> Self {
        match self {
            Black => White,
            White => Black,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    // get an index for a piece, helpful for arrays
    pub fn index(self) -> usize {
        match self {
            King => 0,
            Queen => 1,
            Rook => 2,
            Bishop => 3,
            Knight => 4,
            Pawn => 5,
        }
    }

    // Get the alg name for this kind of piece
    pub fn alg(self) -> &'static str {
        match self {
            Pawn => "p",
            Knight => "n",
            Bishop => "b",
            Rook => "r",
            Queen => "q",
            King => "k",
        }
    }
}

pub const BOARD_START: usize = 2;
pub const BOARD_END: usize = 10;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Point(pub usize, pub usize);

impl FromStr for Point {
    type Err = &'static str;

    // Parse an algebraic pair into a board position
    fn from_str(pair: &str) -> Result<Self, Self::Err> {
        if pair.len() != 2 {
            return Err("Invalid length for algebraic string");
        }

        let c = pair.chars().next().unwrap();
        let r = pair.chars().nth(1).unwrap();
        let col = match c {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return Err("Invalid column"),
        };

        let row = BOARD_END - (r.to_digit(10).unwrap() as usize);
        if !(BOARD_START..BOARD_END).contains(&row) {
            return Err("Invalid row");
        }

        Ok(Point(row, col + BOARD_START))
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            match self.1 {
                2 => "a",
                3 => "b",
                4 => "c",
                5 => "d",
                6 => "e",
                7 => "f",
                8 => "g",
                9 => "h",
                _ => "h",
            },
            match self.0 {
                2 => "8",
                3 => "7",
                4 => "6",
                5 => "5",
                6 => "4",
                7 => "3",
                8 => "2",
                9 => "1",
                _ => "1",
            },
        )
    }
}

#[derive(Clone)]
pub struct BoardState {
    pub board: [[Square; 12]; 12],
    pub to_move: PieceColor,
    pub pawn_double_move: Option<Point>, // if a pawn, on the last move, made a double move, this is set, otherwise this is None
    pub white_king_location: Point,
    pub black_king_location: Point,
    pub white_king_side_castle: bool,
    pub white_queen_side_castle: bool,
    pub black_king_side_castle: bool,
    pub black_queen_side_castle: bool,
    pub order_heuristic: i32, // value set to help order this board, a higher value means this board state will be considered first
    pub last_move: Option<(Point, Point)>, // the start and last position of the last move made
    pub pawn_promotion: Option<Piece>, // set to the chosen pawn promotion type
    pub zobrist_key: u64,
}

impl BoardState {
    // Parse the standard fen string notation (en.wikipedia.org/wiki/Forsyth–Edwards_Notation) and return a board state
    pub fn from_fen(fen: &str) -> Result<BoardState, &str> {
        let mut board = [[Square::Boundary; 12]; 12];
        let mut fen = fen.to_string();
        let zobrist_hasher = ZobristHasher::create_zobrist_hasher();
        let mut zobrist_key = 0;
        trim_newline(&mut fen);
        let fen_config: Vec<&str> = fen.split(' ').collect();
        if fen_config.len() != 6 {
            return Err("Could not parse fen string: Invalid fen string");
        }

        let to_move = match fen_config[1] {
            "w" => PieceColor::White,
            "b" => PieceColor::Black,
            _ => return Err("Could not parse fen string: Next player to move was not provided"),
        };

        if to_move == PieceColor::Black {
            zobrist_key = zobrist_hasher.get_black_to_move_val();
        }

        let castling_privileges = fen_config[2];
        let en_passant = fen_config[3];

        let half_move_clock = fen_config[4].parse::<u8>();
        if half_move_clock.is_err() {
            return Err("Could not parse fen string: Invalid half move value");
        }

        let full_move_clock = fen_config[5].parse::<u8>();
        if full_move_clock.is_err() {
            return Err("Could not parse fen string: Invalid full move value");
        }

        let fen_rows: Vec<&str> = fen_config[0].split('/').collect();

        if fen_rows.len() != 8 {
            return Err("Could not parse fen string: Invalid number of rows provided, 8 expected");
        }

        let mut row: usize = BOARD_START;
        let mut col: usize = BOARD_START;
        let mut white_king_location = Point(0, 0);
        let mut black_king_location = Point(0, 0);
        for fen_row in fen_rows {
            for square in fen_row.chars() {
                if square.is_digit(10) {
                    let square_skip_count = square.to_digit(10).unwrap() as usize;
                    if square_skip_count + col > BOARD_END {
                        return Err("Could not parse fen string: Index out of bounds");
                    }
                    for _ in 0..square_skip_count {
                        board[row][col] = Square::Empty;
                        col += 1;
                    }
                } else {
                    board[row][col] = match Self::piece_from_fen_string_char(square) {
                        Some(piece) => Square::Full(piece),
                        None => return Err("Could not parse fen string: Invalid character found"),
                    };

                    if let Square::Full(Piece { kind, color }) = board[row][col] {
                        zobrist_key ^= zobrist_hasher
                            .get_val_for_piece(Piece { kind, color }, Point(row, col));
                        if kind == King {
                            match color {
                                White => white_king_location = Point(row, col),
                                Black => black_king_location = Point(row, col),
                            };
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

        // Deal with the en passant string
        let mut en_passant_pos: Option<Point> = None;
        if en_passant.len() != 2 {
            if en_passant != "-" {
                return Err("Could not parse fen string: En passant string not valid");
            }
        } else {
            en_passant_pos = en_passant.parse().ok();
            if let Some(point) = en_passant_pos {
                zobrist_key ^= zobrist_hasher.get_val_for_en_passant(point.1);
            }
        }

        let mut board = BoardState {
            board,
            to_move,
            white_king_location,
            black_king_location,
            pawn_double_move: en_passant_pos,
            white_king_side_castle: castling_privileges.find('K') != None,
            white_queen_side_castle: castling_privileges.find('Q') != None,
            black_king_side_castle: castling_privileges.find('k') != None,
            black_queen_side_castle: castling_privileges.find('q') != None,
            order_heuristic: i32::MIN,
            last_move: None,
            pawn_promotion: None,
            zobrist_key,
        };

        if board.white_king_side_castle {
            board.zobrist_key ^= zobrist_hasher.get_val_for_castling(CastlingType::WhiteKingSide);
        }
        if board.white_queen_side_castle {
            board.zobrist_key ^= zobrist_hasher.get_val_for_castling(CastlingType::WhiteQueenSide);
        }
        if board.black_king_side_castle {
            board.zobrist_key ^= zobrist_hasher.get_val_for_castling(CastlingType::BlackKingSide)
        }
        if board.black_queen_side_castle {
            board.zobrist_key ^= zobrist_hasher.get_val_for_castling(CastlingType::BlackQueenSide);
        }

        Ok(board)
    }

    fn piece_from_fen_string_char(piece: char) -> Option<Piece> {
        match piece {
            'r' => Some(Piece {
                color: Black,
                kind: Rook,
            }),
            'n' => Some(Piece {
                color: Black,
                kind: Knight,
            }),
            'b' => Some(Piece {
                color: Black,
                kind: Bishop,
            }),
            'q' => Some(Piece {
                color: Black,
                kind: Queen,
            }),
            'k' => Some(Piece {
                color: Black,
                kind: King,
            }),
            'p' => Some(Piece {
                color: Black,
                kind: Pawn,
            }),
            'R' => Some(Piece {
                color: White,
                kind: Rook,
            }),
            'N' => Some(Piece {
                color: White,
                kind: Knight,
            }),
            'B' => Some(Piece {
                color: White,
                kind: Bishop,
            }),
            'Q' => Some(Piece {
                color: White,
                kind: Queen,
            }),
            'K' => Some(Piece {
                color: White,
                kind: King,
            }),
            'P' => Some(Piece {
                color: White,
                kind: Pawn,
            }),
            _ => None,
        }
    }

    pub fn pretty_print_board(&self) {
        println!("a b c d e f g h");
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                let square = self.board[i][j];
                let cell = format!("{} ", square.fancy_char());
                let cell = match square {
                    Square::Full(Piece { color: White, .. }) => cell.white(),
                    Square::Full(Piece { color: Black, .. }) => cell.black(),
                    _ => cell.white(),
                };

                let cell = if (i + j) % 2 != 0 {
                    cell.on_truecolor(158, 93, 30)
                } else {
                    cell.on_truecolor(205, 170, 125)
                };

                print!("{}", cell);
            }
            println!(" {}", 10 - i);
        }
    }

    pub fn simple_board(&self) -> String {
        let mut board = "\na b c d e f g h\n".to_string();
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                board = format!("{}{} ", board, self.board[i][j].simple_char());
            }
            board = format!("{} {}\n", board, 10 - i);
        }
        board
    }

    pub fn simple_print_board(&self) {
        print!("{}", self.simple_board());
    }

    pub fn swap_color(&mut self, zobrist_hasher: &ZobristHasher) {
        match self.to_move {
            PieceColor::White => self.to_move = PieceColor::Black,
            PieceColor::Black => self.to_move = PieceColor::White,
        }
        // the current play changed so we need to update the key
        self.zobrist_key ^= zobrist_hasher.get_black_to_move_val();
    }

    /*
        Helper function to take away castling rights, updates the zobrist as well if required

        Also protects against unsetting the castling rights more than once, which would mess
        up the zobrist key
    */
    pub fn take_away_castling_rights(
        &mut self,
        castling_type: CastlingType,
        zobrist_hasher: &ZobristHasher,
    ) {
        if castling_type == CastlingType::WhiteKingSide {
            if self.white_king_side_castle {
                self.white_king_side_castle = false;
                self.zobrist_key ^= zobrist_hasher.get_val_for_castling(CastlingType::WhiteKingSide)
            }
        } else if castling_type == CastlingType::WhiteQueenSide {
            if self.white_queen_side_castle {
                self.white_queen_side_castle = false;
                self.zobrist_key ^=
                    zobrist_hasher.get_val_for_castling(CastlingType::WhiteQueenSide);
            }
        } else if castling_type == CastlingType::BlackKingSide {
            if self.black_king_side_castle {
                self.black_king_side_castle = false;
                self.zobrist_key ^=
                    zobrist_hasher.get_val_for_castling(CastlingType::BlackKingSide);
            }
        } else if castling_type == CastlingType::BlackQueenSide && self.black_queen_side_castle {
            self.black_queen_side_castle = false;
            self.zobrist_key ^=
                zobrist_hasher.get_val_for_castling(CastlingType::BlackQueenSide);
        }
    }

    /*
        Helper function to clear the pawn double move condition and update
        the zobrist key if required
    */
    pub fn unset_pawn_double_move(&mut self, zobrist_hasher: &ZobristHasher) {
        if let Some(en_passant_target) = self.pawn_double_move {
            self.pawn_double_move = None;
            self.zobrist_key ^= zobrist_hasher.get_val_for_en_passant(en_passant_target.1);
        }
    }

    /*
        Helper function to move a piece on the board, will also update the zobrist
        hash of the board correctly even with a capture
    */
    pub fn move_piece(&mut self, start: Point, end: Point, zobrist_hasher: &ZobristHasher) {
        if let Square::Full(cur_piece) = self.board[start.0][start.1] {
            self.board[start.0][start.1] = Square::Empty;
            if let Square::Full(target_piece) = self.board[end.0][end.1] {
                self.zobrist_key ^= zobrist_hasher.get_val_for_piece(target_piece, end);
            }
            self.board[end.0][end.1] = Square::Full(cur_piece);
            self.zobrist_key ^= zobrist_hasher.get_val_for_piece(cur_piece, start)
                ^ zobrist_hasher.get_val_for_piece(cur_piece, end);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pieces_recognized() {
        assert_eq!(Piece::bishop(White).color, White);
        assert_eq!(Piece::rook(White).color, White);
        assert_eq!(Piece::king(White).color, White);
        assert_eq!(Piece::pawn(White).color, White);

        assert_eq!(Piece::bishop(Black).color, Black);
        assert_eq!(Piece::rook(Black).color, Black);
        assert_eq!(Piece::king(Black).color, Black);
        assert_eq!(Piece::pawn(Black).color, Black);

        assert_eq!(Piece::pawn(White).kind, Pawn);
        assert_eq!(Piece::knight(White).kind, Knight);
        assert_eq!(Piece::bishop(White).kind, Bishop);
        assert_eq!(Piece::rook(White).kind, Rook);
        assert_eq!(Piece::queen(White).kind, Queen);
        assert_eq!(Piece::king(White).kind, King);

        assert!(Square::Empty.is_empty());
        assert!(!Square::Full(Piece::king(White)).is_empty());
    }

    // Algebraic translation tests

    #[test]
    fn algebraic_translation_correct() {
        let res = "a8".parse::<Point>().unwrap();
        assert_eq!(res.0, BOARD_START);
        assert_eq!(res.1, BOARD_START);

        let res = "h1".parse::<Point>().unwrap();
        assert_eq!(res.0, BOARD_END - 1);
        assert_eq!(res.1, BOARD_END - 1);

        let res = "a6".parse::<Point>().unwrap();
        assert_eq!(res.0, BOARD_START + 2);
        assert_eq!(res.1, BOARD_START);

        let res = "c5".parse::<Point>().unwrap();
        assert_eq!(res.0, BOARD_START + 3);
        assert_eq!(res.1, BOARD_START + 2);
    }

    #[test]
    #[should_panic]
    fn algebraic_translation_panic_col() {
        "z1".parse::<Point>().unwrap();
    }

    #[test]
    #[should_panic]
    fn algebraic_translation_panic_long() {
        "a11".parse::<Point>().unwrap();
    }

    #[test]
    fn points_to_long_algebraic_position_test() {
        let res = Point(2, 2).to_string();
        assert_eq!(res, "a8");

        let res = Point(4, 6).to_string();
        assert_eq!(res, "e6");
    }

    // Fen string tests

    #[test]
    fn empty_board() {
        let b = BoardState::from_fen("8/8/8/8/8/8/8/8 w - - 0 1").unwrap();
        for i in BOARD_START..BOARD_END {
            for j in BOARD_START..BOARD_END {
                assert_eq!(b.board[i][j], Square::Empty);
            }
        }
        assert_eq!(b.zobrist_key, 0);
    }

    #[test]
    fn starting_pos() {
        let b = BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
        assert_eq!(b.board[2][2], Square::from(Piece::rook(Black)));
        assert_eq!(b.board[2][3], Square::from(Piece::knight(Black)));
        assert_eq!(b.board[2][4], Square::from(Piece::bishop(Black)));
        assert_eq!(b.board[2][5], Square::from(Piece::queen(Black)));
        assert_eq!(b.board[2][6], Square::from(Piece::king(Black)));
        assert_eq!(b.board[2][7], Square::from(Piece::bishop(Black)));
        assert_eq!(b.board[2][8], Square::from(Piece::knight(Black)));
        assert_eq!(b.board[2][9], Square::from(Piece::rook(Black)));

        for i in BOARD_START..BOARD_END {
            assert_eq!(b.board[3][i], Square::from(Piece::pawn(Black)));
        }

        for i in 4..8 {
            for j in BOARD_START..BOARD_END {
                assert_eq!(b.board[i][j], Square::Empty);
            }
        }

        assert_eq!(b.board[9][2], Square::from(Piece::rook(White)));
        assert_eq!(b.board[9][3], Square::from(Piece::knight(White)));
        assert_eq!(b.board[9][4], Square::from(Piece::bishop(White)));
        assert_eq!(b.board[9][5], Square::from(Piece::queen(White)));
        assert_eq!(b.board[9][6], Square::from(Piece::king(White)));
        assert_eq!(b.board[9][7], Square::from(Piece::bishop(White)));
        assert_eq!(b.board[9][8], Square::from(Piece::knight(White)));
        assert_eq!(b.board[9][9], Square::from(Piece::rook(White)));

        for i in BOARD_START..BOARD_END {
            assert_eq!(b.board[8][i], Square::from(Piece::pawn(White)));
        }

        assert_eq!(b.zobrist_key, 321564624691785580);
    }

    #[test]
    fn correct_en_passant_privileges() {
        let b = BoardState::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e4 0 1")
            .unwrap();
        assert_eq!(b.pawn_double_move.unwrap().0, BOARD_START + 4);
        assert_eq!(b.pawn_double_move.unwrap().1, BOARD_START + 4);
    }

    #[test]
    fn correct_en_passant_privileges_black() {
        let b = BoardState::from_fen("rnbqkbnr/ppppppp1/8/7p/8/8/PPPPPPPP/RNBQKBNR w KQkq h5 0 1")
            .unwrap();
        assert_eq!(b.pawn_double_move.unwrap().0, BOARD_START + 3);
        assert_eq!(b.pawn_double_move.unwrap().1, BOARD_START + 7);
    }

    #[test]
    fn correct_king_location() {
        let b = BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
        assert_eq!(b.black_king_location, Point(2, 6));
        assert_eq!(b.white_king_location, Point(9, 6));
    }

    #[test]
    fn correct_king_location_two() {
        let b =
            BoardState::from_fen("6rk/1b4np/5pp1/1p6/8/1P3NP1/1B3P1P/5RK1 w KQkq - 0 1").unwrap();
        assert_eq!(b.black_king_location, Point(2, 9));
        assert_eq!(b.white_king_location, Point(9, 8));
    }

    #[test]
    fn correct_starting_player() {
        let mut b =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        assert_eq!(b.to_move, PieceColor::White);
        b = BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1")
            .unwrap();
        assert_eq!(b.to_move, PieceColor::Black);
    }

    #[test]
    fn correct_castling_privileges() {
        let mut b =
            BoardState::from_fen("6rk/1b4np/5pp1/1p6/8/1P3NP1/1B3P1P/5RK1 w KQkq - 0 1").unwrap();
        assert!(b.black_king_side_castle);
        assert!(b.black_queen_side_castle);
        assert!(b.white_king_side_castle);
        assert!(b.white_queen_side_castle);

        b = BoardState::from_fen("6rk/1b4np/5pp1/1p6/8/1P3NP1/1B3P1P/5RK1 w - - 0 1").unwrap();
        assert!(!b.black_king_side_castle);
        assert!(!b.black_queen_side_castle);
        assert!(!b.white_king_side_castle);
        assert!(!b.white_queen_side_castle);

        b = BoardState::from_fen("6rk/1b4np/5pp1/1p6/8/1P3NP1/1B3P1P/5RK1 w Kq - 0 1").unwrap();
        assert!(!b.black_king_side_castle);
        assert!(b.black_queen_side_castle);
        assert!(b.white_king_side_castle);
        assert!(!b.white_queen_side_castle);
    }

    #[test]
    fn random_pos() {
        let b =
            BoardState::from_fen("4R1B1/1kp5/1B1Q4/1P5p/1p2p1pK/8/3pP3/4N1b1 w - - 0 1").unwrap();
        assert_eq!(b.board[2][6], Square::from(Piece::rook(White)));
        assert_eq!(b.board[2][8], Square::from(Piece::bishop(White)));
        assert_eq!(b.board[3][3], Square::from(Piece::king(Black)));
        assert_eq!(b.board[3][4], Square::from(Piece::pawn(Black)));
        assert_eq!(b.board[4][3], Square::from(Piece::bishop(White)));
        assert_eq!(b.board[4][5], Square::from(Piece::queen(White)));
        assert_eq!(b.board[5][3], Square::from(Piece::pawn(White)));
        assert_eq!(b.board[5][9], Square::from(Piece::pawn(Black)));
        assert_eq!(b.board[6][3], Square::from(Piece::pawn(Black)));
        assert_eq!(b.board[6][6], Square::from(Piece::pawn(Black)));
        assert_eq!(b.board[6][8], Square::from(Piece::pawn(Black)));
        assert_eq!(b.board[6][9], Square::from(Piece::king(White)));
        assert_eq!(b.board[8][5], Square::from(Piece::pawn(Black)));
        assert_eq!(b.board[8][6], Square::from(Piece::pawn(White)));
        assert_eq!(b.board[9][6], Square::from(Piece::knight(White)));
        assert_eq!(b.board[9][8], Square::from(Piece::bishop(Black)));
    }

    #[test]
    #[should_panic]
    fn bad_fen_string() {
        BoardState::from_fen("this isn't a fen string").unwrap();
    }

    #[test]
    #[should_panic]
    fn bad_fen_string_bad_char() {
        BoardState::from_fen("rnbqkbnH/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    }

    #[test]
    #[should_panic]
    fn bad_fen_string_too_many_chars() {
        BoardState::from_fen("rnbqkbnrrrrr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
    }
}
