use crate::{board::BoardState, zobrist::ZobristKey};
use std::collections::HashMap;

#[derive(Clone)]
pub struct DrawTable {
    pub table: HashMap<ZobristKey, u8>,
}

impl DrawTable {
    pub fn new() -> DrawTable {
        DrawTable {
            table: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }

    pub fn remove_board_from_draw_table(&mut self, board: &BoardState) {
        if let Some(&val) = self.table.get(&board.zobrist_key) {
            self.table.insert(board.zobrist_key, val - 1);
        }
    }

    pub fn add_board_to_draw_table(&mut self, board: &BoardState) {
        let board_count = *self.table.get(&board.zobrist_key).unwrap_or(&0);
        self.table.insert(board.zobrist_key, board_count + 1);
    }

    /*
        Given the next move as a board determine if making that move would result
        in a three fold repetition
    */
    pub fn is_threefold_repetition(&mut self, board: &BoardState) -> bool {
        let board_count = *self.table.get(&board.zobrist_key).unwrap_or(&0);

        if board_count == 2 {
            // this position has been seen twice before, so making the move again would be a draw
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::DEFAULT_FEN_STRING;

    #[test]
    fn remove_board_from_draw_table_test() {
        let board = BoardState::from_fen(DEFAULT_FEN_STRING).unwrap();
        let mut draw_table: DrawTable = DrawTable::new();
        draw_table.table.insert(board.zobrist_key, 2);
        draw_table.remove_board_from_draw_table(&board);
        assert_eq!(*draw_table.table.get(&board.zobrist_key).unwrap(), 1);
    }

    #[test]
    fn draw_detected_three_fold_rep() {
        let board = BoardState::from_fen(DEFAULT_FEN_STRING).unwrap();
        let mut draw_table: DrawTable = DrawTable::new();
        draw_table.table.insert(board.zobrist_key, 2);
        assert!(draw_table.is_threefold_repetition(&board));
    }

    #[test]
    fn draw_not_detected() {
        let board = BoardState::from_fen(DEFAULT_FEN_STRING).unwrap();
        let mut draw_table: DrawTable = DrawTable::new();
        draw_table.table.insert(board.zobrist_key, 1);
        assert!(!draw_table.is_threefold_repetition(&board));
    }

    #[test]
    fn board_removed_from_draw_table() {
        let board = BoardState::from_fen(DEFAULT_FEN_STRING).unwrap();
        let mut draw_table: DrawTable = DrawTable::new();
        draw_table.table.insert(board.zobrist_key, 2);
        draw_table.remove_board_from_draw_table(&board);
        assert_eq!(*draw_table.table.get(&board.zobrist_key).unwrap(), 1);
    }
}
