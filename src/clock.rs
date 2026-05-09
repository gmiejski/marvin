// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

use std::time::Instant;

pub trait Clock: std::fmt::Debug + Send + Sync {
    fn now(&self) -> Instant;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}
