use gpui::{div, prelude::*, px, App, Window};

use crate::theme::{Colors, Spacing, Typography};

/// 4-column stats grid: speed, active, finished, total
#[derive(IntoElement)]
pub struct StatsBar;

impl RenderOnce for StatsBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .gap(px(Spacing::CARD_GAP))
            .mb(px(32.0))
            .child(stat_card("Speed", "0 B/s"))
            .child(stat_card("Active", "0"))
            .child(stat_card("Finished", "0"))
            .child(stat_card("Total", "0"))
    }
}

/// A single stat card: label + large value.
fn stat_card(label: &str, value: &str) -> gpui::Div {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .py(px(16.0))
        .px(px(16.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(Colors::border())
        .bg(Colors::card())
        .child(
            div()
                .text_size(px(Typography::SIZE_XS))
                .text_color(Colors::muted_foreground())
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(label.to_string()),
        )
        .child(
            div()
                .text_size(px(Typography::SIZE_2XL))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(value.to_string()),
        )
}
