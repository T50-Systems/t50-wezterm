use crate::quad::TripleLayerQuadAllocator;
use crate::tabbar::TabBarState;
use crate::termwindow::render::RenderScreenLineParams;
use crate::utilsprites::RenderMetrics;
use config::ConfigHandle;
use mux::renderable::RenderableDimensions;
use wezterm_term::color::ColorAttribute;
use window::color::LinearRgba;

impl crate::TermWindow {
    pub fn secondary_tab_bar_enabled(&self) -> bool {
        self.show_tab_bar && self.config.enable_secondary_bar
    }

    pub fn top_bar_pixel_height(&self) -> f32 {
        if !self.show_tab_bar {
            return 0.;
        }

        let height = self.tab_bar_pixel_height().unwrap_or(0.);
        if self.config.tab_bar_at_bottom {
            if self.secondary_tab_bar_enabled() {
                height
            } else {
                0.
            }
        } else {
            height
        }
    }

    pub fn bottom_bar_pixel_height(&self) -> f32 {
        if !self.show_tab_bar {
            return 0.;
        }

        let height = self.tab_bar_pixel_height().unwrap_or(0.);
        if self.config.tab_bar_at_bottom {
            height
        } else if self.secondary_tab_bar_enabled() {
            height
        } else {
            0.
        }
    }

    pub fn total_bar_pixel_height(&self) -> f32 {
        self.top_bar_pixel_height() + self.bottom_bar_pixel_height()
    }

    pub fn primary_tab_bar_y(&self) -> anyhow::Result<f32> {
        let border = self.get_os_border();
        let tab_bar_height = self.tab_bar_pixel_height()?;
        Ok(if self.config.tab_bar_at_bottom {
            ((self.dimensions.pixel_height as f32) - (tab_bar_height + border.bottom.get() as f32))
                .max(0.)
        } else {
            border.top.get() as f32
        })
    }

    pub fn secondary_tab_bar_y(&self) -> anyhow::Result<Option<f32>> {
        if !self.secondary_tab_bar_enabled() {
            return Ok(None);
        }

        let border = self.get_os_border();
        let tab_bar_height = self.tab_bar_pixel_height()?;
        Ok(Some(if self.config.tab_bar_at_bottom {
            border.top.get() as f32
        } else {
            ((self.dimensions.pixel_height as f32) - (tab_bar_height + border.bottom.get() as f32))
                .max(0.)
        }))
    }

    fn paint_one_tab_bar(
        &mut self,
        layers: &mut TripleLayerQuadAllocator,
        tab_bar: &TabBarState,
        tab_bar_y: f32,
    ) -> anyhow::Result<()> {
        self.ui_items.append(&mut tab_bar.compute_ui_items(
            tab_bar_y as usize,
            self.render_metrics.cell_size.height as usize,
            self.render_metrics.cell_size.width as usize,
        ));

        let palette = self.palette().clone();
        let window_is_transparent =
            !self.window_background.is_empty() || self.config.window_background_opacity != 1.0;
        let gl_state = self.render_state.as_ref().unwrap();
        let white_space = gl_state.util_sprites.white_space.texture_coords();
        let filled_box = gl_state.util_sprites.filled_box.texture_coords();
        let default_bg = palette
            .resolve_bg(ColorAttribute::Default)
            .to_linear()
            .mul_alpha(if window_is_transparent {
                0.
            } else {
                self.config.text_background_opacity
            });
        let line = tab_bar.line().clone();

        self.render_screen_line(
            RenderScreenLineParams {
                top_pixel_y: tab_bar_y,
                left_pixel_x: 0.,
                pixel_width: self.dimensions.pixel_width as f32,
                stable_line_idx: None,
                line: &line,
                selection: 0..0,
                cursor: &Default::default(),
                palette: &palette,
                dims: &RenderableDimensions {
                    cols: self.dimensions.pixel_width
                        / self.render_metrics.cell_size.width as usize,
                    physical_top: 0,
                    scrollback_rows: 0,
                    scrollback_top: 0,
                    viewport_rows: 1,
                    dpi: self.terminal_size.dpi,
                    pixel_height: self.render_metrics.cell_size.height as usize,
                    pixel_width: self.terminal_size.pixel_width,
                    reverse_video: false,
                },
                config: &self.config,
                cursor_border_color: LinearRgba::default(),
                foreground: palette.foreground.to_linear(),
                pane: None,
                is_active: true,
                selection_fg: LinearRgba::default(),
                selection_bg: LinearRgba::default(),
                cursor_fg: LinearRgba::default(),
                cursor_bg: LinearRgba::default(),
                cursor_is_default_color: true,
                white_space,
                filled_box,
                window_is_transparent,
                default_bg,
                style: None,
                font: None,
                use_pixel_positioning: self.config.experimental_pixel_positioning,
                render_metrics: self.render_metrics,
                shape_key: None,
                password_input: false,
            },
            layers,
        )?;

        Ok(())
    }

    pub fn paint_tab_bar(&mut self, layers: &mut TripleLayerQuadAllocator) -> anyhow::Result<()> {
        if self.config.use_fancy_tab_bar {
            if self.fancy_tab_bar.is_none() {
                let palette = self.palette().clone();
                let tab_bar = self.build_fancy_tab_bar(&palette)?;
                self.fancy_tab_bar.replace(tab_bar);
            }
            if self.secondary_tab_bar_enabled() && self.fancy_secondary_tab_bar.is_none() {
                let palette = self.palette().clone();
                let tab_bar = self.build_fancy_secondary_tab_bar(&palette)?;
                self.fancy_secondary_tab_bar.replace(tab_bar);
            }

            self.ui_items.append(&mut self.paint_fancy_tab_bar()?);
            if self.secondary_tab_bar_enabled() {
                self.ui_items
                    .append(&mut self.paint_fancy_secondary_tab_bar()?);
            }
            return Ok(());
        }

        let primary_y = self.primary_tab_bar_y()?;
        let primary_bar = self.tab_bar.clone();
        self.paint_one_tab_bar(layers, &primary_bar, primary_y)?;

        if self.secondary_tab_bar_enabled() {
            if let Some(secondary_y) = self.secondary_tab_bar_y()? {
                let secondary_bar = self.secondary_tab_bar.clone();
                self.paint_one_tab_bar(layers, &secondary_bar, secondary_y)?;
            }
        }

        Ok(())
    }

    pub fn tab_bar_pixel_height_impl(
        config: &ConfigHandle,
        fontconfig: &wezterm_font::FontConfiguration,
        render_metrics: &RenderMetrics,
    ) -> anyhow::Result<f32> {
        if config.use_fancy_tab_bar {
            let font = fontconfig.title_font()?;
            Ok((font.metrics().cell_height.get() as f32 * 1.75).ceil())
        } else {
            Ok(render_metrics.cell_size.height as f32)
        }
    }

    pub fn tab_bar_pixel_height(&self) -> anyhow::Result<f32> {
        Self::tab_bar_pixel_height_impl(&self.config, &self.fonts, &self.render_metrics)
    }
}
