use gpui::{div, prelude::*, px, Context, Window};

use crate::theme::{Colors, Spacing};
use crate::views::sidebar::Sidebar;
use crate::views::stats_bar::StatsBar;
use crate::views::download_list::DownloadList;

/// Root view
/// owns the full window layout
///
/// This is the only `Render` (stateful) view at the top level.
/// It composes the sidebar and main content area side by side.
pub struct MainWindow;

impl MainWindow {
    pub fn new() -> Self {
        Self
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .bg(Colors::background())
            .text_color(Colors::foreground())
            // Left: fixed-width sidebar
            .child(Sidebar)
            // Right: main content fills remaining space
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_hidden()
                    // Header (drag region + search, placeholder for now)
                    .child(
                        div()
                            .h(px(Spacing::HEADER_HEIGHT))
                            .flex()
                            .items_center()
                            .justify_center()
                            .border_b_1()
                            .border_color(Colors::border())
                            .child("search placeholder"),
                    )
                    // Scrollable content area
                    .child(
                        div()
                            .id("main-content")
                            .flex_1()
                            .flex()
                            .flex_col()
                            .overflow_y_scroll()
                            .px(px(Spacing::CONTENT_PADDING_X))
                            .py(px(Spacing::CONTENT_PADDING_Y))
                            .child(StatsBar)
                            .child(DownloadList),
                    ),
            )
    }
}
