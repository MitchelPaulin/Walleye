pub use crate::board::{
    Board, BISHOP, BLACK, EMPTY, KING, KNIGHT, PAWN, QUEEN, ROOK, SENTINEL, WHITE,
};

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

    let mut row: usize = 2;
    let mut col: usize = 2;
    for fen_row in fen_rows {
        for square in fen_row.chars() {
            if square.is_digit(10) {
                let mut square_skip_count = square.to_digit(10).unwrap() as usize;
                if square_skip_count + col > 10 {
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
        if col != 10 {
            return Err("Could not parse fen string: Complete row was not specified");
        }
        row += 1;
        col = 2;
    }
    Ok(Board {
        board: b,
        to_move: to_move,
    })
}
