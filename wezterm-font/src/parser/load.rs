/// In case the user has a broken configuration, or no configuration,
/// we bundle JetBrains Mono and Noto Color Emoji to act as reasonably
/// sane fallback fonts.
/// This function loads those.
pub(crate) fn load_built_in_fonts(font_info: &mut Vec<ParsedFont>) -> anyhow::Result<()> {
    #[allow(unused_macros)]
    macro_rules! font {
        ($font:literal) => {
            (include_bytes!($font) as &'static [u8], $font)
        };
    }
    let lib = crate::ftwrap::Library::new()?;

    let built_ins: &[&[(&[u8], &str)]] = &[
        #[cfg(any(test, feature = "vendor-jetbrains"))]
        &[
            font!("../../../assets/fonts/JetBrainsMono-BoldItalic.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-Bold.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-ExtraBoldItalic.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-ExtraBold.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-ExtraLightItalic.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-ExtraLight.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-Italic.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-LightItalic.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-Light.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-MediumItalic.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-Medium.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-Regular.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-SemiBoldItalic.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-SemiBold.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-ThinItalic.ttf"),
            font!("../../../assets/fonts/JetBrainsMono-Thin.ttf"),
        ],
        #[cfg(any(test, feature = "vendor-roboto"))]
        &[
            font!("../../../assets/fonts/Roboto-Black.ttf"),
            font!("../../../assets/fonts/Roboto-BlackItalic.ttf"),
            font!("../../../assets/fonts/Roboto-Bold.ttf"),
            font!("../../../assets/fonts/Roboto-BoldItalic.ttf"),
            font!("../../../assets/fonts/Roboto-Italic.ttf"),
            font!("../../../assets/fonts/Roboto-Light.ttf"),
            font!("../../../assets/fonts/Roboto-LightItalic.ttf"),
            font!("../../../assets/fonts/Roboto-Medium.ttf"),
            font!("../../../assets/fonts/Roboto-MediumItalic.ttf"),
            font!("../../../assets/fonts/Roboto-Regular.ttf"),
            font!("../../../assets/fonts/Roboto-Thin.ttf"),
            font!("../../../assets/fonts/Roboto-ThinItalic.ttf"),
        ],
        #[cfg(any(test, feature = "vendor-noto-emoji"))]
        &[font!("../../../assets/fonts/NotoColorEmoji.ttf")],
        #[cfg(any(test, feature = "vendor-nerd-font-symbols"))]
        &[font!(
            "../../../assets/fonts/SymbolsNerdFontMono-Regular.ttf"
        )],
    ];
    for bundle in built_ins {
        for (data, name) in bundle.iter() {
            let locator = FontDataHandle {
                source: FontDataSource::BuiltIn { data, name },
                index: 0,
                variation: 0,
                origin: FontOrigin::BuiltIn,
                coverage: None,
            };
            let face = lib.face_from_locator(&locator)?;
            let mut parsed = ParsedFont::from_face(&face, locator)?;
            parsed.is_built_in_fallback = true;
            font_info.push(parsed);
        }
    }

    Ok(())
}

pub fn best_matching_font(
    source: &FontDataSource,
    font_attr: &FontAttributes,
    origin: FontOrigin,
    pixel_size: u16,
) -> anyhow::Result<Option<ParsedFont>> {
    let mut font_info = vec![];
    parse_and_collect_font_info(source, &mut font_info, origin)?;
    font_info.retain(|font| font.matches_name(font_attr));
    Ok(ParsedFont::best_match(font_attr, pixel_size, font_info))
}

pub(crate) fn parse_and_collect_font_info(
    source: &FontDataSource,
    font_info: &mut Vec<ParsedFont>,
    origin: FontOrigin,
) -> anyhow::Result<()> {
    let lib = crate::ftwrap::Library::new()?;
    let num_faces = lib.query_num_faces(&source)?;

    fn load_one(
        lib: &crate::ftwrap::Library,
        source: &FontDataSource,
        index: u32,
        font_info: &mut Vec<ParsedFont>,
        origin: &FontOrigin,
    ) -> anyhow::Result<()> {
        let locator = FontDataHandle {
            source: source.clone(),
            index,
            variation: 0,
            origin: origin.clone(),
            coverage: None,
        };

        let face = lib.face_from_locator(&locator)?;
        if let Ok(variations) = face.variations() {
            for parsed in variations {
                font_info.push(parsed);
            }
        } else {
            let parsed = ParsedFont::from_locator(&locator)?;
            font_info.push(parsed);
        }
        Ok(())
    }

    for index in 0..num_faces {
        if let Err(err) = load_one(&lib, &source, index, font_info, &origin) {
            log::trace!("error while parsing {:?} index {}: {}", source, index, err);
        }
    }

    Ok(())
}
