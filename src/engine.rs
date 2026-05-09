// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use enigo::{Direction, Enigo, Key, Keyboard, Settings};

use crate::command::{AppEvent, EngineCommand};
use crate::error::AppError;

pub fn spawn_engine(
    cmd_rx: Receiver<EngineCommand>,
    evt_tx: Sender<AppEvent>,
) -> std::io::Result<JoinHandle<()>> {
    thread::Builder::new()
        .name("marvin-engine".into())
        .spawn(move || run(cmd_rx, evt_tx))
}

#[allow(clippy::needless_pass_by_value)]
fn run(cmd_rx: Receiver<EngineCommand>, evt_tx: Sender<AppEvent>) {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(err) => {
            tracing::error!(?err, "failed to initialise input simulator");
            let _ = evt_tx.send(AppEvent::Error(AppError::from(err)));
            return;
        }
    };
    tracing::debug!("engine thread ready");
    engine_loop(&mut enigo, &cmd_rx, &evt_tx);
    tracing::debug!("engine thread exiting");
}

fn engine_loop(enigo: &mut Enigo, cmd_rx: &Receiver<EngineCommand>, evt_tx: &Sender<AppEvent>) {
    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            EngineCommand::Start { content, delay_ms } => {
                let outcome = type_script(&content, delay_ms, enigo, evt_tx, cmd_rx);
                let evt = match outcome {
                    TypingOutcome::Completed => AppEvent::Complete,
                    TypingOutcome::Aborted => AppEvent::Aborted,
                    TypingOutcome::Failed(e) => AppEvent::Error(e),
                };
                let _ = evt_tx.send(evt);
            }
            EngineCommand::Abort => {
                tracing::trace!("Abort received while idle; ignoring");
            }
        }
    }
}

#[derive(Debug)]
enum TypingOutcome {
    Completed,
    Aborted,
    Failed(AppError),
}

fn type_script(
    content: &str,
    delay_ms: u64,
    enigo: &mut Enigo,
    evt_tx: &Sender<AppEvent>,
    cmd_rx: &Receiver<EngineCommand>,
) -> TypingOutcome {
    let chars_total = normalized_chars(content).count();
    tracing::info!(chars_total, delay_ms, "typing script");

    let delay = Duration::from_millis(delay_ms);
    let mut buf = [0u8; 4];

    for (i, ch) in normalized_chars(content).enumerate() {
        loop {
            match cmd_rx.try_recv() {
                Ok(EngineCommand::Abort) => {
                    tracing::info!(at = i, "abort received; stopping");
                    return TypingOutcome::Aborted;
                }
                Ok(EngineCommand::Start { .. }) => {
                    tracing::warn!("Start received while typing; ignoring");
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    tracing::warn!("command channel disconnected mid-typing");
                    return TypingOutcome::Aborted;
                }
            }
        }

        if let Err(err) = type_char(ch, &mut buf, enigo) {
            tracing::error!(?err, at = i, "keystroke failed");
            return TypingOutcome::Failed(AppError::from(err));
        }

        let _ = evt_tx.send(AppEvent::TypingProgress {
            chars_done: i + 1,
            chars_total,
        });

        if !delay.is_zero() {
            thread::sleep(delay);
        }
    }

    TypingOutcome::Completed
}

fn type_char(ch: char, buf: &mut [u8; 4], enigo: &mut Enigo) -> enigo::InputResult<()> {
    match ch {
        '\n' => enigo.key(Key::Return, Direction::Click),
        '\t' => enigo.key(Key::Tab, Direction::Click),
        c => enigo.text(c.encode_utf8(buf)),
    }
}

pub(crate) fn normalized_chars(s: &str) -> impl Iterator<Item = char> + '_ {
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
