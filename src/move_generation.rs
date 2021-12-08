pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};
pub use crate::evaluation::*;

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

// MVV-LVA score, see https://www.chessprogramming.org/MVV-LVA
// addressed as [victim][attacker]
#[rustfmt::skip]
const MVV_LVA: [[i32; 7]; 7] = [
    [0,  0,  0,  0,  0,  0,  0],
    [50, 51, 52, 53, 54, 55, 0],
    [40, 41, 42, 43, 44, 45, 0],
    [30, 31, 32, 33, 34, 35, 0],
    [20, 21, 22, 23, 24, 25, 0],
    [10, 11, 12, 13, 14, 15, 0],
    [ 0,  0,  0,  0,  0,  0, 0],
];

#[derive(PartialEq, Eq)]
pub enum CastlingType {
    WhiteKingSide,
    WhiteQueenSide,
    BlackKingSide,
    BlackQueenSide,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum MoveGenerationMode {
    AllMoves,
    CapturesOnly,
}

const WHITE_KING_SIDE_CASTLE_ALG: Option<(Point, Point)> = Some((Point(9, 6), Point(9, 8)));
const WHITE_QUEEN_SIDE_CASTLE_ALG: Option<(Point, Point)> = Some((Point(9, 6), Point(9, 4)));
const BLACK_KING_SIDE_CASTLE_ALG: Option<(Point, Point)> = Some((Point(2, 6), Point(2, 8)));
const BLACK_QUEEN_SIDE_CASTLE_ALG: Option<(Point, Point)> = Some((Point(2, 6), Point(2, 4)));

/*
    Generate all possible *legal* moves from the given board
    Also sets appropriate variables for the board state
*/
pub fn generate_moves(board: &BoardState, move_gen_mode: MoveGenerationMode) -> Vec<BoardState> {
    //usually there is at minimum 16 moves in a position, so it make sense to preallocate some space to avoid excessive reallocations
    let mut new_moves: Vec<BoardState> = Vec::with_capacity(16);

    for i in BOARD_START..BOARD_END {
        for j in BOARD_START..BOARD_END {
            if let Square::Full(piece) = board.board[i][j] {
                if piece.color == board.to_move {
                    generate_moves_for_piece(
                        piece,
                        board,
                        Point(i, j),
                        &mut new_moves,
                        move_gen_mode,
                    );
                }
            }
        }
    }

    if move_gen_mode == MoveGenerationMode::AllMoves {
        generate_castling_moves(board, &mut new_moves);
    }
    new_moves
}

/*
    Determine if a color is currently in check
*/
pub fn is_check(board: &BoardState, color: PieceColor) -> bool {
    match color {
        White => is_check_cords(board, White, board.white_king_location),
        Black => is_check_cords(board, Black, board.black_king_location),
    }
}

/*
    Generate pseudo-legal moves for a knight
*/
fn knight_moves(
    piece: Piece,
    row: usize,
    col: usize,
    board: &BoardState,
    moves: &mut Vec<Point>,
    move_generation_mode: MoveGenerationMode,
) {
    for (r, c) in &KNIGHT_CORDS {
        let row = (row as i8 + r) as usize;
        let col = (col as i8 + c) as usize;
        let square = board.board[row][col];

        if square.is_empty_or_color(piece.color.opposite()) {
            if move_generation_mode == MoveGenerationMode::CapturesOnly {
                if !square.is_empty() {
                    moves.push(Point(row, col));
                }
            } else {
                moves.push(Point(row, col));
            }
        }
    }
}

/*
    Generate pseudo-legal moves for a pawn
*/
fn pawn_moves(
    piece: Piece,
    row: usize,
    col: usize,
    board: &BoardState,
    moves: &mut Vec<Point>,
    move_generation_mode: MoveGenerationMode,
) {
    match piece.color {
        // white pawns move up board
        White => {
            // check capture
            let left_cap = board.board[row - 1][col - 1];
            let right_cap = board.board[row - 1][col + 1];
            if let Square::Full(Piece { color: Black, .. }) = left_cap {
                moves.push(Point(row - 1, col - 1));
            }
            if let Square::Full(Piece { color: Black, .. }) = right_cap {
                moves.push(Point(row - 1, col + 1));
            }

            // check a normal push
            if move_generation_mode == MoveGenerationMode::AllMoves
                && (board.board[row - 1][col]).is_empty()
            {
                moves.push(Point(row - 1, col));
                // check double push
                if row == 8 && (board.board[row - 2][col]).is_empty() {
                    moves.push(Point(row - 2, col));
                }
            }
        }
        // black pawns move down board
        Black => {
            // check capture
            let left_cap = board.board[row + 1][col + 1];
            let right_cap = board.board[row + 1][col - 1];
            if let Square::Full(Piece { color: White, .. }) = left_cap {
                moves.push(Point(row + 1, col + 1));
            }
            if let Square::Full(Piece { color: White, .. }) = right_cap {
                moves.push(Point(row + 1, col - 1));
            }

            // check a normal push
            if move_generation_mode == MoveGenerationMode::AllMoves
                && (board.board[row + 1][col]).is_empty()
            {
                moves.push(Point(row + 1, col));
                // check double push
                if row == 3 && (board.board[row + 2][col]).is_empty() {
                    moves.push(Point(row + 2, col));
                }
            }
        }
    }
}

/*
    Generate pseudo-legal en passant moves

    Uses the pawn_double_move cords to decide if a en passant capture is legal

    Returns None if no legal move is available, otherwise return the coordinates of the capture
*/

fn pawn_moves_en_passant(
    piece: Piece,
    row: usize,
    col: usize,
    board: &BoardState,
) -> Option<Point> {
    if let Some(double_moved_pawn) = board.pawn_double_move {
        let left_cap;
        let right_cap;

        match piece.color {
            White if row == BOARD_START + 3 => {
                left_cap = Point(row - 1, col - 1);
                right_cap = Point(row - 1, col + 1);
            }
            Black if row == BOARD_START + 4 => {
                left_cap = Point(row + 1, col + 1);
                right_cap = Point(row + 1, col - 1);
            }
            _ => return None,
        }

        if left_cap == double_moved_pawn {
            return Some(left_cap);
        } else if right_cap == double_moved_pawn {
            return Some(right_cap);
        }
    }

    None
}

/*
    Generate pseudo-legal moves for a king
*/
fn king_moves(
    piece: Piece,
    row: usize,
    col: usize,
    board: &BoardState,
    moves: &mut Vec<Point>,
    move_generation_mode: MoveGenerationMode,
) {
    for i in 0..3 {
        let row = row + i - 1;
        for j in 0..3 {
            let col = col + j - 1;
            let square = board.board[row][col];

            if square.is_empty_or_color(piece.color.opposite()) {
                if move_generation_mode == MoveGenerationMode::CapturesOnly {
                    if !square.is_empty() {
                        moves.push(Point(row, col));
                    }
                } else {
                    moves.push(Point(row, col));
                }
            }
        }
    }
}

/*
    Generate pseudo-legal moves for a rook
*/
fn rook_moves(
    piece: Piece,
    row: usize,
    col: usize,
    board: &BoardState,
    moves: &mut Vec<Point>,
    move_generation_mode: MoveGenerationMode,
) {
    for (r, c) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let mut row = row as i8 + r;
        let mut col = col as i8 + c;
        let mut square = board.board[row as usize][col as usize];
        while square.is_empty() {
            if move_generation_mode == MoveGenerationMode::AllMoves {
                moves.push(Point(row as usize, col as usize));
            }
            row += r;
            col += c;
            square = board.board[row as usize][col as usize];
        }

        if square.is_color(piece.color.opposite()) {
            moves.push(Point(row as usize, col as usize));
        }
    }
}

/*
    Generate pseudo-legal moves for a bishop
*/
fn bishop_moves(
    piece: Piece,
    row: usize,
    col: usize,
    board: &BoardState,
    moves: &mut Vec<Point>,
    move_generation_mode: MoveGenerationMode,
) {
    for (r, c) in &[(1, -1), (1, 1), (-1, 1), (-1, -1)] {
        let mut row = row as i8 + r;
        let mut col = col as i8 + c;
        let mut square = board.board[row as usize][col as usize];
        while square.is_empty() {
            if move_generation_mode == MoveGenerationMode::AllMoves {
                moves.push(Point(row as usize, col as usize));
            }
            row += r;
            col += c;
            square = board.board[row as usize][col as usize];
        }

        if square.is_color(piece.color.opposite()) {
            moves.push(Point(row as usize, col as usize));
        }
    }
}

/*
    Generate pseudo-legal moves for a queen
*/
fn queen_moves(
    piece: Piece,
    row: usize,
    col: usize,
    board: &BoardState,
    moves: &mut Vec<Point>,
    move_generation_mode: MoveGenerationMode,
) {
    rook_moves(piece, row, col, board, moves, move_generation_mode);
    bishop_moves(piece, row, col, board, moves, move_generation_mode);
}

/*
    Generate pseudo-legal moves for a piece
    This will not generate en passants and castling, these cases are handled separately
*/
fn get_moves(
    piece: Piece,
    row: usize,
    col: usize,
    board: &BoardState,
    moves: &mut Vec<Point>,
    move_generation_mode: MoveGenerationMode,
) {
    match piece.kind {
        Pawn => pawn_moves(piece, row, col, board, moves, move_generation_mode),
        Rook => rook_moves(piece, row, col, board, moves, move_generation_mode),
        Bishop => bishop_moves(piece, row, col, board, moves, move_generation_mode),
        Knight => knight_moves(piece, row, col, board, moves, move_generation_mode),
        King => king_moves(piece, row, col, board, moves, move_generation_mode),
        Queen => queen_moves(piece, row, col, board, moves, move_generation_mode),
    }
}

/*
    Determine if the given position is check

    Rather than checking each piece to see if it attacks the king
    this function checks all possible attack squares to the king and
    sees if the piece is there, thus it is important the king_location is set
*/
fn is_check_cords(board: &BoardState, color: PieceColor, square_cords: Point) -> bool {
    let attacking_color = color.opposite();
    let attacking_rook = Piece::rook(attacking_color);
    let attacking_queen = Piece::queen(attacking_color);
    let attacking_bishop = Piece::bishop(attacking_color);
    let attacking_knight = Piece::knight(attacking_color);
    let attacking_pawn = Piece::pawn(attacking_color);

    // Check from rook or queen
    for (r, c) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let mut row = square_cords.0 as i8 + r;
        let mut col = square_cords.1 as i8 + c;
        let mut square = board.board[row as usize][col as usize];
        while square.is_empty() {
            row += r;
            col += c;
            square = board.board[row as usize][col as usize];
        }

        if square == attacking_rook || square == attacking_queen {
            return true;
        }
    }

    // Check from bishop or queen
    for (r, c) in &[(1, -1), (1, 1), (-1, 1), (-1, -1)] {
        let mut row = square_cords.0 as i8 + r;
        let mut col = square_cords.1 as i8 + c;
        let mut square = board.board[row as usize][col as usize];
        while square.is_empty() {
            row += r;
            col += c;
            square = board.board[row as usize][col as usize];
        }

        if square == attacking_bishop || square == attacking_queen {
            return true;
        }
    }

    // Check from knight
    for (r, c) in &KNIGHT_CORDS {
        let row = (square_cords.0 as i8 + r) as usize;
        let col = (square_cords.1 as i8 + c) as usize;
        let square = board.board[row][col];

        if square == attacking_knight {
            return true;
        }
    }

    // Check from pawn
    let pawn_row = match color {
        White => square_cords.0 - 1,
        Black => square_cords.0 + 1,
    };

    if board.board[pawn_row][square_cords.1 - 1] == attacking_pawn
        || board.board[pawn_row][square_cords.1 + 1] == attacking_pawn
    {
        return true;
    }

    // Check from king
    // By using the king location here we can just check if they are within one square of each other
    (board.black_king_location.0 as i8 - board.white_king_location.0 as i8).abs() <= 1
        && (board.black_king_location.1 as i8 - board.white_king_location.1 as i8).abs() <= 1
}

/*
    Determine if castling is a legal move

    Rules
    1. The castling must be kingside or queen side.
    2. Neither the king nor the chosen rook has previously moved.
    3. There are no pieces between the king and the chosen rook.
    4. The king is not currently in check.
    5. The king does not pass through a square that is attacked by an enemy piece.
    6. The king does not end up in check. (True of any legal move.)

    This method will check all but rule 2

    This method will check the board state to determine if is should go ahead with the castling check
    If the associated castling privilege variable is set to true, the following will be assumed by this function
    1. The king and associated rook have not moved yet this game
    2. The king and associated rook are in the correct castling positions

    Thus its the responsibility of other functions to update the castling privilege variables when the king or associated rook moves (including castling)

*/
fn can_castle(board: &BoardState, castling_type: &CastlingType) -> bool {
    match castling_type {
        CastlingType::WhiteKingSide => can_castle_white_king_side(board),
        CastlingType::WhiteQueenSide => can_castle_white_queen_side(board),
        CastlingType::BlackKingSide => can_castle_black_king_side(board),
        CastlingType::BlackQueenSide => can_castle_black_queen_side(board),
    }
}

fn can_castle_white_king_side(board: &BoardState) -> bool {
    if !board.white_king_side_castle {
        return false;
    }
    // check that squares required for castling are empty
    if !(board.board[BOARD_END - 1][BOARD_END - 3]).is_empty()
        || !(board.board[BOARD_END - 1][BOARD_END - 2]).is_empty()
    {
        return false;
    }
    // check that the king currently isn't in check
    if is_check(board, White) {
        return false;
    }
    //check that the squares required for castling are not threatened
    if is_check_cords(board, White, Point(BOARD_END - 1, BOARD_END - 3))
        || is_check_cords(board, White, Point(BOARD_END - 1, BOARD_END - 2))
    {
        return false;
    }

    true
}

fn can_castle_white_queen_side(board: &BoardState) -> bool {
    if !board.white_queen_side_castle {
        return false;
    }
    // check that squares required for castling are empty
    if !(board.board[BOARD_END - 1][BOARD_START + 1]).is_empty()
        || !(board.board[BOARD_END - 1][BOARD_START + 2]).is_empty()
        || !(board.board[BOARD_END - 1][BOARD_START + 3]).is_empty()
    {
        return false;
    }
    // check that the king currently isn't in check
    if is_check(board, White) {
        return false;
    }
    //check that the squares required for castling are not threatened
    if is_check_cords(board, White, Point(BOARD_END - 1, BOARD_START + 3))
        || is_check_cords(board, White, Point(BOARD_END - 1, BOARD_START + 2))
    {
        return false;
    }

    true
}

fn can_castle_black_king_side(board: &BoardState) -> bool {
    if !board.black_king_side_castle {
        return false;
    }
    // check that squares required for castling are empty
    if !(board.board[BOARD_START][BOARD_END - 3]).is_empty()
        || !(board.board[BOARD_START][BOARD_END - 2]).is_empty()
    {
        return false;
    }
    // check that the king currently isn't in check
    if is_check(board, Black) {
        return false;
    }
    //check that the squares required for castling are not threatened
    if is_check_cords(board, Black, Point(BOARD_START, BOARD_END - 3))
        || is_check_cords(board, Black, Point(BOARD_START, BOARD_END - 2))
    {
        return false;
    }

    true
}

fn can_castle_black_queen_side(board: &BoardState) -> bool {
    if !board.black_queen_side_castle {
        return false;
    }
    // check that squares required for castling are empty
    if !(board.board[BOARD_START][BOARD_START + 1]).is_empty()
        || !(board.board[BOARD_START][BOARD_START + 2]).is_empty()
        || !(board.board[BOARD_START][BOARD_START + 3]).is_empty()
    {
        return false;
    }
    // check that the king currently isn't in check
    if is_check(board, Black) {
        return false;
    }
    //check that the squares required for castling are not threatened
    if is_check_cords(board, Black, Point(BOARD_START, BOARD_START + 2))
        || is_check_cords(board, Black, Point(BOARD_START, BOARD_START + 3))
    {
        return false;
    }

    true
}

/*
    Given the coordinates of a piece and that pieces color, generate all possible pseudo-legal moves for that piece
*/
fn generate_moves_for_piece(
    piece: Piece,
    board: &BoardState,
    square_cords: Point,
    new_moves: &mut Vec<BoardState>,
    move_generation_mode: MoveGenerationMode,
) {
    let mut moves: Vec<Point> = Vec::new();
    let Piece { color, kind } = piece;
    get_moves(
        piece,
        square_cords.0,
        square_cords.1,
        board,
        &mut moves,
        move_generation_mode,
    );

    // make all the valid moves of this piece
    for mov in moves {
        let mut new_board = board.clone();
        new_board.pawn_promotion = None;
        new_board.swap_color();

        // update king location if we are moving the king
        if kind == King {
            match color {
                White => new_board.white_king_location = mov,
                Black => new_board.black_king_location = mov,
            }
        }

        let target_square = new_board.board[mov.0][mov.1];
        if let Square::Full(target_piece) = target_square {
            new_board.order_heuristic = MVV_LVA[target_piece.index()][piece.index()];
        } else {
            // by default all moves are given a minimum score
            new_board.order_heuristic = i32::MIN;
        }

        // move the piece, this will take care of any captures as well, excluding en passant
        new_board.board[mov.0][mov.1] = piece.into();
        new_board.board[square_cords.0][square_cords.1] = Square::Empty;
        new_board.last_move = Some((square_cords, mov));

        // if you make your move, and you are in check, this move is not valid
        if is_check(&new_board, color) {
            continue;
        }

        // if the rook or king move, take away castling privileges
        if kind == King {
            if color == White {
                new_board.white_king_side_castle = false;
                new_board.white_queen_side_castle = false;
            } else {
                new_board.black_queen_side_castle = false;
                new_board.black_king_side_castle = false;
            }
        } else if square_cords.0 == BOARD_END - 1 && square_cords.1 == BOARD_END - 1 {
            new_board.white_king_side_castle = false;
        } else if square_cords.0 == BOARD_END - 1 && square_cords.1 == BOARD_START {
            new_board.white_queen_side_castle = false;
        } else if square_cords.0 == BOARD_START && square_cords.1 == BOARD_START {
            new_board.black_queen_side_castle = false;
        } else if square_cords.0 == BOARD_START && square_cords.1 == BOARD_END - 1 {
            new_board.black_king_side_castle = false;
        }

        // if the rook is captured, take away castling privileges
        if mov.0 == BOARD_END - 1 && mov.1 == BOARD_END - 1 {
            new_board.white_king_side_castle = false;
        } else if mov.0 == BOARD_END - 1 && mov.1 == BOARD_START {
            new_board.white_queen_side_castle = false;
        } else if mov.0 == BOARD_START && mov.1 == BOARD_START {
            new_board.black_queen_side_castle = false;
        } else if mov.0 == BOARD_START && mov.1 == BOARD_END - 1 {
            new_board.black_king_side_castle = false;
        }

        // checks if the pawn has moved two spaces, if it has it can be captured en passant, record the space *behind* the pawn ie the valid capture square
        if move_generation_mode == MoveGenerationMode::AllMoves {
            if kind == Pawn && (square_cords.0 as i8 - mov.0 as i8).abs() == 2 {
                if color == White {
                    new_board.pawn_double_move = Some(Point(mov.0 + 1, mov.1));
                } else {
                    new_board.pawn_double_move = Some(Point(mov.0 - 1, mov.1));
                }
            } else {
                // the most recent move was not a double pawn move, unset any possibly existing pawn double move
                new_board.pawn_double_move = None;
            }
            // deal with pawn promotions
            if mov.0 == BOARD_START && color == White && kind == Pawn {
                promote_pawn(&new_board, White, square_cords, mov, new_moves);
            } else if mov.0 == BOARD_END - 1 && color == Black && kind == Pawn {
                promote_pawn(&new_board, Black, square_cords, mov, new_moves);
            } else {
                new_moves.push(new_board);
            }
        } else {
            new_moves.push(new_board);
        }
    }

    // take care of en passant captures
    if board.pawn_double_move.is_some() && kind == Pawn {
        let en_passant = pawn_moves_en_passant(piece, square_cords.0, square_cords.1, board);
        if let Some(mov) = en_passant {
            let mut new_board = board.clone();
            new_board.last_move = Some((square_cords, mov));
            new_board.swap_color();
            new_board.pawn_double_move = None;
            new_board.board[mov.0][mov.1] = piece.into();
            new_board.board[square_cords.0][square_cords.1] = Square::Empty;
            if color == White {
                new_board.board[mov.0 + 1][mov.1] = Square::Empty;
            } else {
                new_board.board[mov.0 - 1][mov.1] = Square::Empty;
            }

            // if you make a move, and you do not end up in check, then this move is valid
            if !is_check(&new_board, board.to_move) {
                new_moves.push(new_board);
            }
        }
    }
}

/*
    Given the current board, attempt to castle
    If castling is possible add the move the the list of possible moves
    Will also update appropriate castling variables if castling was successful
*/
fn generate_castling_moves(board: &BoardState, new_moves: &mut Vec<BoardState>) {
    if board.to_move == White && can_castle(board, &CastlingType::WhiteKingSide) {
        let mut new_board = board.clone();
        new_board.swap_color();
        new_board.pawn_double_move = None;
        new_board.white_king_side_castle = false;
        new_board.white_queen_side_castle = false;
        new_board.white_king_location = Point(BOARD_END - 1, BOARD_END - 2);
        new_board.board[BOARD_END - 1][BOARD_START + 4] = Square::Empty;
        new_board.board[BOARD_END - 1][BOARD_END - 1] = Square::Empty;
        new_board.board[BOARD_END - 1][BOARD_END - 2] = Piece::king(White).into();
        new_board.board[BOARD_END - 1][BOARD_END - 3] = Piece::rook(White).into();
        new_board.last_move = WHITE_KING_SIDE_CASTLE_ALG;
        new_moves.push(new_board);
    }

    if board.to_move == White && can_castle(board, &CastlingType::WhiteQueenSide) {
        let mut new_board = board.clone();
        new_board.swap_color();
        new_board.pawn_double_move = None;
        new_board.white_king_side_castle = false;
        new_board.white_queen_side_castle = false;
        new_board.white_king_location = Point(BOARD_END - 1, BOARD_START + 2);
        new_board.board[BOARD_END - 1][BOARD_START + 4] = Square::Empty;
        new_board.board[BOARD_END - 1][BOARD_START] = Square::Empty;
        new_board.board[BOARD_END - 1][BOARD_START + 2] = Piece::king(White).into();
        new_board.board[BOARD_END - 1][BOARD_START + 3] = Piece::rook(White).into();
        new_board.last_move = WHITE_QUEEN_SIDE_CASTLE_ALG;
        new_moves.push(new_board);
    }

    if board.to_move == Black && can_castle(board, &CastlingType::BlackKingSide) {
        let mut new_board = board.clone();
        new_board.swap_color();
        new_board.pawn_double_move = None;
        new_board.black_king_side_castle = false;
        new_board.black_queen_side_castle = false;
        new_board.black_king_location = Point(BOARD_START, BOARD_END - 2);
        new_board.board[BOARD_START][BOARD_START + 4] = Square::Empty;
        new_board.board[BOARD_START][BOARD_END - 1] = Square::Empty;
        new_board.board[BOARD_START][BOARD_END - 2] = Piece::king(Black).into();
        new_board.board[BOARD_START][BOARD_END - 3] = Piece::rook(Black).into();
        new_board.last_move = BLACK_KING_SIDE_CASTLE_ALG;
        new_moves.push(new_board);
    }

    if board.to_move == Black && can_castle(board, &CastlingType::BlackQueenSide) {
        let mut new_board = board.clone();
        new_board.swap_color();
        new_board.pawn_double_move = None;
        new_board.black_king_side_castle = false;
        new_board.black_queen_side_castle = false;
        new_board.black_king_location = Point(BOARD_START, BOARD_START + 2);
        new_board.board[BOARD_START][BOARD_START + 4] = Square::Empty;
        new_board.board[BOARD_START][BOARD_START] = Square::Empty;
        new_board.board[BOARD_START][BOARD_START + 2] = Piece::king(Black).into();
        new_board.board[BOARD_START][BOARD_START + 3] = Piece::rook(Black).into();
        new_board.last_move = BLACK_QUEEN_SIDE_CASTLE_ALG;
        new_moves.push(new_board);
    }
}

/*
    Executes a pawn promotion on the given cords

    This function assumes that the board state is a valid pawn promotion and does not do additional checks
*/
const PAWN_PROMOTION_SCORE: i32 = 800; // queen value - pawn value
fn promote_pawn(
    board: &BoardState,
    color: PieceColor,
    start: Point,
    target: Point,
    moves: &mut Vec<BoardState>,
) {
    for kind in &[Queen, Knight, Bishop, Rook] {
        let mut new_board = board.clone();
        new_board.pawn_double_move = None;
        let promotion_piece = Piece { color, kind: *kind };
        new_board.board[target.0][target.1] = Square::Full(promotion_piece);
        new_board.last_move = Some((start, target));
        new_board.pawn_promotion = Some(promotion_piece);
        new_board.order_heuristic = PAWN_PROMOTION_SCORE; // a pawn promotion is usually a good idea
        moves.push(new_board);
    }
}

/*
    Generate all valid moves recursively given the current board state

    Will generate up until cur_depth = depth
*/
pub fn generate_moves_test(
    board: &BoardState,
    cur_depth: usize,
    depth: usize,
    move_counts: &mut [u32],
    should_evaluate: bool,
) {
    if cur_depth == depth {
        if should_evaluate {
            // we don't do anything with this score, we just calculate it at the leaf for
            // performance testing purposes
            get_evaluation(board);
        }
        return;
    }

    let moves = generate_moves(board, MoveGenerationMode::AllMoves);
    move_counts[cur_depth] += moves.len() as u32;
    for mov in moves {
        generate_moves_test(&mov, cur_depth + 1, depth, move_counts, should_evaluate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_sanity_test() {
        let b = BoardState::from_fen("8/8/8/8/3K4/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, White));
    }

    #[test]
    fn knight_checks() {
        let mut b = BoardState::from_fen("8/8/4n3/8/3K4/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, White));

        b = BoardState::from_fen("8/8/8/8/8/8/1RK5/nRB5 w - - 0 1").unwrap();
        assert!(is_check(&b, White));

        b = BoardState::from_fen("8/8/8/8/3k4/5N2/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, Black));

        b = BoardState::from_fen("8/8/8/8/3k4/5n2/8/7N w - - 0 1").unwrap();
        assert!(!is_check(&b, Black));

        b = BoardState::from_fen("8/8/2N5/8/3k4/5n2/8/7N w - - 0 1").unwrap();
        assert!(is_check(&b, Black));
    }

    #[test]
    fn pawn_checks() {
        let mut b = BoardState::from_fen("8/8/8/4k3/3P4/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, Black));

        b = BoardState::from_fen("8/8/8/4k3/5P2/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, Black));

        b = BoardState::from_fen("8/8/8/4k3/4P3/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, Black));

        b = BoardState::from_fen("8/8/3PPP2/4k3/8/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, Black));

        b = BoardState::from_fen("8/8/8/8/8/5p2/6K1/8 w - - 0 1").unwrap();
        assert!(is_check(&b, White));

        b = BoardState::from_fen("8/8/8/8/8/7p/6K1/8 w - - 0 1").unwrap();
        assert!(is_check(&b, White));

        b = BoardState::from_fen("8/8/8/8/8/6p1/6K1/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, White));

        b = BoardState::from_fen("8/8/8/8/8/6K1/5ppp/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, White));
    }

    #[test]
    fn rook_checks() {
        let mut b = BoardState::from_fen("8/8/8/R3k3/8/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, Black));

        b = BoardState::from_fen("8/8/8/R1r1k3/8/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, Black));

        b = BoardState::from_fen("8/8/8/R1r1k3/8/8/8/4R3 w - - 0 1").unwrap();
        assert!(is_check(&b, Black));

        b = BoardState::from_fen("4R3/8/8/R1r5/8/8/8/4k3 w - - 0 1").unwrap();
        assert!(is_check(&b, Black));

        b = BoardState::from_fen("8/8/8/R1r5/8/8/7R/4k3 w - - 0 1").unwrap();
        assert!(!is_check(&b, Black));

        b = BoardState::from_fen("4R3/8/8/8/8/3r4/R3K2R/2r1Rr2 w - - 0 1").unwrap();
        assert!(!is_check(&b, White));

        b = BoardState::from_fen("4R3/8/8/8/4K3/3r4/R6R/2r1rr2 w - - 0 1").unwrap();
        assert!(is_check(&b, White));

        b = BoardState::from_fen("4R3/8/8/8/4K2r/3r4/R6R/2r2r2 w - - 0 1").unwrap();
        assert!(is_check(&b, White));

        b = BoardState::from_fen("4r3/8/8/4B3/r2QKP1r/3rR3/R6R/2r1rr2 w - - 0 1").unwrap();
        assert!(!is_check(&b, White));
    }

    #[test]
    fn bishop_checks() {
        let mut b = BoardState::from_fen("8/8/8/1B6/8/8/8/5k2 w - - 0 1").unwrap();
        assert!(is_check(&b, Black));

        b = BoardState::from_fen("8/8/2B1B3/1B3B2/1B1k1B2/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, Black));

        b = BoardState::from_fen("8/8/8/8/5k2/8/8/2B5 w - - 0 1").unwrap();
        assert!(is_check(&b, Black));

        b = BoardState::from_fen("8/8/8/8/5k2/4n3/8/2B5 w - - 0 1").unwrap();
        assert!(!is_check(&b, Black));

        b = BoardState::from_fen("8/8/8/8/3K4/8/8/6b1 w - - 0 1").unwrap();
        assert!(is_check(&b, White));

        b = BoardState::from_fen("8/8/8/8/3K4/4r3/8/6b1 w - - 0 1").unwrap();
        assert!(!is_check(&b, White));

        b = BoardState::from_fen("8/8/8/8/3K4/4r3/8/b5b1 w - - 0 1").unwrap();
        assert!(is_check(&b, White));

        b = BoardState::from_fen("8/8/8/8/3K4/2P1r3/8/b5b1 w - - 0 1").unwrap();
        assert!(!is_check(&b, White));
    }

    #[test]
    fn queen_checks() {
        let mut b = BoardState::from_fen("8/8/8/8/3k1Q2/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, Black));

        b = BoardState::from_fen("8/8/2k5/8/8/8/6Q1/8 w - - 0 1").unwrap();
        assert!(is_check(&b, Black));

        b = BoardState::from_fen("8/8/2K5/8/3q4/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, White));

        b = BoardState::from_fen("8/8/1K6/2Q5/3q4/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, White));

        b = BoardState::from_fen("8/5Q2/1K6/8/3q4/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, White));

        b = BoardState::from_fen("8/5Q2/1K6/1P6/8/8/1q6/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, White));

        b = BoardState::from_fen("8/2P2Q2/1K6/8/8/8/1q6/8 w - - 0 1").unwrap();
        assert!(is_check(&b, White));
    }

    // Knight tests

    #[test]
    fn knight_moves_empty_board() {
        let b = BoardState::from_fen("8/8/8/8/3N4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        knight_moves(
            Piece::knight(White),
            6,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn knight_moves_corner() {
        let b = BoardState::from_fen("N7/8/8/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        knight_moves(
            Piece::knight(White),
            2,
            2,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 2);
    }
    #[test]
    fn knight_moves_with_other_pieces_with_capture() {
        let b = BoardState::from_fen("8/8/5n2/3NQ3/2K2P2/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        knight_moves(
            Piece::knight(White),
            5,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 7);
    }

    // Pawn tests - white pawn

    #[test]
    fn white_pawn_double_push() {
        let b = BoardState::from_fen("8/8/8/8/8/8/P7/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::pawn(White),
            8,
            2,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn white_pawn_has_moved() {
        let b = BoardState::from_fen("8/8/8/8/8/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::pawn(White),
            7,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn white_pawn_cant_move_black_piece_block() {
        let b = BoardState::from_fen("8/8/8/8/3r4/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::pawn(White),
            7,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_pawn_cant_move_white_piece_block() {
        let b = BoardState::from_fen("8/8/8/8/3K4/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::pawn(White),
            7,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_pawn_with_two_captures_and_start() {
        let b = BoardState::from_fen("8/8/8/8/8/n1q5/1P6/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::pawn(White),
            8,
            3,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn white_pawn_with_one_capture() {
        let b = BoardState::from_fen("8/8/Q1b5/1P6/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::pawn(White),
            5,
            3,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn white_pawn_double_push_piece_in_front() {
        let b = BoardState::from_fen("8/8/8/8/8/b7/P7/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::pawn(White),
            8,
            2,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_pawn_en_passant_left() {
        let b = BoardState::from_fen("8/8/8/3pP3/8/8/8/8 w - d6 0 1").unwrap();
        assert!(pawn_moves_en_passant(Piece::pawn(White), 5, 6, &b).is_some());
    }

    #[test]
    fn white_pawn_en_passant_right() {
        let b = BoardState::from_fen("8/8/8/4Pp2/8/8/8/8 w - f6 0 1").unwrap();
        assert!(pawn_moves_en_passant(Piece::pawn(White), 5, 6, &b).is_some());
    }

    #[test]
    fn white_pawn_en_passant_right_2() {
        let b = BoardState::from_fen("7K/8/7k/1Pp5/8/8/8/8 w - c6 0 1").unwrap();
        assert!(pawn_moves_en_passant(Piece::pawn(White), 5, 3, &b).is_some());
    }

    #[test]
    fn white_pawn_en_passant_wrong_row() {
        let b = BoardState::from_fen("8/8/8/8/4Pp2/8/8/8 w - f4 0 1").unwrap();
        assert!(pawn_moves_en_passant(Piece::pawn(White), 6, 6, &b).is_none());
    }

    #[test]
    fn white_en_passant_capture_not_available() {
        let b = BoardState::from_fen("8/8/8/4Pp2/8/8/8/8 w - - 0 1").unwrap();
        assert!(pawn_moves_en_passant(Piece::pawn(White), 5, 6, &b).is_none());
    }

    // Pawn tests - black pawn

    #[test]
    fn black_pawn_double_push() {
        let b = BoardState::from_fen("8/p7/8/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::pawn(Black),
            3,
            2,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn black_pawn_has_moved() {
        let b = BoardState::from_fen("8/8/8/3p4/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::pawn(Black),
            5,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn black_pawn_cant_move_white_piece_block() {
        let b = BoardState::from_fen("8/3p4/3R4/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::pawn(Black),
            3,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn black_pawn_with_two_captures_and_start() {
        let b = BoardState::from_fen("8/3p4/2R1R3/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::pawn(Black),
            3,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn black_pawn_with_one_capture() {
        let b = BoardState::from_fen("8/3p4/3qR3/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::pawn(Black),
            3,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn black_pawn_en_passant_left() {
        let b = BoardState::from_fen("8/8/8/8/1Pp5/8/8/8 w - b3 0 1").unwrap();
        assert!(pawn_moves_en_passant(Piece::pawn(Black), 6, 4, &b).is_some());
    }

    #[test]
    fn black_pawn_en_passant_right() {
        let b = BoardState::from_fen("8/8/8/8/pP6/8/8/8 w - b3 0 1").unwrap();
        assert!(pawn_moves_en_passant(Piece::pawn(Black), 6, 2, &b).is_some());
    }

    #[test]
    fn black_pawn_en_passant_wrong_row() {
        let b = BoardState::from_fen("8/8/8/pP6/8/8/8/8 w - b4 0 1").unwrap();
        assert!(pawn_moves_en_passant(Piece::pawn(Black), 5, 2, &b).is_none());
    }

    #[test]
    fn black_en_passant_capture_not_available() {
        let b = BoardState::from_fen("8/8/8/8/pP6/8/8/8 w - - 0 1").unwrap();
        assert!(pawn_moves_en_passant(Piece::pawn(Black), 6, 2, &b).is_none());
    }

    // King tests

    #[test]
    fn king_empty_board_center() {
        let b = BoardState::from_fen("8/8/8/8/3K4/8/8/k7 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        king_moves(
            Piece::king(White),
            6,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(dbg!(ret).len(), 8);
    }

    #[test]
    fn king_start_pos() {
        let b = BoardState::from_fen("8/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        king_moves(
            Piece::king(White),
            9,
            6,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 5);
    }

    #[test]
    fn king_start_pos_other_pieces() {
        let b = BoardState::from_fen("8/8/8/8/8/8/3Pn3/3QKB2 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        king_moves(
            Piece::king(White),
            9,
            6,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn king_black_other_pieces() {
        let b = BoardState::from_fen("8/8/8/8/8/3Pn3/3QkB2/3R1q2 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        king_moves(
            Piece::king(Black),
            8,
            6,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 6);
    }

    // Rook tests

    #[test]
    fn rook_center_of_empty_board() {
        let b = BoardState::from_fen("8/8/8/8/3R4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        rook_moves(
            Piece::rook(White),
            6,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 14);
    }

    #[test]
    fn rook_center_of_board() {
        let b = BoardState::from_fen("8/8/8/3q4/2kRp3/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        rook_moves(
            Piece::rook(White),
            6,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn rook_center_of_board_with_white_pieces() {
        let b = BoardState::from_fen("7p/3N4/8/4n3/2kR4/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        rook_moves(
            Piece::rook(White),
            6,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn rook_corner() {
        let b = BoardState::from_fen("7p/3N4/K7/4n3/2kR4/3b4/8/7R w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        rook_moves(
            Piece::rook(White),
            9,
            9,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 14);
    }
    #[test]
    fn black_rook_center_of_board_with_white_pieces() {
        let b = BoardState::from_fen("7p/3N4/8/4n3/2kr4/3b4/8/K7 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        rook_moves(
            Piece::rook(Black),
            6,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 7);
    }

    // Bishop tests

    #[test]
    fn black_bishop_center_empty_board() {
        let b = BoardState::from_fen("8/8/8/3b4/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        bishop_moves(
            Piece::bishop(Black),
            5,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 13);
    }

    #[test]
    fn black_bishop_center_with_captures() {
        let b = BoardState::from_fen("6P1/8/8/3b4/8/1R6/8/3Q4 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        bishop_moves(
            Piece::bishop(Black),
            5,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 12);
    }

    #[test]
    fn black_bishop_center_with_captures_and_black_pieces() {
        let b = BoardState::from_fen("6P1/8/2Q5/3b4/2k1n3/1R6/8/b2Q4 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        bishop_moves(
            Piece::bishop(Black),
            5,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn white_bishop_center_with_captures_and_white_pieces() {
        let b = BoardState::from_fen("8/8/8/4r3/5B2/8/3Q4/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        bishop_moves(
            Piece::bishop(White),
            6,
            7,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 6);
    }

    // Queen tests

    #[test]
    fn white_queen_empty_board() {
        let b = BoardState::from_fen("8/8/8/8/3Q4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        queen_moves(
            Piece::queen(White),
            6,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 27);
    }

    #[test]
    fn white_queen_cant_move() {
        let b = BoardState::from_fen("8/8/8/2NBR3/2PQR3/2RRR3/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        queen_moves(
            Piece::queen(White),
            6,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_queen_with_other_piece() {
        let b = BoardState::from_fen("8/6r1/8/8/3Q4/5N2/8/6P1 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        queen_moves(
            Piece::queen(White),
            6,
            5,
            &b,
            &mut ret,
            MoveGenerationMode::AllMoves,
        );
        assert_eq!(ret.len(), 25);
    }

    // Castling tests

    #[test]
    fn white_king_side_castle() {
        let mut b = BoardState::from_fen("8/8/8/8/8/8/8/4K2R w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, &CastlingType::WhiteKingSide));

        b = BoardState::from_fen("8/8/2b5/8/8/6P1/5P1P/4K2R w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, &CastlingType::WhiteKingSide));

        // Can't castle out of check
        b = BoardState::from_fen("4r3/8/2b5/8/8/6P1/5P1P/4K2R w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::WhiteKingSide));

        // Can't castle through check
        b = BoardState::from_fen("8/8/8/8/8/6Pb/5P1P/4K2R w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::WhiteKingSide));

        // Can't castle with pieces in way
        b = BoardState::from_fen("8/8/8/8/8/6PN/5P1P/4KP1R w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::WhiteKingSide));

        // Can't castle with pieces in way 2
        b = BoardState::from_fen("8/8/8/8/8/6PN/5P1P/4K1PR w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::WhiteKingSide));
    }

    #[test]
    fn white_queen_side_castle() {
        let mut b = BoardState::from_fen("8/8/8/8/8/8/8/R3K3 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, &CastlingType::WhiteQueenSide));

        b = BoardState::from_fen("8/8/8/8/8/2P5/PP1P4/R3K1N1 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, &CastlingType::WhiteQueenSide));

        // Can't castle out of check
        b = BoardState::from_fen("8/8/8/8/8/2P2n2/PP1P4/R3K1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::WhiteQueenSide));

        // Can't castle through check
        b = BoardState::from_fen("8/8/8/8/8/2n5/PP1P4/R3K1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::WhiteQueenSide));

        // Can't castle with pieces in way
        b = BoardState::from_fen("8/8/8/8/8/2P5/PP1P4/R2QK1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::WhiteQueenSide));

        // Can't castle with pieces in way 2
        b = BoardState::from_fen("8/8/8/8/8/2P5/PP1P4/R1Q1K1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::WhiteQueenSide));

        // Can't castle with pieces in way 3
        b = BoardState::from_fen("8/8/8/8/8/2P5/PP1P4/RQ2K1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::WhiteQueenSide));
    }

    #[test]
    fn black_king_side_castle() {
        let mut b = BoardState::from_fen("1p2k2r/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, &CastlingType::BlackKingSide));

        b = BoardState::from_fen("1p2k2r/4bp1p/6p1/8/8/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, &CastlingType::BlackKingSide));

        // Can't castle out of check
        b = BoardState::from_fen("1p2k2r/4bp1p/6p1/8/B7/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::BlackKingSide));

        // Can't castle through check
        b = BoardState::from_fen("1p2k2r/4bp1p/6pB/8/8/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::BlackKingSide));

        // Can't castle with pieces in way
        b = BoardState::from_fen("1p2k1nr/4bp1p/6pn/8/8/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::BlackKingSide));

        // Can't castle with pieces in way 2
        b = BoardState::from_fen("1p2kN1r/4bp1p/6pn/3n4/8/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::BlackKingSide));
    }

    #[test]
    fn black_queen_side_castle() {
        let mut b = BoardState::from_fen("r3k3/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, &CastlingType::BlackQueenSide));

        b = BoardState::from_fen("r3k3/qpb5/3n4/8/8/8/8/8 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, &CastlingType::BlackQueenSide));

        // Can't castle out of check
        b = BoardState::from_fen("r3k3/qpb5/3n4/8/8/8/8/4Q3 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::BlackQueenSide));

        // Can't castle through check
        b = BoardState::from_fen("r3k3/qpb5/3n4/8/7Q/8/8/8 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::BlackQueenSide));

        // Can't castle with pieces in way
        b = BoardState::from_fen("r2Pk3/qpb5/3n4/8/8/8/8/P7 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::BlackQueenSide));

        // Can't castle with pieces in way 2
        b = BoardState::from_fen("r1p1k3/qpb5/3n4/8/8/8/8/P7 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::BlackQueenSide));

        // Can't castle with pieces in way 3
        b = BoardState::from_fen("rn2k3/qpb5/3n4/8/8/8/8/P7 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, &CastlingType::BlackQueenSide));
    }

    #[test]
    fn generate_only_captures_queen() {
        let b = BoardState::from_fen("q3b3/1Q3n2/8/8/1R6/8/8/p6b w KQkq - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        queen_moves(
            Piece::queen(White),
            3,
            3,
            &b,
            &mut ret,
            MoveGenerationMode::CapturesOnly,
        );
        assert_eq!(ret.len(), 3);
    }

    #[test]
    fn generate_only_captures_bishop() {
        let b = BoardState::from_fen("q3b3/1B6/8/8/R7/8/8/p6b w KQkq - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        bishop_moves(
            Piece::bishop(White),
            3,
            3,
            &b,
            &mut ret,
            MoveGenerationMode::CapturesOnly,
        );
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn generate_only_captures_rook() {
        let b = BoardState::from_fen("R3b3/8/8/8/R7/8/8/p7 w KQkq - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        rook_moves(
            Piece::rook(White),
            2,
            2,
            &b,
            &mut ret,
            MoveGenerationMode::CapturesOnly,
        );
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn generate_only_captures_king() {
        let b = BoardState::from_fen("q3b3/1Kr2n2/1B6/8/1R6/8/8/p6b w KQkq - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        king_moves(
            Piece::king(White),
            3,
            3,
            &b,
            &mut ret,
            MoveGenerationMode::CapturesOnly,
        );
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn generate_only_captures_knight() {
        let b = BoardState::from_fen("q3b3/1Nr2n2/1B6/2b5/1R6/8/8/p7 w KQkq - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        knight_moves(
            Piece::knight(White),
            3,
            3,
            &b,
            &mut ret,
            MoveGenerationMode::CapturesOnly,
        );
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn generate_only_captures_pawn() {
        let b = BoardState::from_fen("q3b3/1Pr2n2/1B6/2b5/1R6/8/8/p7 w KQkq - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(
            Piece::knight(White),
            3,
            3,
            &b,
            &mut ret,
            MoveGenerationMode::CapturesOnly,
        );
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn only_captures_correctly_counted() {
        let b = BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
        assert_eq!(
            generate_moves(&b, MoveGenerationMode::CapturesOnly).len(),
            0
        );

        let b = BoardState::from_fen("rnbqkbnr/pppppppp/2N5/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
        assert_eq!(
            generate_moves(&b, MoveGenerationMode::CapturesOnly).len(),
            4
        );

        let b = BoardState::from_fen("K1k4p/8/8/8/8/8/8/B6R w KQkq - 0 1").unwrap();
        assert_eq!(
            generate_moves(&b, MoveGenerationMode::CapturesOnly).len(),
            2
        );
    }

    #[test]
    fn mvv_lva_sanity() {
        assert!(MVV_LVA[Pawn.index()][Pawn.index()] > MVV_LVA[Pawn.index()][Bishop.index()]);
        assert!(MVV_LVA[Rook.index()][Pawn.index()] > MVV_LVA[Knight.index()][Bishop.index()]);
        assert!(MVV_LVA[Queen.index()][Knight.index()] > MVV_LVA[Rook.index()][Bishop.index()]);
        assert!(MVV_LVA[Queen.index()][Queen.index()] > MVV_LVA[Rook.index()][Rook.index()]);
    }

    // Perft tests - move generation. Table of values taken from https://www.chessprogramming.org/Perft_Results

    #[test]
    fn perft_test_position_1() {
        let mut moves_states = [0; 5];
        let b = BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
        generate_moves_test(&b, 0, 5, &mut moves_states, false);
        assert_eq!(moves_states[0], 20);
        assert_eq!(moves_states[1], 400);
        assert_eq!(moves_states[2], 8902);
        assert_eq!(moves_states[3], 197281);
        assert_eq!(moves_states[4], 4865609);
    }

    #[test]
    fn perft_test_position_2() {
        let mut moves_states = [0; 4];
        let b = BoardState::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();
        generate_moves_test(&b, 0, 4, &mut moves_states, false);
        assert_eq!(moves_states[0], 48);
        assert_eq!(moves_states[1], 2039);
        assert_eq!(moves_states[2], 97862);
        assert_eq!(moves_states[3], 4085603);
    }

    #[test]
    fn perft_test_position_3() {
        let mut moves_states = [0; 5];
        let b = BoardState::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        generate_moves_test(&b, 0, 5, &mut moves_states, false);
        assert_eq!(moves_states[0], 14);
        assert_eq!(moves_states[1], 191);
        assert_eq!(moves_states[2], 2812);
        assert_eq!(moves_states[3], 43238);
        assert_eq!(moves_states[4], 674624);
    }

    #[test]
    fn perft_test_position_4() {
        let mut moves_states = [0; 4];
        let b = BoardState::from_fen(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        )
        .unwrap();
        generate_moves_test(&b, 0, 4, &mut moves_states, false);
        assert_eq!(moves_states[0], 6);
        assert_eq!(moves_states[1], 264);
        assert_eq!(moves_states[2], 9467);
        assert_eq!(moves_states[3], 422333);
    }

    #[test]
    fn perft_test_position_4_mirrored() {
        let mut moves_states = [0; 4];
        let b = BoardState::from_fen(
            "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
        )
        .unwrap();
        generate_moves_test(&b, 0, 4, &mut moves_states, false);
        assert_eq!(moves_states[0], 6);
        assert_eq!(moves_states[1], 264);
        assert_eq!(moves_states[2], 9467);
        assert_eq!(moves_states[3], 422333);
    }

    #[test]
    fn perft_test_position_5() {
        let mut moves_states = [0; 4];
        let b = BoardState::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
            .unwrap();
        generate_moves_test(&b, 0, 4, &mut moves_states, false);
        assert_eq!(moves_states[0], 44);
        assert_eq!(moves_states[1], 1486);
        assert_eq!(moves_states[2], 62379);
        assert_eq!(moves_states[3], 2103487);
    }

    #[test]
    fn perft_test_position_6() {
        let mut moves_states = [0; 4];
        let b = BoardState::from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        )
        .unwrap();
        generate_moves_test(&b, 0, 4, &mut moves_states, false);
        assert_eq!(moves_states[0], 46);
        assert_eq!(moves_states[1], 2079);
        assert_eq!(moves_states[2], 89890);
        assert_eq!(moves_states[3], 3894594);
    }
}
