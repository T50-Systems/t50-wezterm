#[derive(Debug, Clone, FromDynamic, ToDynamic)]
pub struct TabBarStyle {
    #[dynamic(default = "default_new_tab")]
    pub new_tab: String,
    #[dynamic(default = "default_new_tab")]
    pub new_tab_hover: String,
    #[dynamic(default = "default_window_hide")]
    pub window_hide: String,
    #[dynamic(default = "default_window_hide")]
    pub window_hide_hover: String,
    #[dynamic(default = "default_window_maximize")]
    pub window_maximize: String,
    #[dynamic(default = "default_window_maximize")]
    pub window_maximize_hover: String,
    #[dynamic(default = "default_window_close")]
    pub window_close: String,
    #[dynamic(default = "default_window_close")]
    pub window_close_hover: String,
}

impl Default for TabBarStyle {
    fn default() -> Self {
        Self {
            new_tab: default_new_tab(),
            new_tab_hover: default_new_tab(),
            window_hide: default_window_hide(),
            window_hide_hover: default_window_hide(),
            window_maximize: default_window_maximize(),
            window_maximize_hover: default_window_maximize(),
            window_close: default_window_close(),
            window_close_hover: default_window_close(),
        }
    }
}

fn default_new_tab() -> String {
    " + ".to_string()
}

fn default_window_hide() -> String {
    " . ".to_string()
}

fn default_window_maximize() -> String {
    " - ".to_string()
}

fn default_window_close() -> String {
    " X ".to_string()
}

#[derive(Debug, Clone, FromDynamic, ToDynamic)]
pub struct WindowFrameConfig {
    #[dynamic(default = "default_inactive_titlebar_bg")]
    pub inactive_titlebar_bg: RgbaColor,
    #[dynamic(default = "default_active_titlebar_bg")]
    pub active_titlebar_bg: RgbaColor,
    #[dynamic(default = "default_inactive_titlebar_fg")]
    pub inactive_titlebar_fg: RgbaColor,
    #[dynamic(default = "default_active_titlebar_fg")]
    pub active_titlebar_fg: RgbaColor,
    #[dynamic(default = "default_inactive_titlebar_border_bottom")]
    pub inactive_titlebar_border_bottom: RgbaColor,
    #[dynamic(default = "default_active_titlebar_border_bottom")]
    pub active_titlebar_border_bottom: RgbaColor,
    #[dynamic(default = "default_button_fg")]
    pub button_fg: RgbaColor,
    #[dynamic(default = "default_button_bg")]
    pub button_bg: RgbaColor,
    #[dynamic(default = "default_button_hover_fg")]
    pub button_hover_fg: RgbaColor,
    #[dynamic(default = "default_button_hover_bg")]
    pub button_hover_bg: RgbaColor,

    #[dynamic(default)]
    pub font: Option<TextStyle>,
    #[dynamic(default)]
    pub font_size: Option<f64>,

    #[dynamic(try_from = "crate::units::PixelUnit", default = "default_zero_pixel")]
    pub border_left_width: Dimension,
    #[dynamic(try_from = "crate::units::PixelUnit", default = "default_zero_pixel")]
    pub border_right_width: Dimension,
    #[dynamic(try_from = "crate::units::PixelUnit", default = "default_zero_pixel")]
    pub border_top_height: Dimension,
    #[dynamic(try_from = "crate::units::PixelUnit", default = "default_zero_pixel")]
    pub border_bottom_height: Dimension,

    pub border_left_color: Option<RgbaColor>,
    pub border_right_color: Option<RgbaColor>,
    pub border_top_color: Option<RgbaColor>,
    pub border_bottom_color: Option<RgbaColor>,
}

const fn default_zero_pixel() -> Dimension {
    Dimension::Pixels(0.)
}

impl Default for WindowFrameConfig {
    fn default() -> Self {
        Self {
            inactive_titlebar_bg: default_inactive_titlebar_bg(),
            active_titlebar_bg: default_active_titlebar_bg(),
            inactive_titlebar_fg: default_inactive_titlebar_fg(),
            active_titlebar_fg: default_active_titlebar_fg(),
            inactive_titlebar_border_bottom: default_inactive_titlebar_border_bottom(),
            active_titlebar_border_bottom: default_active_titlebar_border_bottom(),
            button_fg: default_button_fg().into(),
            button_bg: default_button_bg().into(),
            button_hover_fg: default_button_hover_fg(),
            button_hover_bg: default_button_hover_bg(),
            font: None,
            font_size: None,
            border_left_width: default_zero_pixel(),
            border_right_width: default_zero_pixel(),
            border_top_height: default_zero_pixel(),
            border_bottom_height: default_zero_pixel(),
            border_left_color: None,
            border_right_color: None,
            border_top_color: None,
            border_bottom_color: None,
        }
    }
}

fn default_inactive_titlebar_bg() -> RgbaColor {
    RgbColor::new_8bpc(0x33, 0x33, 0x33).into()
}

fn default_active_titlebar_bg() -> RgbaColor {
    RgbColor::new_8bpc(0x33, 0x33, 0x33).into()
}

fn default_inactive_titlebar_fg() -> RgbaColor {
    RgbColor::new_8bpc(0xcc, 0xcc, 0xcc).into()
}

fn default_active_titlebar_fg() -> RgbaColor {
    RgbColor::new_8bpc(0xff, 0xff, 0xff).into()
}

fn default_inactive_titlebar_border_bottom() -> RgbaColor {
    RgbColor::new_8bpc(0x2b, 0x20, 0x42).into()
}

fn default_active_titlebar_border_bottom() -> RgbaColor {
    RgbColor::new_8bpc(0x2b, 0x20, 0x42).into()
}

fn default_button_hover_fg() -> RgbaColor {
    RgbColor::new_8bpc(0xff, 0xff, 0xff).into()
}

fn default_button_fg() -> RgbaColor {
    RgbColor::new_8bpc(0xcc, 0xcc, 0xcc).into()
}

fn default_button_hover_bg() -> RgbaColor {
    RgbColor::new_8bpc(0x1f, 0x1f, 0x1f).into()
}

fn default_button_bg() -> RgbaColor {
    RgbColor::new_8bpc(0x33, 0x33, 0x33).into()
}
