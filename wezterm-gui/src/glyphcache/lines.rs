impl GlyphCache {
    fn line_sprite(&mut self, key: LineKey, metrics: &RenderMetrics) -> anyhow::Result<Sprite> {
        let mut buffer = Image::new(
            metrics.cell_size.width as usize,
            metrics.cell_size.height as usize,
        );
        let black = SrgbaPixel::rgba(0, 0, 0, 0);
        let white = SrgbaPixel::rgba(0xff, 0xff, 0xff, 0xff);

        let cell_rect = Rect::new(Point::new(0, 0), metrics.cell_size);

        let draw_single = |buffer: &mut Image| {
            for row in 0..metrics.underline_height {
                buffer.draw_line(
                    Point::new(
                        cell_rect.origin.x,
                        cell_rect.origin.y + metrics.descender_row + row,
                    ),
                    Point::new(
                        cell_rect.origin.x + metrics.cell_size.width,
                        cell_rect.origin.y + metrics.descender_row + row,
                    ),
                    white,
                );
            }
        };

        let draw_dotted = |buffer: &mut Image| {
            for row in 0..metrics.underline_height {
                let y = (cell_rect.origin.y + metrics.descender_row + row) as usize;
                if y >= metrics.cell_size.height as usize {
                    break;
                }

                let mut color = white;
                let segment_length = (metrics.cell_size.width / 4) as usize;
                let mut count = segment_length;
                let range =
                    buffer.horizontal_pixel_range_mut(0, metrics.cell_size.width as usize, y);
                for c in range.iter_mut() {
                    *c = color.as_srgba32();
                    count -= 1;
                    if count == 0 {
                        color = if color == white { black } else { white };
                        count = segment_length;
                    }
                }
            }
        };

        let draw_dashed = |buffer: &mut Image| {
            for row in 0..metrics.underline_height {
                let y = (cell_rect.origin.y + metrics.descender_row + row) as usize;
                if y >= metrics.cell_size.height as usize {
                    break;
                }
                let mut color = white;
                let third = (metrics.cell_size.width / 3) as usize + 1;
                let mut count = third;
                let range =
                    buffer.horizontal_pixel_range_mut(0, metrics.cell_size.width as usize, y);
                for c in range.iter_mut() {
                    *c = color.as_srgba32();
                    count -= 1;
                    if count == 0 {
                        color = if color == white { black } else { white };
                        count = third;
                    }
                }
            }
        };

        let draw_curly = |buffer: &mut Image| {
            let max_y = metrics.cell_size.height as usize - 1;
            let x_factor = (2. * std::f32::consts::PI) / metrics.cell_size.width as f32;

            // Have the wave go from the descender to the bottom of the cell
            let wave_height =
                metrics.cell_size.height - (cell_rect.origin.y + metrics.descender_row);

            let half_height = (wave_height as f32 / 4.).max(1.);
            let y = ((cell_rect.origin.y + metrics.descender_row) as usize)
                .saturating_sub(half_height as usize);

            fn add(x: usize, y: usize, val: u8, max_y: usize, buffer: &mut Image) {
                let y = y.min(max_y);
                let pixel = buffer.pixel_mut(x, y);
                let (current, _, _, _) = SrgbaPixel::with_srgba_u32(*pixel).as_rgba();
                let value = current.saturating_add(val);
                *pixel = SrgbaPixel::rgba(value, value, value, value).as_srgba32();
            }

            for x in 0..metrics.cell_size.width as usize {
                let vertical = -half_height * (x as f32 * x_factor).sin() + half_height;
                let v1 = vertical.floor();
                let v2 = vertical.ceil();

                for row in 0..metrics.underline_height as usize {
                    let value = (255. * (vertical - v1).abs()) as u8;
                    add(
                        x,
                        row.saturating_add(y).saturating_add(v1 as usize),
                        255u8.saturating_sub(value),
                        max_y,
                        buffer,
                    );
                    add(
                        x,
                        row.saturating_add(y).saturating_add(v2 as usize),
                        value,
                        max_y,
                        buffer,
                    );
                }
            }
        };

        let draw_double = |buffer: &mut Image| {
            let first_line = metrics
                .descender_row
                .min(metrics.descender_plus_two - 2 * metrics.underline_height);

            for row in 0..metrics.underline_height {
                buffer.draw_line(
                    Point::new(cell_rect.origin.x, cell_rect.origin.y + first_line + row),
                    Point::new(
                        cell_rect.origin.x + metrics.cell_size.width,
                        cell_rect.origin.y + first_line + row,
                    ),
                    white,
                );
                buffer.draw_line(
                    Point::new(
                        cell_rect.origin.x,
                        cell_rect.origin.y + metrics.descender_plus_two + row,
                    ),
                    Point::new(
                        cell_rect.origin.x + metrics.cell_size.width,
                        cell_rect.origin.y + metrics.descender_plus_two + row,
                    ),
                    white,
                );
            }
        };

        let draw_strike = |buffer: &mut Image| {
            for row in 0..metrics.underline_height {
                buffer.draw_line(
                    Point::new(
                        cell_rect.origin.x,
                        cell_rect.origin.y + metrics.strike_row + row,
                    ),
                    Point::new(
                        cell_rect.origin.x + metrics.cell_size.width,
                        cell_rect.origin.y + metrics.strike_row + row,
                    ),
                    white,
                );
            }
        };

        let draw_overline = |buffer: &mut Image| {
            for row in 0..metrics.underline_height {
                buffer.draw_line(
                    Point::new(cell_rect.origin.x, cell_rect.origin.y + row),
                    Point::new(
                        cell_rect.origin.x + metrics.cell_size.width,
                        cell_rect.origin.y + row,
                    ),
                    white,
                );
            }
        };

        buffer.clear_rect(cell_rect, black);
        if key.overline {
            draw_overline(&mut buffer);
        }
        match key.underline {
            Underline::None => {}
            Underline::Single => draw_single(&mut buffer),
            Underline::Curly => draw_curly(&mut buffer),
            Underline::Dashed => draw_dashed(&mut buffer),
            Underline::Dotted => draw_dotted(&mut buffer),
            Underline::Double => draw_double(&mut buffer),
        }
        if key.strike_through {
            draw_strike(&mut buffer);
        }
        let sprite = self.atlas.allocate(&buffer)?;
        self.line_glyphs.insert(key, sprite.clone());
        Ok(sprite)
    }

    /// Figure out what we're going to draw for the underline.
    /// If the current cell is part of the current URL highlight
    /// then we want to show the underline.
    pub fn cached_line_sprite(
        &mut self,
        is_highlited_hyperlink: bool,
        is_strike_through: bool,
        underline: Underline,
        overline: bool,
        metrics: &RenderMetrics,
    ) -> anyhow::Result<Sprite> {
        let effective_underline = match (is_highlited_hyperlink, underline) {
            (true, Underline::None) => Underline::Single,
            (true, Underline::Single) => Underline::Double,
            (true, _) => Underline::Single,
            (false, u) => u,
        };

        let key = LineKey {
            strike_through: is_strike_through,
            overline,
            underline: effective_underline,
            size: metrics.into(),
        };

        if let Some(s) = self.line_glyphs.get(&key) {
            return Ok(s.clone());
        }

        self.line_sprite(key, metrics)
    }
}
