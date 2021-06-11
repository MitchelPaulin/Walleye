pub use crate::board::*;
use std::io::{self, BufRead, Write};

pub fn play_game_uci(board: &BoardState) {
    let mut buffer = String::new();
    let mut stdin = io::stdin();
    let mut file = std::fs::File::create("log.txt").expect("Could not create log file");
    while true {
        stdin.lock().read_line(&mut buffer).unwrap();
        file.write_all(("ENGINE<< ".to_string() + &buffer).as_bytes()).expect("write failed");
        buffer = String::new();
    }
}
