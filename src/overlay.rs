// Copyright 2026 variHQ OÜ
// SPDX-License-Identifier: BSD-3-Clause

use eframe::egui;

use crate::app::PlaybackState;

const PROGRESS_BAR_WIDTH: f32 = 200.0;
const OVERLAY_BG_ALPHA: u8 = 180;

pub fn render_overlay(ctx: &egui::Context, state: &PlaybackState) {
    if !matches!(
        state,
        PlaybackState::Countdown { .. } | PlaybackState::Typing { .. }
    ) {
        return;
    }

    let screen = ctx.content_rect();
    let cx = screen.center();

    egui::Area::new(egui::Id::new("marvin_overlay"))
        .fixed_pos(cx)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_black_alpha(OVERLAY_BG_ALPHA))
                .show(ui, |ui| match state {
                    PlaybackState::Countdown { remaining_secs } => {
                        ui.heading(format!("▶  {remaining_secs:.1}s"));
                    }
                    PlaybackState::Typing { .. } => {
                        let progress = state.progress().unwrap_or(0.0);
                        ui.add(egui::ProgressBar::new(progress).desired_width(PROGRESS_BAR_WIDTH));
                    }
                    PlaybackState::Idle | PlaybackState::Done => {}
                });
        });
}
