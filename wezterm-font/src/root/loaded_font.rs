#[derive(Debug, Error)]
#[error("Font fallback recalculated")]
pub struct ClearShapeCache {}

static FONT_ID: ::std::sync::atomic::AtomicUsize = ::std::sync::atomic::AtomicUsize::new(0);
pub type LoadedFontId = usize;
pub fn alloc_font_id() -> LoadedFontId {
    FONT_ID.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed)
}

lazy_static::lazy_static! {
    static ref LAST_WARNING: Mutex<Option<(Instant, usize)>> = Mutex::new(None);
}

pub struct LoadedFont {
    rasterizers: RefCell<HashMap<FallbackIdx, Box<dyn FontRasterizer>>>,
    handles: RefCell<Vec<ParsedFont>>,
    shaper: RefCell<Box<dyn FontShaper>>,
    metrics: FontMetrics,
    pixel_geometry: DisplayPixelGeometry,
    font_size: f64,
    dpi: u32,
    font_config: Weak<FontConfigInner>,
    pending_fallback: Arc<Mutex<Vec<ParsedFont>>>,
    text_style: TextStyle,
    id: LoadedFontId,
    /// Glyphs for which no font was found and for which we should
    /// stop searching
    tried_glyphs: RefCell<HashSet<char>>,
}

impl std::fmt::Debug for LoadedFont {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("LoadedFont")
            .field("handles", &self.handles)
            .field("metrics", &self.metrics)
            .field("font_size", &self.font_size)
            .field("dpi", &self.dpi)
            .field("pending_fallback", &self.pending_fallback)
            .field("text_style", &self.text_style)
            .finish()
    }
}

impl LoadedFont {
    pub fn metrics(&self) -> FontMetrics {
        self.metrics
    }

    pub fn style(&self) -> &TextStyle {
        &self.text_style
    }

    pub fn id(&self) -> LoadedFontId {
        self.id
    }

    fn insert_fallback_handles(&self, extra_handles: Vec<ParsedFont>) -> anyhow::Result<bool> {
        let mut loaded = false;
        {
            let mut handles = self.handles.borrow_mut();
            for h in extra_handles {
                if !handles.iter().any(|existing| *existing == h) {
                    handles.push(h);
                    loaded = true;
                }
            }
            if loaded {
                log::trace!("revised fallback: {:#?}", handles);
            }
        }
        if loaded {
            if let Some(font_config) = self.font_config.upgrade() {
                *self.shaper.borrow_mut() =
                    new_shaper(&*font_config.config.borrow(), &self.handles.borrow())?;
            }
        }
        Ok(loaded)
    }

    pub fn blocking_shape(
        &self,
        text: &str,
        presentation: Option<Presentation>,
        direction: Direction,
        range: Option<Range<usize>>,
        presentation_width: Option<&PresentationWidth>,
    ) -> anyhow::Result<Vec<GlyphInfo>> {
        loop {
            let (tx, rx) = channel();

            let (async_resolve, res) = match self.shape_impl(
                text,
                move || {
                    let _ = tx.send(());
                },
                |_| {},
                presentation,
                direction,
                range.clone(),
                presentation_width,
            ) {
                Ok(tuple) => tuple,
                Err(err) if err.downcast_ref::<ClearShapeCache>().is_some() => {
                    continue;
                }
                Err(err) => return Err(err),
            };

            if !async_resolve {
                return Ok(res);
            }
            if rx.recv().is_err() {
                return Ok(res);
            }
        }
    }

    pub fn shape<F: FnOnce() + Send + 'static, FS: FnOnce(&mut Vec<char>)>(
        &self,
        text: &str,
        completion: F,
        filter_out_synthetic: FS,
        presentation: Option<Presentation>,
        direction: Direction,
        range: Option<Range<usize>>,
        presentation_width: Option<&PresentationWidth>,
    ) -> anyhow::Result<Vec<GlyphInfo>> {
        let (_async_resolve, res) = self.shape_impl(
            text,
            completion,
            filter_out_synthetic,
            presentation,
            direction,
            range,
            presentation_width,
        )?;
        Ok(res)
    }

    fn shape_impl<F: FnOnce() + Send + 'static, FS: FnOnce(&mut Vec<char>)>(
        &self,
        text: &str,
        completion: F,
        filter_out_synthetic: FS,
        presentation: Option<Presentation>,
        direction: Direction,
        range: Option<Range<usize>>,
        presentation_width: Option<&PresentationWidth>,
    ) -> anyhow::Result<(bool, Vec<GlyphInfo>)> {
        let mut no_glyphs = vec![];

        {
            let mut pending = self.pending_fallback.lock().unwrap();
            if !pending.is_empty() {
                match self.insert_fallback_handles(pending.split_off(0)) {
                    Ok(true) => return Err(ClearShapeCache {})?,
                    Ok(false) => {}
                    Err(err) => {
                        log::error!("Error adding fallback: {:#}", err);
                    }
                }
            }
        }

        let result = self.shaper.borrow().shape(
            text,
            self.font_size,
            self.dpi,
            &mut no_glyphs,
            presentation,
            direction,
            range,
            presentation_width,
        );

        no_glyphs.retain(|&c| c != '\u{FE0F}' && c != '\u{FE0E}');
        filter_out_synthetic(&mut no_glyphs);

        let mut tried_glyphs = self.tried_glyphs.borrow_mut();
        no_glyphs.retain(|c| !tried_glyphs.contains(c));
        for c in &no_glyphs {
            tried_glyphs.insert(*c);
        }

        no_glyphs.sort();
        no_glyphs.dedup();

        let mut async_resolve = false;

        if !no_glyphs.is_empty() {
            if let Some(font_config) = self.font_config.upgrade() {
                font_config.schedule_fallback_resolve(
                    no_glyphs,
                    &self.pending_fallback,
                    completion,
                );
                async_resolve = true;
            }
        }

        result.map(|r| (async_resolve, r))
    }

    pub fn metrics_for_idx(&self, font_idx: usize) -> anyhow::Result<FontMetrics> {
        self.shaper
            .borrow()
            .metrics_for_idx(font_idx, self.font_size, self.dpi)
    }

    pub fn brightness_adjust(&self, font_idx: usize) -> f32 {
        let synthesize_dim = self
            .handles
            .borrow()
            .get(font_idx)
            .map(|p| p.synthesize_dim)
            .unwrap_or(false);
        if synthesize_dim {
            0.5
        } else {
            1.0
        }
    }

    pub fn rasterize_glyph(
        &self,
        glyph_pos: u32,
        fallback: FallbackIdx,
    ) -> anyhow::Result<RasterizedGlyph> {
        let mut rasterizers = self.rasterizers.borrow_mut();
        if let Some(raster) = rasterizers.get(&fallback) {
            raster.rasterize_glyph(glyph_pos, self.font_size, self.dpi)
        } else {
            let raster_selection = self
                .font_config
                .upgrade()
                .map_or(FontRasterizerSelection::default(), |c| {
                    c.config.borrow().font_rasterizer
                });
            let raster = new_rasterizer(
                raster_selection,
                &(self.handles.borrow())[fallback],
                self.pixel_geometry,
            )?;
            let result = raster.rasterize_glyph(glyph_pos, self.font_size, self.dpi);
            rasterizers.insert(fallback, raster);
            result
        }
    }

    pub fn clone_handles(&self) -> Vec<ParsedFont> {
        self.handles.borrow().clone()
    }
}
