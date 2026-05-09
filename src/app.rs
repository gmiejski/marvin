// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use eframe::egui;

use crate::clock::Clock;
use crate::command::{AppEvent, EngineCommand};
use crate::config::{ABORT_KEY, COUNTDOWN_DEFAULT_SECS, TYPING_DELAY_DEFAULT_MS};
use crate::error::AppError;
use crate::overlay::render_overlay;
use crate::platform::WindowPlatform;

const TICK_INTERVAL: Duration = Duration::from_millis(33);
const DONE_LINGER: Duration = Duration::from_secs(2);
const TYPING_DELAY_RANGE: std::ops::RangeInclusive<u64> = 1..=500;
const COUNTDOWN_RANGE: std::ops::RangeInclusive<u8> = 1..=30;
const SCRIPT_EDITOR_HEIGHT: f32 = 200.0;
const SCRIPT_EDITOR_ROWS: usize = 10;

#[derive(Debug)]
pub struct AppState {
    pub script_content: String,
    pub typing_delay_ms: u64,
    pub countdown_secs: u8,
    pub playback: PlaybackState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            script_content: String::new(),
            typing_delay_ms: TYPING_DELAY_DEFAULT_MS,
            countdown_secs: COUNTDOWN_DEFAULT_SECS,
            playback: PlaybackState::Idle,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum PlaybackState {
    #[default]
    Idle,
    Countdown {
        remaining_secs: f32,
    },
    Typing {
        chars_done: usize,
        chars_total: usize,
    },
    Done,
}

impl PlaybackState {
    #[must_use]
    pub fn progress(&self) -> Option<f32> {
        match *self {
            Self::Typing {
                chars_done,
                chars_total,
            } => Some(chars_done as f32 / chars_total.max(1) as f32),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Countdown { .. } | Self::Typing { .. })
    }
}

pub struct MarvinAppDeps {
    pub cmd_tx: Sender<EngineCommand>,
    pub app_rx: Receiver<AppEvent>,
    pub platform: Box<dyn WindowPlatform>,
    pub clock: Box<dyn Clock>,
}

impl std::fmt::Debug for MarvinAppDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarvinAppDeps")
            .field("platform", &self.platform)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct MarvinApp {
    state: AppState,
    cmd_tx: Sender<EngineCommand>,
    app_rx: Receiver<AppEvent>,
    platform: Box<dyn WindowPlatform>,
    clock: Box<dyn Clock>,
    countdown_started_at: Option<Instant>,
    done_at: Option<Instant>,
    error_banner: Option<String>,
}

impl MarvinApp {
    #[must_use]
    pub fn new(deps: MarvinAppDeps) -> Self {
        Self {
            state: AppState::default(),
            cmd_tx: deps.cmd_tx,
            app_rx: deps.app_rx,
            platform: deps.platform,
            clock: deps.clock,
            countdown_started_at: None,
            done_at: None,
            error_banner: None,
        }
    }

    fn apply_engine_event(&mut self, evt: AppEvent) {
        match evt {
            AppEvent::Trigger => self.handle_trigger(),
            AppEvent::TypingProgress {
                chars_done,
                chars_total,
            } => {
                self.state.playback = PlaybackState::Typing {
                    chars_done,
                    chars_total,
                };
            }
            AppEvent::Complete => {
                self.transition_to_done();
            }
            AppEvent::Aborted => {
                self.transition_to_idle();
            }
            AppEvent::Error(err) => {
                self.error_banner = Some(err.to_string());
                tracing::warn!(%err, "background error");
                self.transition_to_idle();
            }
        }
    }

    fn transition_to_idle(&mut self) {
        self.state.playback = PlaybackState::Idle;
        self.countdown_started_at = None;
        self.done_at = None;
        report_platform(
            &mut self.error_banner,
            self.platform.set_click_through(false),
        );
    }

    fn transition_to_done(&mut self) {
        self.state.playback = PlaybackState::Done;
        self.countdown_started_at = None;
        self.done_at = Some(self.clock.now());
        report_platform(
            &mut self.error_banner,
            self.platform.set_click_through(false),
        );
    }

    fn handle_trigger(&mut self) {
        if self.state.playback.is_active() {
            return;
        }
        if self.state.script_content.is_empty() {
            self.error_banner = Some("Script is empty".into());
            return;
        }

        self.error_banner = None;
        self.done_at = None;
        self.countdown_started_at = Some(self.clock.now());
        self.state.playback = PlaybackState::Countdown {
            remaining_secs: f32::from(self.state.countdown_secs),
        };
        report_platform(
            &mut self.error_banner,
            self.platform.set_click_through(true),
        );
    }

    fn tick_countdown(&mut self) {
        let PlaybackState::Countdown { .. } = self.state.playback else {
            return;
        };
        let Some(started) = self.countdown_started_at else {
            return;
        };

        let total = Duration::from_secs(u64::from(self.state.countdown_secs));
        let elapsed = self.clock.now().saturating_duration_since(started);

        if elapsed >= total {
            self.countdown_started_at = None;
            report_platform(
                &mut self.error_banner,
                self.platform.set_click_through(false),
            );
            self.state.playback = PlaybackState::Typing {
                chars_done: 0,
                chars_total: self.state.script_content.chars().count(),
            };
            if self
                .cmd_tx
                .send(EngineCommand::Start {
                    content: self.state.script_content.clone(),
                    delay_ms: self.state.typing_delay_ms,
                })
                .is_err()
            {
                self.error_banner = Some(AppError::EngineGone.to_string());
                self.state.playback = PlaybackState::Idle;
            }
        } else {
            let remaining = total.saturating_sub(elapsed).as_secs_f32();
            self.state.playback = PlaybackState::Countdown {
                remaining_secs: remaining,
            };
        }
    }

    fn tick_done(&mut self) {
        if !matches!(self.state.playback, PlaybackState::Done) {
            return;
        }
        if let Some(done_at) = self.done_at
            && self.clock.now().saturating_duration_since(done_at) >= DONE_LINGER
        {
            self.state.playback = PlaybackState::Idle;
            self.done_at = None;
        }
    }

    fn handle_abort_key(&mut self, ctx: &egui::Context) {
        if !self.state.playback.is_active() {
            return;
        }
        let pressed = ctx.input(|i| i.key_pressed(ABORT_KEY));
        if !pressed {
            return;
        }
        if self.cmd_tx.send(EngineCommand::Abort).is_err() {
            self.error_banner = Some(AppError::EngineGone.to_string());
        }
        self.transition_to_idle();
    }

    fn render_main(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let drag_bg = ui.interact(
                ui.max_rect(),
                ui.id().with("window_drag_bg"),
                egui::Sense::click_and_drag(),
            );
            if drag_bg.drag_started_by(egui::PointerButton::Primary) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            ui.heading("MARVIN");
            ui.add_space(4.0);

            if let Some(msg) = self.error_banner.clone() {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::LIGHT_RED, format!("⚠ {msg}"));
                    if ui.small_button("✕").clicked() {
                        self.error_banner = None;
                    }
                });
                ui.add_space(4.0);
            }

            ui.label("Script");
            egui::ScrollArea::vertical()
                .max_height(SCRIPT_EDITOR_HEIGHT)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_sized(
                        [ui.available_width(), SCRIPT_EDITOR_HEIGHT],
                        egui::TextEdit::multiline(&mut self.state.script_content)
                            .code_editor()
                            .desired_rows(SCRIPT_EDITOR_ROWS),
                    );
                });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Delay (ms)");
                ui.add(
                    egui::DragValue::new(&mut self.state.typing_delay_ms)
                        .range(TYPING_DELAY_RANGE)
                        .speed(1.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Countdown (s)");
                ui.add(
                    egui::DragValue::new(&mut self.state.countdown_secs)
                        .range(COUNTDOWN_RANGE)
                        .speed(1.0),
                );
            });

            ui.add_space(8.0);
            ui.separator();
            ui.label("Shift+Click to start  ·  Esc to stop");
        });
    }
}

fn report_platform(banner: &mut Option<String>, r: Result<(), AppError>) {
    if let Err(err) = r {
        tracing::warn!(%err, "platform call failed");
        *banner = Some(err.to_string());
    }
}

impl eframe::App for MarvinApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(evt) = self.app_rx.try_recv() {
            self.apply_engine_event(evt);
        }

        self.tick_countdown();
        self.tick_done();
        self.handle_abort_key(ctx);

        if self.state.playback.is_active() || matches!(self.state.playback, PlaybackState::Done) {
            ctx.request_repaint_after(TICK_INTERVAL);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.render_main(ui);
        render_overlay(&ctx, &self.state.playback);
    }
}
