impl HarfbuzzShaper {
    fn do_shape(
        &self,
        mut font_idx: FallbackIdx,
        s: &str,
        font_size: f64,
        dpi: u32,
        no_glyphs: &mut Vec<char>,
        presentation: Option<Presentation>,
        direction: Direction,
        range: Range<usize>,
        presentation_width: Option<&PresentationWidth>,
    ) -> anyhow::Result<Vec<GlyphInfo>> {
        let mut buf = harfbuzz::Buffer::new()?;
        // We deliberately omit setting the script and leave it to harfbuzz
        // to infer from the buffer contents so that it can correctly
        // enable appropriate preprocessing for eg: Hangul.
        // <https://github.com/wezterm/wezterm/issues/1474> and
        // <https://github.com/wezterm/wezterm/issues/1573>
        // buf.set_script(harfbuzz::hb_script_t::HB_SCRIPT_LATIN);
        buf.set_direction(match direction {
            Direction::LeftToRight => harfbuzz::hb_direction_t::HB_DIRECTION_LTR,
            Direction::RightToLeft => harfbuzz::hb_direction_t::HB_DIRECTION_RTL,
        });
        buf.set_language(self.lang);

        buf.add_str(s, range.clone());
        buf.guess_segment_properties();
        buf.set_cluster_level(
            harfbuzz::hb_buffer_cluster_level_t::HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES,
        );

        let shaped_any;

        // We set this to true when we've run out of fallback fonts to try.
        // In that case, we accept shaper info with codepoint==0 and
        // will use the notdef glyph from the base font.
        let mut no_more_fallbacks = false;

        loop {
            match self.load_fallback(font_idx, dpi).context("load_fallback")? {
                Some(mut pair) => {
                    if let Some(p) = presentation {
                        if pair.presentation != p {
                            log::trace!(
                                "wanted presentation is {p:?} != font \
                                     presentation {:?} so skip \
                                     font_idx={font_idx}",
                                pair.presentation
                            );
                            font_idx += 1;
                            continue;
                        }
                    }
                    let point_size = font_size * self.handles[font_idx].scale.unwrap_or(1.);

                    // Tell harfbuzz to recompute important font metrics!
                    if *pair.last_size_and_dpi.borrow() != Some((point_size, dpi)) {
                        let selected_font_size = pair.face.set_font_size(point_size, dpi)?;

                        let pixel_size = if selected_font_size.is_scaled {
                            (point_size * dpi as f64 / 72.) as u32
                        } else {
                            selected_font_size.height as u32
                        };

                        let mut font = pair.font.borrow_mut();

                        if USE_OT_FACE {
                            font.set_ppem(pixel_size, pixel_size);
                            font.set_ptem(point_size as f32);
                            let scale = pixel_size as i32 * 64;
                            font.set_font_scale(scale, scale);
                        }

                        font.font_changed();

                        if USE_OT_FUNCS {
                            font.set_ot_funcs();
                        }
                        pair.last_size_and_dpi
                            .borrow_mut()
                            .replace((point_size, dpi));
                    }

                    let mut font = pair.font.borrow_mut();
                    shaped_any = pair.shaped_any;
                    font.shape(&mut buf, pair.features.as_slice());
                    log::trace!(
                        "shaped font_idx={} {:?} presentation={presentation:?} as: {}",
                        font_idx,
                        &s[range.start..range.end],
                        buf.serialize(Some(&*font))
                    );
                    break;
                }
                None => {
                    for c in s.chars() {
                        no_glyphs.push(c);
                    }

                    if presentation.is_some() {
                        log::debug!(
                            "Ran out of fallback options, retry shape with no presentation"
                        );
                        // Ran out of fallbacks and we have an explicit presentation.
                        // This is a little awkward; we want to record the missing
                        // glyphs so that we can resolve them async, but we also
                        // want to try the current set of fonts without forcing
                        // the presentation match as we might find the results
                        // that way.
                        // Let's restart the shape but pretend that no specific
                        // presentation was used.
                        // We'll probably match the emoji presentation for something,
                        // but might potentially discover the text presentation for
                        // that glyph in a fallback font and swap it out a little
                        // later after a flash of showing the emoji one.
                        return self.do_shape(
                            0,
                            s,
                            font_size,
                            dpi,
                            no_glyphs,
                            None,
                            direction,
                            range,
                            presentation_width,
                        );
                    }

                    // One more go around to pick up the base font and
                    // accept using the notdef glyph from that.
                    no_more_fallbacks = true;
                    font_idx = 0;
                    continue;
                }
            }
        }

        let hb_infos = buf.glyph_infos();
        let positions = buf.glyph_positions();

        let mut cluster = Vec::with_capacity(s.len());
        let mut info_clusters: Vec<Vec<Info>> = Vec::with_capacity(s.len());

        // At this point we have a list of glyphs from the shaper.
        // Each glyph will have `info.cluster` set to the byte index
        // into `s`.  Multiple byte positions can be coalesced into
        // the same `info.cluster` value, representing text that combines
        // into a ligature.
        // It is important for the terminal to understand this relationship
        // because the cell width of that range of text depends on the unicode
        // version at the time that the text was added to the terminal.
        // To calculate the width per glyph:
        // * Make a pass over the clusters to identify the `info.cluster` starting
        //   positions of all of the glyphs
        // * Sort by info.cluster
        // * Dedup
        // * We can now get the byte length of each cluster by looking at the difference
        //   between the `info.cluster` values.
        // * `presentation_width` can be used to resolve the cell width of those
        //   byte ranges.
        // * Then distribute the glyphs across that cell width when assigning them
        //   to a GlyphInfo.
        //
        // A further complication is that the shaper may cluster in surprising ways.
        // For example, the sequence "\u{28}\u{ff9f}" is a single grapheme with
        // width of 2 cells, but harfbuzz returns that as two different clusters.
        //
        // In situations where we know better (eg: when we have presentation_width)
        // we can fixup the cluster information to be based on the cell index for
        // the harfbuzz byte start.
        // <https://github.com/wezterm/wezterm/issues/2572>

        let mut cluster_resolver = ClusterResolver {
            presentation_width,
            ..Default::default()
        };

        cluster_resolver.build(hb_infos, s, &range);
        log::debug!("cluster_resolver: {cluster_resolver:#?}");

        let info_iter = hb_infos.iter().zip(positions.iter()).peekable();
        for (info, pos) in info_iter {
            let cluster_info = match cluster_resolver.get_mut(info.cluster as usize) {
                Some(i) => i,
                None => panic!(
                    "expected cluster info.cluster {} to be in cluster_resolver",
                    info.cluster
                ),
            };
            let len = cluster_info.byte_len;

            let mut info = Info {
                // Take care to use our adjusted cluster_info.start rather
                // than the info.cluster value, otherwise the combination
                // of info.cluster + info.len will be out of bounds later
                // in this code
                cluster: cluster_info.start,
                len,
                codepoint: info.codepoint,
                x_advance: pos.x_advance,
                y_advance: pos.y_advance,
                x_offset: pos.x_offset,
                y_offset: pos.y_offset,
            };
            log::debug!("hb info.cluster {} -> {info:?}", info.cluster);

            if info.codepoint == 0 && !no_more_fallbacks {
                cluster_info.incomplete = true;
            }

            if let Some(ref mut cluster) = info_clusters.last_mut() {
                // Don't fragment runs of unresolved codepoints; they could be a sequence
                // that shapes together in a fallback font.
                if info.codepoint == 0 && !no_more_fallbacks {
                    let prior = cluster.last_mut().unwrap();
                    // This logic essentially merges `info` into `prior` by
                    // extending the length of prior by `info`.
                    // We can only do that if they are contiguous.
                    // Take care, as the shaper may have re-ordered things!
                    if prior.codepoint == 0 || prior.cluster == info.cluster {
                        if prior.cluster + prior.len == info.cluster {
                            // Coalesce with prior
                            prior.len += info.len;
                            continue;
                        } else if info.cluster + info.len == prior.cluster {
                            // We actually precede prior; we must have been
                            // re-ordered by the shaper. Re-arrange and
                            // coalesce
                            std::mem::swap(&mut info, prior);
                            prior.len += info.len;
                            continue;
                        } else if info.cluster + info.len == prior.cluster + prior.len {
                            // Overlaps and coincide with the end of prior; this one folds away.
                            // This can happen with NFD rather than NFC text.
                            // <https://github.com/wezterm/wezterm/issues/2032>
                            continue;
                        }
                        // log::info!("prior={:#?}, info={:#?}", prior, info);
                    }
                }

                // It is important that this bit happens after we've had the
                // opportunity to coalesce runs of unresolved codepoints,
                // otherwise we can produce incorrect shaping
                // <https://github.com/wezterm/wezterm/issues/2482>
                if cluster.last().unwrap().cluster == info.cluster {
                    cluster.push(info);
                    continue;
                }
            }
            info_clusters.push(vec![info]);
        }
        //  log::error!("do_shape: font_idx={} {:?} {:#?}", font_idx, &s[range.clone()], info_clusters);
        log::debug!("font_idx={font_idx} info_clusters: {:#?}", info_clusters);

        let mut direct_clusters = 0;

        for infos in &info_clusters {
            let cluster_info = cluster_resolver
                .get(infos[0].cluster)
                .expect("assigned above");
            let sub_range = cluster_info.start..cluster_info.start + cluster_info.byte_len;
            let substr = &s[sub_range.clone()];

            if cluster_info.incomplete {
                // One or more entries didn't have a corresponding glyph,
                // so try a fallback

                /*
                if font_idx == 0 {
                    log::error!("incomplete cluster for text={:?} {:?}", s, info_clusters);
                }
                */

                let first_info = &infos[0];

                let mut shape = match self.do_shape(
                    font_idx + 1,
                    s,
                    font_size,
                    dpi,
                    no_glyphs,
                    presentation,
                    direction,
                    // NOT! substr; this is a coalesced sequence of incomplete clusters!
                    first_info.cluster..first_info.cluster + first_info.len,
                    presentation_width,
                ) {
                    Ok(shape) => Ok(shape),
                    Err(e) => {
                        error!("{:?} for {:?}", e, substr);
                        self.do_shape(
                            0,
                            &make_question_string(substr),
                            font_size,
                            dpi,
                            no_glyphs,
                            presentation,
                            direction,
                            sub_range,
                            presentation_width,
                        )
                    }
                }?;

                cluster.append(&mut shape);
                continue;
            }

            let total_width: f64 = infos.iter().map(|info| info.x_advance as f64).sum();
            let mut remaining_cells = cluster_info.cell_width;

            for info in infos.iter() {
                // Proportional width based on relative pixel dimensions vs. other glyphs in
                // this same cluster
                // Note that weighted_cell_width can legitimately compute as zero here
                // for the case where a combining mark composes over another glyph
                // However, some symbol fonts have broken advance metrics and we don't
                // want those glyphs to end up with zero width, so if this run is zero
                // width then we round up to 1 cell.
                // <https://github.com/wezterm/wezterm/issues/1787>
                let weighted_cell_width = if total_width == 0. {
                    1
                } else {
                    (cluster_info.cell_width as f64 * info.x_advance as f64 / total_width).ceil()
                        as u8
                };
                let weighted_cell_width = weighted_cell_width.min(remaining_cells);
                remaining_cells = remaining_cells.saturating_sub(weighted_cell_width);

                let glyph = make_glyphinfo(substr, weighted_cell_width, font_idx, info);

                cluster.push(glyph);
                direct_clusters += 1;
            }
        }

        if !shaped_any {
            if let Some(opt_pair) = self.fonts.get(font_idx) {
                if direct_clusters == 0 {
                    // If we've never shaped anything from this font, and we didn't
                    // shape it just now, then we're probably a fallback font from
                    // the system and unlikely to be useful to keep around, so we
                    // unload it.
                    log::trace!(
                        "Shaper didn't resolve glyphs from {:?}, so unload it",
                        self.handles[font_idx]
                    );
                    opt_pair.borrow_mut().take();
                } else if let Some(pair) = &mut *opt_pair.borrow_mut() {
                    // We shaped something: mark this pair up so that it sticks around
                    pair.shaped_any = true;
                }
            }
        }

        Ok(cluster)
    }
}
