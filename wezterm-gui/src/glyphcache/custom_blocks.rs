impl GlyphCache {
    pub fn block_sprite(
        &mut self,
        render_metrics: &super::utilsprites::RenderMetrics,
        key: SizedBlockKey,
    ) -> anyhow::Result<::window::bitmaps::atlas::Sprite> {
        let raster = wezterm_custom_glyph::rasterize_block(
            key.block,
            &wezterm_custom_glyph::RasterizeGlyphParams {
                underline_height: render_metrics.underline_height,
                cell_size: wezterm_custom_glyph::CellSize::new(
                    render_metrics.cell_size.width,
                    render_metrics.cell_size.height,
                ),
                anti_alias: config::configuration().anti_alias_custom_block_glyphs,
            },
        );
        let image = ::window::bitmaps::Image::from_raw(raster.width, raster.height, raster.data);
        let sprite = self.atlas.allocate(&image)?;
        self.block_glyphs.insert(key, sprite.clone());
        Ok(sprite)
    }
}
