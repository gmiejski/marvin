// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

use std::io::Read;
use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use enigo::{Enigo, Settings};
use marvin::typing::{normalized_chars, type_char};

const COUNTDOWN_SECS: u64 = 0;
const DEFAULT_TYPING_DELAY_MS: u64 = 15;

fn main() {
    let text = match std::env::args().nth(1) {
        Some(arg) if arg == "-" => read_stdin(),
        Some(arg) => unescape_newlines(&arg),
        None => {
            eprintln!("Usage: marvin-cli <text>  or  marvin-cli -");
            std::process::exit(1);
        }
    };

    if text.is_empty() {
        eprintln!("Empty input, nothing to type.");
        std::process::exit(1);
    }

    let delay_ms = load_delay_ms();

    thread::sleep(Duration::from_secs(COUNTDOWN_SECS));

    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Failed to init input simulator: {err}");
            std::process::exit(1);
        }
    };

    let delay = Duration::from_millis(delay_ms);
    let mut buf = [0u8; 4];

    for ch in normalized_chars(&text) {
        if let Err(err) = type_char(ch, &mut buf, &mut enigo) {
            eprintln!("Keystroke failed: {err}");
            std::process::exit(1);
        }
        if !delay.is_zero() {
            thread::sleep(delay);
        }
    }
}

fn unescape_newlines(s: &str) -> String {
    s.replace("\\n", "\n").replace("\\t", "\t")
}

fn read_stdin() -> String {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .expect("failed to read stdin");
    buf
}

fn config_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".config")
        .join("marvin")
        .join("config.toml")
}

fn load_delay_ms() -> u64 {
    let path = config_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return DEFAULT_TYPING_DELAY_MS,
    };

    for line in content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("delay_ms") {
            let value = value.trim().strip_prefix('=').unwrap_or("").trim();
            if let Ok(ms) = value.parse::<u64>() {
                return ms;
            }
        }
    }

    DEFAULT_TYPING_DELAY_MS
}
