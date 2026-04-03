use gpui::{Context, IntoElement, OwnedMenu, ParentElement, Render, Window, div, prelude::*, px};

use crate::app_menu::{self, OwnedMenuItemLike};
use crate::ui::prelude::*;

pub struct AppMenuBar {
    menus: Vec<OwnedMenu>,
    open_menu: Option<usize>,
}

impl AppMenuBar {
    pub fn new(menus: Vec<OwnedMenu>, cx: &mut Context<Self>) -> Self {
        let _ = cx;
        Self {
            menus,
            open_menu: None,
        }
    }

    fn toggle_menu(&mut self, index: usize, cx: &mut Context<Self>) {
        self.open_menu = if self.open_menu == Some(index) {
            None
        } else {
            Some(index)
        };
        cx.notify();
    }

    fn close_menu(&mut self, cx: &mut Context<Self>) {
        if self.open_menu.take().is_some() {
            cx.notify();
        }
    }
}

impl Render for AppMenuBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .id("app-menu-bar")
            .items_center()
            .gap(px(4.0))
            .on_mouse_down_out(cx.listener(|this, _, _, cx| this.close_menu(cx)))
            .children(self.menus.iter().enumerate().map(|(index, menu)| {
                let is_open = self.open_menu == Some(index);
                div()
                    .relative()
                    .child(
                        div()
                            .id(("menu-trigger", index))
                            .px(px(10.0))
                            .py(px(6.0))
                            .rounded(px(6.0))
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(Colors::foreground())
                            .bg(if is_open {
                                Colors::muted().into()
                            } else {
                                gpui::transparent_black()
                            })
                            .hover(|style| style.bg(Colors::muted()))
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_menu(index, cx);
                            }))
                            .child(app_menu::menu_label(menu)),
                    )
                    .when(is_open, |this| {
                        this.child(render_menu_popup(index, menu, window, cx))
                    })
            }))
    }
}

fn render_menu_popup(
    index: usize,
    menu: &OwnedMenu,
    window: &mut Window,
    cx: &mut Context<AppMenuBar>,
) -> impl IntoElement {
    let _ = window;
    div()
        .id(("menu-popup", index))
        .absolute()
        .top(px(34.0))
        .left_0()
        .min_w(px(210.0))
        .p(px(6.0))
        .rounded(px(10.0))
        .border_1()
        .border_color(Colors::border())
        .bg(Colors::card())
        .shadow_lg()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .children(
            app_menu::owned_menu_items(menu)
                .enumerate()
                .map(|(item_index, item)| match item {
                    OwnedMenuItemLike::Separator => div()
                        .id(("menu-separator", index * 1000 + item_index))
                        .my(px(4.0))
                        .h(px(1.0))
                        .bg(Colors::border())
                        .into_any_element(),
                    OwnedMenuItemLike::Action {
                        name,
                        action,
                        checked,
                        disabled,
                    } => {
                        let action = action.boxed_clone();
                        div()
                            .id(("menu-item", index * 1000 + item_index))
                            .flex()
                            .items_center()
                            .gap(px(10.0))
                            .px(px(10.0))
                            .py(px(8.0))
                            .rounded(px(7.0))
                            .text_sm()
                            .text_color(if disabled {
                                Colors::muted_foreground()
                            } else {
                                Colors::foreground()
                            })
                            .when(!disabled, |this| {
                                this.cursor_pointer()
                                    .hover(|style| style.bg(Colors::muted()))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_menu = None;
                                        window.dispatch_action(action.boxed_clone(), cx);
                                        cx.notify();
                                    }))
                            })
                            .child(
                                div()
                                    .w(px(12.0))
                                    .text_xs()
                                    .text_color(Colors::active())
                                    .child(if checked { "✓" } else { "" }),
                            )
                            .child(div().flex_1().child(name.to_string()))
                            .into_any_element()
                    }
                }),
        )
}
