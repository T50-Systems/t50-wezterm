impl super::TermWindow {
    pub fn render_element<'a>(
        &self,
        element: &ComputedElement,
        gl_state: &RenderState,
        inherited_colors: Option<&ElementColors>,
    ) -> anyhow::Result<()> {
        let layer = gl_state.layer_for_zindex(element.zindex)?;
        let mut layers = layer.quad_allocator();

        let colors = match &element.hover_colors {
            Some(hc) => {
                let hovering =
                    match &self.current_mouse_event {
                        Some(event) => {
                            let mouse_x = event.coords.x as f32;
                            let mouse_y = event.coords.y as f32;
                            mouse_x >= element.bounds.min_x()
                                && mouse_x <= element.bounds.max_x()
                                && mouse_y >= element.bounds.min_y()
                                && mouse_y <= element.bounds.max_y()
                        }
                        None => false,
                    } && matches!(self.current_mouse_capture, None | Some(MouseCapture::UI));
                if hovering {
                    hc
                } else {
                    &element.colors
                }
            }
            None => &element.colors,
        };

        self.render_element_background(element, colors, &mut layers, inherited_colors)?;
        let left = self.dimensions.pixel_width as f32 / -2.0;
        let top = self.dimensions.pixel_height as f32 / -2.0;
        match &element.content {
            ComputedElementContent::Text(cells) => {
                let mut pos_x = element.content_rect.min_x();
                for cell in cells {
                    if pos_x >= element.content_rect.max_x() {
                        break;
                    }
                    match cell {
                        ElementCell::Sprite(sprite) => {
                            let width = sprite.coords.width();
                            let height = sprite.coords.height();
                            let pos_y = top + element.content_rect.min_y();

                            if pos_x + width as f32 > element.content_rect.max_x() {
                                break;
                            }

                            let mut quad = layers.allocate(2)?;
                            quad.set_position(
                                pos_x + left,
                                pos_y,
                                pos_x + left + width as f32,
                                pos_y + height as f32,
                            );
                            self.resolve_text(colors, inherited_colors).apply(&mut quad);
                            quad.set_texture(sprite.texture_coords());
                            quad.set_hsv(None);
                            pos_x += width as f32;
                        }
                        ElementCell::Glyph(glyph) => {
                            if let Some(texture) = glyph.texture.as_ref() {
                                let pos_y = element.content_rect.min_y() as f32 + top
                                    - (glyph.y_offset + glyph.bearing_y).get() as f32
                                    + element.baseline;

                                if pos_x + glyph.x_advance.get() as f32
                                    > element.content_rect.max_x()
                                {
                                    break;
                                }
                                let pos_x = pos_x + (glyph.x_offset + glyph.bearing_x).get() as f32;
                                let width = texture.coords.size.width as f32 * glyph.scale as f32;
                                let height = texture.coords.size.height as f32 * glyph.scale as f32;

                                let mut quad = layers.allocate(1)?;
                                quad.set_position(
                                    pos_x + left,
                                    pos_y,
                                    pos_x + left + width,
                                    pos_y + height,
                                );
                                self.resolve_text(colors, inherited_colors).apply(&mut quad);
                                quad.set_texture(texture.texture_coords());
                                quad.set_has_color(glyph.has_color);
                                quad.set_hsv(None);
                            }
                            pos_x += glyph.x_advance.get() as f32;
                        }
                    }
                }
            }
            ComputedElementContent::Children(kids) => {
                drop(layers);

                for kid in kids {
                    self.render_element(kid, gl_state, Some(colors))?;
                }
            }
            ComputedElementContent::Poly { poly, line_width } => {
                if element.content_rect.width() >= poly.width {
                    let mut quad = self.poly_quad(
                        &mut layers,
                        1,
                        element.content_rect.origin,
                        poly.poly,
                        *line_width,
                        euclid::size2(poly.width, poly.height),
                        LinearRgba::TRANSPARENT,
                    )?;
                    self.resolve_text(colors, inherited_colors).apply(&mut quad);
                }
            }
        }

        Ok(())
    }

    fn resolve_text(
        &self,
        colors: &ElementColors,
        inherited_colors: Option<&ElementColors>,
    ) -> ResolvedColor {
        match &colors.text {
            InheritableColor::Inherited => match inherited_colors {
                Some(colors) => self.resolve_text(colors, None),
                None => LinearRgba::TRANSPARENT.into(),
            },
            InheritableColor::Color(color) => (*color).into(),
            InheritableColor::Animated {
                color,
                alt_color,
                ease,
                one_shot,
            } => {
                if let Some((mix_value, next)) = ease.borrow_mut().intensity(*one_shot) {
                    self.update_next_frame_time(Some(next));
                    ResolvedColor {
                        color: *color,
                        alt_color: *alt_color,
                        mix_value,
                    }
                } else {
                    (*color).into()
                }
            }
        }
    }

    fn resolve_bg(
        &self,
        colors: &ElementColors,
        inherited_colors: Option<&ElementColors>,
    ) -> ResolvedColor {
        match &colors.bg {
            InheritableColor::Inherited => match inherited_colors {
                Some(colors) => self.resolve_bg(colors, None),
                None => LinearRgba::TRANSPARENT.into(),
            },
            InheritableColor::Color(color) => (*color).into(),
            InheritableColor::Animated {
                color,
                alt_color,
                ease,
                one_shot,
            } => {
                if let Some((mix_value, next)) = ease.borrow_mut().intensity(*one_shot) {
                    self.update_next_frame_time(Some(next));
                    ResolvedColor {
                        color: *color,
                        alt_color: *alt_color,
                        mix_value,
                    }
                } else {
                    (*color).into()
                }
            }
        }
    }

    fn render_element_background<'a>(
        &self,
        element: &ComputedElement,
        colors: &ElementColors,
        layers: &mut TripleLayerQuadAllocator,
        inherited_colors: Option<&ElementColors>,
    ) -> anyhow::Result<()> {
        let mut top_left_width = 0.;
        let mut top_left_height = 0.;
        let mut top_right_width = 0.;
        let mut top_right_height = 0.;

        let mut bottom_left_width = 0.;
        let mut bottom_left_height = 0.;
        let mut bottom_right_width = 0.;
        let mut bottom_right_height = 0.;

        if let Some(c) = &element.border_corners {
            top_left_width = c.top_left.width;
            top_left_height = c.top_left.height;
            top_right_width = c.top_right.width;
            top_right_height = c.top_right.height;

            bottom_left_width = c.bottom_left.width;
            bottom_left_height = c.bottom_left.height;
            bottom_right_width = c.bottom_right.width;
            bottom_right_height = c.bottom_right.height;

            if top_left_width > 0. && top_left_height > 0. {
                self.poly_quad(
                    layers,
                    0,
                    element.border_rect.origin,
                    c.top_left.poly,
                    element.border.top as isize,
                    euclid::size2(top_left_width, top_left_height),
                    colors.border.top,
                )?
                .set_grayscale();
            }
            if top_right_width > 0. && top_right_height > 0. {
                self.poly_quad(
                    layers,
                    0,
                    euclid::point2(
                        element.border_rect.max_x() - top_right_width,
                        element.border_rect.min_y(),
                    ),
                    c.top_right.poly,
                    element.border.top as isize,
                    euclid::size2(top_right_width, top_right_height),
                    colors.border.top,
                )?
                .set_grayscale();
            }
            if bottom_left_width > 0. && bottom_left_height > 0. {
                self.poly_quad(
                    layers,
                    0,
                    euclid::point2(
                        element.border_rect.min_x(),
                        element.border_rect.max_y() - bottom_left_height,
                    ),
                    c.bottom_left.poly,
                    element.border.bottom as isize,
                    euclid::size2(bottom_left_width, bottom_left_height),
                    colors.border.bottom,
                )?
                .set_grayscale();
            }
            if bottom_right_width > 0. && bottom_right_height > 0. {
                self.poly_quad(
                    layers,
                    0,
                    euclid::point2(
                        element.border_rect.max_x() - bottom_right_width,
                        element.border_rect.max_y() - bottom_right_height,
                    ),
                    c.bottom_right.poly,
                    element.border.bottom as isize,
                    euclid::size2(bottom_right_width, bottom_right_height),
                    colors.border.bottom,
                )?
                .set_grayscale();
            }

            // Filling the background is more complex because we can't
            // simply fill the padding rect--we'd clobber the corner
            // graphics.
            // Instead, we consider the element as consisting of:
            //
            //   TL T TR
            //   L  C  R
            //   BL B BR
            //
            // We already rendered the corner pieces, so now we need
            // to do the rest

            // The `T` piece
            let mut quad = self.filled_rectangle(
                layers,
                0,
                euclid::rect(
                    element.border_rect.min_x() + top_left_width,
                    element.border_rect.min_y(),
                    element.border_rect.width() - (top_left_width + top_right_width) as f32,
                    top_left_height.max(top_right_height),
                ),
                LinearRgba::TRANSPARENT,
            )?;
            self.resolve_bg(colors, inherited_colors).apply(&mut quad);

            // The `B` piece
            let mut quad = self.filled_rectangle(
                layers,
                0,
                euclid::rect(
                    element.border_rect.min_x() + bottom_left_width,
                    element.border_rect.max_y() - bottom_left_height.max(bottom_right_height),
                    element.border_rect.width() - (bottom_left_width + bottom_right_width),
                    bottom_left_height.max(bottom_right_height),
                ),
                LinearRgba::TRANSPARENT,
            )?;
            self.resolve_bg(colors, inherited_colors).apply(&mut quad);

            // The `L` piece
            let mut quad = self.filled_rectangle(
                layers,
                0,
                euclid::rect(
                    element.border_rect.min_x(),
                    element.border_rect.min_y() + top_left_height,
                    top_left_width.max(bottom_left_width),
                    element.border_rect.height() - (top_left_height + bottom_left_height),
                ),
                LinearRgba::TRANSPARENT,
            )?;
            self.resolve_bg(colors, inherited_colors).apply(&mut quad);

            // The `R` piece
            let mut quad = self.filled_rectangle(
                layers,
                0,
                euclid::rect(
                    element.border_rect.max_x() - top_right_width,
                    element.border_rect.min_y() + top_right_height,
                    top_right_width.max(bottom_right_width),
                    element.border_rect.height() - (top_right_height + bottom_right_height),
                ),
                LinearRgba::TRANSPARENT,
            )?;
            self.resolve_bg(colors, inherited_colors).apply(&mut quad);

            // The `C` piece
            let mut quad = self.filled_rectangle(
                layers,
                0,
                euclid::rect(
                    element.border_rect.min_x() + top_left_width,
                    element.border_rect.min_y() + top_right_height.min(top_left_height),
                    element.border_rect.width() - (top_left_width + top_right_width),
                    element.border_rect.height()
                        - (top_right_height.min(top_left_height)
                            + bottom_right_height.min(bottom_left_height)),
                ),
                LinearRgba::TRANSPARENT,
            )?;
            self.resolve_bg(colors, inherited_colors).apply(&mut quad);
        } else if colors.bg != InheritableColor::Color(LinearRgba::TRANSPARENT) {
            let mut quad =
                self.filled_rectangle(layers, 0, element.padding, LinearRgba::TRANSPARENT)?;
            self.resolve_bg(colors, inherited_colors).apply(&mut quad);
        }

        if element.border_rect == element.padding {
            // There's no border to be drawn
            return Ok(());
        }

        if element.border.top > 0. && colors.border.top != LinearRgba::TRANSPARENT {
            self.filled_rectangle(
                layers,
                0,
                euclid::rect(
                    element.border_rect.min_x() + top_left_width as f32,
                    element.border_rect.min_y(),
                    element.border_rect.width() - (top_left_width + top_right_width) as f32,
                    element.border.top,
                ),
                colors.border.top,
            )?;
        }
        if element.border.bottom > 0. && colors.border.bottom != LinearRgba::TRANSPARENT {
            self.filled_rectangle(
                layers,
                0,
                euclid::rect(
                    element.border_rect.min_x() + bottom_left_width as f32,
                    element.border_rect.max_y() - element.border.bottom,
                    element.border_rect.width() - (bottom_left_width + bottom_right_width) as f32,
                    element.border.bottom,
                ),
                colors.border.bottom,
            )?;
        }
        if element.border.left > 0. && colors.border.left != LinearRgba::TRANSPARENT {
            self.filled_rectangle(
                layers,
                0,
                euclid::rect(
                    element.border_rect.min_x(),
                    element.border_rect.min_y() + top_left_height as f32,
                    element.border.left,
                    element.border_rect.height() - (top_left_height + bottom_left_height) as f32,
                ),
                colors.border.left,
            )?;
        }
        if element.border.right > 0. && colors.border.right != LinearRgba::TRANSPARENT {
            self.filled_rectangle(
                layers,
                0,
                euclid::rect(
                    element.border_rect.max_x() - element.border.right,
                    element.border_rect.min_y() + top_right_height as f32,
                    element.border.left,
                    element.border_rect.height() - (top_right_height + bottom_right_height) as f32,
                ),
                colors.border.right,
            )?;
        }

        Ok(())
    }
}
