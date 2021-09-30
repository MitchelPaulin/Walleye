use crate::configs;
pub use crate::board::*;

pub const KILLER_MOVE_PLY_SIZE: usize = 2;
type MoveArray = [Option<(Point, Point)>; configs::MAX_DEPTH as usize];
type KillerMoveArray =
    [[Option<(Point, Point)>; KILLER_MOVE_PLY_SIZE]; configs::MAX_DEPTH as usize];

/*
    Keep track of global information about the current search context
*/
pub struct Search {
    pub killer_moves: KillerMoveArray, // the killer moves for this search
    pub pv_moves: MoveArray, // the principle variation for this search
    pub cur_line: MoveArray, // the current line being considered for this search
    pub nodes_searched: u32
}

impl Search {
    pub fn new_search() -> Search {
        Search {
            killer_moves: [[None; KILLER_MOVE_PLY_SIZE]; configs::MAX_DEPTH as usize],
            pv_moves: [None; configs::MAX_DEPTH as usize],
            cur_line: [None; configs::MAX_DEPTH as usize], 
            nodes_searched: 0
        }
    }

    pub fn node_searched(&mut self) {
        self.nodes_searched += 1;
    }

    pub fn insert_killer_move(&mut self, ply_from_root: i32, mov: &BoardState) {
        let ply = ply_from_root as usize;
        for i in 0..(KILLER_MOVE_PLY_SIZE - 1) {
            self.killer_moves[ply][i + 1] = self.killer_moves[ply][i];
        }
        self.killer_moves[ply][0] = mov.last_move;
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
        self.cur_line = [None; configs::MAX_DEPTH as usize];
    }
}