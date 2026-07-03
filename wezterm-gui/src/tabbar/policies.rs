#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TabWidthPolicy {
    title_width: usize,
    titles_len: usize,
    number_of_tabs: usize,
    new_tab_len: usize,
    use_fancy_tab_bar: bool,
    tab_max_width: usize,
}

impl TabWidthPolicy {
    fn max_width(self) -> usize {
        let available_cells = self
            .title_width
            .saturating_sub(self.number_of_tabs.saturating_sub(1) + self.new_tab_len);

        if self.use_fancy_tab_bar || available_cells >= self.titles_len {
            usize::MAX
        } else {
            available_cells / self.number_of_tabs
        }
        .min(self.tab_max_width)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IntegratedTitleButtonReservation {
    title_width: usize,
    reserve: bool,
}

impl IntegratedTitleButtonReservation {
    fn title_width_after_reservation(self, button_widths: &[usize]) -> usize {
        if !self.reserve {
            return self.title_width;
        }

        self.title_width
            .saturating_sub(button_widths.iter().sum::<usize>())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StatusLineLayout {
    title_width: usize,
    current_x: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StatusLinePlan {
    center_width: usize,
    right_width: usize,
    right_trim_left: usize,
}

impl StatusLineLayout {
    fn right_width(self) -> usize {
        self.title_width.saturating_sub(self.current_x)
    }

    fn right_only_plan(self, right_status_len: usize) -> StatusLinePlan {
        let right_width = self.right_width();
        StatusLinePlan {
            center_width: 0,
            right_width,
            right_trim_left: right_status_len.saturating_sub(right_width),
        }
    }

    fn center_and_right_plan(self, right_status_len: usize) -> StatusLinePlan {
        let center_width = self
            .title_width
            .saturating_sub(self.current_x + right_status_len);
        let right_width = self
            .title_width
            .saturating_sub(self.current_x + center_width);
        StatusLinePlan {
            center_width,
            right_width,
            right_trim_left: right_status_len.saturating_sub(right_width),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct StatusTextRenderer {
    filler: Cell,
}

impl StatusTextRenderer {
    fn new(background: RgbaColor) -> Self {
        Self {
            filler: Cell::blank_with_attrs(
                CellAttributes::default()
                    .set_background(ColorSpec::TrueColor(*background))
                    .clone(),
            ),
        }
    }

    fn attrs(&self) -> &CellAttributes {
        self.filler.attrs()
    }

    fn filler(&self) -> Cell {
        self.filler.clone()
    }

    fn parse(&self, text: &str) -> Line {
        parse_status_text(text, self.attrs().clone())
    }

    fn fit_to_width(&self, status_line: &mut Line, width: usize) {
        if status_line.len() > width {
            status_line.resize(width, SEQ_ZERO);
        }
        while status_line.len() < width {
            status_line.insert_cell(status_line.len(), self.filler(), width, SEQ_ZERO);
        }
    }

    fn trim_left(&self, status_line: &mut Line, cells_to_remove: usize) {
        for _ in 0..cells_to_remove {
            status_line.remove_cell(0, SEQ_ZERO);
        }
    }
}

struct TabBarStyleAdapter<'a> {
    config: &'a ConfigHandle,
}

impl<'a> TabBarStyleAdapter<'a> {
    fn new(config: &'a ConfigHandle) -> Self {
        Self { config }
    }

    fn new_tab_line(&self, options: &TabBarBuildOptions, attrs: CellAttributes) -> Line {
        parse_status_text(
            &self.config.tab_bar_style.new_tab,
            Self::style_attrs(options, attrs),
        )
    }

    fn new_tab_hover_line(&self, options: &TabBarBuildOptions, attrs: CellAttributes) -> Line {
        parse_status_text(
            &self.config.tab_bar_style.new_tab_hover,
            Self::style_attrs(options, attrs),
        )
    }

    fn integrated_title_button_widths(&self) -> Vec<usize> {
        let window_hide = parse_status_text(
            &self.config.tab_bar_style.window_hide,
            CellAttributes::default(),
        );
        let window_hide_hover = parse_status_text(
            &self.config.tab_bar_style.window_hide_hover,
            CellAttributes::default(),
        );
        let window_maximize = parse_status_text(
            &self.config.tab_bar_style.window_maximize,
            CellAttributes::default(),
        );
        let window_maximize_hover = parse_status_text(
            &self.config.tab_bar_style.window_maximize_hover,
            CellAttributes::default(),
        );
        let window_close = parse_status_text(
            &self.config.tab_bar_style.window_close,
            CellAttributes::default(),
        );
        let window_close_hover = parse_status_text(
            &self.config.tab_bar_style.window_close_hover,
            CellAttributes::default(),
        );

        self.config
            .integrated_title_buttons
            .iter()
            .map(|button| match button {
                IntegratedTitleButton::Hide => window_hide.len().max(window_hide_hover.len()),
                IntegratedTitleButton::Maximize => {
                    window_maximize.len().max(window_maximize_hover.len())
                }
                IntegratedTitleButton::Close => window_close.len().max(window_close_hover.len()),
            })
            .collect()
    }

    fn style_attrs(options: &TabBarBuildOptions, attrs: CellAttributes) -> CellAttributes {
        if options.use_fancy_tab_bar {
            CellAttributes::default()
        } else {
            attrs
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TabBarBuildOptions {
    show_tabs: bool,
    show_new_tab_button: bool,
    use_fancy_tab_bar: bool,
    tab_max_width: usize,
    zero_based_indices: bool,
    tab_bar_at_bottom: bool,
    integrated_title_buttons_enabled: bool,
    integrated_title_button_style: IntegratedTitleButtonStyle,
    integrated_title_button_alignment: IntegratedTitleButtonAlignment,
}

impl TabBarBuildOptions {
    fn from_config(config: &ConfigHandle) -> Self {
        Self {
            show_tabs: config.show_tabs_in_tab_bar,
            show_new_tab_button: config.show_new_tab_button_in_tab_bar,
            use_fancy_tab_bar: config.use_fancy_tab_bar,
            tab_max_width: config.tab_max_width,
            zero_based_indices: config.tab_and_split_indices_are_zero_based,
            tab_bar_at_bottom: config.tab_bar_at_bottom,
            integrated_title_buttons_enabled: config
                .window_decorations
                .contains(window::WindowDecorations::INTEGRATED_BUTTONS),
            integrated_title_button_style: config.integrated_title_button_style,
            integrated_title_button_alignment: config.integrated_title_button_alignment,
        }
    }

    fn reserve_right_title_button_space(&self, content_mode: TabBarContentMode) -> bool {
        content_mode == TabBarContentMode::Full
            && self.integrated_title_buttons_enabled
            && self.integrated_title_button_style != IntegratedTitleButtonStyle::MacOsNative
            && self.integrated_title_button_alignment == IntegratedTitleButtonAlignment::Right
    }
}
