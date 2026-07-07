pub struct Library {
    lib: FT_Library,
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            FT_Done_FreeType(self.lib);
        }
    }
}

impl Library {
    pub fn new() -> anyhow::Result<Library> {
        let mut lib = ptr::null_mut();
        let res = unsafe { FT_Init_FreeType(&mut lib as *mut _) };
        let lib = ft_result(res, lib).context("FT_Init_FreeType")?;
        let mut lib = Library { lib };

        let config = configuration();
        if let Some(vers) = config.freetype_interpreter_version {
            let interpreter_version: FT_UInt = vers;
            unsafe {
                FT_Property_Set(
                    lib.lib,
                    b"truetype\0" as *const u8 as *const FT_String,
                    b"interpreter-version\0" as *const u8 as *const FT_String,
                    &interpreter_version as *const FT_UInt as *const _,
                );
            }
        }

        {
            let no_long_names: FT_Bool = if config.freetype_pcf_long_family_names {
                0
            } else {
                1
            };
            unsafe {
                FT_Property_Set(
                    lib.lib,
                    b"pcf\0" as *const u8 as *const FT_String,
                    b"no-long-family-names\0" as *const u8 as *const FT_String,
                    &no_long_names as *const FT_Bool as *const _,
                );
            }
        }

        // Due to patent concerns, the freetype library disables the LCD
        // filtering feature by default, and since we always build our
        // own copy of freetype, it is likewise disabled by default for
        // us too.  As a result, this call will generally fail.
        // Freetype is still able to render a decent result without it!
        lib.set_lcd_filter(FT_LcdFilter::FT_LCD_FILTER_DEFAULT).ok();

        Ok(lib)
    }

    /// Returns the number of faces in a given font.
    /// For a TTF this will be 1.
    /// For a TTC, it will be the number of contained fonts
    pub fn query_num_faces(&self, source: &FontDataSource) -> anyhow::Result<u32> {
        let face = self.new_face(source, -1).context("query_num_faces")?;
        let num_faces = unsafe { (*face).num_faces }.try_into();

        unsafe {
            FT_Done_Face(face);
        }

        Ok(num_faces?)
    }

    pub fn face_from_locator(&self, handle: &FontDataHandle) -> anyhow::Result<Face> {
        let source = handle.clone();

        let mut index = handle.index;
        if handle.variation != 0 {
            index |= handle.variation << 16;
        }

        let face = self
            .new_face(&source.source, index as _)
            .with_context(|| format!("face_from_locator({:?})", handle))?;

        Ok(Face {
            face,
            lib: self.lib,
            source,
            size: None,
            palette: None,
        })
    }

    fn new_face(&self, source: &FontDataSource, face_index: FT_Long) -> anyhow::Result<FT_Face> {
        let mut face = ptr::null_mut();

        // FT_Open_Face will take ownership of this and closes it in both
        // the error case and the success case (although the latter is when
        // the face is dropped).
        let stream = FreeTypeStream::from_source(source)?;

        let args = FT_Open_Args {
            flags: FT_OPEN_STREAM,
            memory_base: ptr::null(),
            memory_size: 0,
            pathname: ptr::null_mut(),
            stream,
            driver: ptr::null_mut(),
            num_params: 0,
            params: ptr::null_mut(),
        };

        let res = unsafe { FT_Open_Face(self.lib, &args, face_index, &mut face as *mut _) };

        ft_result(res, face)
            .with_context(|| format!("FT_Open_Face(\"{:?}\", face_index={})", source, face_index))
    }

    pub fn set_lcd_filter(&mut self, filter: FT_LcdFilter) -> anyhow::Result<()> {
        unsafe {
            ft_result(FT_Library_SetLcdFilter(self.lib, filter), ())
                .context("FT_Library_SetLcdFilter")
        }
    }
}
