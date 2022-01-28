use crate::{evaluation::{Point, BoardState}, zobrist::ZobristKey};
use std::{collections::HashMap};

#[derive(Clone)]
pub enum NodeType {
    LowerBound,
    Exact,
    UpperBound,
}

#[derive(Clone)]
pub struct TableEntry {
    depth: u8,
    eval: i32,
    node_type: NodeType,
    best_move: Option<(Point, Point)>
}

#[derive(Clone)]
pub struct TranspositionTable {
    pub table: HashMap<ZobristKey, TableEntry>,
}


impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        TranspositionTable {
            table: HashMap::new()
        }
    }

    pub fn insert(&mut self, depth: u8, eval: i32, node_type: NodeType, best_move: Option<(Point, Point)>, board: &BoardState) {

        let entry = TableEntry {
            depth,
            eval,
            node_type,
            best_move
        };

        self.table.insert(board.zobrist_key, entry);
    }

    pub fn probe(&self, board: &BoardState) -> Option<&TableEntry> {
        self.table.get(&board.zobrist_key)
    }
}