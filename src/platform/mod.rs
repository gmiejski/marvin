// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

use crate::error::AppError;

pub trait WindowPlatform: std::fmt::Debug {
    fn set_click_through(&self, enabled: bool) -> Result<(), AppError>;
    fn set_always_on_top(&self, enabled: bool) -> Result<(), AppError>;
}

#[cfg(target_os = "macos")]
pub mod macos;
pub mod stub;

#[must_use]
pub fn native() -> Box<dyn WindowPlatform> {
    #[cfg(target_os = "macos")]
    {
        if let Some(p) = macos::MacOSPlatform::new() {
            return Box::new(p);
        }
        tracing::warn!("MacOSPlatform::new() returned None; falling back to stub");
    }
    Box::new(stub::StubPlatform::new())
}
