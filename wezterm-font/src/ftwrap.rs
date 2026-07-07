//! Higher level freetype bindings

use crate::locator::{FontDataHandle, FontDataSource};
use crate::parser::ParsedFont;
use crate::rasterizer::colr::DrawOp;
use anyhow::{anyhow, Context};
use config::{configuration, FreeTypeLoadFlags, FreeTypeLoadTarget};
pub use freetype::*;
use memmap2::{Mmap, MmapOptions};
use rangeset::RangeSet;
use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::{c_int, c_void, CStr};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::mem::MaybeUninit;
use std::os::raw::{c_uchar, c_ulong};
use std::path::Path;
use std::ptr;
use std::sync::Arc;

#[inline]
pub fn succeeded(error: FT_Error) -> bool {
    error == freetype::FT_Err_Ok as FT_Error
}

/// Translate an error and value into a result
pub fn ft_result<T>(err: FT_Error, t: T) -> anyhow::Result<T> {
    if succeeded(err) {
        Ok(t)
    } else {
        unsafe {
            let reason = FT_Error_String(err);
            if reason.is_null() {
                Err(anyhow!("FreeType error {:?} 0x{:x}", err, err))
            } else {
                let reason = std::ffi::CStr::from_ptr(reason);
                Err(anyhow!(
                    "FreeType error {:?} 0x{:x}: {}",
                    err,
                    err,
                    reason.to_string_lossy()
                ))
            }
        }
    }
}

fn render_mode_to_load_target(render_mode: FT_Render_Mode) -> u32 {
    // enable FT_LOAD_TARGET bits.  There are no flags defined
    // for these in the bindings so we do some bit magic for
    // ourselves.  This is how the FT_LOAD_TARGET_() macro
    // assembles these bits.
    ((render_mode as u32) & 15) << 16
}

pub fn compute_load_flags_from_config(
    freetype_load_flags: Option<FreeTypeLoadFlags>,
    freetype_load_target: Option<FreeTypeLoadTarget>,
    freetype_render_target: Option<FreeTypeLoadTarget>,
    dpi: Option<u32>,
) -> (i32, FT_Render_Mode) {
    let config = configuration();

    let load_flags = freetype_load_flags
        .or(config.freetype_load_flags)
        .unwrap_or_else(|| match dpi {
            Some(dpi) if dpi >= 100 => FreeTypeLoadFlags::default_hidpi(),
            _ => FreeTypeLoadFlags::default(),
        })
        .bits()
        | FT_LOAD_COLOR;

    fn target_to_render(t: FreeTypeLoadTarget) -> FT_Render_Mode {
        match t {
            FreeTypeLoadTarget::Mono => FT_Render_Mode::FT_RENDER_MODE_MONO,
            FreeTypeLoadTarget::Normal => FT_Render_Mode::FT_RENDER_MODE_NORMAL,
            FreeTypeLoadTarget::Light => FT_Render_Mode::FT_RENDER_MODE_LIGHT,
            FreeTypeLoadTarget::HorizontalLcd => FT_Render_Mode::FT_RENDER_MODE_LCD,
            FreeTypeLoadTarget::VerticalLcd => FT_Render_Mode::FT_RENDER_MODE_LCD_V,
        }
    }

    let load_target = target_to_render(freetype_load_target.unwrap_or(config.freetype_load_target));
    let render = target_to_render(
        freetype_render_target.unwrap_or(
            config
                .freetype_render_target
                .unwrap_or(config.freetype_load_target),
        ),
    );

    let load_flags = load_flags | render_mode_to_load_target(load_target);

    (load_flags as i32, render)
}

pub struct Face {
    pub face: FT_Face,
    source: FontDataHandle,
    size: Option<FaceSize>,
    lib: FT_Library,
    palette: Option<&'static mut [FT_Color]>,
}

impl Drop for Face {
    fn drop(&mut self) {
        unsafe {
            FT_Done_Face(self.face);
        }
    }
}

struct FaceSize {
    size: f64,
    dpi: u32,
    cell_width: f64,
    cell_height: f64,
    cap_height: Option<f64>,
    cap_height_to_height_ratio: Option<f64>,
    is_scaled: bool,
}

#[derive(Debug)]
struct ComputedCellMetrics {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug)]
pub struct SelectedFontSize {
    pub width: f64,
    pub height: f64,
    pub cap_height: Option<f64>,
    pub cap_height_to_height_ratio: Option<f64>,
    pub is_scaled: bool,
}

#[derive(Debug, thiserror::Error)]
#[error("Glyph is SVG")]
pub struct IsSvg;

#[derive(Debug, thiserror::Error)]
#[error("Glyph is COLR1 or later")]
pub struct IsColr1OrLater;

include!("ftwrap/face_names.rs");
include!("ftwrap/face_size.rs");
include!("ftwrap/face_color.rs");
include!("ftwrap/face_glyph.rs");
include!("ftwrap/library.rs");
include!("ftwrap/stream.rs");
include!("ftwrap/types.rs");
