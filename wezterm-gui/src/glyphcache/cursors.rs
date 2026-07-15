impl GlyphCache {
    pub fn cursor_sprite(
        &mut self,
        shape: Option<termwiz::surface::CursorShape>,
        metrics: &super::utilsprites::RenderMetrics,
        width: u8,
    ) -> anyhow::Result<::window::bitmaps::atlas::Sprite> {
        if let Some(sprite) = self.cursor_glyphs.get(&(shape, width)) {
            return Ok(sprite.clone());
        }

        let mut metrics = metrics.scale_cell_width(width as f64);
        if let Some(d) = &self.fonts.config().cursor_thickness {
            metrics.underline_height = d.evaluate_as_pixels(wezterm_config_types::DimensionContext {
                dpi: self.fonts.get_dpi() as f32,
                pixel_max: metrics.underline_height as f32,
                pixel_cell: metrics.cell_size.height as f32,
            }) as isize;
        }

        let image = match shape {
            None => {
                let mut image = ::window::bitmaps::Image::new(
                    metrics.cell_size.width as usize,
                    metrics.cell_size.height as usize,
                );
                let cell_rect = ::window::Rect::new(::window::Point::new(0, 0), metrics.cell_size);
                image.clear_rect(cell_rect, ::window::color::SrgbaPixel::rgba(0, 0, 0, 0));
                image
            }
            Some(termwiz::surface::CursorShape::Default) => {
                let mut image = ::window::bitmaps::Image::new(
                    metrics.cell_size.width as usize,
                    metrics.cell_size.height as usize,
                );
                let cell_rect = ::window::Rect::new(::window::Point::new(0, 0), metrics.cell_size);
                image.clear_rect(cell_rect, ::window::color::SrgbaPixel::rgba(0xff, 0xff, 0xff, 0xff));
                image
            }
            Some(termwiz::surface::CursorShape::BlinkingBlock | termwiz::surface::CursorShape::SteadyBlock) => {
                let raster = wezterm_custom_glyph::rasterize_polys(
                    &[wezterm_custom_glyph::Poly {
                        path: &[
                            wezterm_custom_glyph::PolyCommand::MoveTo(
                                wezterm_custom_glyph::BlockCoord::Zero,
                                wezterm_custom_glyph::BlockCoord::Zero,
                            ),
                            wezterm_custom_glyph::PolyCommand::LineTo(
                                wezterm_custom_glyph::BlockCoord::One,
                                wezterm_custom_glyph::BlockCoord::Zero,
                            ),
                            wezterm_custom_glyph::PolyCommand::LineTo(
                                wezterm_custom_glyph::BlockCoord::One,
                                wezterm_custom_glyph::BlockCoord::One,
                            ),
                            wezterm_custom_glyph::PolyCommand::LineTo(
                                wezterm_custom_glyph::BlockCoord::Zero,
                                wezterm_custom_glyph::BlockCoord::One,
                            ),
                            wezterm_custom_glyph::PolyCommand::LineTo(
                                wezterm_custom_glyph::BlockCoord::Zero,
                                wezterm_custom_glyph::BlockCoord::Zero,
                            ),
                        ],
                        intensity: wezterm_custom_glyph::BlockAlpha::Full,
                        style: wezterm_custom_glyph::PolyStyle::OutlineHeavy,
                    }],
                    &wezterm_custom_glyph::RasterizeGlyphParams {
                        underline_height: metrics.underline_height,
                        cell_size: wezterm_custom_glyph::CellSize::new(
                            metrics.cell_size.width,
                            metrics.cell_size.height,
                        ),
                        anti_alias: true,
                    },
                );
                ::window::bitmaps::Image::from_raw(raster.width, raster.height, raster.data)
            }
            Some(termwiz::surface::CursorShape::BlinkingBar | termwiz::surface::CursorShape::SteadyBar) => {
                let raster = wezterm_custom_glyph::rasterize_polys(
                    &[wezterm_custom_glyph::Poly {
                        path: &[
                            wezterm_custom_glyph::PolyCommand::MoveTo(
                                wezterm_custom_glyph::BlockCoord::Zero,
                                wezterm_custom_glyph::BlockCoord::Zero,
                            ),
                            wezterm_custom_glyph::PolyCommand::LineTo(
                                wezterm_custom_glyph::BlockCoord::Zero,
                                wezterm_custom_glyph::BlockCoord::One,
                            ),
                        ],
                        intensity: wezterm_custom_glyph::BlockAlpha::Full,
                        style: wezterm_custom_glyph::PolyStyle::OutlineHeavy,
                    }],
                    &wezterm_custom_glyph::RasterizeGlyphParams {
                        underline_height: metrics.underline_height,
                        cell_size: wezterm_custom_glyph::CellSize::new(
                            metrics.cell_size.width,
                            metrics.cell_size.height,
                        ),
                        anti_alias: true,
                    },
                );
                ::window::bitmaps::Image::from_raw(raster.width, raster.height, raster.data)
            }
            Some(
                termwiz::surface::CursorShape::BlinkingUnderline
                | termwiz::surface::CursorShape::SteadyUnderline,
            ) => {
                let raster = wezterm_custom_glyph::rasterize_polys(
                    &[wezterm_custom_glyph::Poly {
                        path: &[
                            wezterm_custom_glyph::PolyCommand::MoveTo(
                                wezterm_custom_glyph::BlockCoord::Zero,
                                wezterm_custom_glyph::BlockCoord::One,
                            ),
                            wezterm_custom_glyph::PolyCommand::LineTo(
                                wezterm_custom_glyph::BlockCoord::One,
                                wezterm_custom_glyph::BlockCoord::One,
                            ),
                        ],
                        intensity: wezterm_custom_glyph::BlockAlpha::Full,
                        style: wezterm_custom_glyph::PolyStyle::OutlineHeavy,
                    }],
                    &wezterm_custom_glyph::RasterizeGlyphParams {
                        underline_height: metrics.underline_height,
                        cell_size: wezterm_custom_glyph::CellSize::new(
                            metrics.cell_size.width,
                            metrics.cell_size.height,
                        ),
                        anti_alias: true,
                    },
                );
                ::window::bitmaps::Image::from_raw(raster.width, raster.height, raster.data)
            }
        };

        let sprite = self.atlas.allocate(&image)?;
        self.cursor_glyphs.insert((shape, width), sprite.clone());
        Ok(sprite)
    }
}
