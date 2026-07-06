#[test]
fn ligatures_jetbrains() {
    config::use_test_configuration();
    let _ = env_logger::Builder::new()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();
    let config = config::configuration();

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
        cluster_and_shape(&render_metrics, &mut glyph_cache, &style, &font, "ab"),
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
        glyph_idx: 214,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 1.0,
        bitmap_pixel_width: 8,
    },
]
"
    );

    k9::snapshot!(
        cluster_and_shape(&render_metrics, &mut glyph_cache, &style, &font, "a b"),
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
        glyph_idx: 958,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 0,
    },
    GlyphPosition {
        glyph_idx: 214,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 1.0,
        bitmap_pixel_width: 8,
    },
]
"
    );

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

    k9::snapshot!(
        cluster_and_shape(&render_metrics, &mut glyph_cache, &style, &font, "e_or_"),
        "
[
    GlyphPosition {
        glyph_idx: 225,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 1.0,
        bitmap_pixel_width: 8,
    },
    GlyphPosition {
        glyph_idx: 860,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 9,
    },
    GlyphPosition {
        glyph_idx: 290,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 1.0,
        bitmap_pixel_width: 8,
    },
    GlyphPosition {
        glyph_idx: 320,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 1.0,
        bitmap_pixel_width: 8,
    },
    GlyphPosition {
        glyph_idx: 860,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 9,
    },
]
"
    );

    k9::snapshot!(
        cluster_and_shape(&render_metrics, &mut glyph_cache, &style, &font, "a  b"),
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
        glyph_idx: 958,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 0,
    },
    GlyphPosition {
        glyph_idx: 958,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 0,
    },
    GlyphPosition {
        glyph_idx: 214,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 1.0,
        bitmap_pixel_width: 8,
    },
]
"
    );

    k9::snapshot!(
        cluster_and_shape(&render_metrics, &mut glyph_cache, &style, &font, "<-"),
        "
[
    GlyphPosition {
        glyph_idx: 1742,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 0,
    },
    GlyphPosition {
        glyph_idx: 1588,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: -9.0,
        bitmap_pixel_width: 17,
    },
]
"
    );

    k9::snapshot!(
        cluster_and_shape(&render_metrics, &mut glyph_cache, &style, &font, "<>"),
        "
[
    GlyphPosition {
        glyph_idx: 1742,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 0,
    },
    GlyphPosition {
        glyph_idx: 1613,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: -8.0,
        bitmap_pixel_width: 16,
    },
]
"
    );

    k9::snapshot!(
        cluster_and_shape(&render_metrics, &mut glyph_cache, &style, &font, "|=>"),
        "
[
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
        glyph_idx: 1562,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: -18.0,
        bitmap_pixel_width: 27,
    },
]
"
    );

    let block_bottom_one_eighth = "\u{2581}";
    k9::snapshot!(
        cluster_and_shape(
            &render_metrics,
            &mut glyph_cache,
            &style,
            &font,
            block_bottom_one_eighth
        ),
        "
[
    GlyphPosition {
        glyph_idx: 1178,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 10,
    },
]
"
    );

    let powerline_extra_honeycomb = "\u{e0cc}";
    k9::snapshot!(
        cluster_and_shape(
            &render_metrics,
            &mut glyph_cache,
            &style,
            &font,
            powerline_extra_honeycomb,
        ),
        "
[
    GlyphPosition {
        glyph_idx: 58,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: -0.8333333,
        bitmap_pixel_width: 12,
    },
]
"
    );

    k9::snapshot!(
        cluster_and_shape(&render_metrics, &mut glyph_cache, &style, &font, "<!--"),
        "
[
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
        glyph_idx: 1742,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 0,
    },
    GlyphPosition {
        glyph_idx: 1595,
        num_cells: 1,
        x_offset: 0.0,
        bearing_x: -28.0,
        bitmap_pixel_width: 37,
    },
]
"
    );

    let deaf_man_medium_light_skin_tone = "\u{1F9CF}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}";
    println!(
        "deaf_man_medium_light_skin_tone: {}",
        deaf_man_medium_light_skin_tone
    );
    k9::snapshot!(
        cluster_and_shape(
            &render_metrics,
            &mut glyph_cache,
            &style,
            &font,
            deaf_man_medium_light_skin_tone
        ),
        "
[
    GlyphPosition {
        glyph_idx: 2712,
        num_cells: 2,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 16,
    },
]
"
    );

    let england_flag = "\u{1F3F4}\u{E0067}\u{E0062}\u{E0065}\u{E006E}\u{E0067}\u{E007F}";
    println!("england_flag: {}", england_flag);
    k9::snapshot!(
        cluster_and_shape(
            &render_metrics,
            &mut glyph_cache,
            &style,
            &font,
            england_flag
        ),
        "
[
    GlyphPosition {
        glyph_idx: 3855,
        num_cells: 2,
        x_offset: 0.0,
        bearing_x: 0.0,
        bitmap_pixel_width: 20,
    },
]
"
    );
}
