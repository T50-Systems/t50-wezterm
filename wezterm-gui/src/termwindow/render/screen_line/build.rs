impl crate::TermWindow {
    fn build_line_element_shape(
        &self,
        params: LineToElementParams,
    ) -> anyhow::Result<(Rc<Vec<LineToElementShape>>, bool)> {
        let (bidi_enabled, bidi_direction) = params.line.bidi_info();
        let bidi_hint = if bidi_enabled {
            Some(bidi_direction)
        } else {
            None
        };
        let cell_clusters = if let Some((cursor_x, composing)) =
            params.shape_key.as_ref().and_then(|k| k.composing.as_ref())
        {
            // Create an updated line with the composition overlaid
            let mut line = params.line.clone();
            let seqno = line.current_seqno();
            line.overlay_text_with_attribute(*cursor_x, &composing, CellAttributes::blank(), seqno);
            line.cluster(bidi_hint)
        } else {
            params.line.cluster(bidi_hint)
        };

        let gl_state = self.render_state.as_ref().unwrap();
        let mut shaped = vec![];
        let mut last_style = None;
        let mut x_pos = 0.;
        let mut expires = None;
        let mut invalidate_on_hover_change = false;

        for cluster in &cell_clusters {
            if !matches!(last_style.as_ref(), Some(ClusterStyleCache{attrs,..}) if *attrs == &cluster.attrs)
            {
                let attrs = &cluster.attrs;
                let style = self.fonts.match_style(params.config, attrs);
                let hyperlink = attrs.hyperlink();
                let is_highlited_hyperlink =
                    same_hyperlink(hyperlink, self.current_highlight.as_ref());
                if hyperlink.is_some() {
                    invalidate_on_hover_change = true;
                }
                // underline and strikethrough
                let underline_tex_rect = gl_state
                    .glyph_cache
                    .borrow_mut()
                    .cached_line_sprite(
                        is_highlited_hyperlink,
                        attrs.strikethrough(),
                        attrs.underline(),
                        attrs.overline(),
                        &self.render_metrics,
                    )?
                    .texture_coords();
                let bg_is_default = attrs.background() == ColorAttribute::Default;
                let bg_color = params.palette.resolve_bg(attrs.background()).to_linear();

                let fg_color = resolve_fg_color_attr(
                    &attrs,
                    attrs.foreground(),
                    &params.palette,
                    &params.config,
                    style,
                );
                let (fg_color, bg_color, bg_is_default) = {
                    let mut fg = fg_color;
                    let mut bg = bg_color;
                    let mut bg_default = bg_is_default;

                    // Check the line reverse_video flag and flip.
                    if attrs.reverse() == !params.reverse_video {
                        std::mem::swap(&mut fg, &mut bg);
                        bg_default = false;
                    }

                    // Check for blink, and if this is the "not-visible"
                    // part of blinking then set fg = bg.  This is a cheap
                    // means of getting it done without impacting other
                    // features.
                    let blink_rate = match attrs.blink() {
                        Blink::None => None,
                        Blink::Slow => {
                            Some((params.config.text_blink_rate, self.blink_state.borrow_mut()))
                        }
                        Blink::Rapid => Some((
                            params.config.text_blink_rate_rapid,
                            self.rapid_blink_state.borrow_mut(),
                        )),
                    };
                    if let Some((blink_rate, mut colorease)) = blink_rate {
                        if blink_rate != 0 {
                            let (intensity, next) = colorease.intensity_continuous();

                            let (r1, g1, b1, a) = bg.tuple();
                            let (r, g, b, _a) = fg.tuple();
                            fg = LinearRgba::with_components(
                                r1 + (r - r1) * intensity,
                                g1 + (g - g1) * intensity,
                                b1 + (b - b1) * intensity,
                                a,
                            );

                            update_next_frame_time(&mut expires, Some(next));
                            self.update_next_frame_time(Some(next));
                        }
                    }

                    (fg, bg, bg_default)
                };

                let glyph_color = fg_color;
                let underline_color = match attrs.underline_color() {
                    ColorAttribute::Default => fg_color,
                    c => resolve_fg_color_attr(&attrs, c, &params.palette, &params.config, style),
                };

                let (bg_r, bg_g, bg_b, _) = bg_color.tuple();
                let bg_color = LinearRgba::with_components(
                    bg_r,
                    bg_g,
                    bg_b,
                    if params.window_is_transparent && bg_is_default {
                        0.0
                    } else {
                        params.config.text_background_opacity
                    },
                );

                last_style.replace(ClusterStyleCache {
                    attrs,
                    style,
                    underline_tex_rect: underline_tex_rect.clone(),
                    bg_color,
                    fg_color: glyph_color,
                    underline_color,
                });
            }

            let style_params = last_style.as_ref().expect("we just set it up").clone();

            let glyph_info = self.cached_cluster_shape(
                style_params.style,
                &cluster,
                &gl_state,
                None,
                &self.render_metrics,
            )?;
            let pixel_width = glyph_info
                .iter()
                .map(|info| info.glyph.x_advance.get() as f32)
                .sum();

            shaped.push(LineToElementShape {
                underline_tex_rect: style_params.underline_tex_rect,
                bg_color: style_params.bg_color,
                fg_color: style_params.fg_color,
                underline_color: style_params.underline_color,
                pixel_width,
                cluster: cluster.clone(),
                glyph_info,
                x_pos,
            });

            x_pos += pixel_width;
        }

        let shaped = Rc::new(shaped);

        if let Some(shape_key) = params.shape_key {
            self.line_to_ele_shape_cache.borrow_mut().put(
                shape_key.clone(),
                LineToElementShapeItem {
                    expires,
                    shaped: Rc::clone(&shaped),
                    invalidate_on_hover_change,
                    current_highlight: if invalidate_on_hover_change {
                        self.current_highlight.clone()
                    } else {
                        None
                    },
                },
            );
        }

        Ok((shaped, invalidate_on_hover_change))
    }
}
