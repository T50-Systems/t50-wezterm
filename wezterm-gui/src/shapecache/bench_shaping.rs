#[test]
fn bench_shaping() {
    config::use_test_configuration();

    // let mut glyph_cache = GlyphCache::new_in_memory(&fonts, 128, &render_metrics).unwrap();
    // let render_metrics = RenderMetrics::new(&fonts).unwrap();

    benchmarking::warm_up();

    for &n in &[100, 1000, 10_000] {
        let bench_result = benchmarking::measure_function(move |measurer| {
            let text: String = (0..n).map(|_| ' ').collect();

            let fonts = Rc::new(
                FontConfiguration::new(
                    None,
                    config::configuration()
                        .dpi
                        .unwrap_or_else(|| ::window::default_dpi()) as usize,
                )
                .unwrap(),
            );
            let style = TextStyle::default();
            let font = fonts.resolve_font(&style).unwrap();
            let line = Line::from_text(&text, &CellAttributes::default(), SEQ_ZERO, None);
            let cell_clusters = line.cluster(None);
            let cluster = &cell_clusters[0];
            let presentation_width = PresentationWidth::with_cluster(&cluster);

            measurer.measure(|| {
                let _x = font
                    .shape(
                        &cluster.text,
                        || {},
                        |_| {},
                        None,
                        Direction::LeftToRight,
                        None,
                        Some(&presentation_width),
                    )
                    .unwrap();
                // println!("{:?}", &x[0..2]);
            });
        })
        .unwrap();
        println!("{}: {:?}", n, bench_result.elapsed());
    }
}
