use gpui::{div, Div, Styled};

pub use crate::ui::icon::*;
pub use crate::ui::logo::*;
pub use crate::theme::*;

pub fn h_flex() -> Div {
    div().flex()
}

pub fn v_flex() -> Div {
    div().flex().flex_col()
}
