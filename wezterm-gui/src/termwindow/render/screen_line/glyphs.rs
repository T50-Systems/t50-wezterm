impl crate::TermWindow {
    fn render_screen_line_glyphs(
        &self,
        shaped: &[LineToElementShape],
        params: &RenderScreenLineParams,
        layers: &mut TripleLayerQuadAllocator,
        hsv: Option<HsbTransform>,
        direction: Direction,
        num_cols: usize,
        cell_width: f32,
        cell_height: f32,
        gl_x: f32,
        pos_y: f32,
        cursor_range_pixels: &Range<f32>,
        selection_pixel_range: &Range<f32>,
        width_scale: f32,
        height_scale: f32,
    ) -> anyhow::Result<()> {
        let gl_state = self.render_state.as_ref().unwrap();
        let mut overlay_images = vec![];

        // Number of cells we've rendered, starting from the edge of the line
        let mut visual_cell_idx = 0;

        let mut cluster_x_pos = match direction {
            Direction::LeftToRight => 0.,
            Direction::RightToLeft => params.pixel_width,
        };

        for item in shaped.iter() {
            let cluster = &item.cluster;
            let glyph_info = &item.glyph_info;
            let images = cluster.attrs.images().unwrap_or_else(|| vec![]);
            let valign_adjust = match cluster.attrs.vertical_align() {
                termwiz::cell::VerticalAlign::BaseLine => 0.,
                termwiz::cell::VerticalAlign::SuperScript => {
                    params.render_metrics.cell_size.height as f32 * -0.25
                }
                termwiz::cell::VerticalAlign::SubScript => {
                    params.render_metrics.cell_size.height as f32 * 0.25
                }
            };

            // Pre-decrement by the cluster width when doing RTL,
            // so that we can render it right-justified
            if direction == Direction::RightToLeft {
                cluster_x_pos -= if params.use_pixel_positioning {
                    item.pixel_width
                } else {
                    cluster.width as f32 * cell_width
                };
            }

            for info in glyph_info.iter() {
                let glyph = &info.glyph;

                if params.use_pixel_positioning
                    && params.left_pixel_x + cluster_x_pos + glyph.x_advance.get() as f32
                        >= params.left_pixel_x + params.pixel_width
                {
                    break;
                }

                for glyph_idx in 0..info.pos.num_cells as usize {
                    for img in &images {
                        if img.z_index() < 0 {
                            self.populate_image_quad(
                                &img,
                                gl_state,
                                layers,
                                0,
                                visual_cell_idx + glyph_idx,
                                &params,
                                hsv,
                                item.fg_color,
                            )?;
                        }
                    }
                }

                {
                    // First, resolve this glyph to a texture
                    let mut texture = glyph.texture.as_ref().cloned();

                    let mut top = cell_height
                        + (params.render_metrics.descender.get() as f32 + valign_adjust
                            - (glyph.y_offset + glyph.bearing_y).get() as f32)
                            * height_scale;

                    if self.config.custom_block_glyphs {
                        if let Some(block) = &info.block_key {
                            texture.replace(
                                gl_state
                                    .glyph_cache
                                    .borrow_mut()
                                    .cached_block(*block, &params.render_metrics)
                                    .context("cached_block")?,
                            );
                            // Custom glyphs don't have the same offsets as computed
                            // by the shaper, and are rendered relative to the cell
                            // top left, rather than the baseline.
                            top = 0.;
                        }
                    }

                    if let Some(texture) = texture {
                        // TODO: clipping, but we can do that based on pixels

                        let pos_x = cluster_x_pos
                            + if params.use_pixel_positioning {
                                (glyph.x_offset + glyph.bearing_x).get() as f32
                            } else {
                                0.
                            };

                        if pos_x > params.pixel_width {
                            log::trace!("breaking on overflow {} > {}", pos_x, params.pixel_width);
                            break;
                        }
                        let pos_x = pos_x + params.left_pixel_x;

                        // We need to conceptually slice this texture into
                        // up into strips that consider the cursor and selection
                        // background ranges. For ligatures that span cells, we'll
                        // need to explicitly render each strip independently so that
                        // we can set its foreground color to the appropriate color
                        // for the cursor/selection/regular background upon which
                        // it will be drawn.

                        let adjust = (glyph.x_offset + glyph.bearing_x).get() as f32;
                        let texture_range = pos_x + adjust
                            ..pos_x + adjust + (texture.coords.size.width as f32 * width_scale);

                        // First bucket the ranges according to cursor position
                        let (left, mid, right) = range3(&texture_range, &cursor_range_pixels);
                        // Then sub-divide the non-cursor ranges according to selection
                        let (la, lb, lc) = range3(&left, &selection_pixel_range);
                        let (ra, rb, rc) = range3(&right, &selection_pixel_range);

                        // and render each of these strips
                        for range in [la, lb, lc, mid, ra, rb, rc] {
                            if range.is_empty() {
                                continue;
                            }

                            let is_cursor = cursor_range_pixels.contains(&range.start);
                            let selected =
                                !is_cursor && selection_pixel_range.contains(&range.start);

                            let ComputeCellFgBgResult {
                                fg_color: glyph_color,
                                bg_color,
                                fg_color_alt,
                                fg_color_mix,
                                ..
                            } = self.compute_cell_fg_bg(ComputeCellFgBgParams {
                                cursor: if is_cursor { Some(params.cursor) } else { None },
                                selected,
                                fg_color: item.fg_color,
                                bg_color: item.bg_color,
                                is_active_pane: params.is_active,
                                config: params.config,
                                selection_fg: params.selection_fg,
                                selection_bg: params.selection_bg,
                                cursor_fg: params.cursor_fg,
                                cursor_bg: params.cursor_bg,
                                cursor_is_default_color: params.cursor_is_default_color,
                                cursor_border_color: params.cursor_border_color,
                                pane: params.pane,
                            });

                            if glyph_color == bg_color || cluster.attrs.invisible() {
                                // Essentially invisible: don't render it, as anti-aliasing
                                // can cause a ghostly outline of the invisible glyph to appear.
                                continue;
                            }

                            let pixel_rect = euclid::rect(
                                texture.coords.origin.x + (range.start - (pos_x + adjust)) as isize,
                                texture.coords.origin.y,
                                ((range.end - range.start) / width_scale) as isize,
                                texture.coords.size.height,
                            );

                            let texture_rect = texture.texture.to_texture_coords(pixel_rect);

                            let mut quad = layers.allocate(1).context("layers.allocate(1)")?;
                            quad.set_position(
                                gl_x + range.start,
                                pos_y + top,
                                gl_x + range.end,
                                pos_y + top + texture.coords.size.height as f32 * height_scale,
                            );
                            quad.set_fg_color(glyph_color);
                            quad.set_alt_color_and_mix_value(fg_color_alt, fg_color_mix);
                            quad.set_texture(texture_rect);
                            quad.set_hsv(if glyph.brightness_adjust != 1.0 {
                                let hsv = hsv.unwrap_or_else(|| HsbTransform::default());
                                Some(HsbTransform {
                                    brightness: hsv.brightness * glyph.brightness_adjust,
                                    ..hsv
                                })
                            } else {
                                hsv
                            });
                            quad.set_has_color(glyph.has_color);
                        }
                    }
                }

                for glyph_idx in 0..info.pos.num_cells as usize {
                    for img in &images {
                        if img.z_index() >= 0 {
                            overlay_images.push((
                                visual_cell_idx + glyph_idx,
                                img.clone(),
                                item.fg_color,
                            ));
                        }
                    }
                }
                visual_cell_idx += info.pos.num_cells as usize;
                cluster_x_pos += if params.use_pixel_positioning {
                    glyph.x_advance.get() as f32 * width_scale
                } else {
                    info.pos.num_cells as f32 * cell_width
                };
            }

            match direction {
                Direction::RightToLeft => {
                    // And decrement it again
                    cluster_x_pos -= if params.use_pixel_positioning {
                        item.pixel_width * width_scale
                    } else {
                        cluster.width as f32 * cell_width
                    };
                }
                Direction::LeftToRight => {}
            }
        }

        for (cell_idx, img, glyph_color) in overlay_images {
            self.populate_image_quad(
                &img,
                gl_state,
                layers,
                2,
                phys(cell_idx, num_cols, direction),
                &params,
                hsv,
                glyph_color,
            )
            .context("populate_image_quad")?;
        }
        Ok(())
    }
}
