use gpui::{div, prelude::*, px, App, Entity, Window};

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
        let entity = self.downloads.clone();
        let d = self.downloads.read(cx);
        let rows: Vec<DownloadRow> = (0..d.len())
            .map(|i| {
                let id = d.ids[i];
                let progress = match d.total_bytes[i] {
                    Some(total) if total > 0 => d.downloaded_bytes[i] as f32 / total as f32,
                    _ => 0.0,
                };
                let state = match d.statuses[i] {
                    DownloadStatus::Downloading => DownloadState::Active,
                    DownloadStatus::Paused => DownloadState::Paused,
                    DownloadStatus::Finished => DownloadState::Finished,
                    DownloadStatus::Error => DownloadState::Error,
                    _ => DownloadState::Queued,
                };

                let on_pause_resume: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>> =
                    match state {
                        DownloadState::Active | DownloadState::Queued => {
                            let e = entity.clone();
                            Some(Box::new(move |_w, cx| {
                                e.update(cx, |dl, cx| dl.pause(id, cx));
                            }))
                        }
                        DownloadState::Paused => {
                            let e = entity.clone();
                            Some(Box::new(move |_w, cx| {
                                e.update(cx, |dl, cx| dl.resume(id, cx));
                            }))
                        }
                        _ => None,
                    };

                let on_remove: Box<dyn Fn(&mut Window, &mut App) + 'static> = {
                    let e = entity.clone();
                    Box::new(move |_w, cx| {
                        e.update(cx, |dl, cx| dl.remove(id, cx));
                    })
                };

                DownloadRow {
                    id,
                    filename: d.filenames[i].clone(),
                    destination: d.destinations[i].clone(),
                    progress,
                    speed: format_speed(d.speeds[i]).into(),
                    state,
                    on_pause_resume,
                    on_remove: Some(on_remove),
                }
            })
            .collect();

        v_flex()
            .child(
                div()
                    .text_sm()
                    .text_color(Colors::muted_foreground())
                    .font_weight(gpui::FontWeight::EXTRA_BOLD)
                    .mb(px(14.0))
                    .child(rust_i18n::t!("downloads.section_label").to_string()),
            )
            .child(v_flex().gap(px(Spacing::LIST_GAP)).children(rows))
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
