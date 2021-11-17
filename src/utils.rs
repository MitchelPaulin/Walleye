use std::time::Instant;

/*
    Remove new line characters from the end of a string

    Works on windows and linux
*/
pub fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

pub fn clean_input(buffer: &str) -> String {
    let mut cleaned = String::new();
    let mut prev_char = ' ';
    for c in buffer.chars() {
        if !c.is_whitespace() {
            cleaned.push(c);
        } else if c.is_whitespace() && !prev_char.is_whitespace() {
            cleaned.push(' ');
        }
        prev_char = c;
    }
    cleaned.trim().to_string()
}

/*
    Helper function to determine if we are out of time for our search
*/
pub fn out_of_time(start: Instant, time_to_move_ms: u128) -> bool {
    Instant::now().duration_since(start).as_millis() >= time_to_move_ms
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trim_windows() {
        let mut word = "hello\r\n".to_string();
        trim_newline(&mut word);
        assert_eq!("hello", word);
    }

    #[test]
    fn trim_linux() {
        let mut word = "hello\n".to_string();
        trim_newline(&mut word);
        assert_eq!("hello", word);
    }

    #[test]
    fn clean_string() {
        assert_eq!(clean_input("   debug     on  \n"), "debug on");
        assert_eq!(clean_input("\t  debug \t  \t\ton\t  \n"), "debug on");
    }
}
