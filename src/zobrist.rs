use crate::board::{Piece, PieceColor::*, Point};
pub use crate::move_generation::CastlingType;
use rand_chacha::rand_core::{RngCore, SeedableRng};

/*
    For simplicity use a 12x12 board so we do not
    need to convert between an 8x8 and 12x12 board
    coordinate system

    Since this array is not initialized very often it
    should have a negligible performance impact
*/
const BOARD_SIZE: usize = 12;
// 6 pieces * 2 colors
const PIECE_TYPES: usize = 12;

pub type ZobristKey = u64;

pub struct ZobristHasher {
    // indexed by [piece][file][rank]
    piece_square_table: [[[ZobristKey; BOARD_SIZE]; BOARD_SIZE]; PIECE_TYPES],
    black_to_move: ZobristKey,
    white_king_side_castle: ZobristKey,
    white_queen_side_castle: ZobristKey,
    black_king_side_castle: ZobristKey,
    black_queen_side_castle: ZobristKey,
    // indexed by file
    en_passant_files: [ZobristKey; BOARD_SIZE],
}

impl ZobristHasher {
    pub fn create_zobrist_hasher() -> ZobristHasher {
        // Here we use a seed so if you have to recreate the hasher you will always get the same values
        // Paul Morphy's birthday
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(6 * 10 * 1837);

        let mut piece_square_table = [[[0; BOARD_SIZE]; BOARD_SIZE]; PIECE_TYPES];
        #[allow(clippy::needless_range_loop)]
        for i in 0..BOARD_SIZE {
            #[allow(clippy::needless_range_loop)]
            for j in 0..BOARD_SIZE {
                #[allow(clippy::needless_range_loop)]
                for k in 0..PIECE_TYPES {
                    piece_square_table[k][j][i] = rng.next_u64();
                }
            }
        }

        let mut en_passant_files = [0; BOARD_SIZE];
        #[allow(clippy::needless_range_loop)]
        for i in 0..BOARD_SIZE {
            en_passant_files[i] = rng.next_u64();
        }

        ZobristHasher {
            piece_square_table,
            black_to_move: rng.next_u64(),
            white_king_side_castle: rng.next_u64(),
            white_queen_side_castle: rng.next_u64(),
            black_king_side_castle: rng.next_u64(),
            black_queen_side_castle: rng.next_u64(),
            en_passant_files,
        }
    }

    pub fn get_val_for_piece(&self, piece: Piece, point: Point) -> ZobristKey {
        // shift everything by 6 for black pieces
        // ensures each piece,color pair gets a unique number in [0,11]
        let index = piece.index() + if piece.color == White { 0 } else { 6 };

        self.piece_square_table[index][point.1][point.0]
    }

    pub fn get_val_for_castling(&self, castling_type: CastlingType) -> ZobristKey {
        match castling_type {
            CastlingType::WhiteKingSide => self.white_king_side_castle,
            CastlingType::WhiteQueenSide => self.white_queen_side_castle,
            CastlingType::BlackKingSide => self.black_king_side_castle,
            CastlingType::BlackQueenSide => self.black_queen_side_castle,
        }
    }

    pub fn get_val_for_en_passant(&self, file: usize) -> ZobristKey {
        self.en_passant_files[file]
    }

    pub fn get_black_to_move_val(&self) -> ZobristKey {
        self.black_to_move
    }
}
