pub use crate::board::{Piece, PieceColor::*, PieceKind::*, Point};
pub use crate::move_generation::CastlingType;
use rand_chacha::rand_core::{RngCore, SeedableRng};

const BOARD_SIZE: usize = 8;
// 6 pieces * 2 colors
const PIECE_TYPES: usize = 12;

#[allow(dead_code)]
pub struct ZobristHasher {
    // indexed by [piece][file][rank]
    piece_square_table: [[[u64; BOARD_SIZE]; BOARD_SIZE]; PIECE_TYPES],
    black_to_move: u64,
    white_king_side_castle: u64,
    white_queen_side_castle: u64,
    black_king_side_castle: u64,
    black_queen_side_castle: u64,
    // indexed by file
    en_passant_files: [u64; BOARD_SIZE],
}

impl ZobristHasher {
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn get_val_for_piece(&self, piece: Piece, point: Point) -> u64 {
        let index = match (piece.color, piece.kind) {
            (White, Pawn) => 0,
            (White, Knight) => 1,
            (White, Bishop) => 2,
            (White, Rook) => 3,
            (White, Queen) => 4,
            (White, King) => 5,
            (Black, Pawn) => 6,
            (Black, Knight) => 7,
            (Black, Bishop) => 8,
            (Black, Rook) => 9,
            (Black, Queen) => 10,
            (Black, King) => 11,
        };

        self.piece_square_table[index][point.1][point.0]
    }

    #[allow(dead_code)]
    pub fn get_val_for_castling(&self, castling_type: CastlingType) -> u64 {
        match castling_type {
            CastlingType::WhiteKingSide => self.white_king_side_castle,
            CastlingType::WhiteQueenSide => self.white_queen_side_castle,
            CastlingType::BlackKingSide => self.black_king_side_castle,
            CastlingType::BlackQueenSide => self.black_queen_side_castle,
        }
    }

    #[allow(dead_code)]
    pub fn get_val_for_en_passant(&self, file: usize) -> u64 {
        self.en_passant_files[file]
    }

    #[allow(dead_code)]
    pub fn get_black_to_move_val(&self) -> u64 {
        self.black_to_move
    }
}
