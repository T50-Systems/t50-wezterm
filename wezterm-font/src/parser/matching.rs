impl ParsedFont {
    /// Perform CSS Fonts Level 3 font matching.
    /// This implementation is derived from the `find_best_match` function
    /// in the font-kit crate which is
    /// Copyright © 2018 The Pathfinder Project Developers.
    /// https://drafts.csswg.org/css-fonts-3/#font-style-matching says
    pub fn best_matching_index<P: std::ops::Deref<Target = Self> + std::fmt::Debug>(
        attr: &FontAttributes,
        fonts: &[P],
        pixel_size: u16,
    ) -> Option<usize> {
        if fonts.is_empty() {
            return None;
        }

        let mut candidates: Vec<usize> = (0..fonts.len()).collect();

        // First, filter by stretch
        let stretch_value = attr.stretch.to_opentype_stretch();
        let stretch = if candidates
            .iter()
            .any(|&idx| fonts[idx].stretch == attr.stretch)
        {
            attr.stretch
        } else if attr.stretch <= FontStretch::Normal {
            // Find the closest stretch, looking at narrower first before
            // looking at wider candidates
            match candidates
                .iter()
                .filter(|&&idx| fonts[idx].stretch < attr.stretch)
                .min_by_key(|&&idx| stretch_value - fonts[idx].stretch.to_opentype_stretch())
            {
                Some(&idx) => fonts[idx].stretch,
                None => {
                    let idx = *candidates.iter().min_by_key(|&&idx| {
                        fonts[idx].stretch.to_opentype_stretch() - stretch_value
                    })?;
                    fonts[idx].stretch
                }
            }
        } else {
            // Look at wider values, then narrower values
            match candidates
                .iter()
                .filter(|&&idx| fonts[idx].stretch > attr.stretch)
                .min_by_key(|&&idx| fonts[idx].stretch.to_opentype_stretch() - stretch_value)
            {
                Some(&idx) => fonts[idx].stretch,
                None => {
                    let idx = *candidates.iter().min_by_key(|&&idx| {
                        stretch_value - fonts[idx].stretch.to_opentype_stretch()
                    })?;
                    fonts[idx].stretch
                }
            }
        };

        // Reduce to matching stretches
        candidates.retain(|&idx| fonts[idx].stretch == stretch);

        // Now match style: italics.
        let styles = match attr.style {
            FontStyle::Normal => [FontStyle::Normal, FontStyle::Italic, FontStyle::Oblique],
            FontStyle::Italic => [FontStyle::Italic, FontStyle::Oblique, FontStyle::Normal],
            FontStyle::Oblique => [FontStyle::Oblique, FontStyle::Italic, FontStyle::Normal],
        };
        let style = *styles
            .iter()
            .find(|&&style| candidates.iter().any(|&idx| fonts[idx].style == style))?;

        // Reduce to matching italics
        candidates.retain(|&idx| fonts[idx].style == style);

        // And now match by font weight
        let query_weight = attr.weight.to_opentype_weight();
        let weight = if candidates
            .iter()
            .any(|&idx| fonts[idx].weight == attr.weight)
        {
            // Exact match for the requested weight
            attr.weight
        } else if attr.weight == FontWeight::REGULAR
            && candidates
                .iter()
                .any(|&idx| fonts[idx].weight == FontWeight::MEDIUM)
        {
            // https://drafts.csswg.org/css-fonts-3/#font-style-matching says
            // that if they want weight=400 and we don't have 400,
            // look at weight 500 first
            FontWeight::MEDIUM
        } else if attr.weight == FontWeight::MEDIUM
            && candidates
                .iter()
                .any(|&idx| fonts[idx].weight == FontWeight::REGULAR)
        {
            // Similarly, look at regular before Medium if they wanted
            // Medium and we didn't have it
            FontWeight::REGULAR
        } else if attr.weight <= FontWeight::MEDIUM {
            // Find best lighter weight, else best heavier weight
            match candidates
                .iter()
                .filter(|&&idx| fonts[idx].weight <= attr.weight)
                .min_by_key(|&&idx| query_weight - fonts[idx].weight.to_opentype_weight())
            {
                Some(&idx) => fonts[idx].weight,
                None => {
                    let idx = *candidates.iter().min_by_key(|&&idx| {
                        fonts[idx].weight.to_opentype_weight() - query_weight
                    })?;
                    fonts[idx].weight
                }
            }
        } else {
            // Find best heavier weight, else best lighter weight
            match candidates
                .iter()
                .filter(|&&idx| fonts[idx].weight >= attr.weight)
                .min_by_key(|&&idx| fonts[idx].weight.to_opentype_weight() - query_weight)
            {
                Some(&idx) => fonts[idx].weight,
                None => {
                    let idx = *candidates.iter().min_by_key(|&&idx| {
                        query_weight - fonts[idx].weight.to_opentype_weight()
                    })?;
                    fonts[idx].weight
                }
            }
        };

        // Reduce to matching weight
        candidates.retain(|&idx| fonts[idx].weight == weight);

        // Check for best matching pixel strike, but only if all
        // candidates have pixel strikes
        if candidates
            .iter()
            .all(|&idx| !fonts[idx].pixel_sizes.is_empty())
        {
            if let Some((_distance, idx)) = candidates
                .iter()
                .map(|&idx| {
                    let distance = fonts[idx]
                        .pixel_sizes
                        .iter()
                        .map(|&size| ((pixel_size as i32) - (size as i32)).abs())
                        .min()
                        .unwrap_or(i32::MAX);
                    (distance, idx)
                })
                .min()
            {
                return Some(idx);
            }
        }

        // The first one in this set is our best match
        candidates.into_iter().next()
    }

    pub fn best_match(
        attr: &FontAttributes,
        pixel_size: u16,
        mut fonts: Vec<Self>,
    ) -> Option<Self> {
        let refs: Vec<&Self> = fonts.iter().collect();
        let idx = Self::best_matching_index(attr, &refs, pixel_size)?;
        fonts.drain(idx..=idx).next().map(|p| p.synthesize(attr))
    }

    /// Update self to reflect whether the rasterizer might need to synthesize
    /// italic for this font.
    pub fn synthesize(mut self, attr: &FontAttributes) -> Self {
        self.harfbuzz_features = attr.harfbuzz_features.clone();
        self.freetype_render_target = attr.freetype_render_target;
        self.freetype_load_target = attr.freetype_load_target;
        self.freetype_load_flags = attr.freetype_load_flags;
        self.scale = attr.scale.map(|f| *f);

        self.synthesize_italic = self.style == FontStyle::Normal && attr.style != FontStyle::Normal;
        self.synthesize_bold = attr.weight >= FontWeight::DEMIBOLD
            && attr.weight > self.weight
            && self.weight <= FontWeight::REGULAR;
        self.synthesize_dim = attr.weight < FontWeight::REGULAR
            && attr.weight < self.weight
            && self.weight >= FontWeight::REGULAR;

        match attr.assume_emoji_presentation {
            Some(assume) => {
                self.assume_emoji_presentation = assume;
            }
            None => {
                // If they explicitly list an emoji font, assume that they
                // want it to be used for emoji presentation.
                // We match on "moji" rather than "emoji" as there are
                // emoji fonts that are moji rather than emoji :-/
                // This heuristic is awful, TBH.
                if !self.is_built_in_fallback
                    && !attr.is_synthetic
                    && self.names.full_name.to_lowercase().contains("moji")
                {
                    self.assume_emoji_presentation = true;
                }
            }
        }

        self
    }
}
