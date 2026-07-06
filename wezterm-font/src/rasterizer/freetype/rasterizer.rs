impl FreeTypeRasterizer {
    fn rasterize_mono(
        &self,
        pitch: usize,
        ft_glyph: &FT_GlyphSlotRec_,
        data: &[u8],
        is_scaled: bool,
    ) -> RasterizedGlyph {
        let width = ft_glyph.bitmap.width as usize;
        let height = ft_glyph.bitmap.rows as usize;
        let size = (width * height * 4) as usize;
        let mut rgba = vec![0u8; size];
        for y in 0..height {
            let src_offset = y * pitch;
            let dest_offset = y * width * 4;
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
                        for j in 0..4 {
                            rgba[dest_offset + (x * 4) + j] = 0xff;
                        }
                    }
                    b <<= 1;
                    x += 1;
                }
            }
        }
        RasterizedGlyph {
            data: rgba,
            height,
            width,
            bearing_x: PixelLength::new(ft_glyph.bitmap_left as f64),
            bearing_y: PixelLength::new(ft_glyph.bitmap_top as f64),
            has_color: false,
            is_scaled,
        }
    }

    fn rasterize_gray(
        &self,
        pitch: usize,
        ft_glyph: &FT_GlyphSlotRec_,
        data: &[u8],
        is_scaled: bool,
    ) -> RasterizedGlyph {
        let width = ft_glyph.bitmap.width as usize;
        let height = ft_glyph.bitmap.rows as usize;
        let size = (width * height * 4) as usize;
        let mut rgba = vec![0u8; size];
        for y in 0..height {
            let src_offset = y * pitch;
            let dest_offset = y * width * 4;
            for x in 0..width {
                let linear_gray = data[src_offset + x];
                let gray = linear_u8_to_srgb8(linear_gray);

                // Texture is SRGBA, which in OpenGL means
                // that the RGB values are gamma adjusted
                // non-linear values, but the A value is
                // linear!

                rgba[dest_offset + (x * 4)] = gray;
                rgba[dest_offset + (x * 4) + 1] = gray;
                rgba[dest_offset + (x * 4) + 2] = gray;
                rgba[dest_offset + (x * 4) + 3] = linear_gray;
            }
        }
        RasterizedGlyph {
            data: rgba,
            height,
            width,
            bearing_x: PixelLength::new(ft_glyph.bitmap_left as f64),
            bearing_y: PixelLength::new(ft_glyph.bitmap_top as f64),
            has_color: false,
            is_scaled,
        }
    }

    fn rasterize_lcd(
        &self,
        pitch: usize,
        ft_glyph: &FT_GlyphSlotRec_,
        data: &[u8],
        is_scaled: bool,
    ) -> RasterizedGlyph {
        let width = ft_glyph.bitmap.width as usize / 3;
        let height = ft_glyph.bitmap.rows as usize;
        let size = (width * height * 4) as usize;
        let mut rgba = vec![0u8; size];
        for y in 0..height {
            let src_offset = y * pitch as usize;
            let dest_offset = y * width * 4;
            for x in 0..width {
                let red = data[src_offset + (x * 3)];
                let green = data[src_offset + (x * 3) + 1];
                let blue = data[src_offset + (x * 3) + 2];

                let linear_alpha = red.max(green).max(blue);

                // Texture is SRGBA, which in OpenGL means
                // that the RGB values are gamma adjusted
                // non-linear values, but the A value is
                // linear!

                let red = linear_u8_to_srgb8(red);
                let green = linear_u8_to_srgb8(green);
                let blue = linear_u8_to_srgb8(blue);

                let (red, blue) = match self.display_pixel_geometry {
                    DisplayPixelGeometry::RGB => (red, blue),
                    DisplayPixelGeometry::BGR => (blue, red),
                };

                rgba[dest_offset + (x * 4)] = red;
                rgba[dest_offset + (x * 4) + 1] = green;
                rgba[dest_offset + (x * 4) + 2] = blue;
                rgba[dest_offset + (x * 4) + 3] = linear_alpha;
            }
        }

        RasterizedGlyph {
            data: rgba,
            height,
            width,
            bearing_x: PixelLength::new(ft_glyph.bitmap_left as f64),
            bearing_y: PixelLength::new(ft_glyph.bitmap_top as f64),
            has_color: self.has_color,
            is_scaled,
        }
    }

    fn rasterize_lcd_v(
        &self,
        pitch: usize,
        ft_glyph: &FT_GlyphSlotRec_,
        data: &[u8],
        is_scaled: bool,
    ) -> RasterizedGlyph {
        let width = ft_glyph.bitmap.width as usize;
        let height = ft_glyph.bitmap.rows as usize / 3;
        let size = width * height * 4;
        let mut rgba = vec![0u8; size];
        for y in 0..height {
            let src_offset = y * pitch * 3;
            let dest_offset = y * width * 4;
            for x in 0..width {
                let red = data[src_offset + x];
                let green = data[src_offset + x + pitch];
                let blue = data[src_offset + x + 2 * pitch];

                let linear_alpha = red.max(green).max(blue);

                // Texture is SRGBA, which in OpenGL means
                // that the RGB values are gamma adjusted
                // non-linear values, but the A value is
                // linear!

                let red = linear_u8_to_srgb8(red);
                let green = linear_u8_to_srgb8(green);
                let blue = linear_u8_to_srgb8(blue);

                let (red, blue) = match self.display_pixel_geometry {
                    DisplayPixelGeometry::RGB => (red, blue),
                    DisplayPixelGeometry::BGR => (blue, red),
                };

                rgba[dest_offset + (x * 4)] = red;
                rgba[dest_offset + (x * 4) + 1] = green;
                rgba[dest_offset + (x * 4) + 2] = blue;
                rgba[dest_offset + (x * 4) + 3] = linear_alpha;
            }
        }

        RasterizedGlyph {
            data: rgba,
            height,
            width,
            bearing_x: PixelLength::new(ft_glyph.bitmap_left as f64),
            bearing_y: PixelLength::new(ft_glyph.bitmap_top as f64),
            has_color: self.has_color,
            is_scaled,
        }
    }

    fn rasterize_bgra(
        &self,
        pitch: usize,
        ft_glyph: &FT_GlyphSlotRec_,
        data: &'static [u8],
        is_scaled: bool,
    ) -> anyhow::Result<RasterizedGlyph> {
        let width = ft_glyph.bitmap.width as usize;
        let height = ft_glyph.bitmap.rows as usize;

        if width == 0 || height == 0 {
            // Handle this case separately; the ImageBuffer
            // constructor doesn't like 0-size dimensions
            return Ok(RasterizedGlyph {
                data: vec![],
                height: 0,
                width: 0,
                bearing_x: PixelLength::new(0.),
                bearing_y: PixelLength::new(0.),
                has_color: false,
                is_scaled,
            });
        }

        let mut source_image = image::ImageBuffer::<image::Rgba<u8>, &[u8]>::from_raw(
            width as u32,
            height as u32,
            data,
        )
        .with_context(|| {
            format!(
                "build image from data with \
                 width={width}, height={height} and pitch={pitch}.\
                 Expected pitch={}. format is {:?}",
                width * 4,
                ft_glyph.format
            )
        })?;

        // emoji glyphs don't always fill the bitmap size, so we compute
        // the non-transparent bounds

        let mut cropped = crate::rasterizer::crop_to_non_transparent(&mut source_image).to_image();
        crate::rasterizer::swap_red_and_blue(&mut cropped);

        let dest_width = cropped.width() as usize;
        let dest_height = cropped.height() as usize;

        Ok(RasterizedGlyph {
            data: cropped.into_vec(),
            height: dest_height,
            width: dest_width,
            bearing_x: PixelLength::new(
                f64::from(ft_glyph.bitmap_left) * (dest_width as f64 / width as f64),
            ),
            bearing_y: PixelLength::new(
                f64::from(ft_glyph.bitmap_top) * (dest_height as f64 / height as f64),
            ),
            has_color: self.has_color,
            is_scaled,
        })
    }

    pub fn from_locator(
        parsed: &ParsedFont,
        display_pixel_geometry: DisplayPixelGeometry,
    ) -> anyhow::Result<Self> {
        log::trace!("Rasterizier wants {:?}", parsed);
        let lib = ftwrap::Library::new()?;
        let mut face = lib.face_from_locator(&parsed.handle)?;
        let has_color = unsafe {
            (((*face.face).face_flags as u32) & (ftwrap::FT_FACE_FLAG_COLOR as u32)) != 0
        };

        if parsed.synthesize_italic {
            face.set_transform(Some(FT_Matrix {
                xx: FT_Fixed::from_num(1),                // scale x
                yy: FT_Fixed::from_num(1),                // scale y
                xy: FT_Fixed::from_num(FAKE_ITALIC_SKEW), // skew x
                yx: FT_Fixed::from_num(0),                // skew y
            }));
        }

        Ok(Self {
            _lib: lib,
            face: RefCell::new(face),
            has_color,
            synthesize_bold: parsed.synthesize_bold,
            freetype_load_flags: parsed.freetype_load_flags,
            freetype_load_target: parsed.freetype_load_target,
            freetype_render_target: parsed.freetype_render_target,
            display_pixel_geometry,
            scale: parsed.scale.unwrap_or(1.),
            hb_raster: HarfbuzzRasterizer::from_locator(&parsed)?,
        })
    }

    fn rasterize_outlines(
        &self,
        glyph_pos: u32,
        load_flags: FT_Int32,
    ) -> anyhow::Result<RasterizedGlyph> {
        let mut face = self.face.borrow_mut();
        let paint = face.get_color_glyph_paint(
            glyph_pos,
            FT_Color_Root_Transform::FT_COLOR_INCLUDE_ROOT_TRANSFORM,
        )?;

        // The root transform produces extents that are larger than
        // our nominal pixel size. I'm not sure why that is, but the
        // factor corresponds to the metrics.(x|y)_scale in the root
        // transform.
        // It is desirable to retain the root transform as it includes
        // any skew that may have been applied to the font.
        // So let's extract the offending scaling factors and we'll
        // compensate when we rasterize the paths.
        let (scale_x, scale_y) = unsafe {
            let upem = (*face.face).units_per_EM as f64;
            let metrics = (*(*face.face).size).metrics;
            log::trace!("upem={upem}, metrics: {metrics:#?}");

            (
                1. / metrics.x_scale.to_num::<f64>(),
                1. / metrics.y_scale.to_num::<f64>(),
            )
        };

        let palette = face.get_palette_data()?;
        log::trace!("Palette: {palette:#?}");
        face.select_palette(0)?;

        let clip_box = face.get_color_glyph_clip_box(glyph_pos)?;
        log::trace!("got clip_box: {clip_box:?}");
        let mut walker = Walker {
            load_flags,
            face: &mut face,
            ops: vec![],
        };
        walker.walk_paint(paint, 0)?;

        log::trace!("ops: {:#?}", walker.ops);

        rasterize_from_ops(walker.ops, scale_x, -scale_y)
    }
}

fn rasterize_from_ops(
    ops: Vec<PaintOp>,
    scale_x: f64,
    scale_y: f64,
) -> anyhow::Result<RasterizedGlyph> {
    let (surface, has_color) = record_to_cairo_surface(ops, scale_x, scale_y)?;
    let (left, top, width, height) = surface.ink_extents();
    log::trace!("extents: left={left} top={top} width={width} height={height}");

    if width as usize == 0 || height as usize == 0 {
        return Ok(RasterizedGlyph {
            data: vec![],
            height: 0,
            width: 0,
            bearing_x: PixelLength::new(0.),
            bearing_y: PixelLength::new(0.),
            has_color: false,
            is_scaled: true,
        });
    }

    let mut bounds_adjust = Matrix::identity();
    bounds_adjust.translate(left * -1., top * -1.);
    log::trace!("dims: {width}x{height} {bounds_adjust:?}");

    let target = ImageSurface::create(Format::ARgb32, width as i32, height as i32)?;
    {
        let context = Context::new(&target)?;
        context.transform(bounds_adjust);
        context.set_antialias(cairo::Antialias::Best);
        context.set_source_surface(surface, 0., 0.)?;
        context.paint()?;
    }

    let mut data = target.take_data()?.to_vec();
    argb_to_rgba(&mut data);

    Ok(RasterizedGlyph {
        data,
        height: height as usize,
        width: width as usize,
        bearing_x: PixelLength::new(left.min(0.)),
        bearing_y: PixelLength::new(top * -1.),
        has_color,
        is_scaled: true,
    })
}
