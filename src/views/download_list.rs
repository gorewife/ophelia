use gpui::{div, prelude::*, px, Entity, Window};

use crate::app::Downloads;
use crate::engine::DownloadStatus;
use crate::ui::prelude::*;
use crate::views::download_row::{DownloadRow, DownloadState};

pub struct DownloadList {
    downloads: Entity<Downloads>,
}

impl DownloadList {
    pub fn new(downloads: Entity<Downloads>, cx: &mut Context<Self>) -> Self {
        cx.observe(&downloads, |_, _, cx| cx.notify()).detach();
        Self { downloads }
    }
}

impl Render for DownloadList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let d = self.downloads.read(cx);
        let rows: Vec<DownloadRow> = (0..d.len())
            .map(|i| {
                let progress = match d.total_bytes[i] {
                    Some(total) if total > 0 => d.downloaded_bytes[i] as f32 / total as f32,
                    _ => 0.0,
                };
                DownloadRow {
                    filename: d.filenames[i].clone(),
                    url: d.destinations[i].clone(),
                    progress,
                    speed: format_speed(d.speeds[i]).into(),
                    state: match d.statuses[i] {
                        DownloadStatus::Downloading => DownloadState::Active,
                        DownloadStatus::Finished => DownloadState::Finished,
                        _ => DownloadState::Queued,
                    },
                }
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_sm()
                    .text_color(Colors::muted_foreground())
                    .font_weight(gpui::FontWeight::EXTRA_BOLD)
                    .mb(px(14.0))
                    .child("RECENT"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(Spacing::LIST_GAP))
                    .children(rows),
            )
    }
}

fn format_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec == 0 {
        return String::new();
    }
    const MB: u64 = 1_000_000;
    const KB: u64 = 1_000;
    if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.0} KB/s", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}
