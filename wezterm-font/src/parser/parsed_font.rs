impl ParsedFont {
    pub fn from_locator(handle: &FontDataHandle) -> anyhow::Result<Self> {
        let lib = crate::ftwrap::Library::new()?;
        let face = lib.face_from_locator(handle)?;
        Self::from_face(&face, handle.clone())
    }

    pub fn aka(&self) -> String {
        if self.names.aliases.is_empty() {
            String::new()
        } else {
            format!("(AKA: {}) ", self.names.aliases.join(", "))
        }
    }

    pub fn lua_name(&self) -> String {
        format!(
            "wezterm.font(\"{}\", {{weight={}, stretch=\"{}\", style=\"{}\"}})",
            self.names.family, self.weight, self.stretch, self.style
        )
    }

    pub fn lua_fallback(handles: &[Self]) -> String {
        let mut code = "wezterm.font_with_fallback({\n".to_string();

        for p in handles {
            code.push_str(&format!("  -- {}\n", p.handle.diagnostic_string()));
            if p.synthesize_italic {
                code.push_str("  -- Will synthesize italics\n");
            }
            if p.synthesize_bold {
                code.push_str("  -- Will synthesize bold\n");
            } else if p.synthesize_dim {
                code.push_str("  -- Will synthesize dim\n");
            }
            if p.assume_emoji_presentation {
                code.push_str("  -- Assumed to have Emoji Presentation\n");
            }
            if !p.pixel_sizes.is_empty() {
                code.push_str(&format!("  -- Pixel sizes: {:?}\n", p.pixel_sizes));
            }
            if !p.palettes.is_empty() {
                for pal in &p.palettes {
                    let mut info = format!(
                        "  -- Palette: {} {}",
                        pal.palette_index,
                        pal.name.to_string()
                    );
                    if pal.usable_with_light_bg {
                        info.push_str(" (with light bg)");
                    }
                    if pal.usable_with_dark_bg {
                        info.push_str(" (with dark bg)");
                    }
                    info.push('\n');
                    code.push_str(&info);
                }
            }
            for aka in &p.names.aliases {
                code.push_str(&format!("  -- AKA: \"{}\"\n", aka));
            }

            if p.weight == FontWeight::REGULAR
                && p.stretch == FontStretch::Normal
                && p.style == FontStyle::Normal
                && p.freetype_render_target.is_none()
                && p.freetype_load_target.is_none()
                && p.freetype_load_flags.is_none()
                && p.harfbuzz_features.is_none()
                && p.scale.is_none()
            {
                code.push_str(&format!("  \"{}\",\n", p.names.family));
            } else {
                code.push_str(&format!("  {{family=\"{}\"", p.names.family));
                if p.weight != FontWeight::REGULAR {
                    code.push_str(&format!(", weight={}", p.weight));
                }
                if p.stretch != FontStretch::Normal {
                    code.push_str(&format!(", stretch=\"{}\"", p.stretch));
                }
                if p.style != FontStyle::Normal {
                    code.push_str(&format!(", style=\"{}\"", p.style));
                }
                if let Some(scale) = p.scale {
                    code.push_str(&format!(", scale={}", scale));
                }
                if let Some(item) = p.freetype_load_flags {
                    code.push_str(&format!(", freetype_load_flags=\"{}\"", item.to_string()));
                }
                if let Some(item) = p.freetype_load_target {
                    code.push_str(&format!(", freetype_load_target=\"{:?}\"", item));
                }
                if let Some(item) = p.freetype_render_target {
                    code.push_str(&format!(", freetype_render_target=\"{:?}\"", item));
                }
                if let Some(feat) = &p.harfbuzz_features {
                    code.push_str(", harfbuzz_features={");
                    for (idx, f) in feat.iter().enumerate() {
                        if idx > 0 {
                            code.push_str(", ");
                        }
                        code.push('"');
                        code.push_str(f);
                        code.push('"');
                    }
                    code.push('}');
                }
                code.push_str("},\n")
            }
            code.push_str("\n");
        }
        code.push_str("})");
        code
    }

    pub fn from_face(face: &crate::ftwrap::Face, handle: FontDataHandle) -> anyhow::Result<Self> {
        let style = if face.italic() {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        };
        let (ot_weight, width) = face.weight_and_width();
        let weight = FontWeight::from_opentype_weight(ot_weight);
        let stretch = FontStretch::from_opentype_stretch(width);
        let cap_height = face.cap_height();
        let pixel_sizes = face.pixel_sizes();

        let palettes = match face.get_palette_data() {
            Ok(info) => info
                .palettes
                .iter()
                .map(|p| FontPaletteInfo {
                    name: p.name.to_string(),
                    palette_index: p.palette_index,
                    usable_with_light_bg: (p.flags
                        & crate::ftwrap::FT_PALETTE_FOR_LIGHT_BACKGROUND as u16)
                        != 0,
                    usable_with_dark_bg: (p.flags
                        & crate::ftwrap::FT_PALETTE_FOR_DARK_BACKGROUND as u16)
                        != 0,
                })
                .collect(),
            Err(_) => vec![],
        };

        let has_svg = unsafe {
            (((*face.face).face_flags as u32) & (crate::ftwrap::FT_FACE_FLAG_SVG as u32)) != 0
        };

        if has_svg {
            if config::configuration().ignore_svg_fonts {
                anyhow::bail!("skipping svg font because ignore_svg_fonts=true");
            }
        }

        let has_color = unsafe {
            (((*face.face).face_flags as u32) & (crate::ftwrap::FT_FACE_FLAG_COLOR as u32)) != 0
        };
        let assume_emoji_presentation = has_color;

        let names = Names::from_ft_face(&face);
        // Objectively gross, but freetype's italic property is very coarse grained.
        // fontconfig resorts to name matching, so we do too :-/
        let style = match style {
            FontStyle::Normal => {
                let lower = names.full_name.to_lowercase();
                if lower.contains("italic") || lower.contains("kursiv") {
                    FontStyle::Italic
                } else if lower.contains("oblique") {
                    FontStyle::Oblique
                } else {
                    FontStyle::Normal
                }
            }
            FontStyle::Italic => {
                let lower = names.full_name.to_lowercase();
                if lower.contains("oblique") {
                    FontStyle::Oblique
                } else {
                    FontStyle::Italic
                }
            }
            // Currently "impossible" because freetype only knows italic or normal
            FontStyle::Oblique => FontStyle::Oblique,
        };

        let weight = match weight {
            FontWeight::REGULAR => {
                let lower = names.full_name.to_lowercase();
                let mut weight = weight;
                for (label, candidate) in &[
                    ("extrablack", FontWeight::EXTRABLACK),
                    // must match after other black variants
                    ("black", FontWeight::BLACK),
                    ("extrabold", FontWeight::EXTRABOLD),
                    ("demibold", FontWeight::DEMIBOLD),
                    // must match after other bold variants
                    ("bold", FontWeight::BOLD),
                    ("medium", FontWeight::MEDIUM),
                    ("book", FontWeight::BOOK),
                    ("demilight", FontWeight::DEMILIGHT),
                    ("extralight", FontWeight::EXTRALIGHT),
                    // must match after other light variants
                    ("light", FontWeight::LIGHT),
                    ("thin", FontWeight::THIN),
                ] {
                    if lower.contains(label) {
                        weight = *candidate;
                        break;
                    }
                }
                weight
            }
            weight => weight,
        };

        let stretch = match stretch {
            FontStretch::Normal => {
                let lower = names.full_name.to_lowercase();
                let mut stretch = stretch;
                for (label, value) in &[
                    ("ultracondensed", FontStretch::UltraCondensed),
                    ("extracondensed", FontStretch::ExtraCondensed),
                    ("semicondensed", FontStretch::SemiCondensed),
                    // must match after other condensed variants
                    ("condensed", FontStretch::Condensed),
                    ("semiexpanded", FontStretch::SemiExpanded),
                    ("extraexpanded", FontStretch::ExtraExpanded),
                    ("ultraexpanded", FontStretch::UltraExpanded),
                    // must match after other expanded variants
                    ("expanded", FontStretch::Expanded),
                ] {
                    if lower.contains(label) {
                        stretch = *value;
                        break;
                    }
                }

                stretch
            }
            stretch => stretch,
        };

        Ok(Self {
            names,
            weight,
            stretch,
            style,
            synthesize_italic: false,
            synthesize_bold: false,
            synthesize_dim: false,
            is_built_in_fallback: false,
            assume_emoji_presentation,
            handle,
            coverage: Mutex::new(RangeSet::new()),
            cap_height,
            pixel_sizes,
            harfbuzz_features: None,
            freetype_render_target: None,
            freetype_load_target: None,
            freetype_load_flags: None,
            scale: None,
            palettes,
        })
    }

    /// Computes the intersection of the wanted set of codepoints with
    /// the set of codepoints covered by this font entry.
    /// Computes the codepoint coverage for this font entry if we haven't
    /// already done so.
    pub fn coverage_intersection(&self, wanted: &RangeSet<u32>) -> anyhow::Result<RangeSet<u32>> {
        let mut cov = self.coverage.lock().unwrap();
        if cov.is_empty() {
            let t = std::time::Instant::now();
            let lib = crate::ftwrap::Library::new()?;
            let face = lib.face_from_locator(&self.handle)?;
            *cov = face.compute_coverage();
            let elapsed = t.elapsed();
            metrics::histogram!("font.compute.codepoint.coverage").record(elapsed);
            log::debug!(
                "{} codepoint coverage computed in {:?}",
                self.names.full_name,
                elapsed
            );
        }
        Ok(wanted.intersection(&cov))
    }

    pub fn names(&self) -> &Names {
        &self.names
    }

    pub fn weight(&self) -> FontWeight {
        self.weight
    }

    pub fn stretch(&self) -> FontStretch {
        self.stretch
    }

    pub fn style(&self) -> FontStyle {
        self.style
    }

    pub fn matches_name(&self, attr: &FontAttributes) -> bool {
        if attr.family == self.names.family {
            return true;
        }
        if let Some(path) = self.handle.path_str() {
            if attr.family == path {
                return true;
            }
        }
        self.matches_full_or_ps_name(attr) || self.matches_alias(attr)
    }

    pub fn matches_alias(&self, attr: &FontAttributes) -> bool {
        for a in &self.names.aliases {
            if *a == attr.family {
                return true;
            }
        }
        false
    }

    pub fn matches_full_or_ps_name(&self, attr: &FontAttributes) -> bool {
        if attr.family == self.names.full_name {
            return true;
        }
        if let Some(ps) = self.names.postscript_name.as_ref() {
            if attr.family == *ps {
                return true;
            }
        }
        false
    }
}
