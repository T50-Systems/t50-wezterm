impl GlyphCache {
    /// Resolve a glyph from the cache, rendering the glyph on-demand if
    /// the cache doesn't already hold the desired glyph.
    pub fn cached_glyph(
        &mut self,
        info: &GlyphInfo,
        style: &TextStyle,
        followed_by_space: bool,
        font: &Rc<LoadedFont>,
        metrics: &RenderMetrics,
        num_cells: u8,
    ) -> anyhow::Result<Rc<CachedGlyph>> {
        let key = BorrowedGlyphKey {
            font_idx: info.font_idx,
            glyph_pos: info.glyph_pos,
            num_cells: num_cells,
            style,
            followed_by_space,
            metric: metrics.into(),
            id: font.id(),
        };

        if let Some(entry) = self.glyph_cache.get(&key as &dyn GlyphKeyTrait) {
            metrics::histogram!("glyph_cache.glyph_cache.hit.rate").record(1.);
            return Ok(Rc::clone(entry));
        }
        metrics::histogram!("glyph_cache.glyph_cache.miss.rate").record(1.);

        let glyph = match self.load_glyph(info, font, followed_by_space, num_cells) {
            Ok(g) => g,
            Err(err) => {
                if err
                    .root_cause()
                    .downcast_ref::<OutOfTextureSpace>()
                    .is_some()
                {
                    // Ensure that we propagate this signal to expand
                    // our available teexture space
                    return Err(err);
                }

                // But otherwise: don't allow glyph loading errors to propagate,
                // as that will result in incomplete window painting.
                // Log the error and substitute instead.
                log::error!(
                    "load_glyph failed; using blank instead. Error: {:#}. {:?} {:?}",
                    err,
                    info,
                    style
                );
                Rc::new(CachedGlyph {
                    brightness_adjust: 1.0,
                    has_color: false,
                    texture: None,
                    x_advance: PixelLength::zero(),
                    x_offset: PixelLength::zero(),
                    y_offset: PixelLength::zero(),
                    bearing_x: PixelLength::zero(),
                    bearing_y: PixelLength::zero(),
                    scale: 1.0,
                })
            }
        };
        self.glyph_cache.insert(key.to_owned(), Rc::clone(&glyph));
        Ok(glyph)
    }

    pub fn config_changed(&mut self) {
        let config = self.fonts.config();
        self.image_cache.update_config(&config);
        self.cursor_glyphs.clear();
    }

    /// Perform the load and render of a glyph
    #[allow(clippy::float_cmp)]
    fn load_glyph(
        &mut self,
        info: &GlyphInfo,
        font: &Rc<LoadedFont>,
        followed_by_space: bool,
        num_cells: u8,
    ) -> anyhow::Result<Rc<CachedGlyph>> {
        let base_metrics;
        let idx_metrics;
        let brightness_adjust;
        let glyph;

        {
            base_metrics = font.metrics();
            glyph = font.rasterize_glyph(info.glyph_pos, info.font_idx)?;

            idx_metrics = font.metrics_for_idx(info.font_idx)?;
            brightness_adjust = font.brightness_adjust(info.font_idx);
        }

        let aspect = (idx_metrics.cell_width / idx_metrics.cell_height).get();

        // 0.7 is used for this as that is ~ the threshold for \u24e9 on a mac,
        // which is looks squareish and for which it is desirable to allow to
        // overflow.  0.5 is the typical monospace font aspect ratio.
        let is_square_or_wide = aspect >= 0.7;

        let allow_width_overflow = if is_square_or_wide {
            match self.fonts.config().allow_square_glyphs_to_overflow_width {
                AllowSquareGlyphOverflow::Never => false,
                AllowSquareGlyphOverflow::Always => true,
                AllowSquareGlyphOverflow::WhenFollowedBySpace => followed_by_space,
            }
        } else {
            false
        };

        // We shouldn't need to render a glyph that occupies zero cells, but that
        // can happen somehow; see <https://github.com/wezterm/wezterm/issues/1042>
        // so let's treat 0 cells as 1 cell so that we don't try to divide by
        // zero below.
        let num_cells = num_cells.max(1) as f64;

        // Maximum width allowed for this glyph based on its unicode width and
        // the dimensions of a cell
        let max_pixel_width = base_metrics.cell_width.get() * (num_cells + 0.25);

        let scale;

        // This helps to compensate for the !idx_metrics.is_scaled && glyph.is_scaled
        // case which happens when using the harfbuzz rasterizer with a bitmap font.
        // The default value is no compensation.
        let mut metrics_only_scale = 1.0;

        if info.font_idx == 0 {
            // We are the base font
            scale = if allow_width_overflow || glyph.width as f64 <= max_pixel_width {
                1.0
            } else {
                // Scale the glyph to fit in its number of cells
                1.0 / num_cells
            };
        } else if !glyph.is_scaled {
            // A bitmap font that isn't scaled to the requested height.
            let y_scale = base_metrics.cell_height.get() / idx_metrics.cell_height.get();
            let y_scaled_width = y_scale * glyph.width as f64;

            if allow_width_overflow || y_scaled_width <= max_pixel_width {
                // prefer height-wise scaling
                scale = y_scale;
            } else {
                // otherwise just make it fit the width
                scale = max_pixel_width / glyph.width as f64;
            }
        } else {
            // a scalable fallback font

            let f_width = glyph.width as f64;

            if allow_width_overflow || f_width <= max_pixel_width {
                scale = 1.0;
            } else {
                scale = max_pixel_width / f_width;
            }

            if !idx_metrics.is_scaled {
                // A special case: the shaper (eg: harfbuzz) processed
                // a bitmap font (eg: older versions of Noto Color Emoji)
                // to produce shaping info at the bitmap strike size,
                // which is 128 for that font.  The advance is expressed
                // at that size and not at the size of the font.
                // If we get to this condition, the rasterizer used a mode
                // where it has already scaled the glyph, so the dimensions
                // in the bitmap are correct, but the shaper metrics need
                // to be adjusted.
                let y_scale = base_metrics.cell_height.get() / idx_metrics.cell_height.get();
                metrics_only_scale = y_scale;
            }

            #[cfg(debug_assertions)]
            {
                log::debug!(
                    "{text} allow_width_overflow={allow_width_overflow} \
                     is_square_or_wide={is_square_or_wide} aspect={aspect} \
                     max_pixel_width={max_pixel_width} glyph.width={glyph_width} \
                     -> scale={scale} metrics_only_scale={metrics_only_scale}",
                    text = info.text,
                    glyph_width = glyph.width,
                );
            }
        };

        let descender_adjust = if info.font_idx == 0 {
            PixelLength::new(0.0)
        } else {
            idx_metrics.force_y_adjust
        };

        let (cell_width, cell_height) = (base_metrics.cell_width, base_metrics.cell_height);

        let glyph = if glyph.width == 0 || glyph.height == 0 {
            // a whitespace glyph
            CachedGlyph {
                brightness_adjust: 1.0,
                has_color: glyph.has_color,
                texture: None,
                x_offset: info.x_offset * scale,
                y_offset: info.y_offset * scale,
                x_advance: info.x_advance * scale,
                bearing_x: PixelLength::zero(),
                bearing_y: descender_adjust,
                scale,
            }
        } else {
            let raw_im = Image::with_rgba32(
                glyph.width as usize,
                glyph.height as usize,
                4 * glyph.width as usize,
                &glyph.data,
            );

            let bearing_x = glyph.bearing_x * scale * metrics_only_scale;
            // No metrics_only_scale adjustment to bearing_y is needed because
            // the value comes from the rasterized glyph and not from the
            // shaper stage.
            let bearing_y = descender_adjust + (glyph.bearing_y * scale);
            let x_offset = info.x_offset * scale * metrics_only_scale;
            let y_offset = info.y_offset * scale * metrics_only_scale;
            let x_advance = info.x_advance * scale * metrics_only_scale;

            log::trace!(
                "bearing_x={bearing_x:?} bearing_y={bearing_y:?} \
                 x_offset={x_offset:?} y_offset={y_offset:?} x_advance={x_advance:?}"
            );

            let (scale, raw_im) = if scale != 1.0 {
                log::trace!(
                    "physically scaling {:?} by {} bcos {}x{} > {:?}x{:?}. aspect={}",
                    info,
                    scale,
                    glyph.width,
                    glyph.height,
                    cell_width,
                    cell_height,
                    aspect,
                );
                (1.0, raw_im.scale_by(scale))
            } else {
                (scale, raw_im)
            };

            let tex = self.atlas.allocate(&raw_im)?;

            let g = CachedGlyph {
                brightness_adjust,
                has_color: glyph.has_color,
                texture: Some(tex),
                x_offset,
                y_offset,
                x_advance,
                bearing_x,
                bearing_y,
                scale,
            };

            if info.font_idx != 0 {
                // It's generally interesting to examine eg: emoji or ligatures
                // that we might have fallen back to
                log::trace!("{:?} {:?}", info, g);
            }

            g
        };

        Ok(Rc::new(glyph))
    }
}
