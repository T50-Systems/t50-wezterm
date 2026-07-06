impl Drop for Face {
    fn drop(&mut self) {
        unsafe {
            hb_face_destroy(self.face);
        }
    }
}

impl Face {
    pub fn from_locator(handle: &FontDataHandle) -> anyhow::Result<Self> {
        let blob = Blob::from_source(&handle.source)?;
        let mut index = handle.index;
        if handle.variation != 0 {
            index |= handle.variation << 16;
        }

        let face = unsafe { hb_face_create(blob.blob, index) };
        if face.is_null() {
            anyhow::bail!("failed to create harfbuzz Face");
        }

        Ok(Self { face })
    }

    #[allow(dead_code)]
    pub fn get_upem(&self) -> c_uint {
        unsafe { hb_face_get_upem(self.face) }
    }
}

pub struct Font {
    font: *mut hb_font_t,
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe {
            hb_font_destroy(self.font);
        }
    }
}

impl Font {
    /// Create a harfbuzz face from a freetype font
    pub fn new(face: freetype::FT_Face) -> Font {
        // hb_ft_font_create_referenced always returns a
        // pointer to something, or derefs a nullptr internally
        // if everything fails, so there's nothing for us to
        // test here.
        Font {
            font: unsafe { hb_ft_font_create_referenced(face as _) },
        }
    }

    pub fn from_locator(handle: &FontDataHandle) -> anyhow::Result<Self> {
        let face = Face::from_locator(handle)?;
        let font = unsafe { hb_font_create(face.face) };
        if font.is_null() {
            anyhow::bail!("failed to create harfbuzz Font");
        }
        Ok(Self { font })
    }

    #[allow(dead_code)]
    pub fn get_face(&self) -> Face {
        let face = unsafe { hb_font_get_face(self.font) };
        unsafe {
            hb_face_reference(face);
        }
        Face { face }
    }

    pub fn set_ot_funcs(&mut self) {
        unsafe {
            hb_ot_font_set_funcs(self.font);
        }
    }

    #[allow(dead_code)]
    pub fn set_ft_funcs(&mut self) {
        unsafe {
            hb_ft_font_set_funcs(self.font);
        }
    }

    pub fn set_synthetic_slant(&mut self, slant: f32) {
        unsafe {
            hb_font_set_synthetic_slant(self.font, slant);
        }
    }

    pub fn set_synthetic_bold(&mut self, x_embolden: f32, y_embolden: f32, in_place: bool) {
        unsafe {
            hb_font_set_synthetic_bold(
                self.font,
                x_embolden,
                y_embolden,
                if in_place { 1 } else { 0 },
            );
        }
    }

    pub fn set_font_scale(&self, x_scale: c_int, y_scale: c_int) {
        unsafe {
            hb_font_set_scale(self.font, x_scale, y_scale);
        }
    }

    pub fn set_ppem(&self, x_ppem: u32, y_ppem: u32) {
        unsafe {
            hb_font_set_ppem(self.font, x_ppem, y_ppem);
        }
    }

    pub fn set_ptem(&self, ptem: f32) {
        unsafe {
            hb_font_set_ptem(self.font, ptem);
        }
    }

    pub fn font_changed(&mut self) {
        unsafe {
            hb_ft_font_changed(self.font);
        }
    }

    pub fn set_load_flags(&mut self, load_flags: freetype::FT_Int32) {
        unsafe {
            hb_ft_font_set_load_flags(self.font, load_flags);
        }
    }

    /// Perform shaping.  On entry, Buffer holds the text to shape.
    /// Once done, Buffer holds the output glyph and position info
    pub fn shape(&mut self, buf: &mut Buffer, features: &[hb_feature_t]) {
        unsafe { hb_shape(self.font, buf.buf, features.as_ptr(), features.len() as u32) }
    }

    /// Fetches a list of the caret positions defined for a ligature glyph in the GDEF table of the
    /// font. The list returned will begin at the offset provided.
    /// Note that a ligature that is formed from n characters will have n-1 caret positions. The
    /// first character is not represented in the array, since its caret position is the glyph
    /// position.
    /// The positions returned by this function are 'unshaped', and will have to be fixed up for
    /// kerning that may be applied to the ligature glyp
    #[allow(dead_code)]
    pub fn get_ligature_carets(
        &self,
        direction: hb_direction_t,
        glyph_pos: u32,
    ) -> Vec<hb_position_t> {
        let mut positions = [0 as hb_position_t; 8];

        unsafe {
            let mut array_size = positions.len() as c_uint;
            let n_carets = hb_ot_layout_get_ligature_carets(
                self.font,
                direction,
                glyph_pos,
                0,
                &mut array_size,
                positions.as_mut_ptr(),
            ) as usize;

            if n_carets > positions.len() {
                let mut positions = vec![0 as hb_position_t; n_carets];
                array_size = positions.len() as c_uint;
                hb_ot_layout_get_ligature_carets(
                    self.font,
                    direction,
                    glyph_pos,
                    0,
                    &mut array_size,
                    positions.as_mut_ptr(),
                );

                return positions;
            }

            positions[..n_carets].to_vec()
        }
    }

    #[allow(unused)]
    pub fn draw_glyph(&self, glyph_pos: u32, funcs: &DrawFuncs, draw_data: *mut c_void) {
        unsafe { hb_font_draw_glyph(self.font, glyph_pos, funcs.funcs, draw_data) }
    }

    #[allow(unused)]
    pub fn paint_glyph(
        &self,
        glyph_pos: u32,
        funcs: &FontFuncs,
        paint_data: *mut c_void,
        palette_index: ::std::os::raw::c_uint,
        foreground: hb_color_t,
    ) {
        unsafe {
            hb_font_paint_glyph(
                self.font,
                glyph_pos,
                funcs.funcs,
                paint_data,
                palette_index,
                foreground,
            )
        }
    }

    pub fn get_paint_ops_for_glyph(
        &self,
        glyph_pos: u32,
        palette_index: ::std::os::raw::c_uint,
        foreground: hb_color_t,
        // TODO: pass a callback for querying custom palette colors
        // from the application
    ) -> anyhow::Result<Vec<PaintOp>> {
        let mut ops = vec![];

        let funcs = FontFuncs::new()?;

        macro_rules! func {
            ($hbfunc:ident, $method:ident) => {
                $hbfunc(
                    funcs.funcs,
                    Some(PaintOp::$method),
                    std::ptr::null_mut(),
                    None,
                );
            };
        }

        unsafe {
            func!(hb_paint_funcs_set_push_transform_func, push_transform);
            func!(hb_paint_funcs_set_pop_transform_func, pop_transform);
            func!(hb_paint_funcs_set_push_clip_glyph_func, push_clip_glyph);
            func!(hb_paint_funcs_set_push_clip_rectangle_func, push_clip_rect);
            func!(hb_paint_funcs_set_pop_clip_func, pop_clip);
            func!(hb_paint_funcs_set_color_func, paint_solid);
            func!(
                hb_paint_funcs_set_linear_gradient_func,
                paint_linear_gradient
            );
            func!(
                hb_paint_funcs_set_radial_gradient_func,
                paint_radial_gradient
            );
            func!(hb_paint_funcs_set_sweep_gradient_func, paint_sweep_gradient);
            func!(hb_paint_funcs_set_image_func, paint_image);
            func!(hb_paint_funcs_set_push_group_func, push_group);
            func!(hb_paint_funcs_set_pop_group_func, pop_group);

            // TODO: hb_paint_funcs_set_custom_palette_color_func
        }

        unsafe {
            hb_font_paint_glyph(
                self.font,
                glyph_pos,
                funcs.funcs,
                &mut ops as *mut Vec<PaintOp> as *mut _,
                palette_index,
                foreground,
            )
        }

        Ok(ops)
    }
}
