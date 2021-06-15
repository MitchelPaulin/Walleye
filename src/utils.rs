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
}
