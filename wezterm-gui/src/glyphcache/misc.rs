impl GlyphCache {
    pub fn cached_color(&mut self, color: RgbColor, alpha: f32) -> anyhow::Result<Sprite> {
        let key = (color, NotNan::new(alpha).unwrap());

        if let Some(s) = self.color.get(&key) {
            return Ok(s.clone());
        }

        let (red, green, blue) = color.to_tuple_rgb8();
        let alpha = (alpha * 255.0) as u8;

        let data = vec![
            red, green, blue, alpha, red, green, blue, alpha, red, green, blue, alpha, red, green,
            blue, alpha,
        ];
        let image = Image::from_raw(2, 2, data);

        let sprite = self.atlas.allocate(&image)?;
        self.color.insert(key, sprite.clone());
        Ok(sprite)
    }

    pub fn cached_block(
        &mut self,
        block: BlockKey,
        metrics: &RenderMetrics,
    ) -> anyhow::Result<Sprite> {
        let key = SizedBlockKey {
            block,
            size: metrics.into(),
        };
        if let Some(s) = self.block_glyphs.get(&key) {
            return Ok(s.clone());
        }
        self.block_sprite(metrics, key)
    }
}
