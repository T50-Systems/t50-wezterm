use crate::ftwrap::{
    composite_mode_to_operator, vector_x_y, FT_Affine23, FT_ColorIndex, FT_ColorLine, FT_ColorStop,
    FT_Fixed, FT_Get_Colorline_Stops, FT_Int32, FT_PaintExtend, IsColr1OrLater, IsSvg,
    SelectedFontSize, FT_LOAD_NO_HINTING,
};
use crate::parser::ParsedFont;
use crate::rasterizer::colr::{
    apply_draw_ops_to_context, paint_linear_gradient, paint_radial_gradient, paint_sweep_gradient,
    ColorLine, ColorStop, PaintOp,
};
use crate::rasterizer::harfbuzz::{argb_to_rgba, HarfbuzzRasterizer};
use crate::rasterizer::{FontRasterizer, FAKE_ITALIC_SKEW};
use crate::units::*;
use crate::{ftwrap, FontRasterizerSelection, RasterizedGlyph};
use ::freetype::{
    FT_Color_Root_Transform, FT_GlyphSlotRec_, FT_Matrix, FT_Opaque_Paint_, FT_PaintFormat_,
};
use anyhow::{bail, Context as _};
use cairo::{Content, Context, Extend, Format, ImageSurface, Matrix, Operator, RecordingSurface};
use config::{DisplayPixelGeometry, FreeTypeLoadFlags, FreeTypeLoadTarget};
use std::cell::RefCell;
use std::f64::consts::PI;
use std::mem;
use std::mem::MaybeUninit;
use wezterm_color_types::{linear_u8_to_srgb8, SrgbaPixel};

pub struct FreeTypeRasterizer {
    has_color: bool,
    face: RefCell<ftwrap::Face>,
    _lib: ftwrap::Library,
    synthesize_bold: bool,
    freetype_load_target: Option<FreeTypeLoadTarget>,
    freetype_render_target: Option<FreeTypeLoadTarget>,
    freetype_load_flags: Option<FreeTypeLoadFlags>,
    display_pixel_geometry: DisplayPixelGeometry,
    scale: f64,
    hb_raster: HarfbuzzRasterizer,
}

impl FontRasterizer for FreeTypeRasterizer {
    fn rasterize_glyph(
        &self,
        glyph_pos: u32,
        size: f64,
        dpi: u32,
    ) -> anyhow::Result<RasterizedGlyph> {
        let SelectedFontSize { is_scaled, .. } = self
            .face
            .borrow_mut()
            .set_font_size(size * self.scale, dpi)?;

        let (load_flags, render_mode) = ftwrap::compute_load_flags_from_config(
            self.freetype_load_flags,
            self.freetype_load_target,
            self.freetype_render_target,
            Some(dpi),
        );

        let mut face = self.face.borrow_mut();
        let ft_glyph = match face.load_and_render_glyph(
            glyph_pos,
            load_flags,
            render_mode,
            self.synthesize_bold,
        ) {
            Ok(g) => g,
            Err(err) => {
                if err.root_cause().downcast_ref::<IsSvg>().is_some()
                    || err.root_cause().downcast_ref::<IsColr1OrLater>().is_some()
                {
                    drop(face);

                    let config = config::configuration();
                    match config.font_colr_rasterizer {
                        FontRasterizerSelection::FreeType => {
                            return self.rasterize_outlines(
                                glyph_pos,
                                load_flags | FT_LOAD_NO_HINTING as i32,
                            );
                        }
                        FontRasterizerSelection::Harfbuzz => {
                            return self.hb_raster.rasterize_glyph(glyph_pos, size, dpi);
                        }
                    }
                }
                return Err(err);
            }
        };

        let mode: ftwrap::FT_Pixel_Mode =
            unsafe { mem::transmute(u32::from(ft_glyph.bitmap.pixel_mode)) };

        // pitch is the number of bytes per source row
        let pitch = ft_glyph.bitmap.pitch.abs() as usize;
        let data = unsafe {
            crate::ftwrap::from_raw_parts(
                ft_glyph.bitmap.buffer,
                ft_glyph.bitmap.rows as usize * pitch,
            )
        };

        let glyph = match mode {
            ftwrap::FT_Pixel_Mode::FT_PIXEL_MODE_LCD => {
                self.rasterize_lcd(pitch, ft_glyph, data, is_scaled)
            }
            ftwrap::FT_Pixel_Mode::FT_PIXEL_MODE_LCD_V => {
                self.rasterize_lcd_v(pitch, ft_glyph, data, is_scaled)
            }
            ftwrap::FT_Pixel_Mode::FT_PIXEL_MODE_BGRA => {
                self.rasterize_bgra(pitch, ft_glyph, data, is_scaled)?
            }
            ftwrap::FT_Pixel_Mode::FT_PIXEL_MODE_GRAY => {
                self.rasterize_gray(pitch, ft_glyph, data, is_scaled)
            }
            ftwrap::FT_Pixel_Mode::FT_PIXEL_MODE_MONO => {
                self.rasterize_mono(pitch, ft_glyph, data, is_scaled)
            }
            mode => bail!("unhandled pixel mode: {:?}", mode),
        };

        Ok(glyph)
    }
}
