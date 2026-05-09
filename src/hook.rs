// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use eframe::egui;

use crate::command::AppEvent;

#[must_use = "the hook thread runs until the JoinHandle is dropped or the run loop exits"]
pub fn spawn_global_hook(ctx: egui::Context, evt_tx: Sender<AppEvent>) -> Option<JoinHandle<()>> {
    #[cfg(target_os = "macos")]
    {
        Some(macos::spawn(ctx, evt_tx))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (ctx, evt_tx);
        tracing::info!("global hook not implemented on this platform");
        None
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::panic::AssertUnwindSafe;
    use std::sync::mpsc::Sender;
    use std::thread::{self, JoinHandle};

    use core_foundation::runloop::CFRunLoop;
    use core_graphics::event::{
        CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
        CGEventType, CallbackResult,
    };
    use eframe::egui;

    use crate::command::AppEvent;
    use crate::error::AppError;

    pub(super) fn spawn(ctx: egui::Context, evt_tx: Sender<AppEvent>) -> JoinHandle<()> {
        #[allow(clippy::expect_used)]
        thread::Builder::new()
            .name("marvin-hook".into())
            .spawn(move || run(&ctx, &evt_tx))
            .expect("spawning a thread should not fail")
    }

    fn run(ctx: &egui::Context, evt_tx: &Sender<AppEvent>) {
        let cb_ctx = ctx.clone();
        let cb_tx = evt_tx.clone();

        let result = CGEventTap::with_enabled(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::LeftMouseDown],
            move |_proxy, _etype, event| {
                let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    if event.get_flags().contains(CGEventFlags::CGEventFlagShift) {
                        tracing::trace!("Shift+LeftClick detected");
                        let _ = cb_tx.send(AppEvent::Trigger);
                        cb_ctx.request_repaint();
                    }
                }));
                CallbackResult::Keep
            },
            CFRunLoop::run_current,
        );

        if result.is_err() {
            tracing::warn!("CGEventTap installation failed (Accessibility denied?)");
            let _ = evt_tx.send(AppEvent::Error(AppError::AccessibilityDenied));
            ctx.request_repaint();
        } else {
            tracing::debug!("hook thread exiting (run loop returned)");
        }
    }
}
