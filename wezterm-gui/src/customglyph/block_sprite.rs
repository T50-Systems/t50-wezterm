use super::*;

impl GlyphCache {
    pub fn block_sprite(
        &mut self,
        render_metrics: &RenderMetrics,
        key: SizedBlockKey,
    ) -> anyhow::Result<Sprite> {
        let metrics = match &key.block {
            BlockKey::PolyWithCustomMetrics {
                underline_height,
                cell_size,
                ..
            } => RenderMetrics {
                descender: PixelLength::new(0.),
                descender_row: 0,
                descender_plus_two: 0,
                underline_height: *underline_height,
                strike_row: 0,
                cell_size: cell_size.clone(),
            },
            _ => render_metrics.clone(),
        };

        let mut buffer = Image::new(
            metrics.cell_size.width as usize,
            metrics.cell_size.height as usize,
        );
        let black = SrgbaPixel::rgba(0, 0, 0, 0);

        let cell_rect = Rect::new(Point::new(0, 0), metrics.cell_size);

        buffer.clear_rect(cell_rect, black);

        if self
            .block_sprite_part1(key.block, &metrics, &mut buffer)
            .is_none()
            && self
                .block_sprite_part2(key.block, &metrics, &mut buffer)
                .is_none()
        {
            self.block_sprite_part3(key.block, &metrics, &mut buffer);
        }

        /*
        log::info!("{:?}", block);
        buffer.log_bits();
        */

        let sprite = self.atlas.allocate(&buffer)?;
        self.block_glyphs.insert(key, sprite.clone());
        Ok(sprite)
    }
}
