// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

mod app;
mod clock;
mod command;
mod config;
mod engine;
mod error;
mod hook;
mod overlay;
mod platform;

use std::sync::mpsc;

use eframe::egui;
use tracing_subscriber::EnvFilter;

use crate::app::{MarvinApp, MarvinAppDeps};
use crate::clock::SystemClock;
use crate::config::{WINDOW_HEIGHT, WINDOW_WIDTH};

fn main() -> eframe::Result<()> {
    init_tracing();

    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (evt_tx, evt_rx) = mpsc::channel();

    #[allow(clippy::expect_used)]
    let engine_handle = engine::spawn_engine(cmd_rx, evt_tx.clone())
        .expect("spawning engine thread should not fail");

    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top();

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let result = eframe::run_native(
        "MARVIN",
        options,
        Box::new(move |cc| {
            let _hook_handle = hook::spawn_global_hook(cc.egui_ctx.clone(), evt_tx);
            let platform = platform::native();
            let _ = platform.set_always_on_top(true);

            Ok(Box::new(MarvinApp::new(MarvinAppDeps {
                cmd_tx,
                app_rx: evt_rx,
                platform,
                clock: Box::new(SystemClock),
            })))
        }),
    );

    drop(engine_handle);
    result
}

fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("marvin=info,warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}
