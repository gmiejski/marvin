// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

use crate::error::AppError;

#[derive(Debug, Clone)]
pub enum EngineCommand {
    Start { content: String, delay_ms: u64 },
    Abort,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    TypingProgress {
        chars_done: usize,
        chars_total: usize,
    },
    Complete,
    Aborted,
    Error(AppError),
    Trigger,
}

impl From<AppError> for AppEvent {
    fn from(err: AppError) -> Self {
        Self::Error(err)
    }
}
