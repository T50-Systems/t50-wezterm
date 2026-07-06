// Higher level harfbuzz bindings
use freetype;

pub use harfbuzz::*;

use crate::locator::{FontDataHandle, FontDataSource};
use crate::rasterizer::colr::{ColorLine, ColorStop, DrawOp};
use anyhow::{ensure, Context, Error};
use cairo::Extend;
use memmap2::{Mmap, MmapOptions};
use std::ffi::CStr;
use std::io::Read;
use std::mem;
use std::ops::Range;
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::sync::Arc;
use wezterm_color_types::SrgbaPixel;

extern "C" {
    fn hb_ft_font_set_load_flags(font: *mut hb_font_t, load_flags: i32);
}

pub const IS_PNG: hb_tag_t = hb_tag(b'p', b'n', b'g', b' ');
#[allow(unused)]
pub const IS_SVG: hb_tag_t = hb_tag(b's', b'v', b'g', b' ');
pub const IS_BGRA: hb_tag_t = hb_tag(b'B', b'G', b'R', b'A');

pub fn language_from_string(s: &str) -> Result<hb_language_t, Error> {
    unsafe {
        let lang = hb_language_from_string(s.as_ptr() as *const c_char, s.len() as i32);
        ensure!(!lang.is_null(), "failed to convert {} to language", s);
        Ok(lang)
    }
}

pub fn feature_from_string(s: &str) -> Result<hb_feature_t, Error> {
    unsafe {
        let mut feature = mem::zeroed();
        ensure!(
            hb_feature_from_string(
                s.as_ptr() as *const c_char,
                s.len() as i32,
                &mut feature as *mut _,
            ) != 0,
            "failed to create feature from {}",
            s
        );
        Ok(feature)
    }
}

#[derive(Debug)]
pub struct Blob {
    blob: *mut hb_blob_t,
}

impl Drop for Blob {
    fn drop(&mut self) {
        unsafe {
            hb_blob_destroy(self.blob);
        }
    }
}

impl Clone for Blob {
    fn clone(&self) -> Self {
        unsafe { hb_blob_reference(self.blob) };
        Self { blob: self.blob }
    }
}

impl Blob {
    pub fn with_reference(blob: *mut hb_blob_t) -> Self {
        unsafe { hb_blob_reference(blob) };
        Self { blob }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let mut len = 0;
            let ptr = hb_blob_get_data(self.blob, &mut len);
            from_raw_parts(ptr as *const u8, len as usize)
        }
    }

    pub fn from_source(source: &FontDataSource) -> anyhow::Result<Self> {
        let blob = match source {
            FontDataSource::OnDisk(p) => {
                let mut file = std::fs::File::open(p)
                    .with_context(|| format!("opening file {}", p.display()))?;

                let meta = file
                    .metadata()
                    .with_context(|| format!("querying metadata for {}", p.display()))?;

                if !meta.is_file() {
                    anyhow::bail!("{} is not a file", p.display());
                }

                let len = meta.len();
                if len as usize > c_uint::MAX as usize {
                    anyhow::bail!(
                        "{} is too large to pass to harfbuzz! (len={})",
                        p.display(),
                        len
                    );
                }

                match unsafe { MmapOptions::new().map(&file) } {
                    Ok(map) => {
                        let data_ptr = map.as_ptr();
                        let data_len = map.len() as u32;
                        let user_data = Arc::new(map);

                        let user_data: *const Mmap = Arc::into_raw(user_data);

                        extern "C" fn release_arc_mmap(user_data: *mut c_void) {
                            let user_data = user_data as *mut Mmap;
                            let user_data: Arc<Mmap> = unsafe { Arc::from_raw(user_data) };
                            drop(user_data);
                        }

                        let blob = unsafe {
                            hb_blob_create_or_fail(
                                data_ptr as *const _,
                                data_len,
                                hb_memory_mode_t::HB_MEMORY_MODE_READONLY,
                                user_data as *mut _,
                                Some(release_arc_mmap),
                            )
                        };

                        if blob.is_null() {
                            release_arc_mmap(user_data as *mut _);
                        }

                        blob
                    }
                    Err(err) => {
                        log::warn!(
                            "Unable to memory map {}: {}, will use regular file IO instead",
                            p.display(),
                            err
                        );
                        let mut data = vec![];
                        file.read_to_end(&mut data)
                            .with_context(|| format!("reading font file {}", p.display()))?;
                        let data = Arc::new(data);

                        let data_ptr = data.as_ptr();
                        let data_len = data.len() as u32;
                        let user_data: *const Vec<u8> = Arc::into_raw(data);

                        extern "C" fn release_arc_vec(user_data: *mut c_void) {
                            let user_data = user_data as *mut Vec<u8>;
                            let user_data: Arc<Vec<u8>> = unsafe { Arc::from_raw(user_data) };
                            drop(user_data);
                        }

                        let blob = unsafe {
                            hb_blob_create_or_fail(
                                data_ptr as *const _,
                                data_len,
                                hb_memory_mode_t::HB_MEMORY_MODE_READONLY,
                                user_data as *mut _,
                                Some(release_arc_vec),
                            )
                        };

                        if blob.is_null() {
                            release_arc_vec(user_data as *mut _);
                        }

                        blob
                    }
                }
            }
            FontDataSource::BuiltIn { data, .. } => unsafe {
                hb_blob_create_or_fail(
                    data.as_ptr() as *const _,
                    data.len() as u32,
                    hb_memory_mode_t::HB_MEMORY_MODE_READONLY,
                    std::ptr::null_mut(),
                    None,
                )
            },
            FontDataSource::Memory { data, .. } => {
                let data_ptr = data.as_ptr();
                let data_len = data.len() as u32;
                let user_data: *const Box<[u8]> = Arc::into_raw(Arc::clone(data));

                extern "C" fn release_arc(user_data: *mut c_void) {
                    let user_data = user_data as *const Box<[u8]>;
                    let user_data: Arc<Box<[u8]>> = unsafe { Arc::from_raw(user_data) };
                    drop(user_data);
                }

                let blob = unsafe {
                    hb_blob_create_or_fail(
                        data_ptr as *const _,
                        data_len,
                        hb_memory_mode_t::HB_MEMORY_MODE_READONLY,
                        user_data as *mut _,
                        Some(release_arc),
                    )
                };

                if blob.is_null() {
                    release_arc(user_data as *mut _);
                }

                blob
            }
        };

        if blob.is_null() {
            anyhow::bail!("failed to wrap font as blob");
        }

        Ok(Self { blob })
    }
}

pub struct Face {
    face: *mut hb_face_t,
}
