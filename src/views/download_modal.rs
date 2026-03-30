//! Add Download modal overlay
//!
//! On open, the clipboard is checked immediately and if it holds a URL its
//! pre-filled so the browser-extension workflow (copy → switch → confirm)

use std::path::PathBuf;

use gpui::{div, prelude::*, px, rgba, Context, EventEmitter, SharedString, Window};

use crate::ui::prelude::*;

pub struct DownloadConfirmed {
    pub url: String,
    pub destination: PathBuf,
}

pub struct DownloadCancelled;

pub struct DownloadModal {
    url: Option<String>,
}

impl EventEmitter<DownloadConfirmed> for DownloadModal {}
impl EventEmitter<DownloadCancelled> for DownloadModal {}

impl DownloadModal {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Pre-fill from clipboard if it looks like a URL.
        let url = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .filter(|s| s.starts_with("http://") || s.starts_with("https://"));
        Self { url }
    }

    fn paste_from_clipboard(&mut self, cx: &mut Context<Self>) {
        self.url = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .filter(|s| s.starts_with("http://") || s.starts_with("https://"));
        cx.notify();
    }

    fn destination_for(url: &str) -> PathBuf {
        let filename = url
            .split('/')
            .last()
            .and_then(|s| s.split('?').next())
            .filter(|s| !s.is_empty())
            .unwrap_or("download");
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join("Downloads").join(filename)
    }
}

impl Render for DownloadModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let url = self.url.clone();
        let dest = url.as_deref().map(Self::destination_for);
        let can_confirm = url.is_some();

        let url_display: SharedString = url
            .as_deref()
            .unwrap_or("Copy a link and click Paste")
            .to_string()
            .into();

        let dest_display: SharedString = dest
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
            .into();

        // Full-window dimmed backdrop
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba(0x00000088))
            // Centered card
            .child(
                div()
                    .w(px(480.0))
                    .rounded(px(14.0))
                    .border_1()
                    .border_color(Colors::border())
                    .bg(Colors::card())
                    .p(px(28.0))
                    .flex()
                    .flex_col()
                    .gap(px(20.0))
                    // Title
                    .child(
                        div()
                            .text_xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(Colors::foreground())
                            .child("Add Download"),
                    )
                    // URL row
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(Colors::muted_foreground())
                                    .child("URL"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .flex_1()
                                            .px(px(12.0))
                                            .py(px(10.0))
                                            .rounded(px(8.0))
                                            .border_1()
                                            .border_color(Colors::input_border())
                                            .bg(Colors::background())
                                            .text_sm()
                                            .text_color(if self.url.is_some() {
                                                Colors::foreground()
                                            } else {
                                                Colors::muted_foreground()
                                            })
                                            .child(url_display),
                                    )
                                    .child(
                                        div()
                                            .id("paste-btn")
                                            .flex()
                                            .items_center()
                                            .px(px(12.0))
                                            .py(px(10.0))
                                            .rounded(px(8.0))
                                            .border_1()
                                            .border_color(Colors::border())
                                            .bg(Colors::background())
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(Colors::foreground())
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.paste_from_clipboard(cx);
                                            }))
                                            .child("Paste"),
                                    ),
                            ),
                    )
                    // Destination row (only shown when URL is set)
                    .when(can_confirm, |el| {
                        el.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(Colors::muted_foreground())
                                        .child("Save to"),
                                )
                                .child(
                                    div()
                                        .px(px(12.0))
                                        .py(px(10.0))
                                        .rounded(px(8.0))
                                        .border_1()
                                        .border_color(Colors::input_border())
                                        .bg(Colors::background())
                                        .text_sm()
                                        .text_color(Colors::muted_foreground())
                                        .child(dest_display),
                                ),
                        )
                    })
                    // Action buttons
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .id("cancel-btn")
                                    .px(px(18.0))
                                    .py(px(10.0))
                                    .rounded(px(8.0))
                                    .border_1()
                                    .border_color(Colors::border())
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(Colors::foreground())
                                    .cursor_pointer()
                                    .on_click(cx.listener(|_, _, _, cx| {
                                        cx.emit(DownloadCancelled);
                                    }))
                                    .child("Cancel"),
                            )
                            .child(
                                div()
                                    .id("confirm-btn")
                                    .px(px(18.0))
                                    .py(px(10.0))
                                    .rounded(px(8.0))
                                    .bg(if can_confirm {
                                        Colors::active()
                                    } else {
                                        Colors::muted()
                                    })
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(Colors::background())
                                    .cursor_pointer()
                                    .when(can_confirm, |el| {
                                        el.on_click(cx.listener(move |this, _, _, cx| {
                                            if let Some(url) = this.url.clone() {
                                                let destination =
                                                    DownloadModal::destination_for(&url);
                                                cx.emit(DownloadConfirmed { url, destination });
                                            }
                                        }))
                                    })
                                    .child("Download"),
                            ),
                    ),
            )
    }
}
