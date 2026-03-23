use gpui::{point, px, Point, Pixels, TitlebarOptions};

pub const TITLEBAR_HEIGHT: f32 = 44.0;
pub const TRAFFIC_LIGHT_AREA: f32 = 72.0;
pub const TRAFFIC_LIGHT_OFFSET: Point<Pixels> = point(px(16.0), px(14.0));

pub fn titlebar_options() -> Option<TitlebarOptions> {
    Some(TitlebarOptions {
        appears_transparent: true,
        traffic_light_position: Some(TRAFFIC_LIGHT_OFFSET),
        ..Default::default()
    })
}
