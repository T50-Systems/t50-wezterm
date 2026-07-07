use crate::quad::{QuadTrait, TripleLayerQuadAllocator, TripleLayerQuadAllocatorTrait};
use crate::termwindow::render::{
    resolve_fg_color_attr, same_hyperlink, update_next_frame_time, ClusterStyleCache,
    ComputeCellFgBgParams, ComputeCellFgBgResult, LineToElementParams, LineToElementShape,
    RenderScreenLineParams, RenderScreenLineResult,
};
use crate::termwindow::LineToElementShapeItem;
use ::window::DeadKeyStatus;
use anyhow::Context;
use config::{HsbTransform, TextStyle};
use std::ops::Range;
use std::rc::Rc;
use std::time::Instant;
use termwiz::cell::{unicode_column_width, Blink};
use termwiz::color::LinearRgba;
use termwiz::surface::CursorShape;
use wezterm_bidi::Direction;
use wezterm_term::color::ColorAttribute;
use wezterm_term::CellAttributes;

fn phys(x: usize, num_cols: usize, direction: Direction) -> usize {
    match direction {
        Direction::LeftToRight => x,
        Direction::RightToLeft => num_cols - x,
    }
}

fn intersection(r1: &Range<f32>, r2: &Range<f32>) -> Range<f32> {
    let start = r1.start.max(r2.start);
    let end = r1.end.min(r2.end);
    if end > start {
        start..end
    } else {
        start..start
    }
}

fn range3(r: &Range<f32>, within: &Range<f32>) -> (Range<f32>, Range<f32>, Range<f32>) {
    if r.is_empty() {
        return (r.clone(), r.clone(), r.clone());
    }
    let i = intersection(r, within);
    if i.is_empty() {
        return (r.clone(), i.clone(), i.clone());
    }

    let left = if i.start > r.start {
        r.start..i.start
    } else {
        r.start..r.start
    };

    let right = if i.end < r.end {
        i.end..r.end
    } else {
        r.end..r.end
    };

    (left, i, right)
}

impl crate::TermWindow {
    /// "Render" a line of the terminal screen into the vertex buffer.
    /// This is nominally a matter of setting the fg/bg color and the
    /// texture coordinates for a given glyph.  There's a little bit
    /// of extra complexity to deal with multi-cell glyphs.
    pub fn render_screen_line(
        &self,
        params: RenderScreenLineParams,
        layers: &mut TripleLayerQuadAllocator,
    ) -> anyhow::Result<RenderScreenLineResult> {
        if params.line.is_double_height_bottom() {
            // The top and bottom lines are required to have the same content.
            // For the sake of simplicity, we render both of them as part of
            // rendering the top row, so we have nothing more to do here.
            return Ok(RenderScreenLineResult {
                invalidate_on_hover_change: false,
            });
        }

        let gl_state = self.render_state.as_ref().unwrap();

        let num_cols = params.dims.cols;

        let hsv = if params.is_active {
            None
        } else {
            Some(params.config.inactive_pane_hsb)
        };

        let width_scale = if !params.line.is_single_width() {
            2.0
        } else {
            1.0
        };

        let height_scale = if params.line.is_double_height_top() {
            2.0
        } else {
            1.0
        };

        let cell_width = params.render_metrics.cell_size.width as f32 * width_scale;
        let cell_height = params.render_metrics.cell_size.height as f32 * height_scale;
        let pos_y = (self.dimensions.pixel_height as f32 / -2.) + params.top_pixel_y;
        let gl_x = self.dimensions.pixel_width as f32 / -2.;

        let start = Instant::now();

        let cursor_idx = if params.pane.is_some()
            && params.is_active
            && params.stable_line_idx == Some(params.cursor.y)
        {
            Some(params.cursor.x)
        } else {
            None
        };

        // Referencing the text being composed, but only if it belongs to this pane
        let composing = if cursor_idx.is_some() {
            if let DeadKeyStatus::Composing(composing) = &self.dead_key_status {
                Some(composing)
            } else {
                None
            }
        } else {
            None
        };

        let mut composition_width = 0;

        let (_bidi_enabled, bidi_direction) = params.line.bidi_info();
        let direction = bidi_direction.direction();

        // Do we need to shape immediately, or can we use the pre-shaped data?
        if let Some(composing) = composing {
            composition_width = unicode_column_width(composing, None);
        }

        let cursor_cell = if params.stable_line_idx == Some(params.cursor.y) {
            params.line.get_cell(params.cursor.x)
        } else {
            None
        };

        let cursor_range = if composition_width > 0 {
            params.cursor.x..params.cursor.x + composition_width
        } else if params.stable_line_idx == Some(params.cursor.y) {
            params.cursor.x..params.cursor.x + cursor_cell.as_ref().map(|c| c.width()).unwrap_or(1)
        } else {
            0..0
        };

        let cursor_range_pixels = params.left_pixel_x + cursor_range.start as f32 * cell_width
            ..params.left_pixel_x + cursor_range.end as f32 * cell_width;

        let mut shaped = None;
        let mut invalidate_on_hover_change = false;

        if let Some(shape_key) = &params.shape_key {
            let mut cache = self.line_to_ele_shape_cache.borrow_mut();
            if let Some(entry) = cache.get(shape_key) {
                let expired = entry.expires.map(|i| Instant::now() >= i).unwrap_or(false);
                let hover_changed = if entry.invalidate_on_hover_change {
                    !same_hyperlink(
                        entry.current_highlight.as_ref(),
                        self.current_highlight.as_ref(),
                    )
                } else {
                    false
                };

                if !expired && !hover_changed {
                    self.update_next_frame_time(entry.expires);
                    shaped.replace(Rc::clone(&entry.shaped));
                }

                invalidate_on_hover_change = entry.invalidate_on_hover_change;
            }
        }

        let shaped = if let Some(shaped) = shaped {
            shaped
        } else {
            let params = LineToElementParams {
                config: params.config,
                line: params.line,
                palette: params.palette,
                window_is_transparent: params.window_is_transparent,
                reverse_video: params.dims.reverse_video,
                shape_key: &params.shape_key,
            };

            let (shaped, invalidate_on_hover) = self.build_line_element_shape(params)?;
            invalidate_on_hover_change = invalidate_on_hover;
            shaped
        };

        let bounding_rect = euclid::rect(
            params.left_pixel_x,
            params.top_pixel_y,
            params.pixel_width,
            cell_height,
        );

        if params.dims.reverse_video {
            let mut quad = self
                .filled_rectangle(
                    layers,
                    0,
                    euclid::rect(
                        params.left_pixel_x,
                        params.top_pixel_y,
                        params.pixel_width,
                        cell_height,
                    ),
                    params.foreground,
                )
                .context("filled_rectangle")?;
            quad.set_hsv(hsv);
        }

        // Assume that we are drawing retro tab bar if there is no
        // stable_line_idx set.
        let is_tab_bar = params.stable_line_idx.is_none();

        // Make a pass to compute background colors.
        // Need to consider:
        // * background when it is not the default color
        // * Reverse video attribute
        for item in shaped.iter() {
            let cluster = &item.cluster;
            let attrs = &cluster.attrs;
            let cluster_width = cluster.width;

            let bg_is_default = attrs.background() == ColorAttribute::Default;
            let bg_color = params.palette.resolve_bg(attrs.background()).to_linear();

            let fg_color = resolve_fg_color_attr(
                &attrs,
                attrs.foreground(),
                &params.palette,
                &params.config,
                &Default::default(),
            );

            let (bg_color, bg_is_default) = {
                let mut fg = fg_color;
                let mut bg = bg_color;
                let mut bg_default = bg_is_default;

                // Check the line reverse_video flag and flip.
                if attrs.reverse() == !params.dims.reverse_video {
                    std::mem::swap(&mut fg, &mut bg);
                    bg_default = false;
                }

                (
                    bg.mul_alpha(self.config.text_background_opacity),
                    bg_default,
                )
            };

            if !bg_is_default {
                let x = params.left_pixel_x
                    + if params.use_pixel_positioning {
                        item.x_pos
                    } else {
                        phys(cluster.first_cell_idx, num_cols, direction) as f32 * cell_width
                    };

                let mut width = if params.use_pixel_positioning {
                    item.pixel_width
                } else {
                    cluster_width as f32 * cell_width
                };

                // If the tab bar is falling just short of the full width of the
                // window, extend it to fit.
                // <https://github.com/wezterm/wezterm/issues/2210>
                if is_tab_bar && (x + width + cell_width) > params.pixel_width {
                    width += cell_width;
                }

                let rect = euclid::rect(x, params.top_pixel_y, width, cell_height);
                if let Some(rect) = rect.intersection(&bounding_rect) {
                    let mut quad = self
                        .filled_rectangle(layers, 0, rect, bg_color)
                        .context("filled_rectangle")?;
                    quad.set_hsv(hsv);
                }
            }

            // Underlines
            if item.underline_tex_rect != params.white_space {
                // Draw one per cell, otherwise curly underlines
                // stretch across the whole span
                for i in 0..cluster_width {
                    let mut quad = layers.allocate(0).context("layers.allocate(0)")?;
                    let x = gl_x
                        + params.left_pixel_x
                        + if params.use_pixel_positioning {
                            item.x_pos
                        } else {
                            phys(cluster.first_cell_idx + i, num_cols, direction) as f32
                                * cell_width
                        };

                    quad.set_position(x, pos_y, x + cell_width, pos_y + cell_height);
                    quad.set_hsv(hsv);
                    quad.set_has_color(false);
                    quad.set_texture(item.underline_tex_rect);
                    quad.set_fg_color(item.underline_color);
                }
            }
        }

        // Render the selection background color.
        // This always uses a physical x position, regardles of the line
        // direction.
        let selection_pixel_range = if !params.selection.is_empty() {
            let start = params.left_pixel_x + (params.selection.start as f32 * cell_width);
            let width = (params.selection.end - params.selection.start) as f32 * cell_width;
            let mut quad = self
                .filled_rectangle(
                    layers,
                    0,
                    euclid::rect(start, params.top_pixel_y, width, cell_height),
                    params.selection_bg,
                )
                .context("filled_rectangle")?;

            quad.set_hsv(hsv);

            start..start + width
        } else {
            0.0..0.0
        };

        // Consider cursor
        if !cursor_range.is_empty() {
            let (fg_color, bg_color) = if let Some(c) = &cursor_cell {
                let attrs = c.attrs();

                let bg_color = params.palette.resolve_bg(attrs.background()).to_linear();

                let fg_color = resolve_fg_color_attr(
                    &attrs,
                    attrs.foreground(),
                    &params.palette,
                    &params.config,
                    &Default::default(),
                );

                (fg_color, bg_color)
            } else {
                (params.foreground, params.default_bg)
            };

            let ComputeCellFgBgResult {
                cursor_shape,
                cursor_border_color,
                cursor_border_color_alt,
                cursor_border_mix,
                ..
            } = self.compute_cell_fg_bg(ComputeCellFgBgParams {
                cursor: Some(params.cursor),
                selected: false,
                fg_color,
                bg_color,
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
            let pos_x = (self.dimensions.pixel_width as f32 / -2.)
                + params.left_pixel_x
                + (phys(params.cursor.x, num_cols, direction) as f32 * cell_width);

            if let Some(shape) = cursor_shape {
                let cursor_layer = match shape {
                    CursorShape::BlinkingBar | CursorShape::SteadyBar => 2,
                    _ => 0,
                };
                let mut quad = layers
                    .allocate(cursor_layer)
                    .with_context(|| format!("layers.allocate({cursor_layer})"))?;
                quad.set_hsv(hsv);
                quad.set_has_color(false);

                let mut draw_basic = true;

                if params.password_input {
                    let attrs = cursor_cell
                        .as_ref()
                        .map(|cell| cell.attrs().clone())
                        .unwrap_or_else(|| CellAttributes::blank());

                    let glyph = self
                        .resolve_lock_glyph(
                            &TextStyle::default(),
                            &attrs,
                            params.font.as_ref(),
                            gl_state,
                            &params.render_metrics,
                        )
                        .context("resolve_lock_glyph")?;

                    if let Some(sprite) = &glyph.texture {
                        let width = sprite.coords.size.width as f32 * glyph.scale as f32;
                        let height =
                            sprite.coords.size.height as f32 * glyph.scale as f32 * height_scale;

                        let pos_y = pos_y
                            + cell_height
                            + (params.render_metrics.descender.get() as f32
                                - (glyph.y_offset + glyph.bearing_y).get() as f32)
                                * height_scale;

                        let pos_x = pos_x + (glyph.x_offset + glyph.bearing_x).get() as f32;
                        quad.set_position(pos_x, pos_y, pos_x + width, pos_y + height);
                        quad.set_texture(sprite.texture_coords());
                        draw_basic = false;
                    }
                }

                if draw_basic {
                    quad.set_position(
                        pos_x,
                        pos_y,
                        pos_x + (cursor_range.end - cursor_range.start) as f32 * cell_width,
                        pos_y + cell_height,
                    );
                    quad.set_texture(
                        gl_state
                            .glyph_cache
                            .borrow_mut()
                            .cursor_sprite(
                                Some(shape),
                                &params.render_metrics,
                                (cursor_range.end - cursor_range.start) as u8,
                            )?
                            .texture_coords(),
                    );
                }

                quad.set_fg_color(cursor_border_color);
                quad.set_alt_color_and_mix_value(cursor_border_color_alt, cursor_border_mix);
            }
        }

        self.render_screen_line_glyphs(
            shaped.as_ref(),
            &params,
            layers,
            hsv,
            direction,
            num_cols,
            cell_width,
            cell_height,
            gl_x,
            pos_y,
            &cursor_range_pixels,
            &selection_pixel_range,
            width_scale,
            height_scale,
        )?;
        metrics::histogram!("render_screen_line").record(start.elapsed());

        Ok(RenderScreenLineResult {
            invalidate_on_hover_change,
        })
    }
}

include!("screen_line/glyphs.rs");
include!("screen_line/build.rs");
