use gpui::{div, prelude::*, px, App, Window};
use crate::ui::prelude::*;

/// Download list container
/// shows recent downloads or empty state.
#[derive(IntoElement)]
pub struct DownloadList;

impl RenderOnce for DownloadList {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            // Section header
            .child(
                div()
                    .text_xs()
                    .text_color(Colors::muted_foreground())
                    .font_weight(gpui::FontWeight::BOLD)
                    .mb(px(12.0))
                    .child("RECENT"),
            )
            // Empty state (placeholder until downloads are wired up)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .py(px(80.0))
                    .child(
                        div()
                            .text_size(px(32.0))
                            .text_color(Colors::muted_foreground())
                            .child("inbox"), // TODO: icon
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(Colors::muted_foreground())
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("No downloads yet"),
                    ),
            )
    }
}
