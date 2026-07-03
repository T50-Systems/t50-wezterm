impl CharSelector {
    pub fn new(_term_window: &mut TermWindow, args: &CharSelectArguments) -> Self {
        let aliases = build_aliases();
        let has_recents = aliases[0].group == CharSelectGroup::RecentlyUsed;
        let group = args.group.unwrap_or_else(|| {
            if has_recents {
                CharSelectGroup::RecentlyUsed
            } else {
                CharSelectGroup::default()
            }
        });

        Self {
            element: RefCell::new(None),
            selection: RefCell::new(String::new()),
            group: RefCell::new(group),
            aliases,
            matches: RefCell::new(None),
            selected_row: RefCell::new(0),
            top_row: RefCell::new(0),
            max_rows_on_screen: RefCell::new(0),
            copy_on_select: args.copy_on_select,
            copy_to: args.copy_to,
        }
    }

    fn compute(
        term_window: &mut TermWindow,
        selection: &str,
        group: CharSelectGroup,
        aliases: &[Alias],
        matches: &MatchResults,
        max_rows_on_screen: usize,
        selected_row: usize,
        top_row: usize,
    ) -> anyhow::Result<Vec<ComputedElement>> {
        let font = term_window
            .fonts
            .char_select_font()
            .expect("to resolve char selection font");
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());

        let top_bar_height = term_window.top_bar_pixel_height();
        let (padding_left, padding_top) = term_window.padding_left_top();
        let border = term_window.get_os_border();
        let top_pixel_y = top_bar_height + padding_top + border.top.get() as f32;

        let label = match group {
            CharSelectGroup::RecentlyUsed => "Recent",
            CharSelectGroup::SmileysAndEmotion => "Emotion",
            CharSelectGroup::PeopleAndBody => "People",
            CharSelectGroup::AnimalsAndNature => "Animals",
            CharSelectGroup::FoodAndDrink => "Food",
            CharSelectGroup::TravelAndPlaces => "Travel",
            CharSelectGroup::Activities => "Activities",
            CharSelectGroup::Objects => "Objects",
            CharSelectGroup::Symbols => "Symbols",
            CharSelectGroup::Flags => "Flags",
            CharSelectGroup::NerdFonts => "NerdFonts",
            CharSelectGroup::UnicodeNames => "Unicode",
            CharSelectGroup::ShortCodes => "Short Codes",
        };

        let mut elements = vec![Element::new(
            &font,
            ElementContent::Text(format!("{label}: {selection}_")),
        )
        .colors(ElementColors {
            border: BorderColor::default(),
            bg: LinearRgba::TRANSPARENT.into(),
            text: term_window.config.char_select_fg_color.to_linear().into(),
        })
        .display(DisplayType::Block)];

        for (display_idx, alias) in matches
            .matches
            .iter()
            .map(|&idx| &aliases[idx])
            .enumerate()
            .skip(top_row)
            .take(max_rows_on_screen)
        {
            let (bg, text) = if display_idx == selected_row {
                (
                    term_window.config.char_select_fg_color.to_linear().into(),
                    term_window.config.char_select_bg_color.to_linear().into(),
                )
            } else {
                (
                    LinearRgba::TRANSPARENT.into(),
                    term_window.config.char_select_fg_color.to_linear().into(),
                )
            };
            elements.push(
                Element::new(
                    &font,
                    ElementContent::Text(format!(
                        "{} {} ({})",
                        alias.glyph(),
                        alias.name(),
                        alias.codepoints()
                    )),
                )
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
                .display(DisplayType::Block),
            );
        }

        let element = Element::new(&font, ElementContent::Children(elements))
            .colors(ElementColors {
                border: BorderColor::new(
                    term_window.config.char_select_bg_color.to_linear().into(),
                ),
                bg: term_window.config.char_select_bg_color.to_linear().into(),
                text: term_window.config.char_select_fg_color.to_linear().into(),
            })
            .margin(BoxDimension {
                left: Dimension::Cells(1.25),
                right: Dimension::Cells(1.25),
                top: Dimension::Cells(1.25),
                bottom: Dimension::Cells(1.25),
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
            }));

        let dimensions = term_window.dimensions;
        let size = term_window.terminal_size;

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
                    padding_left,
                    top_pixel_y,
                    size.cols as f32 * term_window.render_metrics.cell_size.width as f32,
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

    fn do_move(&self, how: Move) {
        let page_size = *self.max_rows_on_screen.borrow();
        let current_row = *self.selected_row.borrow();
        let dest = match how {
            Move::Up(n) => current_row.saturating_sub(n),
            Move::PageUp => current_row.saturating_sub(page_size),
            Move::Down(n) => current_row.saturating_add(n),
            Move::PageDown => current_row.saturating_add(page_size),
        };
        *self.selected_row.borrow_mut() = dest;
        self.nav_selection();
    }

    /// handles selection constraints, moving list, keeping selection centered
    fn nav_selection(&self) {
        let max_rows_on_screen = *self.max_rows_on_screen.borrow();
        let limit = self
            .matches
            .borrow()
            .as_ref()
            .map(|m| m.matches.len())
            .unwrap_or_else(|| self.aliases.len());
        {
            let mut row = self.selected_row.borrow_mut();
            let mut top_row = self.top_row.borrow_mut();
            *row = row.min(limit.saturating_sub(1));
            if *row < *top_row {
                *top_row = *row;
            }
            if *row + *top_row > max_rows_on_screen / 2 {
                *top_row = row.saturating_sub(max_rows_on_screen / 2);
            }
        }
    }
}
