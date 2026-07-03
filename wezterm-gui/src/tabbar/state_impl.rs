impl TabBarState {
    pub fn default() -> Self {
        Self {
            line: Line::with_width(1, SEQ_ZERO),
            items: vec![TabEntry {
                item: TabBarItem::None,
                title: Line::from_text(" ", &CellAttributes::blank(), 1, None),
                x: 1,
                width: 1,
            }],
        }
    }

    pub fn line(&self) -> &Line {
        &self.line
    }

    pub fn items(&self) -> &[TabEntry] {
        &self.items
    }

    fn integrated_title_buttons(
        mouse_x: Option<usize>,
        x: &mut usize,
        config: &ConfigHandle,
        items: &mut Vec<TabEntry>,
        line: &mut Line,
        colors: &TabBarColors,
    ) {
        let default_cell = if config.use_fancy_tab_bar {
            CellAttributes::default()
        } else {
            colors.new_tab().as_cell_attributes()
        };

        let default_cell_hover = if config.use_fancy_tab_bar {
            CellAttributes::default()
        } else {
            colors.new_tab_hover().as_cell_attributes()
        };

        let window_hide =
            parse_status_text(&config.tab_bar_style.window_hide, default_cell.clone());
        let window_hide_hover = parse_status_text(
            &config.tab_bar_style.window_hide_hover,
            default_cell_hover.clone(),
        );

        let window_maximize =
            parse_status_text(&config.tab_bar_style.window_maximize, default_cell.clone());
        let window_maximize_hover = parse_status_text(
            &config.tab_bar_style.window_maximize_hover,
            default_cell_hover.clone(),
        );

        let window_close =
            parse_status_text(&config.tab_bar_style.window_close, default_cell.clone());
        let window_close_hover = parse_status_text(
            &config.tab_bar_style.window_close_hover,
            default_cell_hover.clone(),
        );

        for button in &config.integrated_title_buttons {
            use IntegratedTitleButton as Button;
            let title = match button {
                Button::Hide => {
                    let hover = is_tab_hover(mouse_x, *x, window_hide_hover.len());

                    if hover {
                        &window_hide_hover
                    } else {
                        &window_hide
                    }
                }
                Button::Maximize => {
                    let hover = is_tab_hover(mouse_x, *x, window_maximize_hover.len());

                    if hover {
                        &window_maximize_hover
                    } else {
                        &window_maximize
                    }
                }
                Button::Close => {
                    let hover = is_tab_hover(mouse_x, *x, window_close_hover.len());

                    if hover {
                        &window_close_hover
                    } else {
                        &window_close
                    }
                }
            };

            line.append_line(title.to_owned(), SEQ_ZERO);

            let width = title.len();
            items.push(TabEntry {
                item: TabBarItem::WindowButton(*button),
                title: title.to_owned(),
                x: *x,
                width,
            });

            *x += width;
        }
    }

    fn append_center_pane_status(
        x: &mut usize,
        pane_info: &[PaneInformation],
        items: &mut Vec<TabEntry>,
        line: &mut Line,
        default_attrs: &CellAttributes,
        zero_based: bool,
    ) {
        if pane_info.len() <= 1 {
            return;
        }

        for (idx, pane) in pane_info.iter().enumerate() {
            let formatter = PaneLabelFormatter::new(zero_based);
            let label = formatter.label_for(pane.pane_index, &pane.title);
            let pane_line = parse_status_text(&label, default_attrs.clone());
            let width = pane_line.len();
            items.push(TabEntry {
                item: TabBarItem::PaneStatus {
                    pane_id: pane.pane_id,
                    active: pane.is_active,
                },
                title: pane_line.clone(),
                x: *x,
                width,
            });
            line.append_line(pane_line, SEQ_ZERO);
            *x += width;

            if idx + 1 < pane_info.len() {
                let sep = parse_status_text(" ", default_attrs.clone());
                line.append_line(sep, SEQ_ZERO);
                *x += 1;
            }
        }
    }

    /// Build a new tab bar from the current state
    /// mouse_x is some if the mouse is on the same row as the tab bar.
    /// title_width is the total number of cell columns in the window.
    /// window allows access to the tabs associated with the window.
    pub fn new_primary(
        title_width: usize,
        mouse_x: Option<usize>,
        tab_info: &[TabInformation],
        pane_info: &[PaneInformation],
        colors: Option<&TabBarColors>,
        config: &ConfigHandle,
        left_status: &str,
        right_status: &str,
    ) -> Self {
        Self::build(
            title_width,
            mouse_x,
            tab_info,
            pane_info,
            colors,
            config,
            left_status,
            "",
            right_status,
            TabBarContentMode::Full,
        )
    }

    pub fn new_status_bar(
        title_width: usize,
        tab_info: &[TabInformation],
        pane_info: &[PaneInformation],
        colors: Option<&TabBarColors>,
        config: &ConfigHandle,
        left_status: &str,
        center_status: &str,
        right_status: &str,
    ) -> Self {
        Self::build(
            title_width,
            None,
            tab_info,
            pane_info,
            colors,
            config,
            left_status,
            center_status,
            right_status,
            TabBarContentMode::StatusOnly,
        )
    }

    fn build(
        title_width: usize,
        mouse_x: Option<usize>,
        tab_info: &[TabInformation],
        pane_info: &[PaneInformation],
        colors: Option<&TabBarColors>,
        config: &ConfigHandle,
        left_status: &str,
        center_status: &str,
        right_status: &str,
        content_mode: TabBarContentMode,
    ) -> Self {
        TabBarBuilder::new(
            title_width,
            mouse_x,
            tab_info,
            pane_info,
            colors,
            config,
            left_status,
            center_status,
            right_status,
            content_mode,
        )
        .build()
    }

    pub fn compute_ui_items(&self, y: usize, cell_height: usize, cell_width: usize) -> Vec<UIItem> {
        let mut items = vec![];

        for entry in self.items.iter() {
            items.push(UIItem {
                x: entry.x * cell_width,
                width: entry.width * cell_width,
                y,
                height: cell_height,
                item_type: UIItemType::TabBar(entry.item),
            });
        }

        items
    }
}
