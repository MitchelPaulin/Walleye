#![allow(dead_code)]
pub use crate::board::PieceColor;
pub use crate::board::*;

type Point = (usize, usize);

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

/*
    Generate pseudo-legal moves for a knight
*/
pub fn knight_moves(row: usize, col: usize, board: &BoardState, moves: &mut Vec<Point>) {
    for mods in KNIGHT_CORDS.iter() {
        let piece = board.board[row][col];
        let _row = (row as i8 + mods.0) as usize;
        let _col = (col as i8 + mods.1) as usize;
        let square = board.board[_row][_col];

        if is_outside_board(square) {
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
pub fn pawn_moves(row: usize, col: usize, board: &BoardState, moves: &mut Vec<Point>) {
    let piece = board.board[row][col];

    // white pawns move up board
    if is_white(piece) {
        // check capture
        let left_cap = board.board[row - 1][col - 1];
        let right_cap = board.board[row - 1][col + 1];
        if !is_outside_board(left_cap) && is_black(left_cap) {
            moves.push((row - 1, col - 1));
        }
        if !is_outside_board(right_cap) && is_black(right_cap) {
            moves.push((row - 1, col + 1));
        }

        // check a normal push
        if is_empty(board.board[row - 1][col]) {
            moves.push((row - 1, col));
            // check double push
            if row == 8 && is_empty(board.board[row - 2][col]) {
                moves.push((row - 2, col));
            }
        }
    } else {
        // check capture
        let left_cap = board.board[row + 1][col + 1];
        let right_cap = board.board[row + 1][col - 1];
        if !is_outside_board(left_cap) && is_white(left_cap) {
            moves.push((row + 1, col + 1));
        }
        if !is_outside_board(right_cap) && is_white(right_cap) {
            moves.push((row + 1, col - 1));
        }

        // check a normal push
        if is_empty(board.board[row + 1][col]) {
            moves.push((row + 1, col));
            // check double push
            if row == 3 && is_empty(board.board[row + 2][col]) {
                moves.push((row + 2, col));
            }
        }
    }
}

/*
    Generate pseudo-legal en passant moves

    Uses the pawn_double_move cords to decide if a en passant capture is legal

    Returns None if no legal move is available, otherwise return the coordinates of the capture
*/

pub fn pawn_moves_en_passant(row: usize, col: usize, board: &BoardState) -> Option<Point> {
    if board.pawn_double_move.is_none() {
        return None;
    }

    let piece = board.board[row][col];
    let left_cap;
    let right_cap;

    if is_white(piece) && row == BOARD_START + 3 {
        left_cap = (row - 1, col - 1);
        right_cap = (row - 1, col + 1);
    } else if is_black(piece) && row == BOARD_START + 4 {
        left_cap = (row + 1, col + 1);
        right_cap = (row + 1, col - 1);
    } else {
        return None;
    }

    if left_cap == board.pawn_double_move.unwrap() {
        return Some(left_cap);
    } else if right_cap == board.pawn_double_move.unwrap() {
        return Some(right_cap);
    }

    return None;
}

/*
    Generate pseudo-legal moves for a king
*/
pub fn king_moves(row: usize, col: usize, board: &BoardState, moves: &mut Vec<Point>) {
    let piece = board.board[row][col];
    for i in 0..3 {
        for j in 0..3 {
            let _row = row + i - 1;
            let _col = col + j - 1;
            let square = board.board[_row][_col];
            if is_outside_board(square) {
                continue;
            }

            if is_empty(square) || square & COLOR_MASK != piece & COLOR_MASK {
                moves.push((_row, _col));
            }
        }
    }
}

/*
    Generate pseudo-legal moves for a rook
*/
pub fn rook_moves(row: usize, col: usize, board: &BoardState, moves: &mut Vec<Point>) {
    let piece = board.board[row][col];
    for m in [(1, 0), (-1, 0), (0, 1), (0, -1)].iter() {
        let mut row = row as i8 + m.0;
        let mut col = col as i8 + m.1;
        let mut square = board.board[row as usize][col as usize];
        while is_empty(square) {
            moves.push((row as usize, col as usize));
            row += m.0;
            col += m.1;
            square = board.board[row as usize][col as usize];
        }

        if !is_outside_board(square) && piece & COLOR_MASK != square & COLOR_MASK {
            moves.push((row as usize, col as usize));
        }
    }
}

/*
    Generate pseudo-legal moves for a bishop
*/
pub fn bishop_moves(row: usize, col: usize, board: &BoardState, moves: &mut Vec<Point>) {
    let piece = board.board[row][col];
    for m in [(1, -1), (1, 1), (-1, 1), (-1, -1)].iter() {
        let mut row = row as i8 + m.0;
        let mut col = col as i8 + m.1;
        let mut square = board.board[row as usize][col as usize];
        while is_empty(square) {
            moves.push((row as usize, col as usize));
            row += m.0;
            col += m.1;
            square = board.board[row as usize][col as usize];
        }

        if !is_outside_board(square) && piece & COLOR_MASK != square & COLOR_MASK {
            moves.push((row as usize, col as usize));
        }
    }
}

/*
    Generate pseudo-legal moves for a queen
*/
pub fn queen_moves(row: usize, col: usize, board: &BoardState, moves: &mut Vec<Point>) {
    rook_moves(row, col, board, moves);
    bishop_moves(row, col, board, moves);
}

/*
    Generate pseudo-legal moves for a piece
    This will not generate en passants and castling, these cases are handled separately
*/
pub fn get_moves(row: usize, col: usize, board: &BoardState, moves: &mut Vec<Point>) {
    match board.board[row][col] & PIECE_MASK {
        PAWN => pawn_moves(row, col, board, moves),
        ROOK => rook_moves(row, col, board, moves),
        BISHOP => bishop_moves(row, col, board, moves),
        KNIGHT => knight_moves(row, col, board, moves),
        KING => king_moves(row, col, board, moves),
        QUEEN => queen_moves(row, col, board, moves),
        _ => panic!("Unrecognized piece"),
    }
}

/*
    Determine if a color is currently in check
*/
pub fn is_check(board: &BoardState, color: PieceColor) -> bool {
    match color {
        PieceColor::Black => is_check_cords(board, PieceColor::Black, board.black_king_location),
        PieceColor::White => is_check_cords(board, PieceColor::White, board.white_king_location),
    }
}

/*
    Determine if the given position is check

    Rather than checking each piece to see if it attacks the king
    this function checks all possible attack squares to the king and
    sees if the piece is there, thus it is important the king_location is set
*/
fn is_check_cords(board: &BoardState, color: PieceColor, square_cords: Point) -> bool {
    let attacking_color = match color {
        PieceColor::White => PieceColor::Black,
        _ => PieceColor::White,
    };

    // Check from rook or queen
    for m in [(1, 0), (-1, 0), (0, 1), (0, -1)].iter() {
        let mut row = square_cords.0 as i8 + m.0;
        let mut col = square_cords.1 as i8 + m.1;
        let mut square = board.board[row as usize][col as usize];
        while is_empty(square) {
            row += m.0;
            col += m.1;
            square = board.board[row as usize][col as usize];
        }

        if square == attacking_color.as_mask() | ROOK || square == attacking_color.as_mask() | QUEEN
        {
            return true;
        }
    }

    // Check from bishop or queen
    for m in [(1, -1), (1, 1), (-1, 1), (-1, -1)].iter() {
        let mut row = square_cords.0 as i8 + m.0;
        let mut col = square_cords.1 as i8 + m.1;
        let mut square = board.board[row as usize][col as usize];
        while is_empty(square) {
            row += m.0;
            col += m.1;
            square = board.board[row as usize][col as usize];
        }

        if square == attacking_color.as_mask() | BISHOP
            || square == attacking_color.as_mask() | QUEEN
        {
            return true;
        }
    }

    // Check from knight
    for mods in KNIGHT_CORDS.iter() {
        let row = (square_cords.0 as i8 + mods.0) as usize;
        let col = (square_cords.1 as i8 + mods.1) as usize;
        let square = board.board[row][col];

        if square == KNIGHT | attacking_color.as_mask() {
            return true;
        }
    }

    // Check from pawn
    let _row = match color {
        PieceColor::White => square_cords.0 - 1,
        _ => square_cords.0 + 1,
    };

    let attacking_pawn = attacking_color.as_mask() | PAWN;
    if board.board[_row][square_cords.1 - 1] == attacking_pawn
        || board.board[_row][square_cords.1 + 1] == attacking_pawn
    {
        return true;
    }

    // Check from king
    for i in 0..3 {
        for j in 0..3 {
            let row = square_cords.0 + i - 1;
            let col = square_cords.1 + j - 1;
            let square = board.board[row][col];
            if is_outside_board(square) {
                continue;
            }

            if square == KING | attacking_color.as_mask() {
                return true;
            }
        }
    }

    return false;
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
pub fn can_castle(board: &BoardState, castling_type: CastlingType) -> bool {
    if castling_type == CastlingType::WhiteKingSide {
        if !board.white_king_side_castle {
            return false;
        }
        // check that squares required for castling are empty
        if !is_empty(board.board[BOARD_END - 1][BOARD_END - 3])
            || !is_empty(board.board[BOARD_END - 1][BOARD_END - 2])
        {
            return false;
        }
        // check that the king currently isn't in check
        if is_check(board, PieceColor::White) {
            return false;
        }
        //check that the squares required for castling are not threatened
        if is_check_cords(board, PieceColor::White, (BOARD_END - 1, BOARD_END - 3))
            || is_check_cords(board, PieceColor::White, (BOARD_END - 1, BOARD_END - 2))
        {
            return false;
        }
        return true;
    }

    if castling_type == CastlingType::WhiteQueenSide {
        if !board.white_queen_side_castle {
            return false;
        }
        // check that squares required for castling are empty
        if !is_empty(board.board[BOARD_END - 1][BOARD_START + 1])
            || !is_empty(board.board[BOARD_END - 1][BOARD_START + 2])
            || !is_empty(board.board[BOARD_END - 1][BOARD_START + 3])
        {
            return false;
        }
        // check that the king currently isn't in check
        if is_check(board, PieceColor::White) {
            return false;
        }
        //check that the squares required for castling are not threatened
        if is_check_cords(board, PieceColor::White, (BOARD_END - 1, BOARD_START + 3))
            || is_check_cords(board, PieceColor::White, (BOARD_END - 1, BOARD_START + 2))
        {
            return false;
        }

        return true;
    }

    if castling_type == CastlingType::BlackKingSide {
        if !board.black_king_side_castle {
            return false;
        }
        // check that squares required for castling are empty
        if !is_empty(board.board[BOARD_START][BOARD_END - 3])
            || !is_empty(board.board[BOARD_START][BOARD_END - 2])
        {
            return false;
        }
        // check that the king currently isn't in check
        if is_check(board, PieceColor::Black) {
            return false;
        }
        //check that the squares required for castling are not threatened
        if is_check_cords(board, PieceColor::Black, (BOARD_START, BOARD_END - 3))
            || is_check_cords(board, PieceColor::Black, (BOARD_START, BOARD_END - 2))
        {
            return false;
        }

        return true;
    }

    if castling_type == CastlingType::BlackQueenSide {
        if !board.black_queen_side_castle {
            return false;
        }
        // check that squares required for castling are empty
        if !is_empty(board.board[BOARD_START][BOARD_START + 1])
            || !is_empty(board.board[BOARD_START][BOARD_START + 2])
            || !is_empty(board.board[BOARD_START][BOARD_START + 3])
        {
            return false;
        }
        // check that the king currently isn't in check
        if is_check(board, PieceColor::Black) {
            return false;
        }
        //check that the squares required for castling are not threatened
        if is_check_cords(board, PieceColor::Black, (BOARD_START, BOARD_START + 2))
            || is_check_cords(board, PieceColor::Black, (BOARD_START, BOARD_START + 3))
        {
            return false;
        }

        return true;
    }

    panic!("Shouldn't be here");
}

/*
    Generate all possible moves from the given board
    Also sets appropriate variables for the board state
*/
pub fn generate_moves(board: &BoardState) -> Vec<BoardState> {
    let mut new_moves = Vec::new();

    for i in BOARD_START..BOARD_END {
        for j in BOARD_START..BOARD_END {
            let color = get_color(board.board[i][j]);
            if color.is_some() && color.unwrap() == board.to_move {
                //generate moves
                let mut moves: Vec<Point> = vec![];
                let piece = board.board[i][j];
                get_moves(i, j, &board, &mut moves);

                // make all the valid moves of this piece
                for _move in moves {
                    let mut new_board = board.clone();
                    new_board.swap_color();
                    if color.unwrap() == PieceColor::Black {
                        new_board.full_move_clock += 1;
                    }
                    // update king location if we are moving the king
                    if piece == WHITE | KING {
                        new_board.white_king_location = (_move.0, _move.1);
                    } else if piece == BLACK | KING {
                        new_board.black_king_location = (_move.0, _move.1);
                    }

                    let target_square = new_board.board[_move.0][_move.1];
                    if !is_empty(target_square) {
                        let piece_value = PIECE_VALUES[(target_square & PIECE_MASK) as usize];
                        if board.to_move == PieceColor::White {
                            new_board.black_total_piece_value -= piece_value;
                        } else {
                            new_board.white_total_piece_value -= piece_value;
                        }
                    }

                    // move the piece, this will take care of any captures as well, excluding en passant
                    new_board.board[_move.0][_move.1] = piece;
                    new_board.board[i][j] = EMPTY;

                    // if you make your move, and you are in check, this move is not valid
                    if is_check(&new_board, color.unwrap()) {
                        continue;
                    }

                    // if the rook or king move, take away castling privileges
                    if piece == WHITE | KING {
                        new_board.white_king_side_castle = false;
                        new_board.white_queen_side_castle = false;
                    } else if piece == BLACK | KING {
                        new_board.black_queen_side_castle = false;
                        new_board.black_king_side_castle = false;
                    } else if i == BOARD_END - 1 && j == BOARD_END - 1 {
                        new_board.white_king_side_castle = false;
                    } else if i == BOARD_END - 1 && j == BOARD_START {
                        new_board.white_queen_side_castle = false;
                    } else if i == BOARD_START && j == BOARD_START {
                        new_board.black_queen_side_castle = false;
                    } else if i == BOARD_START && j == BOARD_END - 1 {
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
                    if is_pawn(piece) && (i as i8 - _move.0 as i8).abs() == 2 {
                        if is_white(piece) {
                            new_board.pawn_double_move = Some((_move.0 + 1, _move.1));
                        } else {
                            new_board.pawn_double_move = Some((_move.0 - 1, _move.1));
                        }
                    } else {
                        // the most recent move was not a double pawn move, unset any possibly existing pawn double move
                        new_board.pawn_double_move = None;
                    }

                    // deal with pawn promotions
                    if _move.0 == BOARD_START && piece == WHITE | PAWN {
                        for piece in [QUEEN, KNIGHT, BISHOP, ROOK].iter() {
                            let mut _new_board = new_board.clone();
                            _new_board.pawn_double_move = None;
                            _new_board.board[_move.0][_move.1] = WHITE | piece;
                            _new_board.white_total_piece_value +=
                                PIECE_VALUES[*piece as usize] - PIECE_VALUES[PAWN as usize];
                            new_moves.push(_new_board);
                        }
                    } else if _move.0 == BOARD_END - 1 && piece == BLACK | PAWN {
                        for piece in [QUEEN, KNIGHT, BISHOP, ROOK].iter() {
                            let mut _new_board = new_board.clone();
                            _new_board.pawn_double_move = None;
                            _new_board.board[_move.0][_move.1] = BLACK | piece;
                            _new_board.black_total_piece_value +=
                                PIECE_VALUES[*piece as usize] - PIECE_VALUES[PAWN as usize];
                            new_moves.push(_new_board);
                        }
                    } else {
                        new_moves.push(new_board);
                    }
                }

                // take care of en passant captures
                if is_pawn(piece) {
                    let en_passant = pawn_moves_en_passant(i, j, &board);
                    if en_passant.is_some() {
                        let _move = en_passant.unwrap();
                        let mut new_board = board.clone();
                        new_board.swap_color();
                        new_board.pawn_double_move = None;
                        new_board.board[_move.0][_move.1] = piece;
                        new_board.board[i][j] = EMPTY;
                        if is_white(piece) {
                            new_board.board[_move.0 + 1][_move.1] = EMPTY;
                            new_board.black_total_piece_value -= PIECE_VALUES[PAWN as usize];
                        } else {
                            new_board.board[_move.0 - 1][_move.1] = EMPTY;
                            new_board.white_total_piece_value -= PIECE_VALUES[PAWN as usize];
                        }

                        // if you make a move, and you do not end up in check, then this move is valid
                        if !is_check(&new_board, board.to_move) {
                            new_moves.push(new_board);
                        }
                    }
                }
            }
        }
    }

    // take care of castling
    if board.to_move == PieceColor::White && can_castle(&board, CastlingType::WhiteKingSide) {
        let mut new_board = board.clone();
        new_board.swap_color();
        new_board.pawn_double_move = None;
        new_board.white_king_side_castle = false;
        new_board.white_queen_side_castle = false;
        new_board.white_king_location = (BOARD_END - 1, BOARD_END - 2);
        new_board.board[BOARD_END - 1][BOARD_START + 4] = EMPTY;
        new_board.board[BOARD_END - 1][BOARD_END - 1] = EMPTY;
        new_board.board[BOARD_END - 1][BOARD_END - 2] = WHITE | KING;
        new_board.board[BOARD_END - 1][BOARD_END - 3] = WHITE | ROOK;
        new_moves.push(new_board);
    }

    if board.to_move == PieceColor::White && can_castle(&board, CastlingType::WhiteQueenSide) {
        let mut new_board = board.clone();
        new_board.swap_color();
        new_board.pawn_double_move = None;
        new_board.white_king_side_castle = false;
        new_board.white_queen_side_castle = false;
        new_board.white_king_location = (BOARD_END - 1, BOARD_START + 2);
        new_board.board[BOARD_END - 1][BOARD_START + 4] = EMPTY;
        new_board.board[BOARD_END - 1][BOARD_START] = EMPTY;
        new_board.board[BOARD_END - 1][BOARD_START + 2] = WHITE | KING;
        new_board.board[BOARD_END - 1][BOARD_START + 3] = WHITE | ROOK;
        new_moves.push(new_board);
    }

    if board.to_move == PieceColor::Black && can_castle(&board, CastlingType::BlackKingSide) {
        let mut new_board = board.clone();
        new_board.swap_color();
        new_board.pawn_double_move = None;
        new_board.black_king_side_castle = false;
        new_board.black_queen_side_castle = false;
        new_board.black_king_location = (BOARD_START, BOARD_END - 2);
        new_board.board[BOARD_START][BOARD_START + 4] = EMPTY;
        new_board.board[BOARD_START][BOARD_END - 1] = EMPTY;
        new_board.board[BOARD_START][BOARD_END - 2] = BLACK | KING;
        new_board.board[BOARD_START][BOARD_END - 3] = BLACK | ROOK;
        new_moves.push(new_board);
    }

    if board.to_move == PieceColor::Black && can_castle(&board, CastlingType::BlackQueenSide) {
        let mut new_board = board.clone();
        new_board.swap_color();
        new_board.pawn_double_move = None;
        new_board.black_king_side_castle = false;
        new_board.black_queen_side_castle = false;
        new_board.black_king_location = (BOARD_START, BOARD_START + 2);
        new_board.board[BOARD_START][BOARD_START + 4] = EMPTY;
        new_board.board[BOARD_START][BOARD_START] = EMPTY;
        new_board.board[BOARD_START][BOARD_START + 2] = BLACK | KING;
        new_board.board[BOARD_START][BOARD_START + 3] = BLACK | ROOK;
        new_moves.push(new_board);
    }
    return new_moves;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_sanity_test() {
        let b = board_from_fen("8/8/8/8/3K4/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::White));
    }

    #[test]
    fn knight_checks() {
        let mut b = board_from_fen("8/8/4n3/8/3K4/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::White));

        b = board_from_fen("8/8/8/8/8/8/1RK5/nRB5 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::White));

        b = board_from_fen("8/8/8/8/3k4/5N2/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/8/8/3k4/5n2/8/7N w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/2N5/8/3k4/5n2/8/7N w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::Black));
    }

    #[test]
    fn pawn_checks() {
        let mut b = board_from_fen("8/8/8/4k3/3P4/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/8/4k3/5P2/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/8/4k3/4P3/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/3PPP2/4k3/8/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/8/8/8/5p2/6K1/8 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::White));

        b = board_from_fen("8/8/8/8/8/7p/6K1/8 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::White));

        b = board_from_fen("8/8/8/8/8/6p1/6K1/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::White));

        b = board_from_fen("8/8/8/8/8/6K1/5ppp/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::White));
    }

    #[test]
    fn rook_checks() {
        let mut b = board_from_fen("8/8/8/R3k3/8/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/8/R1r1k3/8/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/8/R1r1k3/8/8/8/4R3 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::Black));

        b = board_from_fen("4R3/8/8/R1r5/8/8/8/4k3 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/8/R1r5/8/8/7R/4k3 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::Black));

        b = board_from_fen("4R3/8/8/8/8/3r4/R3K2R/2r1Rr2 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::White));

        b = board_from_fen("4R3/8/8/8/4K3/3r4/R6R/2r1rr2 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::White));

        b = board_from_fen("4R3/8/8/8/4K2r/3r4/R6R/2r2r2 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::White));

        b = board_from_fen("4r3/8/8/4B3/r2QKP1r/3rR3/R6R/2r1rr2 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::White));
    }

    #[test]
    fn bishop_checks() {
        let mut b = board_from_fen("8/8/8/1B6/8/8/8/5k2 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/2B1B3/1B3B2/1B1k1B2/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/8/8/5k2/8/8/2B5 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/8/8/5k2/4n3/8/2B5 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/8/8/3K4/8/8/6b1 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::White));

        b = board_from_fen("8/8/8/8/3K4/4r3/8/6b1 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::White));

        b = board_from_fen("8/8/8/8/3K4/4r3/8/b5b1 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::White));

        b = board_from_fen("8/8/8/8/3K4/2P1r3/8/b5b1 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::White));
    }

    #[test]
    fn queen_checks() {
        let mut b = board_from_fen("8/8/8/8/3k1Q2/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/2k5/8/8/8/6Q1/8 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::Black));

        b = board_from_fen("8/8/2K5/8/3q4/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::White));

        b = board_from_fen("8/8/1K6/2Q5/3q4/8/8/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::White));

        b = board_from_fen("8/5Q2/1K6/8/3q4/8/8/8 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::White));

        b = board_from_fen("8/5Q2/1K6/1P6/8/8/1q6/8 w - - 0 1").unwrap();
        assert!(!is_check(&b, PieceColor::White));

        b = board_from_fen("8/2P2Q2/1K6/8/8/8/1q6/8 w - - 0 1").unwrap();
        assert!(is_check(&b, PieceColor::White));
    }

    // Knight tests

    #[test]
    fn knight_moves_empty_board() {
        let b = board_from_fen("8/8/8/8/3N4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        knight_moves(6, 5, &b, &mut ret);
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn knight_moves_corner() {
        let b = board_from_fen("N7/8/8/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        knight_moves(2, 2, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }
    #[test]
    fn knight_moves_with_other_pieces_with_capture() {
        let b = board_from_fen("8/8/5n2/3NQ3/2K2P2/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        knight_moves(5, 5, &b, &mut ret);
        assert_eq!(ret.len(), 7);
    }

    // Pawn tests - white pawn

    #[test]
    fn white_pawn_double_push() {
        let b = board_from_fen("8/8/8/8/8/8/P7/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        pawn_moves(8, 2, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn white_pawn_has_moved() {
        let b = board_from_fen("8/8/8/8/8/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        pawn_moves(7, 5, &b, &mut ret);
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn white_pawn_cant_move_black_piece_block() {
        let b = board_from_fen("8/8/8/8/3r4/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        pawn_moves(7, 5, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_pawn_cant_move_white_piece_block() {
        let b = board_from_fen("8/8/8/8/3K4/3P4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        pawn_moves(7, 5, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_pawn_with_two_captures_and_start() {
        let b = board_from_fen("8/8/8/8/8/n1q5/1P6/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        pawn_moves(8, 3, &b, &mut ret);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn white_pawn_with_one_capture() {
        let b = board_from_fen("8/8/Q1b5/1P6/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        pawn_moves(5, 3, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn white_pawn_double_push_piece_in_front() {
        let b = board_from_fen("8/8/8/8/8/b7/P7/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        pawn_moves(8, 2, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_pawn_en_passant_left() {
        let b = board_from_fen("8/8/8/3pP3/8/8/8/8 w - d6 0 1").unwrap();
        assert!(pawn_moves_en_passant(5, 6, &b).is_some());
    }

    #[test]
    fn white_pawn_en_passant_right() {
        let b = board_from_fen("8/8/8/4Pp2/8/8/8/8 w - f6 0 1").unwrap();
        assert!(pawn_moves_en_passant(5, 6, &b).is_some());
    }

    #[test]
    fn white_pawn_en_passant_right_2() {
        let b = board_from_fen("7K/8/7k/1Pp5/8/8/8/8 w - c6 0 1").unwrap();
        assert!(pawn_moves_en_passant(5, 3, &b).is_some());
    }

    #[test]
    fn white_pawn_en_passant_wrong_row() {
        let b = board_from_fen("8/8/8/8/4Pp2/8/8/8 w - f4 0 1").unwrap();
        assert!(pawn_moves_en_passant(6, 6, &b).is_none());
    }

    #[test]
    fn white_en_passant_capture_not_available() {
        let b = board_from_fen("8/8/8/4Pp2/8/8/8/8 w - - 0 1").unwrap();
        assert!(pawn_moves_en_passant(5, 6, &b).is_none());
    }

    // Pawn tests - black pawn

    #[test]
    fn black_pawn_double_push() {
        let b = board_from_fen("8/p7/8/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        pawn_moves(3, 2, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn black_pawn_has_moved() {
        let b = board_from_fen("8/8/8/3p4/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        pawn_moves(5, 5, &b, &mut ret);
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn black_pawn_cant_move_white_piece_block() {
        let b = board_from_fen("8/3p4/3R4/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        pawn_moves(3, 5, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn black_pawn_with_two_captures_and_start() {
        let b = board_from_fen("8/3p4/2R1R3/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        pawn_moves(3, 5, &b, &mut ret);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn black_pawn_with_one_capture() {
        let b = board_from_fen("8/3p4/3qR3/8/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        pawn_moves(3, 5, &b, &mut ret);
        assert_eq!(ret.len(), 1);
    }

    #[test]
    fn black_pawn_en_passant_left() {
        let b = board_from_fen("8/8/8/8/1Pp5/8/8/8 w - b3 0 1").unwrap();
        assert!(pawn_moves_en_passant(6, 4, &b).is_some());
    }

    #[test]
    fn black_pawn_en_passant_right() {
        let b = board_from_fen("8/8/8/8/pP6/8/8/8 w - b3 0 1").unwrap();
        assert!(pawn_moves_en_passant(6, 2, &b).is_some());
    }

    #[test]
    fn black_pawn_en_passant_wrong_row() {
        let b = board_from_fen("8/8/8/pP6/8/8/8/8 w - b4 0 1").unwrap();
        assert!(pawn_moves_en_passant(5, 2, &b).is_none());
    }

    #[test]
    fn black_en_passant_capture_not_available() {
        let b = board_from_fen("8/8/8/8/pP6/8/8/8 w - - 0 1").unwrap();
        assert!(pawn_moves_en_passant(6, 2, &b).is_none());
    }

    // King tests

    #[test]
    fn king_empty_board_center() {
        let b = board_from_fen("8/8/8/8/3K4/8/8/k7 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        king_moves(6, 5, &b, &mut ret);
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn king_start_pos() {
        let b = board_from_fen("8/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        king_moves(9, 6, &b, &mut ret);
        assert_eq!(ret.len(), 5);
    }

    #[test]
    fn king_start_pos_other_pieces() {
        let b = board_from_fen("8/8/8/8/8/8/3Pn3/3QKB2 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        king_moves(9, 6, &b, &mut ret);
        assert_eq!(ret.len(), 2);
    }

    #[test]
    fn king_black_other_pieces() {
        let b = board_from_fen("8/8/8/8/8/3Pn3/3QkB2/3R1q2 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        king_moves(8, 6, &b, &mut ret);
        assert_eq!(ret.len(), 6);
    }

    // Rook tests

    #[test]
    fn rook_center_of_empty_board() {
        let b = board_from_fen("8/8/8/8/3R4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        rook_moves(6, 5, &b, &mut ret);
        assert_eq!(ret.len(), 14);
    }

    #[test]
    fn rook_center_of_board() {
        let b = board_from_fen("8/8/8/3q4/2kRp3/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        rook_moves(6, 5, &b, &mut ret);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn rook_center_of_board_with_white_pieces() {
        let b = board_from_fen("7p/3N4/8/4n3/2kR4/3b4/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        rook_moves(6, 5, &b, &mut ret);
        assert_eq!(ret.len(), 8);
    }

    #[test]
    fn rook_corner() {
        let b = board_from_fen("7p/3N4/K7/4n3/2kR4/3b4/8/7R w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        rook_moves(9, 9, &b, &mut ret);
        assert_eq!(ret.len(), 14);
    }
    #[test]
    fn black_rook_center_of_board_with_white_pieces() {
        let b = board_from_fen("7p/3N4/8/4n3/2kr4/3b4/8/K7 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        rook_moves(6, 5, &b, &mut ret);
        assert_eq!(ret.len(), 7);
    }

    // Bishop tests

    #[test]
    fn black_bishop_center_empty_board() {
        let b = board_from_fen("8/8/8/3b4/8/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        bishop_moves(5, 5, &b, &mut ret);
        assert_eq!(ret.len(), 13);
    }

    #[test]
    fn black_bishop_center_with_captures() {
        let b = board_from_fen("6P1/8/8/3b4/8/1R6/8/3Q4 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        bishop_moves(5, 5, &b, &mut ret);
        assert_eq!(ret.len(), 12);
    }

    #[test]
    fn black_bishop_center_with_captures_and_black_pieces() {
        let b = board_from_fen("6P1/8/2Q5/3b4/2k1n3/1R6/8/b2Q4 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        bishop_moves(5, 5, &b, &mut ret);
        assert_eq!(ret.len(), 4);
    }

    #[test]
    fn white_bishop_center_with_captures_and_white_pieces() {
        let b = board_from_fen("8/8/8/4r3/5B2/8/3Q4/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        bishop_moves(6, 7, &b, &mut ret);
        assert_eq!(ret.len(), 6);
    }

    // Queen tests

    #[test]
    fn white_queen_empty_board() {
        let b = board_from_fen("8/8/8/8/3Q4/8/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        queen_moves(6, 5, &b, &mut ret);
        assert_eq!(ret.len(), 27);
    }

    #[test]
    fn white_queen_cant_move() {
        let b = board_from_fen("8/8/8/2NBR3/2PQR3/2RRR3/8/8 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        queen_moves(6, 5, &b, &mut ret);
        assert_eq!(ret.len(), 0);
    }

    #[test]
    fn white_queen_with_other_piece() {
        let b = board_from_fen("8/6r1/8/8/3Q4/5N2/8/6P1 w - - 0 1").unwrap();
        let mut ret: Vec<Point> = vec![];
        queen_moves(6, 5, &b, &mut ret);
        assert_eq!(ret.len(), 25);
    }

    // Castling tests

    #[test]
    fn white_king_side_castle() {
        let mut b = board_from_fen("8/8/8/8/8/8/8/4K2R w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::WhiteKingSide));

        b = board_from_fen("8/8/2b5/8/8/6P1/5P1P/4K2R w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::WhiteKingSide));

        // Can't castle out of check
        b = board_from_fen("4r3/8/2b5/8/8/6P1/5P1P/4K2R w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteKingSide));

        // Can't castle through check
        b = board_from_fen("8/8/8/8/8/6Pb/5P1P/4K2R w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteKingSide));

        // Can't castle with pieces in way
        b = board_from_fen("8/8/8/8/8/6PN/5P1P/4KP1R w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteKingSide));

        // Can't castle with pieces in way 2
        b = board_from_fen("8/8/8/8/8/6PN/5P1P/4K1PR w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteKingSide));
    }

    #[test]
    fn white_queen_side_castle() {
        let mut b = board_from_fen("8/8/8/8/8/8/8/R3K3 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::WhiteQueenSide));

        b = board_from_fen("8/8/8/8/8/2P5/PP1P4/R3K1N1 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::WhiteQueenSide));

        // Can't castle out of check
        b = board_from_fen("8/8/8/8/8/2P2n2/PP1P4/R3K1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteQueenSide));

        // Can't castle through check
        b = board_from_fen("8/8/8/8/8/2n5/PP1P4/R3K1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteQueenSide));

        // Can't castle with pieces in way
        b = board_from_fen("8/8/8/8/8/2P5/PP1P4/R2QK1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteQueenSide));

        // Can't castle with pieces in way 2
        b = board_from_fen("8/8/8/8/8/2P5/PP1P4/R1Q1K1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteQueenSide));

        // Can't castle with pieces in way 3
        b = board_from_fen("8/8/8/8/8/2P5/PP1P4/RQ2K1N1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::WhiteQueenSide));
    }

    #[test]
    fn black_king_side_castle() {
        let mut b = board_from_fen("1p2k2r/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::BlackKingSide));

        b = board_from_fen("1p2k2r/4bp1p/6p1/8/8/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::BlackKingSide));

        // Can't castle out of check
        b = board_from_fen("1p2k2r/4bp1p/6p1/8/B7/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackKingSide));

        // Can't castle through check
        b = board_from_fen("1p2k2r/4bp1p/6pB/8/8/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackKingSide));

        // Can't castle with pieces in way
        b = board_from_fen("1p2k1nr/4bp1p/6pn/8/8/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackKingSide));

        // Can't castle with pieces in way 2
        b = board_from_fen("1p2kN1r/4bp1p/6pn/3n4/8/8/8/1P4P1 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackKingSide));
    }

    #[test]
    fn black_queen_side_castle() {
        let mut b = board_from_fen("r3k3/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::BlackQueenSide));

        b = board_from_fen("r3k3/qpb5/3n4/8/8/8/8/8 w KQkq - 0 1").unwrap();
        assert!(can_castle(&b, CastlingType::BlackQueenSide));

        // Can't castle out of check
        b = board_from_fen("r3k3/qpb5/3n4/8/8/8/8/4Q3 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackQueenSide));

        // Can't castle through check
        b = board_from_fen("r3k3/qpb5/3n4/8/7Q/8/8/8 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackQueenSide));

        // Can't castle with pieces in way
        b = board_from_fen("r2Pk3/qpb5/3n4/8/8/8/8/P7 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackQueenSide));

        // Can't castle with pieces in way 2
        b = board_from_fen("r1p1k3/qpb5/3n4/8/8/8/8/P7 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackQueenSide));

        // Can't castle with pieces in way 3
        b = board_from_fen("rn2k3/qpb5/3n4/8/8/8/8/P7 w KQkq - 0 1").unwrap();
        assert!(!can_castle(&b, CastlingType::BlackQueenSide));
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

        let moves = generate_moves(board);
        move_counts[cur_depth] += moves.len() as u32;
        for mov in moves {
            generate_moves_test(&mov, cur_depth + 1, depth, move_counts);
        }
    }

    // Perft tests - move generation. Table of values taken from https://www.chessprogramming.org/Perft_Results

    #[test]
    fn perft_test_position_1() {
        let mut moves_states = [0; 5];
        let b = board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
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
        let b =
            board_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
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
        let b = board_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
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
        let b = board_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
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
        let b = board_from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1")
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
        let b =
            board_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
        generate_moves_test(&b, 0, 4, &mut moves_states);
        assert_eq!(moves_states[0], 44);
        assert_eq!(moves_states[1], 1486);
        assert_eq!(moves_states[2], 62379);
        assert_eq!(moves_states[3], 2103487);
    }

    #[test]
    fn perft_test_position_6() {
        let mut moves_states = [0; 4];
        let b = board_from_fen(
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
