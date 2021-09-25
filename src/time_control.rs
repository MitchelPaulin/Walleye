use crate::board::PieceColor;

pub const SAFEGUARD: f64 = 100.0; // msecs
const GAME_LENGTH: usize = 30; // moves
const MAX_USAGE: f64 = 0.8; // percentage
const NO_TIME: u128 = 0;

pub struct GameTime {
    // all time is in ms unless otherwise specified
    pub wtime: i128,
    pub btime: i128,
    pub winc: i128,
    pub binc: i128,
    pub movestogo: Option<usize>,
}

/*
    Big thanks to @mvanthoor (https://github.com/mvanthoor) whose chess engine
    the below time control implementation was adapted from
*/
impl GameTime {
    // Calculates the time the engine allocates for searching a single
    // move. This depends on the number of moves still to go in the game.
    pub fn calculate_time_slice(&self, color: PieceColor) -> u128 {
        let mtg = self.moves_to_go() as f64;
        let is_white = color == PieceColor::White;
        let clock = if is_white { self.wtime } else { self.btime } as f64;
        let increment = if is_white { self.winc } else { self.binc } as f64;
        let base_time = clock - SAFEGUARD;

        // return a time slice.
        if base_time <= 0.0 {
            if increment > 0.0 {
                (increment * MAX_USAGE).round() as u128
            } else {
                NO_TIME
            }
        } else {
            ((base_time * MAX_USAGE / mtg) + increment).round() as u128
        }
    }

    // Here we try to come up with some sort of sensible value for "moves
    // to go", if this value is not supplied.
    fn moves_to_go(&self) -> usize {
        // Default to GAME_LENGTH moves if movestogo was not provided
        self.movestogo.unwrap_or(GAME_LENGTH)
    }
}
