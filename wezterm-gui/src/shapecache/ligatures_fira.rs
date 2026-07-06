#[test]
fn ligatures_fira() {
    config::use_test_configuration();
    let _ = env_logger::Builder::new()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();

    let config = config::configuration();

    let mut config: config::Config = (*config).clone();
    config.font = TextStyle {
        font: vec![FontAttributes::new("Fira Code")],
        foreground: None,
    };
    config.font_rules.clear();
    config.compute_extra_defaults(None);
    config::use_this_configuration(config.clone());

    let fonts = Rc::new(
        FontConfiguration::new(
            None,
            config.dpi.unwrap_or_else(|| ::window::default_dpi()) as usize,
        )
        .unwrap(),
    );
    let render_metrics = RenderMetrics::new(&fonts).unwrap();
    let mut glyph_cache = GlyphCache::new_in_memory(&fonts, 128).unwrap();

    let style = TextStyle::default();
    let font = fonts.resolve_font(&style).unwrap();

    k9::snapshot!(
        cluster_and_shape(&render_metrics, &mut glyph_cache, &style, &font, "a..."),
        "
[
    GlyphPosition {
        glyph_idx: 189,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 1.0,
        bitmap_pixel_width: 8,
    },
    GlyphPosition {
        glyph_idx: 1742,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 0,
    },
    GlyphPosition {
        glyph_idx: 1742,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 0,
    },
    GlyphPosition {
        glyph_idx: 896,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: -15.0,
        bitmap_pixel_width: 20,
    },
]
"
    );
}
