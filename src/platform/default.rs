use gpui::{Bounds, Pixels, WindowBounds, WindowOptions};

use crate::platform::WindowChrome;

pub fn window_chrome() -> WindowChrome {
    WindowChrome {
        height: 40.0,
        leading_padding: 20.0,
        horizontal_padding: 20.0,
    }
}

pub fn window_options(bounds: Bounds<Pixels>) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        ..Default::default()
    }
}
