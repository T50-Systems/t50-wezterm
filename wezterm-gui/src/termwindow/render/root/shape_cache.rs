impl crate::TermWindow {
    fn use_reverse_video_cursor(&self, params: &ComputeCellFgBgParams) -> bool {
        self.config.force_reverse_video_cursor
            && params.cursor_is_default_color
            && params.fg_color.contrast_ratio(&params.bg_color)
                >= self.config.reverse_video_cursor_min_contrast
    }

    fn glyph_infos_to_glyphs(
        &self,
        style: &TextStyle,
        glyph_cache: &mut GlyphCache,
        infos: &[GlyphInfo],
        font: &Rc<LoadedFont>,
        metrics: &RenderMetrics,
    ) -> anyhow::Result<Vec<Rc<CachedGlyph>>> {
        let mut glyphs = Vec::with_capacity(infos.len());
        let mut iter = infos.iter().peekable();
        while let Some(info) = iter.next() {
            if self.config.custom_block_glyphs {
                if info.only_char.and_then(BlockKey::from_char).is_some() {
                    // Don't bother rendering the glyph from the font, as it can
                    // have incorrect advance metrics.
                    // Instead, just use our pixel-perfect cell metrics
                    glyphs.push(Rc::new(CachedGlyph {
                        brightness_adjust: 1.0,
                        has_color: false,
                        texture: None,
                        x_advance: PixelLength::new(metrics.cell_size.width as f64),
                        x_offset: PixelLength::zero(),
                        y_offset: PixelLength::zero(),
                        bearing_x: PixelLength::zero(),
                        bearing_y: PixelLength::zero(),
                        scale: 1.0,
                    }));
                    continue;
                }
            }

            let followed_by_space = match iter.peek() {
                Some(next_info) => next_info.is_space,
                None => false,
            };

            glyphs.push(glyph_cache.cached_glyph(
                info,
                &style,
                followed_by_space,
                font,
                metrics,
                info.num_cells,
            )?);
        }
        Ok(glyphs)
    }

    /// Shape the printable text from a cluster
    fn cached_cluster_shape(
        &self,
        style: &TextStyle,
        cluster: &CellCluster,
        gl_state: &RenderState,
        font: Option<&Rc<LoadedFont>>,
        metrics: &RenderMetrics,
    ) -> anyhow::Result<Rc<Vec<ShapedInfo>>> {
        let shape_resolve_start = Instant::now();
        let key = BorrowedShapeCacheKey {
            style,
            text: &cluster.text,
        };
        let glyph_info = match self.lookup_cached_shape(&key) {
            Some(Ok(info)) => info,
            Some(Err(err)) => return Err(err),
            None => {
                let font = match font {
                    Some(f) => Rc::clone(f),
                    None => self.fonts.resolve_font(style)?,
                };
                let window = self.window.as_ref().unwrap().clone();

                let presentation_width = PresentationWidth::with_cluster(&cluster);

                match font.shape(
                    &cluster.text,
                    move || window.notify(TermWindowNotif::InvalidateShapeCache),
                    BlockKey::filter_out_synthetic,
                    Some(cluster.presentation),
                    cluster.direction,
                    None, // FIXME: need more paragraph context
                    Some(&presentation_width),
                ) {
                    Ok(info) => {
                        let glyphs = self.glyph_infos_to_glyphs(
                            &style,
                            &mut gl_state.glyph_cache.borrow_mut(),
                            &info,
                            &font,
                            metrics,
                        )?;
                        let shaped = Rc::new(ShapedInfo::process(&info, &glyphs));

                        self.shape_cache
                            .borrow_mut()
                            .put(key.to_owned(), Ok(Rc::clone(&shaped)));
                        shaped
                    }
                    Err(err) => {
                        if err.root_cause().downcast_ref::<ClearShapeCache>().is_some() {
                            return Err(err);
                        }

                        let res = anyhow!("shaper error: {}", err);
                        self.shape_cache.borrow_mut().put(key.to_owned(), Err(err));
                        return Err(res);
                    }
                }
            }
        };
        metrics::histogram!("cached_cluster_shape").record(shape_resolve_start.elapsed());
        log::trace!(
            "shape_resolve for cluster len {} -> elapsed {:?}",
            cluster.text.len(),
            shape_resolve_start.elapsed()
        );
        Ok(glyph_info)
    }

    fn lookup_cached_shape(
        &self,
        key: &dyn ShapeCacheKeyTrait,
    ) -> Option<anyhow::Result<Rc<Vec<ShapedInfo>>>> {
        match self.shape_cache.borrow_mut().get(key) {
            Some(Ok(info)) => Some(Ok(Rc::clone(info))),
            Some(Err(err)) => Some(Err(anyhow!("cached shaper error: {}", err))),
            None => None,
        }
    }

    pub fn recreate_texture_atlas(&mut self, size: Option<usize>) -> anyhow::Result<()> {
        self.shape_generation += 1;
        self.shape_cache.borrow_mut().clear();
        self.line_to_ele_shape_cache.borrow_mut().clear();
        if let Some(render_state) = self.render_state.as_mut() {
            render_state.recreate_texture_atlas(&self.fonts, &self.render_metrics, size)?;
        }
        Ok(())
    }

    fn shape_hash_for_line(&mut self, line: &Line) -> [u8; 16] {
        let seqno = line.current_seqno();
        let mut id = None;
        if let Some(cached_arc) = line.get_appdata() {
            if let Some(line_state) = cached_arc.downcast_ref::<CachedLineState>() {
                if line_state.seqno == seqno {
                    // Touch the LRU
                    self.line_state_cache.borrow_mut().get(&line_state.id);
                    return line_state.shape_hash;
                }
                id.replace(line_state.id);
            }
        }

        let id = id.unwrap_or_else(|| {
            let id = self.next_line_state_id;
            self.next_line_state_id += 1;
            id
        });

        let shape_hash = line.compute_shape_hash();

        let state = Arc::new(CachedLineState {
            id,
            seqno,
            shape_hash,
        });

        line.set_appdata(Arc::clone(&state));

        self.line_state_cache.borrow_mut().put(id, state);
        shape_hash
    }
}
