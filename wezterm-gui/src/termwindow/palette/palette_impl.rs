impl CommandPalette {
    pub fn new(term_window: &mut TermWindow) -> Self {
        // Showing the CopyMode actions in the palette is useless
        // if the CopyOverlay isn't active, so figure out if that
        // is the case so that we can filter them out in build_commands.
        let filter_copy_mode = term_window
            .get_active_pane_or_overlay()
            .map(|pane| {
                pane.downcast_ref::<crate::termwindow::CopyOverlay>()
                    .is_none()
            })
            .unwrap_or(true);

        let mux_pane = term_window
            .get_active_pane_or_overlay()
            .map(|pane| MuxPane(pane.pane_id()));

        let commands = build_commands(GuiWin::new(term_window), mux_pane, filter_copy_mode);

        Self {
            element: RefCell::new(None),
            selection: RefCell::new(String::new()),
            commands,
            matches: RefCell::new(None),
            selected_row: RefCell::new(0),
            top_row: RefCell::new(0),
            max_rows_on_screen: RefCell::new(0),
        }
    }

    fn compute(
        term_window: &mut TermWindow,
        selection: &str,
        commands: &[ExpandedCommand],
        matches: &MatchResults,
        max_rows_on_screen: usize,
        selected_row: usize,
        top_row: usize,
    ) -> anyhow::Result<Vec<ComputedElement>> {
        let font = term_window
            .fonts
            .command_palette_font()
            .expect("to resolve command palette font");
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());

        let top_bar_height = term_window.top_bar_pixel_height();
        let (padding_left, padding_top) = term_window.padding_left_top();
        let border = term_window.get_os_border();
        let top_pixel_y = top_bar_height + padding_top + border.top.get() as f32;

        let mut elements =
            vec![
                Element::new(&font, ElementContent::Text(format!("> {selection}_")))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg: LinearRgba::TRANSPARENT.into(),
                        text: term_window
                            .config
                            .command_palette_fg_color
                            .to_linear()
                            .into(),
                    })
                    .display(DisplayType::Block),
            ];

        for (display_idx, command) in matches
            .matches
            .iter()
            .map(|&idx| &commands[idx])
            .enumerate()
            .skip(top_row)
            .take(max_rows_on_screen)
        {
            let group = if command.menubar.is_empty() {
                String::new()
            } else {
                format!("{}: ", command.menubar.join(" | "))
            };

            let icon = match &command.icon {
                Some(nf) => NERD_FONTS.get(nf.as_ref()).unwrap_or_else(|| {
                    log::error!("nerdfont {nf} not found in NERD_FONTS");
                    &'?'
                }),
                None => &' ',
            };

            let solid_bg_color: InheritableColor = term_window
                .config
                .command_palette_bg_color
                .to_linear()
                .into();
            let solid_fg_color: InheritableColor = term_window
                .config
                .command_palette_fg_color
                .to_linear()
                .into();

            let (bg, text) = if display_idx == selected_row {
                (solid_fg_color.clone(), solid_bg_color.clone())
            } else {
                (LinearRgba::TRANSPARENT.into(), solid_fg_color.clone())
            };

            let (label_bg, label_text) = if display_idx == selected_row {
                (solid_fg_color.clone(), solid_bg_color.clone())
            } else {
                (solid_bg_color.clone(), solid_fg_color.clone())
            };

            // DRY if the brief and doc are the same
            let label = if command.doc.is_empty()
                || command.brief.to_ascii_lowercase() == command.doc.to_ascii_lowercase()
            {
                format!("{group}{}", command.brief)
            } else {
                format!("{group}{}. {}", command.brief, command.doc)
            };

            let mut row = vec![
                Element::new(&font, ElementContent::Text(icon.to_string()))
                    .min_width(Some(Dimension::Cells(2.))),
                Element::new(&font, ElementContent::Text(label)),
            ];

            if !command.keys.is_empty() {
                let mut keys = command.keys.clone();

                keys.sort_by(|(a_mods, a_key), (b_mods, b_key)| {
                    fn score_mods(mods: &Modifiers) -> usize {
                        let mut score: usize = mods.bits() as usize;
                        // Prefer keys with CMD on macOS, but not on other systems,
                        // where CMD tends to be reserved by the desktop environment
                        if cfg!(target_os = "macos") && mods.contains(Modifiers::SUPER) {
                            score += 1000;
                        } else if !cfg!(target_os = "macos") && !mods.contains(Modifiers::SUPER) {
                            score += 1000;
                        }
                        score
                    }

                    let a_mods = score_mods(a_mods);
                    let b_mods = score_mods(b_mods);

                    match b_mods.cmp(&a_mods) {
                        Ordering::Equal => {}
                        ordering => return ordering,
                    }

                    a_key.cmp(&b_key)
                });

                let separator = if term_window.config.ui_key_cap_rendering
                    == ::window::UIKeyCapRendering::AppleSymbols
                {
                    " "
                } else {
                    "-"
                };

                let mut keys = keys
                    .into_iter()
                    .map(|(mods, keycode)| {
                        let mut mod_string =
                            mods.to_string_with_separator(::window::ModifierToStringArgs {
                                separator,
                                want_none: false,
                                ui_key_cap_rendering: Some(term_window.config.ui_key_cap_rendering),
                            });
                        if !mod_string.is_empty() {
                            mod_string.push_str(separator);
                        }
                        let keycode = crate::inputmap::ui_key(
                            &keycode,
                            term_window.config.ui_key_cap_rendering,
                        );
                        format!("{mod_string}{keycode}")
                    })
                    .collect::<Vec<_>>();

                keys.dedup();
                keys.truncate(term_window.config.palette_max_key_assigments_for_action);

                let key_label = keys.join(", ");

                row.push(
                    Element::new(&font, ElementContent::Text(key_label))
                        .float(Float::Right)
                        .padding(BoxDimension {
                            left: Dimension::Cells(1.25),
                            right: Dimension::Cells(0.5),
                            top: Dimension::Cells(0.),
                            bottom: Dimension::Cells(0.),
                        })
                        .zindex(10)
                        .colors(ElementColors {
                            border: BorderColor::default(),
                            bg: label_bg.clone(),
                            text: label_text.clone(),
                        }),
                );
            }

            elements.push(
                Element::new(&font, ElementContent::Children(row))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg,
                        text,
                    })
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.25),
                        right: Dimension::Cells(0.25),
                        top: Dimension::Cells(0.),
                        bottom: Dimension::Cells(0.),
                    })
                    .min_width(Some(Dimension::Percent(1.)))
                    .display(DisplayType::Block),
            );
        }

        let dimensions = term_window.dimensions;
        let size = term_window.terminal_size;

        // Avoid covering the entire width
        let desired_width = (size.cols / 3).max(120).min(size.cols);

        // Center it
        let avail_pixel_width =
            size.cols as f32 * term_window.render_metrics.cell_size.width as f32;
        let desired_pixel_width =
            desired_width as f32 * term_window.render_metrics.cell_size.width as f32;

        let element = Element::new(&font, ElementContent::Children(elements))
            .colors(ElementColors {
                border: BorderColor::new(
                    term_window
                        .config
                        .command_palette_bg_color
                        .to_linear()
                        .into(),
                ),
                bg: term_window
                    .config
                    .command_palette_bg_color
                    .to_linear()
                    .into(),
                text: term_window
                    .config
                    .command_palette_fg_color
                    .to_linear()
                    .into(),
            })
            .margin(BoxDimension {
                left: Dimension::Cells(0.25),
                right: Dimension::Cells(0.25),
                top: Dimension::Cells(0.25),
                bottom: Dimension::Cells(0.25),
            })
            .padding(BoxDimension {
                left: Dimension::Cells(0.25),
                right: Dimension::Cells(0.25),
                top: Dimension::Cells(0.25),
                bottom: Dimension::Cells(0.25),
            })
            .border(BoxDimension::new(Dimension::Pixels(1.)))
            .border_corners(Some(Corners {
                top_left: SizedPoly {
                    width: Dimension::Cells(0.25),
                    height: Dimension::Cells(0.25),
                    poly: TOP_LEFT_ROUNDED_CORNER,
                },
                top_right: SizedPoly {
                    width: Dimension::Cells(0.25),
                    height: Dimension::Cells(0.25),
                    poly: TOP_RIGHT_ROUNDED_CORNER,
                },
                bottom_left: SizedPoly {
                    width: Dimension::Cells(0.25),
                    height: Dimension::Cells(0.25),
                    poly: BOTTOM_LEFT_ROUNDED_CORNER,
                },
                bottom_right: SizedPoly {
                    width: Dimension::Cells(0.25),
                    height: Dimension::Cells(0.25),
                    poly: BOTTOM_RIGHT_ROUNDED_CORNER,
                },
            }))
            .min_width(Some(Dimension::Pixels(desired_pixel_width)));

        let x_adjust = ((avail_pixel_width - padding_left) - desired_pixel_width) / 2.;

        let computed = term_window.compute_element(
            &LayoutContext {
                height: DimensionContext {
                    dpi: dimensions.dpi as f32,
                    pixel_max: dimensions.pixel_height as f32,
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: DimensionContext {
                    dpi: dimensions.dpi as f32,
                    pixel_max: dimensions.pixel_width as f32,
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds: euclid::rect(
                    padding_left + x_adjust,
                    top_pixel_y,
                    desired_pixel_width,
                    size.rows as f32 * term_window.render_metrics.cell_size.height as f32,
                ),
                metrics: &metrics,
                gl_state: term_window.render_state.as_ref().unwrap(),
                zindex: 100,
            },
            &element,
        )?;

        Ok(vec![computed])
    }

    fn updated_input(&self) {
        *self.selected_row.borrow_mut() = 0;
        *self.top_row.borrow_mut() = 0;
    }

    fn move_up(&self) {
        let mut row = self.selected_row.borrow_mut();
        *row = row.saturating_sub(1);

        let mut top_row = self.top_row.borrow_mut();
        if *row < *top_row {
            *top_row = *row;
        }
    }

    fn move_down(&self) {
        let max_rows_on_screen = *self.max_rows_on_screen.borrow();
        let limit = self
            .matches
            .borrow()
            .as_ref()
            .map(|m| m.matches.len())
            .unwrap_or_else(|| self.commands.len())
            .saturating_sub(1);
        let mut row = self.selected_row.borrow_mut();
        *row = row.saturating_add(1).min(limit);
        let mut top_row = self.top_row.borrow_mut();
        if *row > *top_row + max_rows_on_screen - 1 {
            *top_row = row.saturating_sub(max_rows_on_screen - 1);
        }
    }
}
