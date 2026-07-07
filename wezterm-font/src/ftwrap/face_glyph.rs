impl Face {
    pub fn load_glyph_outlines(
        &mut self,
        glyph_index: FT_UInt,
        load_flags: FT_Int32,
    ) -> anyhow::Result<Vec<DrawOp>> {
        unsafe {
            ft_result(FT_Load_Glyph(self.face, glyph_index, load_flags), ())
                .with_context(|| format!("FT_Load_Glyph {glyph_index}"))?;
            let slot = &mut *(*self.face).glyph;
            if slot.format != FT_Glyph_Format_::FT_GLYPH_FORMAT_OUTLINE {
                anyhow::bail!(
                    "Expected FT_COLR_PAINTFORMAT_GLYPH to be an outline, got {:?}",
                    slot.format
                );
            }

            let funcs = FT_Outline_Funcs_ {
                move_to: Some(move_to),
                line_to: Some(line_to),
                conic_to: Some(conic_to),
                cubic_to: Some(cubic_to),
                shift: 16, // match the same coordinate space as transforms
                delta: FT_Pos::from_font_units(0),
            };

            let mut ops = vec![];

            unsafe extern "C" fn move_to(to: *const FT_Vector, user: *mut c_void) -> c_int {
                let ops = user as *mut Vec<DrawOp>;
                let (to_x, to_y) = vector_x_y(&*to);
                (*ops).push(DrawOp::MoveTo { to_x, to_y });
                0
            }
            unsafe extern "C" fn line_to(to: *const FT_Vector, user: *mut c_void) -> c_int {
                let ops = user as *mut Vec<DrawOp>;
                let (to_x, to_y) = vector_x_y(&*to);
                (*ops).push(DrawOp::LineTo { to_x, to_y });
                0
            }
            unsafe extern "C" fn conic_to(
                control: *const FT_Vector,
                to: *const FT_Vector,
                user: *mut c_void,
            ) -> c_int {
                let ops = user as *mut Vec<DrawOp>;
                let (control_x, control_y) = vector_x_y(&*control);
                let (to_x, to_y) = vector_x_y(&*to);
                (*ops).push(DrawOp::QuadTo {
                    control_x,
                    control_y,
                    to_x,
                    to_y,
                });
                0
            }
            unsafe extern "C" fn cubic_to(
                control1: *const FT_Vector,
                control2: *const FT_Vector,
                to: *const FT_Vector,
                user: *mut c_void,
            ) -> c_int {
                let ops = user as *mut Vec<DrawOp>;
                let (control1_x, control1_y) = vector_x_y(&*control1);
                let (control2_x, control2_y) = vector_x_y(&*control2);
                let (to_x, to_y) = vector_x_y(&*to);
                (*ops).push(DrawOp::CubicTo {
                    control1_x,
                    control1_y,
                    control2_x,
                    control2_y,
                    to_x,
                    to_y,
                });
                0
            }

            ft_result(
                FT_Outline_Decompose(
                    &mut slot.outline,
                    &funcs,
                    &mut ops as *mut Vec<DrawOp> as *mut c_void,
                ),
                (),
            )
            .with_context(|| format!("FT_Outline_Decompose. ops so far: {ops:?}"))?;

            if !ops.is_empty() {
                ops.push(DrawOp::ClosePath);
            }

            Ok(ops)
        }
    }

    pub fn load_and_render_glyph(
        &mut self,
        glyph_index: FT_UInt,
        load_flags: FT_Int32,
        render_mode: FT_Render_Mode,
        synthesize_bold: bool,
    ) -> anyhow::Result<&FT_GlyphSlotRec_> {
        unsafe {
            ft_result(
                FT_Load_Glyph(self.face, glyph_index, load_flags | FT_LOAD_NO_SVG as i32),
                (),
            )
            .with_context(|| {
                format!(
                    "load_and_render_glyph: FT_Load_Glyph glyph_index:{}",
                    glyph_index
                )
            })?;
            let slot = &mut *(*self.face).glyph;

            if slot.format == FT_Glyph_Format_::FT_GLYPH_FORMAT_SVG {
                return Err(IsSvg.into());
            }

            if synthesize_bold {
                FT_GlyphSlot_Embolden(slot as *mut _);
            }

            // Current versions of freetype overload the operation of FT_LOAD_COLOR
            // and the resulting glyph format such that we cannot determine solely
            // from the flags whether we got a regular set of outlines,
            // or its COLR v0 synthesized glyphs, or whether it's COLR v1 or later
            // and it can't render the result.
            // So, we probe here to look for color layer information: if we find it,
            // we don't call freetype's renderer and instead bubble up an error
            // that the embedding application can trap and decide what to do.
            if slot.format == FT_Glyph_Format_::FT_GLYPH_FORMAT_OUTLINE {
                if self
                    .get_color_glyph_paint(
                        glyph_index,
                        FT_Color_Root_Transform::FT_COLOR_NO_ROOT_TRANSFORM,
                    )
                    .is_ok()
                {
                    return Err(IsColr1OrLater.into());
                }
            }

            ft_result(FT_Render_Glyph(slot, render_mode), ())
                .context("load_and_render_glyph: FT_Render_Glyph")?;

            Ok(slot)
        }
    }

    /// Compute the cap-height metric in pixels.
    /// This is pixel-perfect based on the rendered glyph data for `I`,
    /// which is a technique that works for any font regardless
    /// of the integrity of its internal cap-height metric or
    /// whether the font is a bitmap font.
    /// `I` is chosen rather than `O` as `O` glyphs are often optically
    /// compensated and overshoot a little.
    fn compute_cap_height(&mut self) -> anyhow::Result<f64> {
        let glyph_pos = unsafe { FT_Get_Char_Index(self.face, b'I' as _) };
        if glyph_pos == 0 {
            anyhow::bail!("no I from which to compute cap height");
        }
        let (load_flags, render_mode) = compute_load_flags_from_config(None, None, None, None);
        let ft_glyph = self.load_and_render_glyph(glyph_pos, load_flags, render_mode, false)?;

        let mode: FT_Pixel_Mode =
            unsafe { std::mem::transmute(u32::from(ft_glyph.bitmap.pixel_mode)) };

        // pitch is the number of bytes per source row
        let pitch = ft_glyph.bitmap.pitch.abs() as usize;
        let data = unsafe {
            std::slice::from_raw_parts_mut(
                ft_glyph.bitmap.buffer,
                ft_glyph.bitmap.rows as usize * pitch,
            )
        };

        let mut first_row = None;
        let mut last_row = None;

        match mode {
            FT_Pixel_Mode::FT_PIXEL_MODE_LCD => {
                let width = ft_glyph.bitmap.width as usize / 3;
                let height = ft_glyph.bitmap.rows as usize;

                'next_line_lcd: for y in 0..height {
                    let src_offset = y * pitch as usize;
                    for x in 0..width {
                        if data[src_offset + (x * 3)] != 0
                            || data[src_offset + (x * 3) + 1] != 0
                            || data[src_offset + (x * 3) + 2] != 0
                        {
                            if first_row.is_none() {
                                first_row.replace(y);
                            }
                            last_row.replace(y);
                            continue 'next_line_lcd;
                        }
                    }
                }
            }

            FT_Pixel_Mode::FT_PIXEL_MODE_BGRA => {
                let width = ft_glyph.bitmap.width as usize;
                let height = ft_glyph.bitmap.rows as usize;
                'next_line_bgra: for y in 0..height {
                    let src_offset = y * pitch as usize;
                    for x in 0..width {
                        let alpha = data[src_offset + (x * 4) + 3];
                        if alpha != 0 {
                            if first_row.is_none() {
                                first_row.replace(y);
                            }
                            last_row.replace(y);
                            continue 'next_line_bgra;
                        }
                    }
                }
            }
            FT_Pixel_Mode::FT_PIXEL_MODE_GRAY => {
                let width = ft_glyph.bitmap.width as usize;
                let height = ft_glyph.bitmap.rows as usize;
                'next_line_gray: for y in 0..height {
                    let src_offset = y * pitch;
                    for x in 0..width {
                        if data[src_offset + x] != 0 {
                            if first_row.is_none() {
                                first_row.replace(y);
                            }
                            last_row.replace(y);
                            continue 'next_line_gray;
                        }
                    }
                }
            }
            FT_Pixel_Mode::FT_PIXEL_MODE_MONO => {
                let width = ft_glyph.bitmap.width as usize;
                let height = ft_glyph.bitmap.rows as usize;
                'next_line_mono: for y in 0..height {
                    let src_offset = y * pitch;
                    let mut x = 0;
                    for i in 0..pitch {
                        if x >= width {
                            break;
                        }
                        let mut b = data[src_offset + i];
                        for _ in 0..8 {
                            if x >= width {
                                break;
                            }
                            if b & 0x80 == 0x80 {
                                if first_row.is_none() {
                                    first_row.replace(y);
                                }
                                last_row.replace(y);
                                continue 'next_line_mono;
                            }
                            b <<= 1;
                            x += 1;
                        }
                    }
                }
            }
            _ => anyhow::bail!("unhandled pixel mode {:?}", mode),
        }

        match (first_row, last_row) {
            (Some(first), Some(last)) => Ok((last - first) as f64),
            _ => anyhow::bail!("didn't find any rasterized rows?"),
        }
    }

    fn cell_metrics(&mut self) -> ComputedCellMetrics {
        unsafe {
            let metrics = &(*(*self.face).size).metrics;
            let height = metrics.y_scale.to_num::<f64>() * f64::from((*self.face).height) / 64.0;

            let mut width = 0.0;
            let mut num_examined = 0;
            for i in 32..128 {
                let glyph_pos = FT_Get_Char_Index(self.face, i);
                if glyph_pos == 0 {
                    continue;
                }
                let res = FT_Load_Glyph(self.face, glyph_pos, FT_LOAD_COLOR as i32);
                if succeeded(res) {
                    num_examined += 1;
                    let glyph = &(*(*self.face).glyph);
                    if glyph.metrics.horiAdvance.font_units() as f64 > width {
                        width = glyph.metrics.horiAdvance.font_units() as f64;
                    }
                }
            }
            if width == 0.0 {
                // Most likely we're looking at a symbol font with no latin
                // glyphs at all. Let's just pick a selection of glyphs
                for glyph_pos in 1..8 {
                    let res = FT_Load_Glyph(self.face, glyph_pos, FT_LOAD_COLOR as i32);
                    if succeeded(res) {
                        num_examined += 1;
                        let glyph = &(*(*self.face).glyph);
                        if glyph.metrics.horiAdvance.font_units() as f64 > width {
                            width = glyph.metrics.horiAdvance.font_units() as f64;
                        }
                    }
                }
                if width == 0.0 {
                    log::error!(
                        "Couldn't find usable advance metrics out of {} glyphs \
                        sampled from the font, so guessing width == height",
                        num_examined,
                    );
                    width = height * 64.;
                }
            }

            ComputedCellMetrics {
                width: width / 64.0,
                height,
            }
        }
    }
}
