pub fn run_ls_fonts(config: config::ConfigHandle, cmd: &LsFontsCommand) -> anyhow::Result<()> {
    use wezterm_font::parser::ParsedFont;

    if let Err(err) = config::configuration_result() {
        log::error!("{}", err);
        return Ok(());
    }

    // Disable the normal config error UI window, as we don't have
    // a fully baked GUI environment running
    config::assign_error_callback(|err| eprintln!("{}", err));

    let font_config = Rc::new(wezterm_font::FontConfiguration::new(
        Some(config.clone()),
        config.dpi.unwrap_or_else(|| ::window::default_dpi()) as usize,
    )?);

    let render_metrics = crate::utilsprites::RenderMetrics::new(&font_config)?;

    let bidi_hint = if config.bidi_enabled {
        Some(config.bidi_direction)
    } else {
        None
    };

    let unicode_version = config.unicode_version();

    let text = match (&cmd.text, &cmd.codepoints) {
        (Some(text), _) => Some(text.to_string()),
        (_, Some(codepoints)) => {
            let mut s = String::new();
            for cp in codepoints.split(",") {
                let cp = u32::from_str_radix(cp, 16)
                    .with_context(|| format!("{cp} is not a hex number"))?;
                let c = char::from_u32(cp)
                    .ok_or_else(|| anyhow!("{cp} is not a valid unicode codepoint value"))?;
                s.push(c);
            }
            Some(s)
        }
        _ => None,
    };

    if let Some(text) = &text {
        // Emulate the effect of output normalization
        let text = if config.normalize_output_to_unicode_nfc {
            text.nfc().collect()
        } else {
            text.to_string()
        };

        let line = Line::from_text(
            &text,
            &CellAttributes::default(),
            SEQ_ZERO,
            Some(&unicode_version),
        );
        let cell_clusters = line.cluster(bidi_hint);
        let ft_lib = wezterm_font::ftwrap::Library::new()?;

        let mut glyph_cache = GlyphCache::new_in_memory(&font_config, 256)?;

        for cluster in cell_clusters {
            let style = font_config.match_style(&config, &cluster.attrs);
            let font = font_config.resolve_font(style)?;
            let presentation_width = PresentationWidth::with_cluster(&cluster);
            let infos = font
                .blocking_shape(
                    &cluster.text,
                    Some(cluster.presentation),
                    cluster.direction,
                    None,
                    Some(&presentation_width),
                )
                .unwrap();

            // We must grab the handles after shaping, so that we get the
            // revised list that includes system fallbacks!
            let handles = font.clone_handles();
            let faces: Vec<_> = handles
                .iter()
                .map(|p| ft_lib.face_from_locator(&p.handle).ok())
                .collect();

            let mut iter = infos.iter().peekable();

            let mut byte_lens = vec![];
            for c in cluster.text.chars() {
                let len = c.len_utf8();
                for _ in 0..len {
                    byte_lens.push(len);
                }
            }
            println!("{:?}", cluster.direction);

            while let Some(info) = iter.next() {
                let idx = cluster.byte_to_cell_idx(info.cluster as usize);
                let followed_by_space = match line.get_cell(idx + 1) {
                    Some(cell) => cell.str() == " ",
                    None => false,
                };

                let text = if cluster.direction == Direction::LeftToRight {
                    if let Some(next) = iter.peek() {
                        line.columns_as_str(idx..cluster.byte_to_cell_idx(next.cluster as usize))
                    } else {
                        let last_idx = cluster.byte_to_cell_idx(cluster.text.len() - 1);
                        line.columns_as_str(idx..last_idx + 1)
                    }
                } else {
                    let info_len = byte_lens[info.cluster as usize];
                    let last_idx = cluster.byte_to_cell_idx(info.cluster as usize + info_len - 1);
                    line.columns_as_str(idx..last_idx + 1)
                };

                let parsed = &handles[info.font_idx];
                let escaped = format!("{}", text.escape_unicode());
                let mut is_custom = false;

                let cached_glyph = glyph_cache.cached_glyph(
                    &info,
                    &style,
                    followed_by_space,
                    &font,
                    &render_metrics,
                    info.num_cells,
                )?;

                let mut texture = cached_glyph.texture.clone();

                if config.custom_block_glyphs {
                    if let Some(block) = info.only_char.and_then(BlockKey::from_char) {
                        texture.replace(glyph_cache.cached_block(block, &render_metrics)?);
                        println!(
                            "{:2} {:4} {:12} drawn by wezterm because custom_block_glyphs=true: {:?}",
                            info.cluster, text, escaped, block
                        );
                        is_custom = true;
                    }
                }

                if !is_custom {
                    let glyph_name = faces[info.font_idx]
                        .as_ref()
                        .and_then(|face| {
                            face.get_glyph_name(info.glyph_pos)
                                .map(|name| format!("{},", name))
                        })
                        .unwrap_or_else(String::new);

                    println!(
                        "{:2} {:4} {:12} x_adv={:<2} cells={:<2} glyph={}{:<4} {}\n{:38}{}",
                        info.cluster,
                        text,
                        escaped,
                        cached_glyph.x_advance.get(),
                        info.num_cells,
                        glyph_name,
                        info.glyph_pos,
                        parsed.lua_name(),
                        "",
                        parsed.handle.diagnostic_string()
                    );
                }

                if cmd.rasterize_ascii {
                    let mut glyph = String::new();

                    if let Some(texture) = &cached_glyph.texture {
                        use ::window::bitmaps::ImageTexture;
                        if let Some(tex) = texture.texture.downcast_ref::<ImageTexture>() {
                            for y in texture.coords.min_y()..texture.coords.max_y() {
                                for &px in tex.image.borrow().horizontal_pixel_range(
                                    texture.coords.min_x() as usize,
                                    texture.coords.max_x() as usize,
                                    y as usize,
                                ) {
                                    let px = u32::from_be(px);
                                    let (b, g, r, a) = (
                                        (px >> 8) as u8,
                                        (px >> 16) as u8,
                                        (px >> 24) as u8,
                                        (px & 0xff) as u8,
                                    );
                                    // Use regular RGB for other terminals, but then
                                    // set RGBA for wezterm
                                    glyph.push_str(&format!(
                                "\x1b[38:2::{r}:{g}:{b}m\x1b[38:6::{r}:{g}:{b}:{a}m\u{2588}\x1b[0m"
                            ));
                                }
                                glyph.push('\n');
                            }
                        }
                    }

                    if !is_custom {
                        println!(
                            "bearing: x={} y={}, offset: x={} y={}",
                            cached_glyph.bearing_x.get(),
                            cached_glyph.bearing_y.get(),
                            cached_glyph.x_offset.get(),
                            cached_glyph.y_offset.get(),
                        );
                    }
                    println!("{glyph}");
                }
            }
        }
        return Ok(());
    }

    println!("Primary font:");
    let default_font = font_config.default_font()?;
    println!(
        "{}",
        ParsedFont::lua_fallback(&default_font.clone_handles())
    );
    println!();

    for rule in &config.font_rules {
        println!();

        let mut condition = "When".to_string();
        if let Some(intensity) = &rule.intensity {
            condition.push_str(&format!(" Intensity={:?}", intensity));
        }
        if let Some(underline) = &rule.underline {
            condition.push_str(&format!(" Underline={:?}", underline));
        }
        if let Some(italic) = &rule.italic {
            condition.push_str(&format!(" Italic={:?}", italic));
        }
        if let Some(blink) = &rule.blink {
            condition.push_str(&format!(" Blink={:?}", blink));
        }
        if let Some(rev) = &rule.reverse {
            condition.push_str(&format!(" Reverse={:?}", rev));
        }
        if let Some(strikethrough) = &rule.strikethrough {
            condition.push_str(&format!(" Strikethrough={:?}", strikethrough));
        }
        if let Some(invisible) = &rule.invisible {
            condition.push_str(&format!(" Invisible={:?}", invisible));
        }

        println!("{}:", condition);
        let font = font_config.resolve_font(&rule.font)?;
        println!("{}", ParsedFont::lua_fallback(&font.clone_handles()));
        println!();
    }

    println!("Title font:");
    let title_font = font_config.title_font()?;
    println!("{}", ParsedFont::lua_fallback(&title_font.clone_handles()));
    println!();

    if cmd.list_system {
        let font_dirs = font_config.list_fonts_in_font_dirs();
        println!(
            "{} fonts found in your font_dirs + built-in fonts:",
            font_dirs.len()
        );
        for font in font_dirs {
            let pixel_sizes = if font.pixel_sizes.is_empty() {
                "".to_string()
            } else {
                format!(" pixel_sizes={:?}", font.pixel_sizes)
            };
            println!(
                "{} -- {}{}{}",
                font.lua_name(),
                font.aka(),
                font.handle.diagnostic_string(),
                pixel_sizes
            );
        }

        match font_config.list_system_fonts() {
            Ok(sys_fonts) => {
                println!(
                    "{} system fonts found using {:?}:",
                    sys_fonts.len(),
                    config.font_locator
                );
                for font in sys_fonts {
                    let pixel_sizes = if font.pixel_sizes.is_empty() {
                        "".to_string()
                    } else {
                        format!(" pixel_sizes={:?}", font.pixel_sizes)
                    };
                    println!(
                        "{} -- {}{}{}",
                        font.lua_name(),
                        font.aka(),
                        font.handle.diagnostic_string(),
                        pixel_sizes
                    );
                }
            }
            Err(err) => log::error!("Unable to list system fonts: {}", err),
        }
    }

    Ok(())
}
