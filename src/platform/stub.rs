// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

use crate::error::AppError;

#[derive(Debug, Default)]
pub struct StubPlatform;

impl StubPlatform {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl super::WindowPlatform for StubPlatform {
    fn set_click_through(&self, _enabled: bool) -> Result<(), AppError> {
        Ok(())
    }
    fn set_always_on_top(&self, _enabled: bool) -> Result<(), AppError> {
        Ok(())
    }
}
