
fn redraw(&mut self) {
    let showing_title_bar = self.showing_title_bar(&*self.inner.borrow());

    if showing_title_bar {
        self.reshape_title();
    }

    let inner = self.inner.borrow_mut();

    // Don't draw borders if the frame explicitly hidden or fullscreened.
    if self.hidden || inner.fullscreened {
        // Don't draw the borders.
        for p in inner.parts.iter() {
            p.surface.attach(None, 0, 0);
            p.surface.commit();
        }
        return;
    }

    // `parts` can't be empty here, since the initial state for `self.hidden` is true, and
    // they will be created once `self.hidden` will become `false`.
    let parts = &inner.parts;

    let scales: Vec<u32> = parts
        .iter()
        .map(|part| get_surface_scale_factor(&part.surface) as u32)
        .collect();

    let (width, height) = inner.size;

    // Use header scale for all the thing.
    let header_scale = scales[HEAD];

    let scaled_header_height = HEADER_SIZE * header_scale;
    let scaled_header_width = width * header_scale;

    {
        // grab the current pool
        let pool = match self.pools.pool() {
            Some(pool) => pool,
            None => return,
        };
        let lr_surfaces_scale = max(scales[LEFT], scales[RIGHT]);
        let tp_surfaces_scale = max(scales[TOP], scales[BOTTOM]);

        // resize the pool as appropriate
        let pxcount = (scaled_header_height * scaled_header_width)
            + max(
                (width + 2 * BORDER_SIZE) * BORDER_SIZE * tp_surfaces_scale * tp_surfaces_scale,
                (height + HEADER_SIZE) * BORDER_SIZE * lr_surfaces_scale * lr_surfaces_scale,
            );

        pool.resize(4 * pxcount as usize)
            .expect("I/O Error while redrawing the borders");

        if showing_title_bar {
            // draw the header bar
            let mmap = pool.mmap();
            {
                let colors = self.config.colors();
                let color = match self.active {
                    WindowState::Active => colors.active_titlebar_bg,
                    WindowState::Inactive => colors.inactive_titlebar_bg,
                };

                let mut pixmap = PixmapMut::from_bytes(
                    &mut mmap[0..scaled_header_height as usize * scaled_header_width as usize * 4],
                    scaled_header_width,
                    scaled_header_height,
                )
                .expect("make pixmap from existing bitmap");

                pixmap.fill_path(
                    &PathBuilder::from_rect(
                        Rect::from_xywh(
                            0.,
                            0.,
                            scaled_header_width as f32,
                            scaled_header_height as f32,
                        )
                        .unwrap(),
                    ),
                    &color_to_paint(color),
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );

                if let Some(shaped) = self.shaped_title.as_ref() {
                    let mut x = 8.;
                    let limit = scaled_header_width.saturating_sub(4 * HEADER_SIZE) as f64;
                    let identity = Transform::identity();
                    let paint = PixmapPaint::default();
                    for item in &shaped.glyphs {
                        if let Some(data) = PixmapRef::from_bytes(
                            &item.glyph.data,
                            item.glyph.width as u32,
                            item.glyph.height as u32,
                        ) {
                            // FIXME: scale emoji

                            pixmap.draw_pixmap(
                                (x + item.info.x_offset.get() + item.glyph.bearing_x.get()) as i32,
                                (scaled_header_height * 3 / 4) as i32
                                    + (shaped.metrics.descender
                                        - (item.info.y_offset + item.glyph.bearing_y))
                                        .get() as i32,
                                data,
                                &paint,
                                identity,
                                None,
                            );
                        }

                        x += item.info.x_advance.get();
                        if x >= limit {
                            // Don't overflow the buttons
                            break;
                        }
                    }
                }

                draw_buttons(
                    &mut pixmap,
                    width,
                    header_scale,
                    inner.resizable,
                    self.active,
                    &self
                        .pointers
                        .iter()
                        .flat_map(|p| {
                            if p.as_ref().is_alive() {
                                let data: &RefCell<PointerUserData> =
                                    p.as_ref().user_data().get().unwrap();
                                Some(data.borrow().location)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<Location>>(),
                    &self.config,
                );
            }

            // For each pixel in borders
            {
                for b in
                    &mut mmap[scaled_header_height as usize * scaled_header_width as usize * 4..]
                {
                    *b = 0x00;
                }
            }
            if let Err(err) = mmap.flush() {
                log::error!("Failed to flush frame memory map: {}", err);
            }

            // Create the buffers
            // -> head-subsurface
            let buffer = pool.buffer(
                0,
                scaled_header_width as i32,
                scaled_header_height as i32,
                4 * scaled_header_width as i32,
                wl_shm::Format::Argb8888,
            );
            parts[HEAD]
                .subsurface
                .set_position(0, -(HEADER_SIZE as i32));
            parts[HEAD].surface.attach(Some(&buffer), 0, 0);
            if self.surface_version >= 4 {
                parts[HEAD].surface.damage_buffer(
                    0,
                    0,
                    scaled_header_width as i32,
                    scaled_header_height as i32,
                );
            } else {
                // surface is old and does not support damage_buffer, so we damage
                // in surface coordinates and hope it is not rescaled
                parts[HEAD]
                    .surface
                    .damage(0, 0, width as i32, HEADER_SIZE as i32);
            }
            parts[HEAD].surface.commit();
        }

        // -> top-subsurface
        let buffer = pool.buffer(
            4 * (scaled_header_width * scaled_header_height) as i32,
            ((width + 2 * BORDER_SIZE) * scales[TOP]) as i32,
            (BORDER_SIZE * scales[TOP]) as i32,
            (4 * scales[TOP] * (width + 2 * BORDER_SIZE)) as i32,
            wl_shm::Format::Argb8888,
        );
        parts[TOP].subsurface.set_position(
            -(BORDER_SIZE as i32),
            -(if showing_title_bar {
                HEADER_SIZE as i32
            } else {
                0
            } + BORDER_SIZE as i32),
        );
        parts[TOP].surface.attach(Some(&buffer), 0, 0);
        if self.surface_version >= 4 {
            parts[TOP].surface.damage_buffer(
                0,
                0,
                ((width + 2 * BORDER_SIZE) * scales[TOP]) as i32,
                (BORDER_SIZE * scales[TOP]) as i32,
            );
        } else {
            // surface is old and does not support damage_buffer, so we damage
            // in surface coordinates and hope it is not rescaled
            parts[TOP]
                .surface
                .damage(0, 0, (width + 2 * BORDER_SIZE) as i32, BORDER_SIZE as i32);
        }
        parts[TOP].surface.commit();

        // -> bottom-subsurface
        let buffer = pool.buffer(
            4 * (scaled_header_width * scaled_header_height) as i32,
            ((width + 2 * BORDER_SIZE) * scales[BOTTOM]) as i32,
            (BORDER_SIZE * scales[BOTTOM]) as i32,
            (4 * scales[BOTTOM] * (width + 2 * BORDER_SIZE)) as i32,
            wl_shm::Format::Argb8888,
        );
        parts[BOTTOM]
            .subsurface
            .set_position(-(BORDER_SIZE as i32), height as i32);
        parts[BOTTOM].surface.attach(Some(&buffer), 0, 0);
        if self.surface_version >= 4 {
            parts[BOTTOM].surface.damage_buffer(
                0,
                0,
                ((width + 2 * BORDER_SIZE) * scales[BOTTOM]) as i32,
                (BORDER_SIZE * scales[BOTTOM]) as i32,
            );
        } else {
            // surface is old and does not support damage_buffer, so we damage
            // in surface coordinates and hope it is not rescaled
            parts[BOTTOM].surface.damage(
                0,
                0,
                (width + 2 * BORDER_SIZE) as i32,
                BORDER_SIZE as i32,
            );
        }
        parts[BOTTOM].surface.commit();

        // -> left-subsurface
        let buffer = pool.buffer(
            4 * (scaled_header_width * scaled_header_height) as i32,
            (BORDER_SIZE * scales[LEFT]) as i32,
            ((height + HEADER_SIZE) * scales[LEFT]) as i32,
            4 * (BORDER_SIZE * scales[LEFT]) as i32,
            wl_shm::Format::Argb8888,
        );
        parts[LEFT]
            .subsurface
            .set_position(-(BORDER_SIZE as i32), -(HEADER_SIZE as i32));
        parts[LEFT].surface.attach(Some(&buffer), 0, 0);
        if self.surface_version >= 4 {
            parts[LEFT].surface.damage_buffer(
                0,
                0,
                (BORDER_SIZE * scales[LEFT]) as i32,
                ((height + HEADER_SIZE) * scales[LEFT]) as i32,
            );
        } else {
            // surface is old and does not support damage_buffer, so we damage
            // in surface coordinates and hope it is not rescaled
            parts[LEFT]
                .surface
                .damage(0, 0, BORDER_SIZE as i32, (height + HEADER_SIZE) as i32);
        }
        parts[LEFT].surface.commit();

        // -> right-subsurface
        let buffer = pool.buffer(
            4 * (scaled_header_width * scaled_header_height) as i32,
            (BORDER_SIZE * scales[RIGHT]) as i32,
            ((height + HEADER_SIZE) * scales[RIGHT]) as i32,
            4 * (BORDER_SIZE * scales[RIGHT]) as i32,
            wl_shm::Format::Argb8888,
        );
        parts[RIGHT]
            .subsurface
            .set_position(width as i32, -(HEADER_SIZE as i32));
        parts[RIGHT].surface.attach(Some(&buffer), 0, 0);
        if self.surface_version >= 4 {
            parts[RIGHT].surface.damage_buffer(
                0,
                0,
                (BORDER_SIZE * scales[RIGHT]) as i32,
                ((height + HEADER_SIZE) * scales[RIGHT]) as i32,
            );
        } else {
            // surface is old and does not support damage_buffer, so we damage
            // in surface coordinates and hope it is not rescaled
            parts[RIGHT]
                .surface
                .damage(0, 0, BORDER_SIZE as i32, (height + HEADER_SIZE) as i32);
        }
        parts[RIGHT].surface.commit();
    }
}
