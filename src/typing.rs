// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

use enigo::{Direction, Enigo, Key, Keyboard};

pub fn type_char(ch: char, buf: &mut [u8; 4], enigo: &mut Enigo) -> enigo::InputResult<()> {
    match ch {
        '\n' => enigo.key(Key::Return, Direction::Click),
        '\t' => enigo.key(Key::Tab, Direction::Click),
        c => enigo.text(c.encode_utf8(buf)),
    }
}

pub fn normalized_chars(s: &str) -> impl Iterator<Item = char> + '_ {
    let mut prev_cr = false;
    s.chars().filter_map(move |c| match c {
        '\r' => {
            prev_cr = true;
            Some('\n')
        }
        '\n' if prev_cr => {
            prev_cr = false;
            None
        }
        other => {
            prev_cr = false;
            Some(other)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::normalized_chars;

    fn normalize(s: &str) -> String {
        normalized_chars(s).collect()
    }

    #[test]
    fn lf_passes_through() {
        assert_eq!(normalize("a\nb"), "a\nb");
    }

    #[test]
    fn crlf_collapses_to_lf() {
        assert_eq!(normalize("a\r\nb"), "a\nb");
    }

    #[test]
    fn lone_cr_becomes_lf() {
        assert_eq!(normalize("a\rb"), "a\nb");
    }

    #[test]
    fn mixed_crlf_and_cr() {
        assert_eq!(normalize("a\r\nb\rc\nd"), "a\nb\nc\nd");
    }

    #[test]
    fn double_cr_is_two_lfs() {
        assert_eq!(normalize("a\r\rb"), "a\n\nb");
    }

    #[test]
    fn empty_input() {
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn unicode_passes_through() {
        assert_eq!(normalize("héllo→"), "héllo→");
    }
}
