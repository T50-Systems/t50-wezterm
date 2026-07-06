use crate::glyphcache::GlyphCache;
use crate::shapecache::{GlyphPosition, ShapedInfo};
use crate::utilsprites::RenderMetrics;
use config::{FontAttributes, TextStyle};
use std::rc::Rc;
use termwiz::cell::CellAttributes;
use termwiz::surface::{Line, SEQ_ZERO};
use wezterm_bidi::Direction;
use wezterm_font::shaper::PresentationWidth;
use wezterm_font::{FontConfiguration, LoadedFont};

fn cluster_and_shape(
    render_metrics: &RenderMetrics,
    glyph_cache: &mut GlyphCache,
    style: &TextStyle,
    font: &Rc<LoadedFont>,
    text: &str,
) -> Vec<GlyphPosition> {
    let line = Line::from_text(text, &CellAttributes::default(), SEQ_ZERO, None);
    eprintln!("{:?}", line);
    let mut all_infos = vec![];
    let mut all_glyphs = vec![];

    for cluster in line.cluster(None) {
        let presentation_width = PresentationWidth::with_cluster(&cluster);
        let mut infos = font
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
        let mut glyphs = infos
            .iter()
            .map(|info| {
                let cell_idx = cluster.byte_to_cell_idx(info.cluster as usize);
                let num_cells = cluster.byte_to_cell_width(info.cluster as usize);

                let followed_by_space = match line.get_cell(cell_idx + 1) {
                    Some(cell) => cell.str() == " ",
                    None => false,
                };

                glyph_cache
                    .cached_glyph(
                        info,
                        &style,
                        followed_by_space,
                        font,
                        render_metrics,
                        num_cells,
                    )
                    .unwrap()
            })
            .collect::<Vec<_>>();

        all_infos.append(&mut infos);
        all_glyphs.append(&mut glyphs);
    }

    eprintln!("infos: {:#?}", all_infos);
    eprintln!("glyphs: {:#?}", all_glyphs);
    ShapedInfo::process(&all_infos, &all_glyphs)
        .into_iter()
        .map(|p| p.pos)
        .collect()
}
