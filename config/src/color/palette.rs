#[derive(Default, Debug, Clone, PartialEq, FromDynamic, ToDynamic)]
pub struct Palette {
    /// The text color to use when the attributes are reset to default
    pub foreground: Option<RgbaColor>,
    /// The background color to use when the attributes are reset to default
    pub background: Option<RgbaColor>,
    /// The color of the cursor
    pub cursor_fg: Option<RgbaColor>,
    pub cursor_bg: Option<RgbaColor>,
    pub cursor_border: Option<RgbaColor>,
    /// The color of selected text
    pub selection_fg: Option<RgbaColor>,
    pub selection_bg: Option<RgbaColor>,
    /// A list of 8 colors corresponding to the basic ANSI palette
    pub ansi: Option<[RgbaColor; 8]>,
    /// A list of 8 colors corresponding to bright versions of the
    /// ANSI palette
    pub brights: Option<[RgbaColor; 8]>,
    /// A map for setting arbitrary colors ranging from 16 to 256 in the color
    /// palette
    #[dynamic(default)]
    pub indexed: HashMap<u8, RgbaColor>,
    /// Configure the colors and styling of the tab bar
    pub tab_bar: Option<TabBarColors>,
    /// The color of the "thumb" of the scrollbar; the segment that
    /// represents the current viewable area
    pub scrollbar_thumb: Option<RgbaColor>,
    /// The color of the split line between panes
    pub split: Option<RgbaColor>,
    /// The color of the visual bell. If unspecified, the foreground
    /// color is used instead.
    pub visual_bell: Option<RgbaColor>,
    /// The color to use for the cursor when a dead key or leader state is active
    pub compose_cursor: Option<RgbaColor>,

    pub copy_mode_active_highlight_fg: Option<ColorSpec>,
    pub copy_mode_active_highlight_bg: Option<ColorSpec>,
    pub copy_mode_inactive_highlight_fg: Option<ColorSpec>,
    pub copy_mode_inactive_highlight_bg: Option<ColorSpec>,

    pub quick_select_label_fg: Option<ColorSpec>,
    pub quick_select_label_bg: Option<ColorSpec>,
    pub quick_select_match_fg: Option<ColorSpec>,
    pub quick_select_match_bg: Option<ColorSpec>,

    pub input_selector_label_fg: Option<ColorSpec>,
    pub input_selector_label_bg: Option<ColorSpec>,

    pub launcher_label_fg: Option<ColorSpec>,
    pub launcher_label_bg: Option<ColorSpec>,
}
impl_lua_conversion_dynamic!(Palette);

impl Palette {
    pub fn overlay_with(&self, other: &Self) -> Self {
        macro_rules! overlay {
            ($name:ident) => {
                if let Some(c) = &other.$name {
                    Some(c.clone())
                } else {
                    self.$name.clone()
                }
            };
        }
        Self {
            foreground: overlay!(foreground),
            background: overlay!(background),
            cursor_fg: overlay!(cursor_fg),
            cursor_bg: overlay!(cursor_bg),
            cursor_border: overlay!(cursor_border),
            selection_fg: overlay!(selection_fg),
            selection_bg: overlay!(selection_bg),
            ansi: overlay!(ansi),
            brights: overlay!(brights),
            tab_bar: match (&self.tab_bar, &other.tab_bar) {
                (Some(a), Some(b)) => Some(a.overlay_with(&b)),
                (None, Some(b)) => Some(b.clone()),
                (Some(a), None) => Some(a.clone()),
                (None, None) => None,
            },
            indexed: {
                let mut map = self.indexed.clone();
                for (k, v) in &other.indexed {
                    map.insert(*k, *v);
                }
                map
            },
            scrollbar_thumb: overlay!(scrollbar_thumb),
            split: overlay!(split),
            visual_bell: overlay!(visual_bell),
            compose_cursor: overlay!(compose_cursor),
            copy_mode_active_highlight_fg: overlay!(copy_mode_active_highlight_fg),
            copy_mode_active_highlight_bg: overlay!(copy_mode_active_highlight_bg),
            copy_mode_inactive_highlight_fg: overlay!(copy_mode_inactive_highlight_fg),
            copy_mode_inactive_highlight_bg: overlay!(copy_mode_inactive_highlight_bg),
            quick_select_label_fg: overlay!(quick_select_label_fg),
            quick_select_label_bg: overlay!(quick_select_label_bg),
            quick_select_match_fg: overlay!(quick_select_match_fg),
            quick_select_match_bg: overlay!(quick_select_match_bg),
            input_selector_label_fg: overlay!(input_selector_label_fg),
            input_selector_label_bg: overlay!(input_selector_label_bg),
            launcher_label_fg: overlay!(launcher_label_fg),
            launcher_label_bg: overlay!(launcher_label_bg),
        }
    }
}

impl From<ColorPalette> for Palette {
    fn from(cp: ColorPalette) -> Palette {
        let mut p = Palette::default();
        macro_rules! apply_color {
            ($name:ident) => {
                p.$name = Some(cp.$name.into());
            };
        }
        apply_color!(foreground);
        apply_color!(background);
        apply_color!(cursor_fg);
        apply_color!(cursor_bg);
        apply_color!(cursor_border);
        apply_color!(selection_fg);
        apply_color!(selection_bg);
        apply_color!(scrollbar_thumb);
        apply_color!(split);

        let mut ansi = [RgbaColor::default(); 8];
        for (idx, col) in cp.colors.0[0..8].iter().enumerate() {
            ansi[idx] = (*col).into();
        }
        p.ansi = Some(ansi);

        let mut brights = [RgbaColor::default(); 8];
        for (idx, col) in cp.colors.0[8..16].iter().enumerate() {
            brights[idx] = (*col).into();
        }
        p.brights = Some(brights);

        for (idx, col) in cp.colors.0.iter().enumerate().skip(16) {
            p.indexed.insert(idx as u8, (*col).into());
        }

        p
    }
}

impl From<Palette> for ColorPalette {
    fn from(cfg: Palette) -> ColorPalette {
        let mut p = ColorPalette::default();
        macro_rules! apply_color {
            ($name:ident) => {
                if let Some($name) = cfg.$name {
                    p.$name = $name.into();
                }
            };
        }
        apply_color!(foreground);
        apply_color!(background);
        apply_color!(cursor_fg);
        apply_color!(cursor_bg);
        apply_color!(cursor_border);
        apply_color!(selection_fg);
        apply_color!(selection_bg);
        apply_color!(scrollbar_thumb);
        apply_color!(split);

        if let Some(ansi) = cfg.ansi {
            for (idx, col) in ansi.iter().enumerate() {
                p.colors.0[idx] = (*col).into();
            }
        }
        if let Some(brights) = cfg.brights {
            for (idx, col) in brights.iter().enumerate() {
                p.colors.0[idx + 8] = (*col).into();
            }
        }
        for (&idx, &col) in &cfg.indexed {
            if idx < 16 {
                log::warn!(
                    "Ignoring invalid colors.indexed index {}; \
                           use `ansi` or `brights` to specify lower indices",
                    idx
                );
                continue;
            }
            p.colors.0[idx as usize] = col.into();
        }
        p
    }
}
