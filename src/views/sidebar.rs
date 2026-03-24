use gpui::{div, prelude::*, px, Hsla, SharedString, Window};
use crate::ui::prelude::*;

/// Left sidebar
/// logo, new download button, navigation, storage card
///
pub struct Sidebar {
    pub active_item: usize,
    pub collapsed: bool,
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let nav_items: Vec<(IconName, &str)> = vec![
            (IconName::Inbox, "Downloads"),
            (IconName::ArrowDownToLine, "Active"),
            (IconName::CircleCheck, "Finished"),
            (IconName::CirclePause, "Paused"),
        ];

        let width = if self.collapsed { 48.0 } else { Spacing::SIDEBAR_WIDTH };

        div()
            .flex()
            .flex_col()
            .w(px(width))
            .h_full()
            .flex_shrink_0()
            .border_r_1()
            .border_color(Colors::border())
            .bg(Colors::sidebar())
            // Logo row — expanded: horizontal with toggle on right
            .when(!self.collapsed, |el| el.child(
                div()
                    .px(px(12.0))
                    .pt(px(12.0))
                    .mb(px(20.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(OpheliaLogo::new(44.0))
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(Colors::foreground())
                                    .child("ophelia")
                            )
                    )
                    .child(
                        div()
                            .id("collapse-toggle")
                            .flex()
                            .items_center()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.collapsed = !this.collapsed;
                                cx.notify();
                            }))
                            .child(icon_sm(IconName::PanelLeftClose, Colors::muted_foreground()))
                    )
            ))
            // Logo row — collapsed: vertical, toggle below logo
            .when(self.collapsed, |el| el.child(
                div()
                    .pt(px(12.0))
                    .mb(px(20.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(8.0))
                    .child(OpheliaLogo::new(44.0))
                    .child(
                        div()
                            .id("collapse-toggle")
                            .flex()
                            .items_center()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.collapsed = !this.collapsed;
                                cx.notify();
                            }))
                            .child(icon_sm(IconName::PanelLeftOpen, Colors::muted_foreground()))
                    )
            ))

            // Add Download button
            .when(!self.collapsed, |el| el.child(
                div()
                    .px(px(12.0))
                    .mb(px(16.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w_full()
                            .h(px(36.0))
                            .rounded(px(6.0))
                            .bg(Colors::active())
                            .text_color(Colors::background())
                            .text_sm()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("+ Add Download"),
                    ),
            ))
            .when(self.collapsed, |el| el.child(
                div()
                    .flex()
                    .justify_center()
                    .mb(px(16.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(36.0))
                            .h(px(36.0))
                            .rounded(px(6.0))
                            .bg(Colors::active())
                            .child(icon_sm(IconName::Plus, Colors::background())),
                    ),
            ))
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
                    .children(nav_items.into_iter().enumerate().map(|(i, (icon_name, label))| {
                        let is_active = i == self.active_item;
                        nav_item(icon_name, label, is_active, self.collapsed)
                            .id(i)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.active_item = i;
                                cx.notify();
                            }))
                    })),
            )
            // Spacer pushes storage card to bottom
            .child(div().flex_1())
            // Storage card
            .when(!self.collapsed, |el| el.child(
                div()
                    .p(px(12.0))
                    .child(storage_card()),
            ))
    }
}

/// A single navigation row: for now
fn nav_item(icon_name: IconName, label: &str, active: bool, collapsed: bool) -> gpui::Div {
    let bg: Hsla = if active {
        Colors::muted().into()
    } else {
        gpui::transparent_black()
    };
    let text: Hsla = if active {
        Colors::foreground().into()
    } else {
        Colors::muted_foreground().into()
    };

    div()
        .flex()
        .items_center()
        .when(collapsed, |el| el.justify_center())
        .gap(px(12.0))
        .px(px(12.0))
        .py(px(8.0))
        .rounded(px(6.0))
        .bg(bg)
        .text_color(text)
        .text_sm()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(icon(icon_name, px(18.0), text))
        .when(!collapsed, |el| el.child(SharedString::from(label.to_string())))
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
        // I know this is dumb but uhm it looks pretty
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
                        .text_xs()
                        .text_color(Colors::finished())
                        .child(icon_sm(IconName::Database, Colors::finished()))
                        .child("Storage"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(Colors::muted_foreground())
                        .child("62%"),
                ),
        )
        // Available space
        .child(
            div()
                .text_base()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(Colors::muted_foreground())
                .child("284 GB"),
        )
        .child(
            div()
                .text_xs()
                .text_color(Colors::finished())
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
                        .bg(Colors::finished())
                        .w(px(Spacing::SIDEBAR_WIDTH * used_fraction * 0.75)), // rough width
                ),
        )
    }
