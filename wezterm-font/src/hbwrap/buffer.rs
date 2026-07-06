pub struct Buffer {
    buf: *mut hb_buffer_t,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            hb_buffer_destroy(self.buf);
        }
    }
}

#[allow(dead_code)]
extern "C" fn log_buffer_message(
    _buf: *mut hb_buffer_t,
    _font: *mut hb_font_t,
    message: *const c_char,
    _user_data: *mut c_void,
) -> i32 {
    unsafe {
        if !message.is_null() {
            let message = CStr::from_ptr(message);
            log::info!("{message:?}");
        }
    }

    1
}

impl Buffer {
    /// Create a new buffer
    pub fn new() -> Result<Buffer, Error> {
        let buf = unsafe { hb_buffer_create() };
        ensure!(
            unsafe { hb_buffer_allocation_successful(buf) } != 0,
            "hb_buffer_create failed"
        );
        unsafe {
            hb_buffer_set_content_type(
                buf,
                harfbuzz::hb_buffer_content_type_t::HB_BUFFER_CONTENT_TYPE_UNICODE,
            );

            // hb_buffer_set_message_func(buf, Some(log_buffer_message), std::ptr::null_mut(), None);
        };
        Ok(Buffer { buf })
    }

    /// Reset the buffer back to its initial post-creation state
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        unsafe {
            hb_buffer_reset(self.buf);
        }
    }

    pub fn set_cluster_level(&mut self, level: hb_buffer_cluster_level_t) {
        unsafe {
            hb_buffer_set_cluster_level(self.buf, level);
        }
    }

    pub fn set_direction(&mut self, direction: hb_direction_t) {
        unsafe {
            hb_buffer_set_direction(self.buf, direction);
        }
    }

    #[allow(dead_code)]
    pub fn set_script(&mut self, script: hb_script_t) {
        unsafe {
            hb_buffer_set_script(self.buf, script);
        }
    }

    pub fn set_language(&mut self, lang: hb_language_t) {
        unsafe {
            hb_buffer_set_language(self.buf, lang);
        }
    }

    #[allow(dead_code)]
    pub fn add(&mut self, codepoint: hb_codepoint_t, cluster: u32) {
        unsafe {
            hb_buffer_add(self.buf, codepoint, cluster);
        }
    }

    #[allow(dead_code)]
    pub fn reverse(&mut self) {
        unsafe {
            hb_buffer_reverse_clusters(self.buf);
        }
    }

    pub fn add_str(&mut self, paragraph: &str, range: Range<usize>) {
        let bytes = paragraph.as_bytes();
        unsafe {
            hb_buffer_add_utf8(
                self.buf,
                bytes.as_ptr() as *const c_char,
                bytes.len() as i32,
                range.start as u32,
                (range.end - range.start) as i32,
            );
        }
    }

    /// Returns glyph information.  This is only valid after calling
    /// font->shape() on this buffer instance.
    pub fn glyph_infos(&self) -> &[hb_glyph_info_t] {
        unsafe {
            let mut len: u32 = 0;
            let info = hb_buffer_get_glyph_infos(self.buf, &mut len as *mut _);
            from_raw_parts(info, len as usize)
        }
    }

    /// Returns glyph positions.  This is only valid after calling
    /// font->shape() on this buffer instance.
    pub fn glyph_positions(&self) -> &[hb_glyph_position_t] {
        unsafe {
            let mut len: u32 = 0;
            let pos = hb_buffer_get_glyph_positions(self.buf, &mut len as *mut _);
            from_raw_parts(pos, len as usize)
        }
    }

    #[allow(dead_code)]
    pub fn serialize(&self, font: Option<&Font>) -> String {
        unsafe {
            let len = hb_buffer_get_length(self.buf);
            let mut text = vec![0u8; len as usize * 16];
            let buf_len = text.len();
            let mut text_len = 0;
            hb_buffer_serialize(
                self.buf,
                0,
                len,
                text.as_mut_ptr() as *mut _,
                buf_len as _,
                &mut text_len,
                match font {
                    Some(f) => f.font,
                    None => std::ptr::null_mut(),
                },
                harfbuzz::hb_buffer_serialize_format_t::HB_BUFFER_SERIALIZE_FORMAT_TEXT,
                harfbuzz::hb_buffer_serialize_flags_t::HB_BUFFER_SERIALIZE_FLAG_DEFAULT,
            );
            String::from_utf8_lossy(&text[0..text_len as usize]).into()
        }
    }

    pub fn guess_segment_properties(&mut self) {
        unsafe {
            hb_buffer_guess_segment_properties(self.buf);
        }
    }
}

pub struct FontFuncs {
    funcs: *mut hb_paint_funcs_t,
}

impl Drop for FontFuncs {
    fn drop(&mut self) {
        unsafe {
            hb_paint_funcs_destroy(self.funcs);
        }
    }
}

impl FontFuncs {
    pub fn new() -> anyhow::Result<Self> {
        let funcs = unsafe { hb_paint_funcs_create() };
        anyhow::ensure!(!funcs.is_null(), "hb_paint_funcs_create failed");
        Ok(Self { funcs })
    }
}

pub struct DrawFuncs {
    funcs: *mut hb_draw_funcs_t,
}

impl Drop for DrawFuncs {
    fn drop(&mut self) {
        unsafe {
            hb_draw_funcs_destroy(self.funcs);
        }
    }
}

impl DrawFuncs {
    pub fn new() -> anyhow::Result<Self> {
        let funcs = unsafe { hb_draw_funcs_create() };
        anyhow::ensure!(!funcs.is_null(), "hb_draw_funcs_create failed");
        Ok(Self { funcs })
    }
}

pub struct TagString([u8; 4]);

impl std::convert::AsRef<str> for TagString {
    fn as_ref(&self) -> &str {
        std::str::from_utf8(&self.0).expect("tag to be valid ascii")
    }
}

impl std::ops::Deref for TagString {
    type Target = str;
    fn deref(&self) -> &str {
        std::str::from_utf8(&self.0).expect("tag to be valid ascii")
    }
}

impl std::fmt::Display for TagString {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.as_ref().fmt(fmt)
    }
}

pub const fn hb_tag(c1: u8, c2: u8, c3: u8, c4: u8) -> hb_tag_t {
    ((c1 as u32) << 24) | ((c2 as u32) << 16) | ((c3 as u32) << 8) | (c4 as u32)
}

pub fn hb_color(b: u8, g: u8, r: u8, a: u8) -> hb_tag_t {
    hb_tag(b, g, r, a)
}

pub fn hb_tag_to_string(tag: hb_tag_t) -> TagString {
    let mut buf = [0u8; 4];

    // safety: hb_tag_to_string stores 4 bytes to the provided buffer
    unsafe {
        harfbuzz::hb_tag_to_string(tag, &mut buf as *mut u8 as *mut c_char);
    }
    TagString(buf)
}

/// Wrapper around std::slice::from_raw_parts that allows for ptr to be
/// null. In the null ptr case, an empty slice is returned.
/// This is necessary because harfbuzz may sometimes encode
/// empty arrays in that way, and rust 1.78 will panic if a null
/// ptr is passed in.
pub(crate) unsafe fn from_raw_parts<'a, T>(ptr: *const T, size: usize) -> &'a [T] {
    if ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(ptr, size)
    }
}
