use gpui::{Bounds, Pixels, WindowOptions};

#[derive(Clone, Copy)]
pub struct WindowChrome {
    pub height: f32,
    pub leading_padding: f32,
    pub horizontal_padding: f32,
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as imp;

#[cfg(not(target_os = "macos"))]
mod default;
#[cfg(not(target_os = "macos"))]
use default as imp;

pub fn window_chrome() -> WindowChrome {
    imp::window_chrome()
}

pub fn window_options(bounds: Bounds<Pixels>) -> WindowOptions {
    imp::window_options(bounds)
}
