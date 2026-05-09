// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

use std::borrow::Cow;

#[derive(Debug, Clone, thiserror::Error)]
pub enum AppError {
    #[error("input simulation error: {0}")]
    Input(#[from] enigo::InputError),

    #[error("input simulator init failed: {0}")]
    InputInit(#[from] enigo::NewConError),

    #[error("platform error: {0}")]
    Platform(Cow<'static, str>),

    #[error(
        "accessibility permission denied - grant it under \
         System Settings -> Privacy & Security -> Accessibility"
    )]
    AccessibilityDenied,

    #[error("engine thread is no longer running")]
    EngineGone,
}

impl AppError {
    pub fn platform(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Platform(msg.into())
    }
}
