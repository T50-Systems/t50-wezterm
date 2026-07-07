/// A clean, modern and stylish set of decorations.
///
/// This class draws clean and modern decorations with
/// buttons inspired by breeze, material hover shade and
/// a white header background.
///
/// `ConceptFrame` is hiding its `ClientSide` decorations
/// in a `Fullscreen` state and brings them back if those are
/// visible when unsetting `Fullscreen` state.
pub struct ConceptFrame {
    base_surface: wl_surface::WlSurface,
    compositor: Attached<wl_compositor::WlCompositor>,
    subcompositor: Attached<wl_subcompositor::WlSubcompositor>,
    inner: Rc<RefCell<Inner>>,
    pools: DoubleMemPool,
    active: WindowState,
    hidden: bool,
    pointers: Vec<ThemedPointer>,
    themer: ThemeManager,
    surface_version: u32,
    config: ConceptConfig,
    title: Option<String>,
    shaped_title: Option<ShapedTitle>,
}

struct ShapedTitle {
    title: String,
    glyphs: Vec<ShapedGlyph>,
    metrics: FontMetrics,
    state: WindowState,
    dpi: usize,
}

struct ShapedGlyph {
    info: GlyphInfo,
    glyph: RasterizedGlyph,
}

impl ConceptFrame {
    fn reshape_title(&mut self) -> Option<()> {
        let font_config = self.config.font_config.as_ref()?;
        let title = self.title.as_deref().unwrap_or("");
        if title.is_empty() {
            self.title.take();
            self.shaped_title.take();
            return Some(());
        }

        if let Some(existing) = self.shaped_title.as_ref() {
            if existing.title == title
                && existing.state == self.active
                && existing.dpi == font_config.get_dpi()
            {
                return Some(());
            }
        }

        let font = font_config.title_font().ok()?;
        let metrics = font.metrics();
        let infos = font
            .shape(
                title,
                || {
                    // TODO: font fallback completed, trigger title repaint!
                },
                |_| {
                    // We don't do synthesis here, so no need to filter
                },
                None,
                wezterm_bidi::Direction::LeftToRight,
                None,
                None,
            )
            .ok()?;

        let mut glyphs = vec![];
        let colors = self.config.colors();
        let title_color = match self.active {
            WindowState::Active => colors.active_titlebar_fg,
            WindowState::Inactive => colors.inactive_titlebar_fg,
        };

        for info in infos {
            if let Ok(mut glyph) = font.rasterize_glyph(info.glyph_pos, info.font_idx) {
                // fixup colors: they need to be switched to the appropriate
                // pixel format, and for monochrome font data we need to tint
                // it with their preferred title color
                if let Some(mut data) =
                    PixmapMut::from_bytes(&mut glyph.data, glyph.width as u32, glyph.height as u32)
                {
                    for p in data.pixels_mut() {
                        let c = p.demultiply();
                        let (r, g, b, a) = (c.red(), c.green(), c.blue(), c.alpha());
                        if glyph.has_color {
                            *p = ColorU8::from_rgba(b, g, r, a).premultiply();
                        } else {
                            // Apply the preferred title color
                            *p = ColorU8::from_rgba(
                                ((b as f32 / 255.) * (title_color.0 * 255.)) as u8,
                                ((g as f32 / 255.) * (title_color.1 * 255.)) as u8,
                                ((r as f32 / 255.) * (title_color.2 * 255.)) as u8,
                                a,
                            )
                            .premultiply();
                        }
                    }
                }

                glyphs.push(ShapedGlyph { info, glyph });
            }
        }

        self.shaped_title.replace(ShapedTitle {
            title: title.to_string(),
            glyphs,
            metrics,
            state: self.active,
            dpi: font_config.get_dpi(),
        });

        Some(())
    }

    fn showing_title_bar(&self, inner: &Inner) -> bool {
        if self.hidden || inner.fullscreened {
            false
        } else {
            self.config
                .config
                .window_decorations
                .contains(WindowDecorations::TITLE)
        }
    }
}
