struct TabBarBuilder<'a> {
    title_width: usize,
    mouse_x: Option<usize>,
    tab_info: &'a [TabInformation],
    pane_info: &'a [PaneInformation],
    config: &'a ConfigHandle,
    options: TabBarBuildOptions,
    left_status: &'a str,
    center_status: &'a str,
    right_status: &'a str,
    content_mode: TabBarContentMode,
    colors: TabBarColors,
    active_cell_attrs: CellAttributes,
    inactive_hover_attrs: CellAttributes,
    inactive_cell_attrs: CellAttributes,
    new_tab: Line,
    new_tab_hover: Line,
    active_tab_no: usize,
    tab_titles: Vec<TitleText>,
    tab_width_max: usize,
    status_renderer: StatusTextRenderer,
    line: Line,
    x: usize,
    items: Vec<TabEntry>,
    deferred_center_status: Option<String>,
}

impl<'a> TabBarBuilder<'a> {
    fn new(
        title_width: usize,
        mouse_x: Option<usize>,
        tab_info: &'a [TabInformation],
        pane_info: &'a [PaneInformation],
        colors: Option<&TabBarColors>,
        config: &'a ConfigHandle,
        left_status: &'a str,
        center_status: &'a str,
        right_status: &'a str,
        content_mode: TabBarContentMode,
    ) -> Self {
        let options = TabBarBuildOptions::from_config(config);
        let colors = colors.cloned().unwrap_or_else(TabBarColors::default);
        let active_cell_attrs = colors.active_tab().as_cell_attributes();
        let inactive_hover_attrs = colors.inactive_tab_hover().as_cell_attributes();
        let inactive_cell_attrs = colors.inactive_tab().as_cell_attributes();
        let new_tab_hover_attrs = colors.new_tab_hover().as_cell_attributes();
        let new_tab_attrs = colors.new_tab().as_cell_attributes();

        let style = TabBarStyleAdapter::new(config);
        let new_tab = style.new_tab_line(&options, new_tab_attrs);
        let new_tab_hover = style.new_tab_hover_line(&options, new_tab_hover_attrs);

        let mut active_tab_no = 0;
        let tab_titles = Self::collect_tab_titles(
            tab_info,
            pane_info,
            config,
            &options,
            content_mode,
            &mut active_tab_no,
        );
        let tab_width_max =
            Self::tab_width_max(title_width, &tab_titles, &new_tab, &options, content_mode);
        let status_renderer = StatusTextRenderer::new(colors.background());
        Self {
            title_width,
            mouse_x,
            tab_info,
            pane_info,
            config,
            options,
            left_status,
            center_status,
            right_status,
            content_mode,
            colors,
            active_cell_attrs,
            inactive_hover_attrs,
            inactive_cell_attrs,
            new_tab,
            new_tab_hover,
            active_tab_no,
            tab_titles,
            tab_width_max,
            status_renderer,
            line: Line::with_width(0, SEQ_ZERO),
            x: 0,
            items: vec![],
            deferred_center_status: None,
        }
    }

    fn build(mut self) -> TabBarState {
        self.reserve_native_left_title_button_space();
        self.append_left_integrated_title_buttons();
        self.append_left_status();
        self.append_center_status_or_defer();
        self.append_tabs();
        self.append_full_mode_content();

        let title_width = self.title_width_without_right_title_buttons();
        self.append_deferred_center_and_right_status(title_width);
        self.pad_to_title_width(title_width);
        self.append_right_integrated_title_buttons(title_width);

        TabBarState {
            line: self.line,
            items: self.items,
        }
    }

    fn is_full(&self) -> bool {
        self.content_mode == TabBarContentMode::Full
    }

    fn is_status_only(&self) -> bool {
        self.content_mode == TabBarContentMode::StatusOnly
    }

    fn collect_tab_titles(
        tab_info: &[TabInformation],
        pane_info: &[PaneInformation],
        config: &ConfigHandle,
        options: &TabBarBuildOptions,
        content_mode: TabBarContentMode,
        active_tab_no: &mut usize,
    ) -> Vec<TitleText> {
        if content_mode != TabBarContentMode::Full || !options.show_tabs {
            return vec![];
        }

        tab_info
            .iter()
            .map(|tab| {
                if tab.is_active {
                    *active_tab_no = tab.tab_index;
                }
                compute_tab_title(
                    tab,
                    tab_info,
                    pane_info,
                    config,
                    false,
                    options.tab_max_width,
                )
            })
            .collect()
    }

    fn tab_width_max(
        title_width: usize,
        tab_titles: &[TitleText],
        new_tab: &Line,
        options: &TabBarBuildOptions,
        content_mode: TabBarContentMode,
    ) -> usize {
        TabWidthPolicy {
            title_width,
            titles_len: tab_titles.iter().map(|s| s.len).sum(),
            number_of_tabs: tab_titles.len(),
            new_tab_len: if content_mode == TabBarContentMode::Full && options.show_new_tab_button {
                new_tab.len()
            } else {
                0
            },
            use_fancy_tab_bar: options.use_fancy_tab_bar,
            tab_max_width: options.tab_max_width,
        }
        .max_width()
    }

    fn reserve_native_left_title_button_space(&mut self) {
        if self.is_full()
            && self.options.integrated_title_buttons_enabled
            && self.options.integrated_title_button_style == IntegratedTitleButtonStyle::MacOsNative
            && !self.options.use_fancy_tab_bar
            && !self.options.tab_bar_at_bottom
        {
            for _ in 0..10 as usize {
                self.line
                    .insert_cell(0, self.status_renderer.filler(), self.title_width, SEQ_ZERO);
                self.x += 1;
            }
        }
    }

    fn append_left_integrated_title_buttons(&mut self) {
        if self.is_full()
            && self.options.integrated_title_buttons_enabled
            && self.options.integrated_title_button_style != IntegratedTitleButtonStyle::MacOsNative
            && self.options.integrated_title_button_alignment
                == IntegratedTitleButtonAlignment::Left
        {
            TabBarState::integrated_title_buttons(
                self.mouse_x,
                &mut self.x,
                self.config,
                &mut self.items,
                &mut self.line,
                &self.colors,
            );
        }
    }

    fn append_left_status(&mut self) {
        let left_status_line = self.status_renderer.parse(self.left_status);
        if left_status_line.len() > 0 {
            self.items.push(TabEntry {
                item: TabBarItem::LeftStatus,
                title: left_status_line.clone(),
                x: self.x,
                width: left_status_line.len(),
            });
            self.x += left_status_line.len();
            self.line.append_line(left_status_line, SEQ_ZERO);
        }
    }

    fn append_center_status_or_defer(&mut self) {
        if self.center_status.is_empty() {
            return;
        }

        if self.is_status_only() {
            self.deferred_center_status = Some(self.center_status.to_string());
            return;
        }

        let center_status_line = self.status_renderer.parse(self.center_status);
        if center_status_line.len() > 0 {
            self.items.push(TabEntry {
                item: TabBarItem::CenterStatus,
                title: center_status_line.clone(),
                x: self.x,
                width: center_status_line.len(),
            });
            self.x += center_status_line.len();
            self.line.append_line(center_status_line, SEQ_ZERO);
        }
    }

    fn append_tabs(&mut self) {
        for tab_idx in 0..self.tab_titles.len() {
            let tab_title_len = self.tab_titles[tab_idx].len.min(self.tab_width_max);
            let active = tab_idx == self.active_tab_no;
            let hover = !active && is_tab_hover(self.mouse_x, self.x, tab_title_len);

            let tab_title = compute_tab_title(
                &self.tab_info[tab_idx],
                self.tab_info,
                self.pane_info,
                self.config,
                hover,
                tab_title_len,
            );

            let cell_attrs = if active {
                &self.active_cell_attrs
            } else if hover {
                &self.inactive_hover_attrs
            } else {
                &self.inactive_cell_attrs
            };

            let tab_start_idx = self.x;
            let esc = format_as_escapes(tab_title.items.clone()).expect("already parsed ok above");
            let mut tab_line = parse_status_text(
                &esc,
                if self.options.use_fancy_tab_bar {
                    CellAttributes::default()
                } else {
                    cell_attrs.clone()
                },
            );

            let title = tab_line.clone();
            if tab_line.len() > self.tab_width_max {
                tab_line.resize(self.tab_width_max, SEQ_ZERO);
            }
            let width = tab_line.len();

            self.items.push(TabEntry {
                item: TabBarItem::Tab { tab_idx, active },
                title,
                x: tab_start_idx,
                width,
            });
            self.line.append_line(tab_line, SEQ_ZERO);
            self.x += width;
        }
    }

    fn append_full_mode_content(&mut self) {
        if !self.is_full() {
            return;
        }

        TabBarState::append_center_pane_status(
            &mut self.x,
            self.pane_info,
            &mut self.items,
            &mut self.line,
            self.status_renderer.attrs(),
            self.options.zero_based_indices,
        );
        self.append_new_tab_button();
    }

    fn append_new_tab_button(&mut self) {
        if !self.options.show_new_tab_button {
            return;
        }

        let hover = is_tab_hover(self.mouse_x, self.x, self.new_tab_hover.len());
        let new_tab_button = if hover {
            &self.new_tab_hover
        } else {
            &self.new_tab
        };
        let button_start = self.x;
        let width = new_tab_button.len();

        self.line.append_line(new_tab_button.clone(), SEQ_ZERO);
        self.items.push(TabEntry {
            item: TabBarItem::NewTabButton,
            title: new_tab_button.clone(),
            x: button_start,
            width,
        });
        self.x += width;
    }

    fn title_width_without_right_title_buttons(&self) -> usize {
        let button_widths = TabBarStyleAdapter::new(self.config).integrated_title_button_widths();

        IntegratedTitleButtonReservation {
            title_width: self.title_width,
            reserve: self
                .options
                .reserve_right_title_button_space(self.content_mode),
        }
        .title_width_after_reservation(&button_widths)
    }

    fn append_deferred_center_and_right_status(&mut self, title_width: usize) {
        if let Some(center_status) = self.deferred_center_status.take() {
            self.append_deferred_center_status_and_right_status(title_width, center_status);
        } else {
            self.append_right_status(title_width);
        }
    }

    fn append_deferred_center_status_and_right_status(
        &mut self,
        title_width: usize,
        center_status: String,
    ) {
        let mut right_status_line = self.status_renderer.parse(self.right_status);
        let plan = StatusLineLayout {
            title_width,
            current_x: self.x,
        }
        .center_and_right_plan(right_status_line.len());

        let mut center_status_line = self.status_renderer.parse(&center_status);
        self.status_renderer
            .fit_to_width(&mut center_status_line, plan.center_width);

        if center_status_line.len() > 0 {
            self.items.push(TabEntry {
                item: TabBarItem::CenterStatus,
                title: center_status_line.clone(),
                x: self.x,
                width: center_status_line.len(),
            });
            self.x += center_status_line.len();
            self.line.append_line(center_status_line, SEQ_ZERO);
        }

        self.items.push(TabEntry {
            item: TabBarItem::RightStatus,
            title: right_status_line.clone(),
            x: self.x,
            width: plan.right_width,
        });
        self.status_renderer
            .trim_left(&mut right_status_line, plan.right_trim_left);
        self.line.append_line(right_status_line, SEQ_ZERO);
    }

    fn append_right_status(&mut self, title_width: usize) {
        let mut right_status_line = self.status_renderer.parse(self.right_status);
        let plan = StatusLineLayout {
            title_width,
            current_x: self.x,
        }
        .right_only_plan(right_status_line.len());
        self.items.push(TabEntry {
            item: TabBarItem::RightStatus,
            title: right_status_line.clone(),
            x: self.x,
            width: plan.right_width,
        });

        self.status_renderer
            .trim_left(&mut right_status_line, plan.right_trim_left);
        self.line.append_line(right_status_line, SEQ_ZERO);
    }

    fn pad_to_title_width(&mut self, title_width: usize) {
        while self.line.len() < title_width {
            self.line
                .insert_cell(self.x, self.status_renderer.filler(), title_width, SEQ_ZERO);
        }
    }

    fn append_right_integrated_title_buttons(&mut self, title_width: usize) {
        if self.is_full()
            && self.options.integrated_title_buttons_enabled
            && self.options.integrated_title_button_style != IntegratedTitleButtonStyle::MacOsNative
            && self.options.integrated_title_button_alignment
                == IntegratedTitleButtonAlignment::Right
        {
            self.x = title_width;
            TabBarState::integrated_title_buttons(
                self.mouse_x,
                &mut self.x,
                self.config,
                &mut self.items,
                &mut self.line,
                &self.colors,
            );
        }
    }
}
