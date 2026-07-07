impl FontConfigInner {
    /// Create a new empty configuration
    pub fn new(config: Option<ConfigHandle>, dpi: usize) -> anyhow::Result<Self> {
        let config = config.unwrap_or_else(configuration);
        let locator = new_locator(config.font_locator);
        Ok(Self {
            fonts: RefCell::new(HashMap::new()),
            locator,
            metrics: RefCell::new(None),
            title_font: RefCell::new(None),
            pane_select_font: RefCell::new(None),
            char_select_font: RefCell::new(None),
            command_palette_font: RefCell::new(None),
            font_scale: RefCell::new(1.0),
            dpi: RefCell::new(dpi),
            config: RefCell::new(config.clone()),
            font_dirs: RefCell::new(Arc::new(FontDatabase::with_font_dirs(&config)?)),
            built_in: RefCell::new(Arc::new(FontDatabase::with_built_in()?)),
            fallback_channel: RefCell::new(None),
        })
    }

    fn config_changed(&self, config: &ConfigHandle) -> anyhow::Result<()> {
        let mut fonts = self.fonts.borrow_mut();
        *self.config.borrow_mut() = config.clone();
        // Config was reloaded, invalidate our caches
        fonts.clear();
        self.title_font.borrow_mut().take();
        self.pane_select_font.borrow_mut().take();
        self.char_select_font.borrow_mut().take();
        self.command_palette_font.borrow_mut().take();
        self.metrics.borrow_mut().take();
        *self.font_dirs.borrow_mut() = Arc::new(FontDatabase::with_font_dirs(config)?);
        Ok(())
    }

    fn schedule_fallback_resolve<F: FnOnce() + Send + 'static>(
        &self,
        no_glyphs: Vec<char>,
        pending: &Arc<Mutex<Vec<ParsedFont>>>,
        completion: F,
    ) {
        if no_glyphs.is_empty() {
            return;
        }

        let info = FallbackResolveInfo {
            completion: Box::new(completion),
            no_glyphs,
            pending: Arc::clone(pending),
            font_dirs: Arc::clone(&*self.font_dirs.borrow()),
            built_in: Arc::clone(&*self.built_in.borrow()),
            locator: Arc::clone(&self.locator),
            config: self.config.borrow().clone(),
        };

        let mut fallback = self.fallback_channel.borrow_mut();

        if fallback.is_none() {
            let (tx, rx) = channel::<FallbackResolveInfo>();

            std::thread::spawn(move || {
                for info in rx {
                    info.process();
                }
            });

            fallback.replace(tx);
        }

        if let Err(err) = fallback.as_mut().expect("channel to exist").send(info) {
            log::error!("Failed to schedule font fallback resolve: {:#}", err);
        }
    }

    fn compute_title_font(&self, config: &ConfigHandle, make_bold: bool) -> (TextStyle, f64) {
        fn bold(family: &str) -> FontAttributes {
            FontAttributes {
                family: family.to_string(),
                weight: FontWeight::BOLD,
                ..Default::default()
            }
        }

        let mut fonts = vec![if make_bold {
            bold("Roboto")
        } else {
            FontAttributes::new("Roboto")
        }];

        // Fallback to their main font selection, so that we can pick up
        // any fallback fonts they might have configured in the main
        // config and so that they don't have to replicate that list for
        // the title font.
        for font in &config.font.font {
            let mut font = font.clone();
            font.is_fallback = true;
            fonts.push(font);
        }

        let font_size = if cfg!(windows) { 10. } else { 12. };

        (
            TextStyle {
                foreground: None,
                font: fonts,
            },
            font_size,
        )
    }

    fn make_entity_font_impl(
        &self,
        myself: &Rc<Self>,
        entity: Entity,
    ) -> anyhow::Result<Rc<LoadedFont>> {
        let config = self.config.borrow();
        let make_bold = entity != Entity::CommandPalette;
        let (sys_font, sys_size) = self.compute_title_font(&config, make_bold);

        let (font_size, text_style) = match entity {
            Entity::Title => (config.window_frame.font_size.unwrap_or(sys_size), None),
            Entity::CommandPalette => (
                config.command_palette_font_size,
                config.command_palette_font.as_ref(),
            ),
            Entity::CharSelect => (
                config.char_select_font_size,
                config.char_select_font.as_ref(),
            ),
            Entity::PaneSelect => (
                config.pane_select_font_size,
                config.pane_select_font.as_ref(),
            ),
        };

        let text_style =
            text_style.unwrap_or(config.window_frame.font.as_ref().unwrap_or(&sys_font));

        let dpi = *self.dpi.borrow() as u32;
        let pixel_size = (font_size * dpi as f64 / 72.0) as u16;

        let attributes = text_style.font_with_fallback();
        let (handles, _loaded) = self.resolve_font_helper_impl(&attributes, pixel_size)?;

        let shaper = new_shaper(&*config, &handles)?;

        let metrics = shaper.metrics(font_size, dpi).with_context(|| {
            format!(
                "obtaining metrics for font_size={} @ dpi {}",
                font_size, dpi
            )
        })?;

        let loaded = Rc::new(LoadedFont {
            rasterizers: RefCell::new(HashMap::new()),
            handles: RefCell::new(handles),
            shaper: RefCell::new(shaper),
            metrics,
            font_size,
            dpi,
            font_config: Rc::downgrade(myself),
            pending_fallback: Arc::new(Mutex::new(vec![])),
            text_style: text_style.clone(),
            id: alloc_font_id(),
            tried_glyphs: RefCell::new(HashSet::new()),
            pixel_geometry: config.display_pixel_geometry,
        });

        Ok(loaded)
    }

    fn title_font(&self, myself: &Rc<Self>) -> anyhow::Result<Rc<LoadedFont>> {
        let mut title_font = self.title_font.borrow_mut();

        if let Some(entry) = title_font.as_ref() {
            return Ok(Rc::clone(entry));
        }

        let loaded = self.make_entity_font_impl(myself, Entity::Title)?;

        title_font.replace(Rc::clone(&loaded));

        Ok(loaded)
    }

    fn command_palette_font(&self, myself: &Rc<Self>) -> anyhow::Result<Rc<LoadedFont>> {
        let mut command_palette_font = self.command_palette_font.borrow_mut();

        if let Some(entry) = command_palette_font.as_ref() {
            return Ok(Rc::clone(entry));
        }

        let loaded = self.make_entity_font_impl(myself, Entity::CommandPalette)?;

        command_palette_font.replace(Rc::clone(&loaded));

        Ok(loaded)
    }

    fn char_select_font(&self, myself: &Rc<Self>) -> anyhow::Result<Rc<LoadedFont>> {
        let mut char_select_font = self.char_select_font.borrow_mut();

        if let Some(entry) = char_select_font.as_ref() {
            return Ok(Rc::clone(entry));
        }

        let loaded = self.make_entity_font_impl(myself, Entity::CharSelect)?;

        char_select_font.replace(Rc::clone(&loaded));

        Ok(loaded)
    }

    fn pane_select_font(&self, myself: &Rc<Self>) -> anyhow::Result<Rc<LoadedFont>> {
        let mut pane_select_font = self.pane_select_font.borrow_mut();

        if let Some(entry) = pane_select_font.as_ref() {
            return Ok(Rc::clone(entry));
        }

        let loaded = self.make_entity_font_impl(myself, Entity::PaneSelect)?;

        pane_select_font.replace(Rc::clone(&loaded));

        Ok(loaded)
    }

    fn resolve_font_helper_impl(
        &self,
        attributes: &[FontAttributes],
        pixel_size: u16,
    ) -> anyhow::Result<(Vec<ParsedFont>, HashSet<FontAttributes>)> {
        let preferred_attributes = attributes
            .iter()
            .filter(|a| !a.is_fallback)
            .cloned()
            .collect::<Vec<_>>();
        let fallback_attributes = attributes
            .iter()
            .filter(|a| a.is_fallback)
            .cloned()
            .collect::<Vec<_>>();
        let mut loaded = HashSet::new();
        let mut handles = vec![];

        for &attrs in &[&preferred_attributes, &fallback_attributes] {
            let mut candidates = vec![];

            let font_dirs = self.font_dirs.borrow();
            for attr in attrs {
                candidates.append(&mut font_dirs.candidates(attr));
            }

            let mut loaded_ignored = HashSet::new();
            let located = self
                .locator
                .load_fonts(attrs, &mut loaded_ignored, pixel_size)?;
            for font in &located {
                candidates.push(font);
            }

            let built_in = self.built_in.borrow();
            for attr in attrs {
                candidates.append(&mut built_in.candidates(attr));
            }

            let mut is_fallback = false;

            for attr in attrs {
                if attr.is_fallback {
                    is_fallback = true;
                }

                if loaded.contains(attr) {
                    continue;
                }
                let named_candidates: Vec<&ParsedFont> = candidates
                    .iter()
                    .filter_map(|&p| if p.matches_name(attr) { Some(p) } else { None })
                    .collect();
                if let Some(idx) =
                    ParsedFont::best_matching_index(attr, &named_candidates, pixel_size)
                {
                    named_candidates.get(idx).map(|&p| {
                        loaded.insert(attr.clone());
                        handles.push(p.clone().synthesize(attr))
                    });
                }
            }

            if !is_fallback && loaded.is_empty() {
                // We didn't explicitly match any names.
                // When using fontconfig, the system may have expanded a family name
                // like "monospace" into the real font, in which case we wouldn't have
                // found a match in the `named_candidates` vec above, because of the
                // name mismatch.
                // So what we do now is make a second pass over all the located candidates,
                // ignoring their names, and just match based on font attributes.
                let located_candidates: Vec<_> = located.iter().collect();
                for attr in attrs {
                    if let Some(idx) =
                        ParsedFont::best_matching_index(attr, &located_candidates, pixel_size)
                    {
                        located_candidates.get(idx).map(|&p| {
                            loaded.insert(attr.clone());
                            handles.push(p.clone().synthesize(attr))
                        });
                    }
                }
            }
        }

        Ok((handles, loaded))
    }
}
