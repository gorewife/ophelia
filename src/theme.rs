use gpui::{Hsla, hsla};

// ---------------------------------------------------------------------------
// Color palette
// ---------------------------------------------------------------------------

pub struct Colors;

impl Colors {
    // Backgrounds
    pub fn background() -> Hsla {
        hsla(240. / 360., 0.10, 0.06, 1.0)
    }

    pub fn card() -> Hsla {
        hsla(240. / 360., 0.10, 0.09, 1.0)
    }

    pub fn sidebar() -> Hsla {
        hsla(240. / 360., 0.10, 0.07, 1.0)
    }

    pub fn muted() -> Hsla {
        hsla(240. / 360., 0.06, 0.13, 1.0)
    }

    // Foregrounds
    pub fn foreground() -> Hsla {
        hsla(0., 0.0, 0.96, 1.0)
    }

    pub fn muted_foreground() -> Hsla {
        hsla(240. / 360., 0.04, 0.45, 1.0)
    }

    // Accent / interactive
    pub fn primary() -> Hsla {
        hsla(145. / 360., 0.65, 0.55, 1.0)
    }

    pub fn secondary() -> Hsla {
        hsla(220. / 360., 0.55, 0.45, 1.0)
    }

    pub fn accent() -> Hsla {
        hsla(240. / 360., 0.06, 0.13, 1.0)
    }

    pub fn destructive() -> Hsla {
        hsla(15. / 360., 0.75, 0.50, 1.0)
    }

    // Borders / subtle
    pub fn border() -> Hsla {
        hsla(0., 0.0, 1.0, 0.07)
    }

    pub fn input_border() -> Hsla {
        hsla(0., 0.0, 1.0, 0.08)
    }

    pub fn ring() -> Hsla {
        hsla(145. / 360., 0.65, 0.55, 0.4)
    }
}

// ---------------------------------------------------------------------------
// Spacing
// ---------------------------------------------------------------------------

pub struct Spacing;

impl Spacing {
    pub const SIDEBAR_WIDTH: f32 = 224.0; // w-56
    pub const HEADER_HEIGHT: f32 = 40.0; // h-10
    pub const CONTENT_PADDING_X: f32 = 24.0; // px-6
    pub const CONTENT_PADDING_Y: f32 = 20.0; // py-5
    pub const CARD_GAP: f32 = 12.0; // gap-3
    pub const LIST_GAP: f32 = 6.0; // gap-1.5
    pub const ROW_PADDING_X: f32 = 16.0; // px-4
    pub const ROW_PADDING_Y: f32 = 12.0; // py-3
}

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

pub struct Typography;

impl Typography {
    pub const SIZE_2XS: f32 = 10.0;
    pub const SIZE_XS: f32 = 11.0;
    pub const SIZE_SM: f32 = 12.0;
    pub const SIZE_BASE: f32 = 13.0;
    pub const SIZE_MD: f32 = 14.0;
    pub const SIZE_LG: f32 = 16.0;
    pub const SIZE_XL: f32 = 18.0;
    pub const SIZE_2XL: f32 = 24.0;
}
