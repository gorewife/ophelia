use gpui::{div, prelude::*, px, App, SharedString, Window};

use crate::theme::{Colors, Spacing, Typography};

/// Left sidebar
/// logo, new download button, navigation, storage card
///
/// This is `RenderOnce` (stateless). It doesn't own any data; it just
/// describes the layout. Later, you'll pass props into it (active nav
/// filter, storage info, etc.) by adding fields to this struct.
#[derive(IntoElement)]
pub struct Sidebar;

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let nav_items: Vec<(&str, &str)> = vec![
            ("inbox", "Downloads"),
            ("downloading", "Active"),
            ("check_circle", "Finished"),
            ("pause_circle", "Paused"),
        ];

        div()
            .flex()
            .flex_col()
            .w(px(Spacing::SIDEBAR_WIDTH))
            .h_full()
            .flex_shrink_0()
            .border_r_1()
            .border_color(Colors::border())
            .bg(Colors::sidebar())
            // macOS traffic light clearance
            .child(div().h(px(Spacing::HEADER_HEIGHT)).flex_shrink_0())
            // Logo section
            .child(
                div()
                    .px(px(12.0))
                    .mb(px(24.0))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    // TODO: replace with actual OphLogo SVG component
                    .child(
                        div()
                            .size(px(28.0))
                            .rounded_full()
                            .bg(Colors::primary()),
                    )
                    .child(
                        div()
                            .text_size(px(Typography::SIZE_MD))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("ophelia"),
                    ),
            )
            // New Download button
            .child(
                div()
                    .px(px(12.0))
                    .mb(px(16.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w_full()
                            .h(px(32.0))
                            .rounded(px(6.0))
                            .bg(Colors::primary())
                            .text_color(Colors::background())
                            .text_size(px(Typography::SIZE_SM))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child("+ New Download"),
                    ),
            )
            // Separator
            .child(
                div()
                    .mx(px(12.0))
                    .mb(px(8.0))
                    .h(px(1.0))
                    .bg(Colors::border()),
            )
            // Navigation items
            .child(
                div()
                    .px(px(8.0))
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .children(nav_items.into_iter().enumerate().map(|(i, (icon, label))| {
                        let is_active = i == 0; // first item active by default
                        nav_item(icon, label, is_active)
                    })),
            )
            // Spacer pushes storage card to bottom
            .child(div().flex_1())
            // Storage card
            .child(
                div()
                    .p(px(12.0))
                    .child(storage_card()),
            )
    }
}

/// A single navigation row: icon placeholder + label.
fn nav_item(icon: &str, label: &str, active: bool) -> gpui::Div {
    let bg = if active {
        Colors::accent()
    } else {
        gpui::transparent_black()
    };
    let text = if active {
        Colors::foreground()
    } else {
        Colors::muted_foreground()
    };

    div()
        .flex()
        .items_center()
        .gap(px(12.0))
        .px(px(12.0))
        .py(px(8.0))
        .rounded(px(6.0))
        .bg(bg)
        .text_color(text)
        .text_size(px(Typography::SIZE_BASE))
        .font_weight(gpui::FontWeight::MEDIUM)
        // TODO: replace with actual icon rendering
        .child(SharedString::from(icon.to_string()))
        .child(SharedString::from(label.to_string()))
}

/// Storage info card at the bottom of the sidebar.
fn storage_card() -> gpui::Div {
    let used_fraction: f32 = 0.62; // placeholder

    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .p(px(12.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(Colors::border())
        .bg(Colors::card())
        // Header row: icon + "Storage" + percentage
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .text_size(px(Typography::SIZE_XS))
                        .text_color(Colors::muted_foreground())
                        .child("database") // TODO: icon
                        .child("Storage"),
                )
                .child(
                    div()
                        .text_size(px(Typography::SIZE_XS))
                        .text_color(Colors::muted_foreground())
                        .child("62%"),
                ),
        )
        // Available space
        .child(
            div()
                .text_size(px(Typography::SIZE_MD))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child("284 GB"),
        )
        .child(
            div()
                .text_size(px(Typography::SIZE_XS))
                .text_color(Colors::muted_foreground())
                .child("available"),
        )
        // Progress bar
        .child(
            div()
                .w_full()
                .h(px(4.0))
                .rounded_full()
                .bg(Colors::muted())
                .child(
                    div()
                        .h_full()
                        .rounded_full()
                        .bg(Colors::secondary())
                        .w(px(Spacing::SIDEBAR_WIDTH * used_fraction * 0.75)), // rough width
                ),
        )
    }
