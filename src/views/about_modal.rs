use std::rc::Rc;

use gpui::{
    App, Context, Entity, FontWeight, IntoElement, Render, RenderOnce, Window, div, prelude::*, px,
};

use crate::app_menu;
use crate::ui::prelude::*;

type OnExitHandler = dyn Fn(&mut Window, &mut App);

pub struct AboutLayer {
    show: Entity<bool>,
}

impl AboutLayer {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let show = cx.new(|_| false);
        let show_clone = show.clone();

        App::on_action(cx, move |_: &app_menu::About, cx: &mut App| {
            show_clone.update(cx, |show, cx| {
                *show = true;
                cx.notify();
            });
        });

        cx.observe(&show, |_, _, cx| {
            cx.notify();
        })
        .detach();

        Self { show }
    }

    fn hide(&mut self, cx: &mut Context<Self>) {
        self.show.update(cx, |show, cx| {
            *show = false;
            cx.notify();
        });
    }
}

impl Render for AboutLayer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if *self.show.read(cx) {
            let weak = cx.weak_entity();
            div()
                .child(AboutModal::new(move |_, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        this.hide(cx);
                    });
                }))
                .into_any_element()
        } else {
            div().into_any_element()
        }
    }
}

#[derive(IntoElement)]
pub struct AboutModal {
    on_exit: Rc<OnExitHandler>,
}

impl AboutModal {
    fn new(on_exit: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Self {
            on_exit: Rc::new(on_exit),
        }
    }
}

impl RenderOnce for AboutModal {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let on_exit = Rc::clone(&self.on_exit);
        let on_exit_button = Rc::clone(&self.on_exit);

        modal()
            .on_exit(move |window, cx| {
                on_exit(window, cx);
            })
            .child(
                div()
                    .w(px(460.0))
                    .p(px(28.0))
                    .flex()
                    .flex_col()
                    .gap(px(18.0))
                    .child(
                        h_flex()
                            .items_center()
                            .gap(px(16.0))
                            .child(OpheliaLogo::new(52.0))
                            .child(
                                v_flex()
                                    .gap(px(4.0))
                                    .child(
                                        div()
                                            .text_xl()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(Colors::foreground())
                                            .child("Ophelia"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(Colors::muted_foreground())
                                            .child(format!(
                                                "Version {}",
                                                env!("CARGO_PKG_VERSION")
                                            )),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .line_height(px(22.0))
                            .text_color(Colors::muted_foreground())
                            .child("Feature-rich and extensible download manager."),
                    )
                    .child(
                        h_flex().justify_end().child(
                            div()
                                .id("about-close")
                                .px(px(18.0))
                                .py(px(10.0))
                                .rounded(px(8.0))
                                .bg(Colors::active())
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(Colors::background())
                                .cursor_pointer()
                                .on_click(move |_, window, cx| {
                                    on_exit_button(window, cx);
                                })
                                .child("Close"),
                        ),
                    ),
            )
    }
}
