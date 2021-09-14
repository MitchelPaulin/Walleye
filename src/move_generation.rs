pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};

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

#[derive(PartialEq, Eq)]
pub enum CastlingType {
    WhiteKingSide,
    WhiteQueenSide,
    BlackKingSide,
    BlackQueenSide,
}

const WHITE_KING_SIDE_CASTLE_ALG: Option<(Point, Point)> = Some((Point(9, 6), Point(9, 8)));
const WHITE_QUEEN_SIDE_CASTLE_ALG: Option<(Point, Point)> = Some((Point(9, 6), Point(9, 4)));
const BLACK_KING_SIDE_CASTLE_ALG: Option<(Point, Point)> = Some((Point(2, 6), Point(2, 8)));
const BLACK_QUEEN_SIDE_CASTLE_ALG: Option<(Point, Point)> = Some((Point(2, 6), Point(2, 4)));

/*
    Generate all possible *legal* moves from the given board
    Also sets appropriate variables for the board state
*/
pub fn generate_moves(board: &BoardState, generate_only_captures: bool) -> Vec<BoardState> {
    let mut new_moves: Vec<BoardState>;
    if !generate_only_captures {
        // chess has an estimated branching factor of 35, try to speed up memory allocation here by preallocating some space
        new_moves = Vec::with_capacity(35);
    } else {
        new_moves = Vec::with_capacity(5);
    }

    for i in BOARD_START..BOARD_END {
        for j in BOARD_START..BOARD_END {
            if let Square::Full(piece) = board.board[i][j] {
                if piece.color == board.to_move {
                    generate_move_for_piece(
                        piece,
                        board,
                        Point(i, j),
                        &mut new_moves,
                        generate_only_captures,
                    );
                }
            }
        }
    }

    if !generate_only_captures {
        generate_castling_moves(board, &mut new_moves);
    }
    new_moves
}

/*
    Determine if a color is currently in check
*/
pub fn is_check(board: &BoardState, color: PieceColor) -> bool {
    match color {
        Black => is_check_cords(board, Black, board.black_king_location),
        White => is_check_cords(board, White, board.white_king_location),
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
    only_captures: bool,
) {
    for mods in &KNIGHT_CORDS {
        let row = (row as i8 + mods.0) as usize;
        let col = (col as i8 + mods.1) as usize;
        let square = board.board[row][col];

        if !square.is_in_bounds() {
            continue;
        }

        if only_captures {
            if square.is_color(piece.color.opposite()) {
                moves.push(Point(row, col));
            }
        } else if square.is_empty_or_color(piece.color.opposite()) {
            moves.push(Point(row, col));
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
    only_captures: bool,
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
            if !only_captures {
                if (board.board[row - 1][col]).is_empty() {
                    moves.push(Point(row - 1, col));
                    // check double push
                    if row == 8 && (board.board[row - 2][col]).is_empty() {
                        moves.push(Point(row - 2, col));
                    }
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
            if !only_captures {
                if (board.board[row + 1][col]).is_empty() {
                    moves.push(Point(row + 1, col));
                    // check double push
                    if row == 3 && (board.board[row + 2][col]).is_empty() {
                        moves.push(Point(row + 2, col));
                    }
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
    board.pawn_double_move?;

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

    if left_cap == board.pawn_double_move.unwrap() {
        return Some(left_cap);
    } else if right_cap == board.pawn_double_move.unwrap() {
        return Some(right_cap);
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
    only_captures: bool,
) {
    for i in 0..3 {
        for j in 0..3 {
            let row = row + i - 1;
            let col = col + j - 1;
            let square = board.board[row][col];

            if !square.is_in_bounds() {
                continue;
            }

            if only_captures {
                if square.is_color(piece.color.opposite()) {
                    moves.push(Point(row, col));
                }
            } else if square.is_empty_or_color(piece.color.opposite()) {
                moves.push(Point(row, col));
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
    only_captures: bool,
) {
    for m in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let mut row = row as i8 + m.0;
        let mut col = col as i8 + m.1;
        let mut square = board.board[row as usize][col as usize];
        while square.is_empty() {
            if !only_captures {
                moves.push(Point(row as usize, col as usize));
            }
            row += m.0;
            col += m.1;
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
    only_captures: bool,
) {
    for m in &[(1, -1), (1, 1), (-1, 1), (-1, -1)] {
        let mut row = row as i8 + m.0;
        let mut col = col as i8 + m.1;
        let mut square = board.board[row as usize][col as usize];
        while square.is_empty() {
            if !only_captures {
                moves.push(Point(row as usize, col as usize));
            }
            row += m.0;
            col += m.1;
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
    only_captures: bool,
) {
    rook_moves(piece, row, col, board, moves, only_captures);
    bishop_moves(piece, row, col, board, moves, only_captures);
}

/*
    Generate pseudo-legal moves for a piece
    This will not generate en passants and castling, these cases are handled separately
*/
fn get_moves(
    row: usize,
    col: usize,
    board: &BoardState,
    moves: &mut Vec<Point>,
    generate_only_captures: bool,
) {
    if let Square::Full(piece) = board.board[row][col] {
        match piece.kind {
            Pawn => pawn_moves(piece, row, col, board, moves, generate_only_captures),
            Rook => rook_moves(piece, row, col, board, moves, generate_only_captures),
            Bishop => bishop_moves(piece, row, col, board, moves, generate_only_captures),
            Knight => knight_moves(piece, row, col, board, moves, generate_only_captures),
            King => king_moves(piece, row, col, board, moves, generate_only_captures),
            Queen => queen_moves(piece, row, col, board, moves, generate_only_captures),
        }
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

    // Check from rook or queen
    for m in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let mut row = square_cords.0 as i8 + m.0;
        let mut col = square_cords.1 as i8 + m.1;
        let mut square = board.board[row as usize][col as usize];
        while square.is_empty() {
            row += m.0;
            col += m.1;
            square = board.board[row as usize][col as usize];
        }

        let attacking_rook = Piece::rook(attacking_color);
        let attacking_queen = Piece::queen(attacking_color);
        if square == attacking_rook || square == attacking_queen {
            return true;
        }
    }

    // Check from bishop or queen
    for m in &[(1, -1), (1, 1), (-1, 1), (-1, -1)] {
        let mut row = square_cords.0 as i8 + m.0;
        let mut col = square_cords.1 as i8 + m.1;
        let mut square = board.board[row as usize][col as usize];
        while square.is_empty() {
            row += m.0;
            col += m.1;
            square = board.board[row as usize][col as usize];
        }

        let attacking_bishop = Piece::bishop(attacking_color);
        let attacking_queen = Piece::queen(attacking_color);
        if square == attacking_bishop || square == attacking_queen {
            return true;
        }
    }

    // Check from knight
    for mods in &KNIGHT_CORDS {
        let row = (square_cords.0 as i8 + mods.0) as usize;
        let col = (square_cords.1 as i8 + mods.1) as usize;
        let square = board.board[row][col];

        let attacking_knight = Piece::knight(attacking_color);
        if square == attacking_knight {
            return true;
        }
    }

    // Check from pawn
    let pawn_row = match color {
        White => square_cords.0 - 1,
        Black => square_cords.0 + 1,
    };

    let attacking_pawn = Piece::pawn(attacking_color);
    if board.board[pawn_row][square_cords.1 - 1] == attacking_pawn
        || board.board[pawn_row][square_cords.1 + 1] == attacking_pawn
    {
        return true;
    }

    // Check from king
    for i in 0..3 {
        for j in 0..3 {
            let row = square_cords.0 + i - 1;
            let col = square_cords.1 + j - 1;
            let square = board.board[row][col];
            if !square.is_in_bounds() {
                continue;
            }

            let attacking_king = Piece::king(attacking_color);
            if square == attacking_king {
                return true;
            }
        }
    }

    false
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
fn can_castle(board: &BoardState, castling_type: CastlingType) -> bool {
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
    Given the coordinates of a piece and that pieces color, generate all possible pseudo *legal* moves for that piece
*/
fn generate_move_for_piece(
    piece: Piece,
    board: &BoardState,
    square_cords: Point,
    new_moves: &mut Vec<BoardState>,
    generate_only_captures: bool,
) {
    let mut moves: Vec<Point> = Vec::new();
    let Piece { color, kind } = piece;
    get_moves(
        square_cords.0,
        square_cords.1,
        &board,
        &mut moves,
        generate_only_captures,
    );

    // make all the valid moves of this piece
    for _move in moves {
        let mut new_board = board.clone();
        new_board.pawn_promotion = None;
        new_board.swap_color();
        if color == Black {
            new_board.full_move_clock += 1;
        }

        // update king location if we are moving the king
        if kind == King {
            match color {
                White => new_board.white_king_location = _move,
                Black => new_board.black_king_location = _move,
            }
        }

        let target_square = new_board.board[_move.0][_move.1];
        if let Square::Full(target_piece) = target_square {
            new_board.mvv_lva = target_piece.value() - piece.value();
        } else {
            new_board.mvv_lva = 0;
        }

        // move the piece, this will take care of any captures as well, excluding en passant
        new_board.board[_move.0][_move.1] = piece.into();
        new_board.board[square_cords.0][square_cords.1] = Square::Empty;
        new_board.last_move = Some((square_cords, _move));

        // if you make your move, and you are in check, this move is not valid
        if is_check(&new_board, color) {
            continue;
        }

        // if the rook or king move, take away castling privileges
        if color == White && kind == King {
            new_board.white_king_side_castle = false;
            new_board.white_queen_side_castle = false;
        } else if color == Black && kind == King {
            new_board.black_queen_side_castle = false;
            new_board.black_king_side_castle = false;
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
        if _move.0 == BOARD_END - 1 && _move.1 == BOARD_END - 1 {
            new_board.white_king_side_castle = false;
        } else if _move.0 == BOARD_END - 1 && _move.1 == BOARD_START {
            new_board.white_queen_side_castle = false;
        } else if _move.0 == BOARD_START && _move.1 == BOARD_START {
            new_board.black_queen_side_castle = false;
        } else if _move.0 == BOARD_START && _move.1 == BOARD_END - 1 {
            new_board.black_king_side_castle = false;
        }

        // checks if the pawn has moved two spaces, if it has it can be captured en passant, record the space *behind* the pawn ie the valid capture square
        if !generate_only_captures {
            if kind == Pawn && (square_cords.0 as i8 - _move.0 as i8).abs() == 2 {
                if color == White {
                    new_board.pawn_double_move = Some(Point(_move.0 + 1, _move.1));
                } else {
                    new_board.pawn_double_move = Some(Point(_move.0 - 1, _move.1));
                }
            } else {
                // the most recent move was not a double pawn move, unset any possibly existing pawn double move
                new_board.pawn_double_move = None;
            }
            // deal with pawn promotions
            if _move.0 == BOARD_START && color == White && kind == Pawn {
                promote_pawn(&new_board, White, square_cords, _move, new_moves);
            } else if _move.0 == BOARD_END - 1 && color == Black && kind == Pawn {
                promote_pawn(&new_board, Black, square_cords, _move, new_moves);
            } else {
                new_moves.push(new_board);
            }
        } else {
            new_moves.push(new_board);
        }
    }

    // take care of en passant captures
    if kind == Pawn {
        let en_passant = pawn_moves_en_passant(piece, square_cords.0, square_cords.1, &board);
        if let Some(mov) = en_passant {
            let mut new_board = board.clone();
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
    if board.to_move == White && can_castle(&board, CastlingType::WhiteKingSide) {
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

    if board.to_move == White && can_castle(&board, CastlingType::WhiteQueenSide) {
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

    if board.to_move == Black && can_castle(&board, CastlingType::BlackKingSide) {
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

    if board.to_move == Black && can_castle(&board, CastlingType::BlackQueenSide) {
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
        moves.push(new_board);
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
        knight_moves(Piece::knight(White), 6, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn knight_moves_corner() {
        let b = BoardState::from_fen("N7/8/8/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        knight_moves(Piece::knight(White), 2, 2, &b, &mut ret, false);
        assert_eq!(ret.len(), 2);
    }
    #[test]
    fn knight_moves_with_other_pieces_with_capture() {
        let b = BoardState::from_fen("8/8/5n2/3NQ3/2K2P2/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        knight_moves(Piece::knight(White), 5, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 7);
    }

    // Pawn tests - white pawn

    #[test]
    fn white_pawn_double_push() {
        let b = BoardState::from_fen("8/8/8/8/8/8/P7/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(Piece::pawn(White), 8, 2, &b, &mut ret, false);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn white_pawn_has_moved() {
        let b = BoardState::from_fen("8/8/8/8/8/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(Piece::pawn(White), 7, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn white_pawn_cant_move_black_piece_block() {
        let b = BoardState::from_fen("8/8/8/8/3r4/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(Piece::pawn(White), 7, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_pawn_cant_move_white_piece_block() {
        let b = BoardState::from_fen("8/8/8/8/3K4/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(Piece::pawn(White), 7, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_pawn_with_two_captures_and_start() {
        let b = BoardState::from_fen("8/8/8/8/8/n1q5/1P6/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(Piece::pawn(White), 8, 3, &b, &mut ret, false);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn white_pawn_with_one_capture() {
        let b = BoardState::from_fen("8/8/Q1b5/1P6/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(Piece::pawn(White), 5, 3, &b, &mut ret, false);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn white_pawn_double_push_piece_in_front() {
        let b = BoardState::from_fen("8/8/8/8/8/b7/P7/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(Piece::pawn(White), 8, 2, &b, &mut ret, false);
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
        pawn_moves(Piece::pawn(Black), 3, 2, &b, &mut ret, false);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn black_pawn_has_moved() {
        let b = BoardState::from_fen("8/8/8/3p4/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(Piece::pawn(Black), 5, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn black_pawn_cant_move_white_piece_block() {
        let b = BoardState::from_fen("8/3p4/3R4/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(Piece::pawn(Black), 3, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn black_pawn_with_two_captures_and_start() {
        let b = BoardState::from_fen("8/3p4/2R1R3/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(Piece::pawn(Black), 3, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn black_pawn_with_one_capture() {
        let b = BoardState::from_fen("8/3p4/3qR3/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(Piece::pawn(Black), 3, 5, &b, &mut ret, false);
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
        king_moves(Piece::king(White), 6, 5, &b, &mut ret, false);
        assert_eq!(dbg!(ret).len(), 8);
    }

    #[test]
    fn king_start_pos() {
        let b = BoardState::from_fen("8/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        king_moves(Piece::king(White), 9, 6, &b, &mut ret, false);
        assert_eq!(ret.len(), 5);
    }

    #[test]
    fn king_start_pos_other_pieces() {
        let b = BoardState::from_fen("8/8/8/8/8/8/3Pn3/3QKB2 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        king_moves(Piece::king(White), 9, 6, &b, &mut ret, false);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn king_black_other_pieces() {
        let b = BoardState::from_fen("8/8/8/8/8/3Pn3/3QkB2/3R1q2 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        king_moves(Piece::king(Black), 8, 6, &b, &mut ret, false);
        assert_eq!(ret.len(), 6);
    }

    // Rook tests

    #[test]
    fn rook_center_of_empty_board() {
        let b = BoardState::from_fen("8/8/8/8/3R4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        rook_moves(Piece::rook(White), 6, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 14);
    }

    #[test]
    fn rook_center_of_board() {
        let b = BoardState::from_fen("8/8/8/3q4/2kRp3/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        rook_moves(Piece::rook(White), 6, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn rook_center_of_board_with_white_pieces() {
        let b = BoardState::from_fen("7p/3N4/8/4n3/2kR4/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        rook_moves(Piece::rook(White), 6, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn rook_corner() {
        let b = BoardState::from_fen("7p/3N4/K7/4n3/2kR4/3b4/8/7R w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        rook_moves(Piece::rook(White), 9, 9, &b, &mut ret, false);
        assert_eq!(ret.len(), 14);
    }
    #[test]
    fn black_rook_center_of_board_with_white_pieces() {
        let b = BoardState::from_fen("7p/3N4/8/4n3/2kr4/3b4/8/K7 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        rook_moves(Piece::rook(Black), 6, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 7);
    }

    // Bishop tests

    #[test]
    fn black_bishop_center_empty_board() {
        let b = BoardState::from_fen("8/8/8/3b4/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        bishop_moves(Piece::bishop(Black), 5, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 13);
    }

    #[test]
    fn black_bishop_center_with_captures() {
        let b = BoardState::from_fen("6P1/8/8/3b4/8/1R6/8/3Q4 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        bishop_moves(Piece::bishop(Black), 5, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 12);
    }

    #[test]
    fn black_bishop_center_with_captures_and_black_pieces() {
        let b = BoardState::from_fen("6P1/8/2Q5/3b4/2k1n3/1R6/8/b2Q4 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        bishop_moves(Piece::bishop(Black), 5, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn white_bishop_center_with_captures_and_white_pieces() {
        let b = BoardState::from_fen("8/8/8/4r3/5B2/8/3Q4/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        bishop_moves(Piece::bishop(White), 6, 7, &b, &mut ret, false);
        assert_eq!(ret.len(), 6);
    }

    // Queen tests

    #[test]
    fn white_queen_empty_board() {
        let b = BoardState::from_fen("8/8/8/8/3Q4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        queen_moves(Piece::queen(White), 6, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 27);
    }

    #[test]
    fn white_queen_cant_move() {
        let b = BoardState::from_fen("8/8/8/2NBR3/2PQR3/2RRR3/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        queen_moves(Piece::queen(White), 6, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_queen_with_other_piece() {
        let b = BoardState::from_fen("8/6r1/8/8/3Q4/5N2/8/6P1 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        queen_moves(Piece::queen(White), 6, 5, &b, &mut ret, false);
        assert_eq!(ret.len(), 25);
    }

    // Castling tests

    #[test]
    fn white_king_side_castle() {
        let mut b = BoardState::from_fen("8/8/8/8/8/8/8/4K2R w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::WhiteKingSide));

        b = BoardState::from_fen("8/8/2b5/8/8/6P1/5P1P/4K2R w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::WhiteKingSide));

        // Can't castle out of check
        b = BoardState::from_fen("4r3/8/2b5/8/8/6P1/5P1P/4K2R w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteKingSide));

        // Can't castle through check
        b = BoardState::from_fen("8/8/8/8/8/6Pb/5P1P/4K2R w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteKingSide));

        // Can't castle with pieces in way
        b = BoardState::from_fen("8/8/8/8/8/6PN/5P1P/4KP1R w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteKingSide));

        // Can't castle with pieces in way 2
        b = BoardState::from_fen("8/8/8/8/8/6PN/5P1P/4K1PR w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteKingSide));
    }

    #[test]
    fn white_queen_side_castle() {
        let mut b = BoardState::from_fen("8/8/8/8/8/8/8/R3K3 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::WhiteQueenSide));

        b = BoardState::from_fen("8/8/8/8/8/2P5/PP1P4/R3K1N1 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::WhiteQueenSide));

        // Can't castle out of check
        b = BoardState::from_fen("8/8/8/8/8/2P2n2/PP1P4/R3K1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteQueenSide));

        // Can't castle through check
        b = BoardState::from_fen("8/8/8/8/8/2n5/PP1P4/R3K1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteQueenSide));

        // Can't castle with pieces in way
        b = BoardState::from_fen("8/8/8/8/8/2P5/PP1P4/R2QK1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteQueenSide));

        // Can't castle with pieces in way 2
        b = BoardState::from_fen("8/8/8/8/8/2P5/PP1P4/R1Q1K1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteQueenSide));

        // Can't castle with pieces in way 3
        b = BoardState::from_fen("8/8/8/8/8/2P5/PP1P4/RQ2K1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteQueenSide));
    }

    #[test]
    fn black_king_side_castle() {
        let mut b = BoardState::from_fen("1p2k2r/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::BlackKingSide));

        b = BoardState::from_fen("1p2k2r/4bp1p/6p1/8/8/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::BlackKingSide));

        // Can't castle out of check
        b = BoardState::from_fen("1p2k2r/4bp1p/6p1/8/B7/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackKingSide));

        // Can't castle through check
        b = BoardState::from_fen("1p2k2r/4bp1p/6pB/8/8/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackKingSide));

        // Can't castle with pieces in way
        b = BoardState::from_fen("1p2k1nr/4bp1p/6pn/8/8/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackKingSide));

        // Can't castle with pieces in way 2
        b = BoardState::from_fen("1p2kN1r/4bp1p/6pn/3n4/8/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackKingSide));
    }

    #[test]
    fn black_queen_side_castle() {
        let mut b = BoardState::from_fen("r3k3/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::BlackQueenSide));

        b = BoardState::from_fen("r3k3/qpb5/3n4/8/8/8/8/8 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::BlackQueenSide));

        // Can't castle out of check
        b = BoardState::from_fen("r3k3/qpb5/3n4/8/8/8/8/4Q3 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackQueenSide));

        // Can't castle through check
        b = BoardState::from_fen("r3k3/qpb5/3n4/8/7Q/8/8/8 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackQueenSide));

        // Can't castle with pieces in way
        b = BoardState::from_fen("r2Pk3/qpb5/3n4/8/8/8/8/P7 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackQueenSide));

        // Can't castle with pieces in way 2
        b = BoardState::from_fen("r1p1k3/qpb5/3n4/8/8/8/8/P7 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackQueenSide));

        // Can't castle with pieces in way 3
        b = BoardState::from_fen("rn2k3/qpb5/3n4/8/8/8/8/P7 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackQueenSide));
    }

    #[test]
    fn generate_only_captures_queen() {
        let b = BoardState::from_fen("q3b3/1Q3n2/8/8/1R6/8/8/p6b w KQkq - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        queen_moves(Piece::queen(White), 3, 3, &b, &mut ret, true);
        assert_eq!(ret.len(), 3);
    }

    #[test]
    fn generate_only_captures_bishop() {
        let b = BoardState::from_fen("q3b3/1B6/8/8/R7/8/8/p6b w KQkq - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        bishop_moves(Piece::bishop(White), 3, 3, &b, &mut ret, true);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn generate_only_captures_rook() {
        let b = BoardState::from_fen("R3b3/8/8/8/R7/8/8/p7 w KQkq - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        rook_moves(Piece::rook(White), 2, 2, &b, &mut ret, true);
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn generate_only_captures_king() {
        let b = BoardState::from_fen("q3b3/1Kr2n2/1B6/8/1R6/8/8/p6b w KQkq - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        king_moves(Piece::king(White), 3, 3, &b, &mut ret, true);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn generate_only_captures_knight() {
        let b = BoardState::from_fen("q3b3/1Nr2n2/1B6/2b5/1R6/8/8/p7 w KQkq - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        knight_moves(Piece::knight(White), 3, 3, &b, &mut ret, true);
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn generate_only_captures_pawn() {
        let b = BoardState::from_fen("q3b3/1Pr2n2/1B6/2b5/1R6/8/8/p7 w KQkq - 0 1").unwrap();
        let mut ret: Vec<Point> = Vec::new();
        pawn_moves(Piece::knight(White), 3, 3, &b, &mut ret, true);
        assert_eq!(ret.len(), 1);
    }


    #[test]
    fn only_captures_correctly_counted() {

        let b = BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(generate_moves(&b, true).len(), 0);

        let b = BoardState::from_fen("rnbqkbnr/pppppppp/2N5/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(generate_moves(&b, true).len(), 4);

        let b = BoardState::from_fen("K1k4p/8/8/8/8/8/8/B6R w KQkq - 0 1").unwrap();
        assert_eq!(generate_moves(&b, true).len(), 2);
    }

    /*
        Generate all valid moves recursively given the current board state

        Will generate up until cur_depth = depth
    */
    fn generate_moves_test(
        board: &BoardState,
        cur_depth: usize,
        depth: usize,
        move_counts: &mut [u32],
    ) {
        if cur_depth == depth {
            return;
        }

        let moves = generate_moves(board, false);
        move_counts[cur_depth] += moves.len() as u32;
        for mov in moves {
            generate_moves_test(&mov, cur_depth + 1, depth, move_counts);
        }
    }

    // Perft tests - move generation. Table of values taken from https://www.chessprogramming.org/Perft_Results

    #[test]
    fn perft_test_position_1() {
        let mut moves_states = [0; 5];
        let b = BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
        generate_moves_test(&b, 0, 5, &mut moves_states);
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
        generate_moves_test(&b, 0, 4, &mut moves_states);
        assert_eq!(moves_states[0], 48);
        assert_eq!(moves_states[1], 2039);
        assert_eq!(moves_states[2], 97862);
        assert_eq!(moves_states[3], 4085603);
    }

    #[test]
    fn perft_test_position_3() {
        let mut moves_states = [0; 5];
        let b = BoardState::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        generate_moves_test(&b, 0, 5, &mut moves_states);
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
        generate_moves_test(&b, 0, 4, &mut moves_states);
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
        generate_moves_test(&b, 0, 4, &mut moves_states);
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
        generate_moves_test(&b, 0, 4, &mut moves_states);
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
        generate_moves_test(&b, 0, 4, &mut moves_states);
        assert_eq!(moves_states[0], 46);
        assert_eq!(moves_states[1], 2079);
        assert_eq!(moves_states[2], 89890);
        assert_eq!(moves_states[3], 3894594);
    }
}
