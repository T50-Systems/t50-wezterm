use super::*;

fn resolved_metrics(block: BlockKey, metrics: &RasterizeGlyphParams) -> RasterizeGlyphParams {
    match block {
        BlockKey::PolyWithCustomMetrics { metrics: custom, .. } => RasterizeGlyphParams {
            underline_height: custom.underline_height,
            cell_size: custom.cell_size,
            anti_alias: metrics.anti_alias,
        },
        _ => *metrics,
    }
}

pub fn rasterize_polys(polys: &[Poly], metrics: &RasterizeGlyphParams) -> RasterizedGlyph {
    let width = metrics.cell_size.width as usize;
    let height = metrics.cell_size.height as usize;
    let mut buffer = Pixmap::new(width as u32, height as u32).expect("valid glyph size");
    draw_polys(metrics, polys, &mut buffer, poly_aa(metrics.anti_alias), BlendMode::default());
    RasterizedGlyph {
        width,
        height,
        data: buffer.data().to_vec(),
    }
}

pub fn rasterize_block(block: BlockKey, metrics: &RasterizeGlyphParams) -> RasterizedGlyph {
    let metrics = resolved_metrics(block, metrics);
    let width = metrics.cell_size.width as usize;
    let height = metrics.cell_size.height as usize;
    let mut buffer = Pixmap::new(width as u32, height as u32).expect("valid glyph size");

    if crate::block_sprite_part1::block_sprite_part1(block, &metrics, &mut buffer).is_none()
        && crate::block_sprite_part2::block_sprite_part2(block, &metrics, &mut buffer).is_none()
    {
        crate::block_sprite_part3::block_sprite_part3(block, &metrics, &mut buffer);
    }

    RasterizedGlyph {
        width,
        height,
        data: buffer.data().to_vec(),
    }
}
