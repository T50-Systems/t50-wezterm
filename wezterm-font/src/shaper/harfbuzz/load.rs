impl HarfbuzzShaper {
    pub fn new(config: &ConfigHandle, handles: &[ParsedFont]) -> anyhow::Result<Self> {
        let lib = ftwrap::Library::new()?;
        let handles = handles.to_vec();
        let mut fonts = vec![];
        for _ in 0..handles.len() {
            fonts.push(RefCell::new(None));
        }

        let lang = harfbuzz::language_from_string("en")?;

        let features: Vec<harfbuzz::hb_feature_t> = config
            .harfbuzz_features
            .iter()
            .filter_map(|s| harfbuzz::feature_from_string(s).ok())
            .collect();

        Ok(Self {
            fonts,
            handles,
            lib,
            metrics: RefCell::new(HashMap::new()),
            features,
            lang,
        })
    }

    fn load_fallback(
        &self,
        font_idx: FallbackIdx,
        dpi: u32,
    ) -> anyhow::Result<Option<RefMut<'_, FontPair>>> {
        if font_idx >= self.handles.len() {
            return Ok(None);
        }
        match self.fonts.get(font_idx) {
            None => Ok(None),
            Some(opt_pair) => {
                let mut opt_pair = opt_pair.borrow_mut();
                if opt_pair.is_none() {
                    let handle = &self.handles[font_idx];
                    log::trace!("shaper wants {} {:?}", font_idx, handle);
                    let face = self.lib.face_from_locator(&handle.handle)?;

                    let font = if USE_OT_FACE {
                        harfbuzz::Font::from_locator(&handle.handle)?
                    } else {
                        let (load_flags, _) = ftwrap::compute_load_flags_from_config(
                            handle.freetype_load_flags,
                            handle.freetype_load_target,
                            handle.freetype_render_target,
                            Some(dpi),
                        );
                        let mut font = harfbuzz::Font::new(face.face);
                        font.set_load_flags(load_flags);
                        font
                    };

                    let features = match &handle.harfbuzz_features {
                        Some(features) => features
                            .iter()
                            .filter_map(|s| harfbuzz::feature_from_string(s).ok())
                            .collect(),
                        None => self.features.clone(),
                    };

                    *opt_pair = Some(FontPair {
                        face,
                        font: RefCell::new(font),
                        shaped_any: false,
                        presentation: if handle.assume_emoji_presentation {
                            Presentation::Emoji
                        } else {
                            Presentation::Text
                        },
                        features,
                        last_size_and_dpi: RefCell::new(None),
                    });
                }

                Ok(Some(RefMut::map(opt_pair, |opt_pair| {
                    opt_pair.as_mut().unwrap()
                })))
            }
        }
    }
}
