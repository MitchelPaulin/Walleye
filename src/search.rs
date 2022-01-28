pub use crate::board::*;
use crate::zobrist::ZobristKey;

pub const MAX_DEPTH: u8 = 100;
pub const KILLER_MOVE_PLY_SIZE: usize = 2;
type MoveArray = [Option<(Point, Point)>; MAX_DEPTH as usize];
type KillerMoveArray =
    [[ZobristKey; KILLER_MOVE_PLY_SIZE]; MAX_DEPTH as usize];

/*
    Information about the current search
*/
#[derive(Copy, Clone)]
pub struct SearchContext {
    pub killer_moves: KillerMoveArray, // the killer moves for this search
    pub pv_moves: MoveArray,           // the principle variation for this search
    pub cur_line: MoveArray,           // the current line being considered for this search
    pub nodes_searched: u32,
}

impl SearchContext {
    pub fn new_search() -> SearchContext {
        SearchContext {
            killer_moves: [[0; KILLER_MOVE_PLY_SIZE]; MAX_DEPTH as usize],
            pv_moves: [None; MAX_DEPTH as usize],
            cur_line: [None; MAX_DEPTH as usize],
            nodes_searched: 0,
        }
    }

    pub fn node_searched(&mut self) {
        self.nodes_searched += 1;
    }

    pub fn insert_killer_move(&mut self, ply_from_root: i32, mov: &BoardState) {
        let ply = ply_from_root as usize;
        if self.killer_moves[ply].contains(&mov.zobrist_key) {
            return;
        }

        for i in 0..(KILLER_MOVE_PLY_SIZE - 1) {
            self.killer_moves[ply][i + 1] = self.killer_moves[ply][i];
        }
        self.killer_moves[ply][0] = mov.zobrist_key;
    }

    pub fn insert_into_cur_line(&mut self, ply_from_root: i32, mov: &BoardState) {
        self.cur_line[ply_from_root as usize] = mov.last_move;
    }

    pub fn set_principle_variation(&mut self) {
        self.pv_moves.clone_from_slice(&self.cur_line);
    }

    // reset the required data to search the next depth
    pub fn reset_search(&mut self) {
        self.nodes_searched = 0;
        self.cur_line = [None; MAX_DEPTH as usize];
    }
}
