use std::path::{Path, PathBuf};

use gpui::{div, prelude::*, px, relative, EventEmitter, Hsla, SharedString, Window};
use crate::ui::prelude::*;

pub struct AddDownloadClicked;

/// Left sidebar
/// logo, new download button, navigation, storage card
///
pub struct Sidebar {
    pub active_item: usize,
    pub collapsed: bool,
    pub download_dir: PathBuf,
}

impl EventEmitter<AddDownloadClicked> for Sidebar {}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let nav_items: Vec<(IconName, &str)> = vec![
            (IconName::Inbox, "Downloads"),
            (IconName::ArrowDownToLine, "Active"),
            (IconName::CircleCheck, "Finished"),
            (IconName::CirclePause, "Paused"),
        ];

        let width = if self.collapsed { 56.0 } else { Spacing::SIDEBAR_WIDTH };

        div()
            .flex()
            .flex_col()
            .w(px(width))
            .h_full()
            .flex_shrink_0()
            .border_r_1()
            .border_color(Colors::border())
            .bg(Colors::sidebar())
            // Logo row
            .when(!self.collapsed, |el| el.child(
                div()
                    .px(px(16.0))
                    .pt(px(14.0))
                    .mb(px(22.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.0))
                            .child(OpheliaLogo::new(44.0))
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(gpui::FontWeight::EXTRA_BOLD)
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
            // Logo row - collapsed: vertical, toggle below logo
            .when(self.collapsed, |el| el.child(
                div()
                    .pt(px(14.0))
                    .mb(px(22.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(10.0))
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
                    .px(px(16.0))
                    .mb(px(18.0))
                    .child(
                        div()
                            .id("add-download-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .w_full()
                            .h(px(40.0))
                            .rounded(px(8.0))
                            .bg(Colors::active())
                            .text_color(Colors::background())
                            .text_base()
                            .font_weight(gpui::FontWeight::BOLD)
                            .cursor_pointer()
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(AddDownloadClicked);
                            }))
                            .child("+ Add Download"),
                    ),
            ))
            .when(self.collapsed, |el| el.child(
                div()
                    .flex()
                    .justify_center()
                    .mb(px(18.0))
                    .child(
                        div()
                            .id("add-download-btn-collapsed")
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(40.0))
                            .h(px(40.0))
                            .rounded(px(8.0))
                            .bg(Colors::active())
                            .cursor_pointer()
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(AddDownloadClicked);
                            }))
                            .child(icon_sm(IconName::Plus, Colors::background())),
                    ),
            ))
            // Separator
            .child(
                div()
                    .mx(px(16.0))
                    .mb(px(10.0))
                    .h(px(1.0))
                    .bg(Colors::border()),
            )
            // Navigation items
            .child(
                div()
                    .px(px(10.0))
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
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
                    .p(px(16.0))
                    .child(storage_card(&self.download_dir)),
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
        .px(px(14.0))
        .py(px(10.0))
        .rounded(px(8.0))
        .bg(bg)
        .text_color(text)
        .text_sm()
        .font_weight(gpui::FontWeight::BOLD)
        .when(active, |el| el.border_l_2().border_color(Colors::ring()))
        .child(icon(icon_name, px(20.0), text))
        .when(!collapsed, |el| el.child(SharedString::from(label.to_string())))
}

fn query_disk(path: &Path) -> (u64, u64) {
    use std::ffi::CString;
    let Ok(cpath) = CString::new(path.to_string_lossy().as_bytes()) else { return (0, 0) };
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(cpath.as_ptr(), &mut stat) } != 0 { return (0, 0) }
    let block  = stat.f_frsize as u64;
    let total  = block * stat.f_blocks as u64;
    let avail  = block * stat.f_bavail as u64;
    (total.saturating_sub(avail), total)
}

fn format_gb(bytes: u64) -> String {
    const GB: f64 = 1_000_000_000.0;
    const TB: f64 = 1_000_000_000_000.0;
    let b = bytes as f64;
    if b >= TB { format!("{:.1} TB", b / TB) } else { format!("{:.1} GB", b / GB) }
}

fn storage_card(path: &Path) -> gpui::Div {
    let (used, total) = query_disk(path);
    let fraction = if total > 0 { (used as f32 / total as f32).clamp(0.0, 1.0) } else { 0.0 };

    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .p(px(14.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(Colors::border())
        .bg(Colors::card())
        // Header
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .text_sm()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(Colors::finished())
                .child(icon_sm(IconName::Database, Colors::finished()))
                .child("Storage"),
        )
        // "x.x GB / x.x TB"
        .child(
            div()
                .flex()
                .items_end()
                .gap(px(3.0))
                .child(
                    div()
                        .text_base()
                        .font_weight(gpui::FontWeight::EXTRA_BOLD)
                        .text_color(Colors::foreground())
                        .child(format_gb(used)),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(Colors::muted_foreground())
                        .mb(px(1.0))
                        .child("/"),
                )
                .child(
                    div()
                        .text_base()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(Colors::muted_foreground())
                        .child(format_gb(total)),
                ),
        )
        // Label
        .child(
            div()
                .text_sm()
                .text_color(Colors::finished())
                .child("used"),
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
                        .w(relative(fraction)),
                ),
        )
}
