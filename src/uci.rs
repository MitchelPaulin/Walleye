pub use crate::board::*;
use std::io::{self, BufRead, Write};

pub fn play_game_uci() {
    let mut buffer = String::new();
    let mut board = board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let log = std::fs::File::create("/home/mitch/Desktop/log.txt").expect("Could not create log file");
    buffer = read_from_gui(&log);
    if buffer != "uci\n" {
        log_error("Expected uci protocol but got ".to_string() + &buffer, &log);
        return;
    }
    
    send_to_gui("id name ChessEngine\n".to_string(), &log);
    send_to_gui("id author Mitchel Paulin\n".to_string(), &log);
    send_to_gui("uciok\n".to_string(), &log);

    buffer = read_from_gui(&log);
    if buffer != "isready\n" {
        log_error("Expected isready protocol but got ".to_string() + &buffer, &log);
        return;
    }

    send_to_gui("readyok\n".to_string(), &log);

    while true {
        buffer = read_from_gui(&log);
        if buffer == "quit\n" {
            break;
        } else if buffer == "ucinewgame\n" {

        } else if buffer == "position startpos\n" {
            board = board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        } else if buffer == "isready\n" {   
            send_to_gui("readyok\n".to_string(), &log);
        } else {
            log_error("Unrecognized command ".to_string() + &buffer, &log);
        }
    }

}

fn log_error(message: String, mut log: &std::fs::File) {
    log.write_all(("<ERROR> ".to_string() + &message).as_bytes()).expect("write failed");

}

fn send_to_gui(message: String, mut log: &std::fs::File) {
    print!("{}", message);
    log.write_all(("ENGINE >> ".to_string() + &message).as_bytes()).expect("write failed");
}

fn read_from_gui(mut log: &std::fs::File) -> String {
    let stdin = io::stdin();
    let mut buffer = String::new();
    stdin.lock().read_line(&mut buffer).unwrap();
    log.write_all(("ENGINE << ".to_string() + &buffer).as_bytes()).expect("write failed");
    return buffer;
}