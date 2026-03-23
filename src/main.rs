mod app;
mod engine;
mod theme;
mod ui;
mod views;

use gpui::{prelude::*, px, size, App, Bounds, WindowBounds, WindowOptions};
use gpui_platform::application;
use views::main_window::MainWindow;

fn run() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.), px(600.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| MainWindow::new()),
        )
        .unwrap();
        cx.activate(true);
    });
}

fn main() {
    run();
}
