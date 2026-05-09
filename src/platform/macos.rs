// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

use std::cell::OnceCell;

use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSWindow};
use objc2_foundation::MainThreadMarker;

use crate::error::AppError;

const NSNORMAL_WINDOW_LEVEL: isize = 0;
const NSFLOATING_WINDOW_LEVEL: isize = 3;

#[derive(Debug)]
pub struct MacOSPlatform {
    window: OnceCell<Retained<NSWindow>>,
    mtm: MainThreadMarker,
}

impl MacOSPlatform {
    #[must_use]
    pub fn new() -> Option<Self> {
        Some(Self {
            window: OnceCell::new(),
            mtm: MainThreadMarker::new()?,
        })
    }

    fn window(&self) -> Result<&NSWindow, AppError> {
        if let Some(w) = self.window.get() {
            return Ok(w);
        }

        let app = NSApplication::sharedApplication(self.mtm);

        let win = app.mainWindow().or_else(|| {
            let windows = app.windows();
            (windows.count() > 0).then(|| windows.objectAtIndex(0))
        });

        let Some(win) = win else {
            return Err(AppError::platform("NSWindow not yet available"));
        };

        Ok(self.window.get_or_init(|| win))
    }
}

impl super::WindowPlatform for MacOSPlatform {
    fn set_click_through(&self, enabled: bool) -> Result<(), AppError> {
        let win = self.window()?;
        win.setIgnoresMouseEvents(enabled);
        Ok(())
    }

    fn set_always_on_top(&self, enabled: bool) -> Result<(), AppError> {
        let win = self.window()?;
        let level = if enabled {
            NSFLOATING_WINDOW_LEVEL
        } else {
            NSNORMAL_WINDOW_LEVEL
        };
        win.setLevel(level);
        Ok(())
    }
}
