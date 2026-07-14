#[cfg(test)]
mod test {
    use super::*;
    use crate::FontDatabase;
    use wezterm_config_types::FontAttributes;

    #[test]
    fn ligatures() {
        let _ = env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Trace)
            .try_init();

        let db = FontDatabase::with_built_in().unwrap();
        let handle = db
            .resolve(
                &FontAttributes {
                    family: "JetBrains Mono".into(),
                    stretch: Default::default(),
                    weight: Default::default(),
                    is_fallback: false,
                    is_synthetic: false,
                    style: Default::default(),
                    freetype_load_flags: None,
                    freetype_load_target: None,
                    freetype_render_target: None,
                    harfbuzz_features: None,
                    scale: None,
                    assume_emoji_presentation: None,
                },
                14,
            )
            .unwrap()
            .clone();

        let config = config::configuration();

        let shaper = HarfbuzzShaper::new(&config, &[handle]).unwrap();
        {
            let mut no_glyphs = vec![];
            let info = shaper
                .shape(
                    "abc",
                    10.,
                    72,
                    &mut no_glyphs,
                    None,
                    Direction::LeftToRight,
                    None,
                    None,
                )
                .unwrap();
            assert!(no_glyphs.is_empty(), "{:?}", no_glyphs);
            k9::snapshot!(
                info,
                r#"
[
    GlyphInfo {
        text: "a",
        only_char: Some(
            'a',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 0,
        font_idx: 0,
        glyph_pos: 189,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
    GlyphInfo {
        text: "b",
        only_char: Some(
            'b',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 1,
        font_idx: 0,
        glyph_pos: 214,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
    GlyphInfo {
        text: "c",
        only_char: Some(
            'c',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 2,
        font_idx: 0,
        glyph_pos: 215,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
]
"#
            );
        }
        {
            let mut no_glyphs = vec![];
            let info = shaper
                .shape(
                    "<",
                    10.,
                    72,
                    &mut no_glyphs,
                    None,
                    Direction::LeftToRight,
                    None,
                    None,
                )
                .unwrap();
            assert!(no_glyphs.is_empty(), "{:?}", no_glyphs);
            k9::snapshot!(
                info,
                r#"
[
    GlyphInfo {
        text: "<",
        only_char: Some(
            '<',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 0,
        font_idx: 0,
        glyph_pos: 1052,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
]
"#
            );
        }
        {
            // This is a ligatured sequence, but you wouldn't know
            // from this info :-/
            let mut no_glyphs = vec![];
            let info = shaper
                .shape(
                    "<-",
                    10.,
                    72,
                    &mut no_glyphs,
                    None,
                    Direction::LeftToRight,
                    None,
                    None,
                )
                .unwrap();
            assert!(no_glyphs.is_empty(), "{:?}", no_glyphs);
            k9::snapshot!(
                info,
                r#"
[
    GlyphInfo {
        text: "<",
        only_char: Some(
            '<',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 0,
        font_idx: 0,
        glyph_pos: 1742,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
    GlyphInfo {
        text: "-",
        only_char: Some(
            '-',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 1,
        font_idx: 0,
        glyph_pos: 1588,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
]
"#
            );
        }
        {
            let mut no_glyphs = vec![];
            let info = shaper
                .shape(
                    "<--",
                    10.,
                    72,
                    &mut no_glyphs,
                    None,
                    Direction::LeftToRight,
                    None,
                    None,
                )
                .unwrap();
            assert!(no_glyphs.is_empty(), "{:?}", no_glyphs);
            k9::snapshot!(
                info,
                r#"
[
    GlyphInfo {
        text: "<",
        only_char: Some(
            '<',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 0,
        font_idx: 0,
        glyph_pos: 1742,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
    GlyphInfo {
        text: "-",
        only_char: Some(
            '-',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 1,
        font_idx: 0,
        glyph_pos: 1742,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
    GlyphInfo {
        text: "-",
        only_char: Some(
            '-',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 2,
        font_idx: 0,
        glyph_pos: 1589,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
]
"#
            );
        }

        {
            let mut no_glyphs = vec![];
            let info = shaper
                .shape(
                    "x x",
                    10.,
                    72,
                    &mut no_glyphs,
                    None,
                    Direction::LeftToRight,
                    None,
                    None,
                )
                .unwrap();
            assert!(no_glyphs.is_empty(), "{:?}", no_glyphs);
            k9::snapshot!(
                info,
                r#"
[
    GlyphInfo {
        text: "x",
        only_char: Some(
            'x',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 0,
        font_idx: 0,
        glyph_pos: 367,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
    GlyphInfo {
        text: " ",
        only_char: Some(
            ' ',
        ),
        is_space: true,
        num_cells: 1,
        cluster: 1,
        font_idx: 0,
        glyph_pos: 958,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
    GlyphInfo {
        text: "x",
        only_char: Some(
            'x',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 2,
        font_idx: 0,
        glyph_pos: 367,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
]
"#
            );
        }

        {
            let mut no_glyphs = vec![];
            let info = shaper
                .shape(
                    "x\u{3000}x",
                    10.,
                    72,
                    &mut no_glyphs,
                    None,
                    Direction::LeftToRight,
                    None,
                    None,
                )
                .unwrap();
            assert!(no_glyphs.is_empty(), "{:?}", no_glyphs);
            k9::snapshot!(
                info,
                r#"
[
    GlyphInfo {
        text: "x",
        only_char: Some(
            'x',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 0,
        font_idx: 0,
        glyph_pos: 367,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
    GlyphInfo {
        text: "\u{3000}",
        only_char: Some(
            '\u{3000}',
        ),
        is_space: false,
        num_cells: 2,
        cluster: 1,
        font_idx: 0,
        glyph_pos: 958,
        x_advance: 10.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
    GlyphInfo {
        text: "x",
        only_char: Some(
            'x',
        ),
        is_space: false,
        num_cells: 1,
        cluster: 4,
        font_idx: 0,
        glyph_pos: 367,
        x_advance: 6.0,
        y_advance: 0.0,
        x_offset: 0.0,
        y_offset: 0.0,
    },
]
"#
            );
        }
    }
}
