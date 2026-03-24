use gpui::{div, prelude::*, px, App, Window};
use crate::ui::prelude::*;
use crate::views::download_row::{DownloadRow, DownloadState};

#[derive(IntoElement)]
pub struct DownloadList;

impl RenderOnce for DownloadList {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let rows: Vec<DownloadRow> = vec![
            DownloadRow {
                filename: "ubuntu-24.04.3-desktop-amd64.iso".into(),
                url: "releases.ubuntu.com/24.04/ubuntu-24.04.iso".into(),
                progress: 0.62,
                speed: "3.4 MB/s".into(),
                state: DownloadState::Active,
            },
            DownloadRow {
                filename: "Figma.dmg".into(),
                url: "desktop.figma.com/mac/Figma.dmg".into(),
                progress: 1.0,
                speed: "—".into(),
                state: DownloadState::Finished,
            },
            DownloadRow {
                filename: "node-v20.11.0.pkg".into(),
                url: "nodejs.org/dist/v20.11.0/node-v20.11.0.pkg".into(),
                progress: 0.0,
                speed: "—".into(),
                state: DownloadState::Queued,
            },
        ];

        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_xs()
                    .text_color(Colors::muted_foreground())
                    .font_weight(gpui::FontWeight::BOLD)
                    .mb(px(12.0))
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
