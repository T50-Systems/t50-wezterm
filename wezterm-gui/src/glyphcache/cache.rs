/// A number of items here are HashMaps rather than LfuCaches;
/// eviction is managed by recreating Self when the Atlas is filled
pub struct GlyphCache {
    glyph_cache: HashMap<GlyphKey, Rc<CachedGlyph>>,
    pub atlas: Atlas,
    pub fonts: Rc<FontConfiguration>,
    pub image_cache: LfuCache<[u8; 32], DecodedImage>,
    frame_cache: HashMap<[u8; 32], Sprite>,
    line_glyphs: HashMap<LineKey, Sprite>,
    pub block_glyphs: HashMap<SizedBlockKey, Sprite>,
    pub cursor_glyphs: HashMap<(Option<CursorShape>, u8), Sprite>,
    pub color: HashMap<(RgbColor, NotNan<f32>), Sprite>,
    min_frame_duration: Duration,
}

impl GlyphCache {
    pub fn new_in_memory(fonts: &Rc<FontConfiguration>, size: usize) -> anyhow::Result<Self> {
        let surface: Rc<dyn Texture2d> = Rc::new(ImageTexture::new(size, size));
        let atlas = Atlas::new(&surface).expect("failed to create new texture atlas");

        Ok(Self {
            fonts: Rc::clone(fonts),
            glyph_cache: HashMap::new(),
            image_cache: LfuCache::new(
                "glyph_cache.image_cache.hit.rate",
                "glyph_cache.image_cache.miss.rate",
                |config| config.glyph_cache_image_cache_size,
                &fonts.config(),
            ),
            frame_cache: HashMap::new(),
            atlas,
            line_glyphs: HashMap::new(),
            block_glyphs: HashMap::new(),
            cursor_glyphs: HashMap::new(),
            color: HashMap::new(),
            min_frame_duration: Duration::from_millis(1000 / fonts.config().max_fps as u64),
        })
    }
}

impl GlyphCache {
    pub fn new_gl(
        backend: &RenderContext,
        fonts: &Rc<FontConfiguration>,
        size: usize,
    ) -> anyhow::Result<Self> {
        let surface = backend.allocate_texture_atlas(size)?;
        let atlas = Atlas::new(&surface).expect("failed to create new texture atlas");

        Ok(Self {
            fonts: Rc::clone(fonts),
            glyph_cache: HashMap::new(),
            image_cache: LfuCache::new(
                "glyph_cache.image_cache.hit.rate",
                "glyph_cache.image_cache.miss.rate",
                |config| config.glyph_cache_image_cache_size,
                &fonts.config(),
            ),
            frame_cache: HashMap::new(),
            atlas,
            line_glyphs: HashMap::new(),
            block_glyphs: HashMap::new(),
            cursor_glyphs: HashMap::new(),
            color: HashMap::new(),
            min_frame_duration: Duration::from_millis(1000 / fonts.config().max_fps as u64),
        })
    }
}
