impl FontConfigInner {
    fn resolve_font_helper(
        &self,
        style: &TextStyle,
        config: &ConfigHandle,
        pixel_size: u16,
    ) -> anyhow::Result<(Box<dyn FontShaper>, Vec<ParsedFont>)> {
        let attributes = style.font_with_fallback();

        let (handles, loaded) = self.resolve_font_helper_impl(&attributes, pixel_size)?;

        for attr in &attributes {
            if !attr.is_synthetic && !attr.is_fallback && !loaded.contains(attr) {
                let styled_extra = if attr.weight != FontWeight::default()
                    || attr.style != FontStyle::default()
                    || attr.stretch != FontStretch::default()
                {
                    ". An alternative variant of the font was requested; \
                    TrueType and OpenType fonts don't have an automatic way to \
                    produce these font variants, so a separate font file containing \
                    the bold or italic variant must be installed"
                } else {
                    ""
                };

                let is_primary = config.font.font.iter().any(|a| a == attr);
                let derived_from_primary = config.font.font.iter().any(|a| a.family == attr.family);

                let explanation = if is_primary {
                    // This is the primary font selection
                    format!(
                        "Unable to load a font specified by your font={} configuration",
                        attr
                    )
                } else if derived_from_primary {
                    // it came from font_rules and may have been derived from
                    // their primary font (we can't know for sure)
                    format!(
                        "Unable to load a font matching one of your font_rules: {}. \
                        Note that wezterm will synthesize font_rules to select bold \
                        and italic fonts based on your primary font configuration",
                        attr
                    )
                } else {
                    format!(
                        "Unable to load a font matching one of your font_rules: {}",
                        attr
                    )
                };

                config::show_error(&format!(
                    "{}. Fallback(s) are being used instead, and the terminal \
                    may not render as intended{}. See \
                    https://wezterm.org/config/fonts.html for more information",
                    explanation, styled_extra
                ));
            }
        }

        Ok((new_shaper(&*config, &handles)?, handles))
    }

    /// Given a text style, load (with caching) the font that best
    /// matches according to the fontconfig pattern.
    fn resolve_font(&self, myself: &Rc<Self>, style: &TextStyle) -> anyhow::Result<Rc<LoadedFont>> {
        let config = self.config.borrow();
        let is_default = *style == config.font;
        let def_font = if !is_default && config.use_cap_height_to_scale_fallback_fonts {
            Some(self.default_font(myself)?)
        } else {
            None
        };

        let mut fonts = self.fonts.borrow_mut();

        if let Some(entry) = fonts.get(style) {
            return Ok(Rc::clone(entry));
        }

        let mut font_size = config.font_size * *self.font_scale.borrow();
        let dpi = *self.dpi.borrow() as u32;
        let pixel_size = (font_size * dpi as f64 / 72.0) as u16;

        let (mut shaper, mut handles) = self.resolve_font_helper(style, &config, pixel_size)?;

        let mut metrics = shaper.metrics(font_size, dpi).with_context(|| {
            format!(
                "obtaining metrics for font_size={} @ dpi {}",
                font_size, dpi
            )
        })?;

        if let Some(def_font) = def_font {
            let def_metrics = def_font.metrics();
            match (def_metrics.cap_height, metrics.cap_height) {
                (Some(d), Some(m)) => {
                    // Scale by the ratio of the pixel heights of the default
                    // and this font; this causes the `I` glyphs to appear to
                    // have the same height.
                    let scale = d.get() / m.get();
                    if scale != 1.0 {
                        let scaled_pixel_size = (pixel_size as f64 * scale) as u16;
                        let scaled_font_size = font_size * scale;
                        log::trace!(
                            "using cap height adjusted: pixel_size {} -> {}, font_size {} -> {}, {:?}",
                            pixel_size,
                            scaled_pixel_size,
                            font_size,
                            scaled_font_size,
                            metrics,
                        );
                        let (alt_shaper, alt_handles) =
                            self.resolve_font_helper(style, &config, scaled_pixel_size)?;
                        shaper = alt_shaper;
                        handles = alt_handles;

                        metrics = shaper.metrics(scaled_font_size, dpi).with_context(|| {
                            format!(
                                "obtaining cap-height adjusted metrics for font_size={} @ dpi {}",
                                scaled_font_size, dpi
                            )
                        })?;

                        font_size = scaled_font_size;
                    }
                }
                _ => {}
            }
        }

        let loaded = Rc::new(LoadedFont {
            rasterizers: RefCell::new(HashMap::new()),
            handles: RefCell::new(handles),
            shaper: RefCell::new(shaper),
            metrics,
            font_size,
            dpi,
            font_config: Rc::downgrade(myself),
            pending_fallback: Arc::new(Mutex::new(vec![])),
            text_style: style.clone(),
            id: alloc_font_id(),
            tried_glyphs: RefCell::new(HashSet::new()),
            pixel_geometry: config.display_pixel_geometry,
        });

        fonts.insert(style.clone(), Rc::clone(&loaded));

        Ok(loaded)
    }

    pub fn change_scaling(&self, font_scale: f64, dpi: usize) -> (f64, usize) {
        let prior_font = *self.font_scale.borrow();
        let prior_dpi = *self.dpi.borrow();

        *self.dpi.borrow_mut() = dpi;
        *self.font_scale.borrow_mut() = font_scale;
        self.fonts.borrow_mut().clear();
        self.metrics.borrow_mut().take();
        self.title_font.borrow_mut().take();

        (prior_font, prior_dpi)
    }

    /// Returns the baseline font specified in the configuration
    pub fn default_font(&self, myself: &Rc<Self>) -> anyhow::Result<Rc<LoadedFont>> {
        self.resolve_font(myself, &self.config.borrow().font)
    }

    pub fn get_font_scale(&self) -> f64 {
        *self.font_scale.borrow()
    }

    pub fn get_dpi(&self) -> usize {
        *self.dpi.borrow()
    }

    pub fn default_font_metrics(&self, myself: &Rc<Self>) -> Result<FontMetrics, Error> {
        {
            let metrics = self.metrics.borrow();
            if let Some(metrics) = metrics.as_ref() {
                return Ok(*metrics);
            }
        }

        let font = self.default_font(myself)?;
        let metrics = font.metrics();

        *self.metrics.borrow_mut() = Some(metrics);

        Ok(metrics)
    }

    /// Apply the defined font_rules from the user configuration to
    /// produce the text style that best matches the supplied input
    /// cell attributes.
    pub fn match_style<'a>(
        &self,
        config: &'a ConfigHandle,
        attrs: &CellAttributes,
    ) -> &'a TextStyle {
        // a little macro to avoid boilerplate for matching the rules.
        // If the rule doesn't specify a value for an attribute then
        // it will implicitly match.  If it specifies an attribute
        // then it has to have the same value as that in the input attrs.
        macro_rules! attr_match {
            ($ident:ident, $rule:expr) => {
                if let Some($ident) = $rule.$ident {
                    if $ident != attrs.$ident() {
                        // Does not match
                        continue;
                    }
                }
                // matches so far...
            };
        }

        let would_bright = match attrs.foreground() {
            wezterm_term::color::ColorAttribute::PaletteIndex(idx) if idx < 8 => {
                attrs.intensity() == Intensity::Bold
            }
            _ => false,
        };

        for rule in &config.font_rules {
            if let Some(intensity) = rule.intensity {
                let effective_intensity = match config.bold_brightens_ansi_colors {
                    BoldBrightening::BrightOnly if would_bright => Intensity::Normal,
                    BoldBrightening::No
                    | BoldBrightening::BrightAndBold
                    | BoldBrightening::BrightOnly => attrs.intensity(),
                };
                if intensity != effective_intensity {
                    // Rule does not match
                    continue;
                }
                // matches so far
            }
            attr_match!(underline, &rule);
            attr_match!(italic, &rule);
            attr_match!(blink, &rule);
            attr_match!(reverse, &rule);
            attr_match!(strikethrough, &rule);
            attr_match!(invisible, &rule);

            // If we get here, then none of the rules didn't match,
            // so we therefore assume that it did match overall.
            return &rule.font;
        }
        &config.font
    }
}
