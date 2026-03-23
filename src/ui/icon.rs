use gpui::{px, svg, Hsla, Pixels, SharedString, Styled, Svg};

/// Icon names mapping to Lucide SVG files in assets/icons/
#[derive(Debug, Clone, Copy)]
pub enum IconName {
    Inbox,
    ArrowDownToLine,
    CircleCheck,
    CirclePause,
    HardDrive,
    Plus,
}

impl IconName {
    pub fn path(self) -> SharedString {
        let name = match self {
            Self::Inbox => "inbox",
            Self::ArrowDownToLine => "arrow-down-to-line",
            Self::CircleCheck => "circle-check",
            Self::CirclePause => "circle-pause",
            Self::HardDrive => "hard-drive",
            Self::Plus => "plus",
        };
        SharedString::from(format!("icons/{name}.svg"))
    }
}

pub fn icon(name: IconName, size: Pixels, color: impl Into<Hsla>) -> Svg {
    svg()
        .path(name.path())
        .size(size)
        .flex_shrink_0()
        .text_color(color)
}

/// Default 16px icon
pub fn icon_sm(name: IconName, color: impl Into<Hsla>) -> Svg {
    icon(name, px(16.0), color)
}
