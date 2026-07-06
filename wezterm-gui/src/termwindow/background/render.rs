impl crate::TermWindow {
    pub fn render_backgrounds(
        &self,
        bg_color: LinearRgba,
        top: StableRowIndex,
    ) -> anyhow::Result<bool> {
        let gl_state = self.render_state.as_ref().unwrap();
        let mut layer_idx = -127;
        let mut loaded_any = false;
        for layer in self.window_background.iter() {
            if self.render_background(gl_state, bg_color, layer, layer_idx, top)? {
                loaded_any = true;
                layer_idx = layer_idx.saturating_add(1);
            }
        }
        Ok(loaded_any)
    }

    fn render_background(
        &self,
        gl_state: &RenderState,
        bg_color: LinearRgba,
        layer: &LoadedBackgroundLayer,
        layer_index: i8,
        top: StableRowIndex,
    ) -> anyhow::Result<bool> {
        let render_layer = gl_state.layer_for_zindex(layer_index)?;
        let vbs = render_layer.vb.borrow();
        let mut layer0 = vbs[0].map();

        let color = bg_color.mul_alpha(layer.def.opacity);

        let (sprite, next_due, load_state) = gl_state.glyph_cache.borrow_mut().cached_image(
            &layer.source,
            None,
            self.allow_images,
        )?;
        self.update_next_frame_time(next_due);

        if load_state == LoadState::Loading {
            return Ok(false);
        }

        let pixel_width = self.dimensions.pixel_width as f32;
        let pixel_height = self.dimensions.pixel_height as f32;
        let tex_width = sprite.coords.width() as f32;
        let tex_height = sprite.coords.height() as f32;

        let scale_width = pixel_width / tex_width as f32;
        let scale_height = pixel_height / tex_height as f32;

        let h_context = DimensionContext {
            dpi: self.dimensions.dpi as f32,
            pixel_max: pixel_width,
            pixel_cell: self.render_metrics.cell_size.width as f32,
        };
        let v_context = DimensionContext {
            dpi: self.dimensions.dpi as f32,
            pixel_max: pixel_height,
            pixel_cell: self.render_metrics.cell_size.height as f32,
        };

        // log::info!("tex {tex_width}x{tex_height} aspect={aspect}");

        // Compute the smallest aspect-preserved size that will fit the space
        let (min_aspect_width, min_aspect_height) = {
            let scale = scale_width.min(scale_height);
            (tex_width * scale, tex_height * scale)
        };
        // Compute the largest aspect-preserved size that will fill the space
        let (max_aspect_width, max_aspect_height) = {
            let scale = scale_width.max(scale_height);
            (tex_width * scale, tex_height * scale)
        };

        let width = match layer.def.width {
            BackgroundSize::Contain => min_aspect_width as f32,
            BackgroundSize::Cover => max_aspect_width as f32,
            BackgroundSize::Dimension(n) => n.evaluate_as_pixels(h_context),
        };

        let height = match layer.def.height {
            BackgroundSize::Contain => min_aspect_height as f32,
            BackgroundSize::Cover => max_aspect_height as f32,
            BackgroundSize::Dimension(n) => n.evaluate_as_pixels(v_context),
        };

        let mut origin_x = pixel_width / -2.;
        let top_pixel = pixel_height / -2.;
        let mut origin_y = top_pixel;

        match layer.def.vertical_align {
            BackgroundVerticalAlignment::Top => {}
            BackgroundVerticalAlignment::Bottom => {
                origin_y += pixel_height - height;
            }
            BackgroundVerticalAlignment::Middle => {
                origin_y += (pixel_height - height) / 2.;
            }
        }
        match layer.def.horizontal_align {
            BackgroundHorizontalAlignment::Left => {}
            BackgroundHorizontalAlignment::Right => {
                origin_x += pixel_width - width;
            }
            BackgroundHorizontalAlignment::Center => {
                origin_x += (pixel_width - width) / 2.;
            }
        }

        let vertical_offset = layer
            .def
            .vertical_offset
            .map(|d| d.evaluate_as_pixels(v_context))
            .unwrap_or(0.);
        origin_y += vertical_offset;

        let horizontal_offset = layer
            .def
            .horizontal_offset
            .map(|d| d.evaluate_as_pixels(h_context))
            .unwrap_or(0.);
        origin_x += horizontal_offset;

        let repeat_x = layer
            .def
            .repeat_x_size
            .map(|size| size.evaluate_as_pixels(h_context))
            .unwrap_or(width);
        let repeat_y = layer
            .def
            .repeat_y_size
            .map(|size| size.evaluate_as_pixels(v_context))
            .unwrap_or(height);

        // log::info!("computed {width}x{height}");

        let mut start_tile = 0;
        if let Some(factor) = layer.def.attachment.scroll_factor() {
            let distance = top as f32 * self.render_metrics.cell_size.height as f32 * factor;
            let num_tiles = distance / repeat_y;
            origin_y -= (num_tiles.fract() * repeat_y).floor();
            start_tile = num_tiles.floor() as usize;
        }

        let limit_y = top_pixel + pixel_height;

        let mut emitted = false;

        for y_step in start_tile.. {
            let offset_y = (y_step - start_tile) as f32 * repeat_y;
            let origin_y = origin_y + offset_y;
            if origin_y >= limit_y
                || (y_step > start_tile && layer.def.repeat_y == BackgroundRepeat::NoRepeat)
            {
                break;
            }

            for x_step in 0.. {
                let offset_x = x_step as f32 * repeat_x;
                if offset_x >= pixel_width
                    || (x_step > 0 && layer.def.repeat_x == BackgroundRepeat::NoRepeat)
                {
                    break;
                }
                let origin_x = origin_x + offset_x;
                let mut quad = layer0.allocate()?;
                emitted = true;
                // log::info!("quad {origin_x},{origin_y} {width}x{height}");
                quad.set_position(origin_x, origin_y, origin_x + width, origin_y + height);

                let coords = sprite.texture_coords();
                let mut x1 = coords.min_x();
                let mut x2 = coords.max_x();
                let mut y1 = coords.min_y();
                let mut y2 = coords.max_y();
                if layer.def.repeat_x == BackgroundRepeat::Mirror && x_step % 2 == 1 {
                    std::mem::swap(&mut x1, &mut x2);
                }
                if layer.def.repeat_y == BackgroundRepeat::Mirror && y_step % 2 == 1 {
                    std::mem::swap(&mut y1, &mut y2);
                }

                quad.set_texture_discrete(x1, x2, y1, y2);
                quad.set_is_background_image();
                quad.set_hsv(Some(layer.def.hsb));
                quad.set_fg_color(color);
            }
        }

        Ok(emitted)
    }
}
