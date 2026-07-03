use crate::quad::TripleLayerQuadAllocator;
use crate::tabbar::TabBarState;
use crate::termwindow::render::RenderScreenLineParams;
use crate::utilsprites::RenderMetrics;
use config::ConfigHandle;
use mux::renderable::RenderableDimensions;
use wezterm_term::color::ColorAttribute;
use window::color::LinearRgba;

#[derive(Clone, Copy, Debug, PartialEq)]
struct BarGeometryInput {
    pixel_height: usize,
    border_top: f32,
    border_bottom: f32,
    tab_bar_height: f32,
    show_tab_bar: bool,
    tab_bar_at_bottom: bool,
    secondary_enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct BarGeometry {
    primary_y: Option<f32>,
    secondary_y: Option<f32>,
    top_height: f32,
    bottom_height: f32,
}

impl BarGeometry {
    fn compute(input: BarGeometryInput) -> Self {
        if !input.show_tab_bar {
            return Self {
                primary_y: None,
                secondary_y: None,
                top_height: 0.,
                bottom_height: 0.,
            };
        }

        let bottom_y =
            ((input.pixel_height as f32) - (input.tab_bar_height + input.border_bottom)).max(0.);

        if input.tab_bar_at_bottom {
            Self {
                primary_y: Some(bottom_y),
                secondary_y: input.secondary_enabled.then_some(input.border_top),
                top_height: if input.secondary_enabled {
                    input.tab_bar_height
                } else {
                    0.
                },
                bottom_height: input.tab_bar_height,
            }
        } else {
            Self {
                primary_y: Some(input.border_top),
                secondary_y: input.secondary_enabled.then_some(bottom_y),
                top_height: input.tab_bar_height,
                bottom_height: if input.secondary_enabled {
                    input.tab_bar_height
                } else {
                    0.
                },
            }
        }
    }
}

impl crate::TermWindow {
    pub fn secondary_tab_bar_enabled(&self) -> bool {
        self.show_tab_bar && self.config.enable_secondary_bar
    }

    fn bar_geometry(&self) -> anyhow::Result<BarGeometry> {
        let border = self.get_os_border();
        Ok(BarGeometry::compute(BarGeometryInput {
            pixel_height: self.dimensions.pixel_height,
            border_top: border.top.get() as f32,
            border_bottom: border.bottom.get() as f32,
            tab_bar_height: self.tab_bar_pixel_height()?,
            show_tab_bar: self.show_tab_bar,
            tab_bar_at_bottom: self.config.tab_bar_at_bottom,
            secondary_enabled: self.secondary_tab_bar_enabled(),
        }))
    }

    pub fn top_bar_pixel_height(&self) -> f32 {
        self.bar_geometry().map(|g| g.top_height).unwrap_or(0.)
    }

    pub fn bottom_bar_pixel_height(&self) -> f32 {
        self.bar_geometry().map(|g| g.bottom_height).unwrap_or(0.)
    }

    pub fn total_bar_pixel_height(&self) -> f32 {
        self.top_bar_pixel_height() + self.bottom_bar_pixel_height()
    }

    pub fn primary_tab_bar_y(&self) -> anyhow::Result<f32> {
        Ok(self.bar_geometry()?.primary_y.unwrap_or(0.))
    }

    pub fn secondary_tab_bar_y(&self) -> anyhow::Result<Option<f32>> {
        Ok(self.bar_geometry()?.secondary_y)
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

#[cfg(test)]
mod bar_geometry_tests {
    use super::{BarGeometry, BarGeometryInput};

    fn input(
        show_tab_bar: bool,
        tab_bar_at_bottom: bool,
        secondary_enabled: bool,
    ) -> BarGeometryInput {
        BarGeometryInput {
            pixel_height: 100,
            border_top: 3.,
            border_bottom: 5.,
            tab_bar_height: 10.,
            show_tab_bar,
            tab_bar_at_bottom,
            secondary_enabled,
        }
    }

    #[test]
    fn no_tabbar_has_no_bars_or_offsets() {
        let geometry = BarGeometry::compute(input(false, false, true));

        assert_eq!(geometry.primary_y, None);
        assert_eq!(geometry.secondary_y, None);
        assert_eq!(geometry.top_height, 0.);
        assert_eq!(geometry.bottom_height, 0.);
    }

    #[test]
    fn top_primary_without_secondary_offsets_content_top() {
        let geometry = BarGeometry::compute(input(true, false, false));

        assert_eq!(geometry.primary_y, Some(3.));
        assert_eq!(geometry.secondary_y, None);
        assert_eq!(geometry.top_height, 10.);
        assert_eq!(geometry.bottom_height, 0.);
    }

    #[test]
    fn bottom_primary_without_secondary_offsets_content_bottom() {
        let geometry = BarGeometry::compute(input(true, true, false));

        assert_eq!(geometry.primary_y, Some(85.));
        assert_eq!(geometry.secondary_y, None);
        assert_eq!(geometry.top_height, 0.);
        assert_eq!(geometry.bottom_height, 10.);
    }

    #[test]
    fn top_primary_with_secondary_places_secondary_at_bottom() {
        let geometry = BarGeometry::compute(input(true, false, true));

        assert_eq!(geometry.primary_y, Some(3.));
        assert_eq!(geometry.secondary_y, Some(85.));
        assert_eq!(geometry.top_height, 10.);
        assert_eq!(geometry.bottom_height, 10.);
    }

    #[test]
    fn bottom_primary_with_secondary_places_secondary_at_top() {
        let geometry = BarGeometry::compute(input(true, true, true));

        assert_eq!(geometry.primary_y, Some(85.));
        assert_eq!(geometry.secondary_y, Some(3.));
        assert_eq!(geometry.top_height, 10.);
        assert_eq!(geometry.bottom_height, 10.);
    }

    #[test]
    fn small_window_height_clamps_bottom_bar_to_zero() {
        let geometry = BarGeometry::compute(BarGeometryInput {
            pixel_height: 8,
            border_top: 3.,
            border_bottom: 5.,
            tab_bar_height: 10.,
            show_tab_bar: true,
            tab_bar_at_bottom: true,
            secondary_enabled: true,
        });

        assert_eq!(geometry.primary_y, Some(0.));
        assert_eq!(geometry.secondary_y, Some(3.));
    }
}
