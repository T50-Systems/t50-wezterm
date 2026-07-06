struct Walker<'a> {
    load_flags: FT_Int32,
    face: &'a mut crate::ftwrap::Face,
    ops: Vec<PaintOp>,
}

impl<'a> Walker<'a> {
    fn walk_paint(&mut self, paint: FT_Opaque_Paint_, level: usize) -> anyhow::Result<()> {
        use FT_PaintFormat_::*;

        let paint = self.face.get_paint(paint)?;

        unsafe {
            match paint.format {
                FT_COLR_PAINTFORMAT_COLR_LAYERS => {
                    log::trace!("{level:>3} {:?}", paint.format);
                    let mut iter = paint.u.colr_layers.as_ref().layer_iterator;
                    while let Ok(inner_paint) = self.face.get_paint_layers(&mut iter) {
                        self.walk_paint(inner_paint, level + 1)?;
                    }
                }
                FT_COLR_PAINTFORMAT_SOLID => {
                    let op = PaintOp::PaintSolid(
                        self.decode_color_index(&paint.u.solid.as_ref().color)?,
                    );
                    log::trace!("{level:>3} {:?} {op:x?}", paint.format);
                    self.ops.push(op);
                }
                FT_COLR_PAINTFORMAT_LINEAR_GRADIENT => {
                    let grad = paint.u.linear_gradient.as_ref();
                    log::trace!("{level:>3} {grad:?}");
                    let (x0, y0) = vector_x_y(&grad.p0);
                    let (x1, y1) = vector_x_y(&grad.p1);
                    let (x2, y2) = vector_x_y(&grad.p2);
                    // FIXME: gradient vectors are expressed as font units,
                    // do we need to adjust them here?
                    let paint = PaintOp::PaintLinearGradient {
                        x0,
                        y0,
                        x1,
                        y1,
                        x2,
                        y2,
                        color_line: self.decode_color_line(&grad.colorline)?,
                    };
                    self.ops.push(paint);
                }
                FT_COLR_PAINTFORMAT_RADIAL_GRADIENT => {
                    let grad = paint.u.radial_gradient.as_ref();
                    log::trace!("{level:>3} {grad:?}");
                    let (x0, y0) = vector_x_y(&grad.c0);
                    let (x1, y1) = vector_x_y(&grad.c1);

                    let paint = PaintOp::PaintRadialGradient {
                        x0,
                        y0,
                        x1,
                        y1,
                        r0: grad.r0.font_units() as f32,
                        r1: grad.r1.font_units() as f32,
                        color_line: self.decode_color_line(&grad.colorline)?,
                    };
                    self.ops.push(paint);
                }
                FT_COLR_PAINTFORMAT_SWEEP_GRADIENT => {
                    let grad = paint.u.sweep_gradient.as_ref();
                    log::trace!("{level:>3} {grad:?}");
                    let (x0, y0) = vector_x_y(&grad.center);
                    let start_angle = grad.start_angle.to_num();
                    let end_angle = grad.end_angle.to_num();

                    let paint = PaintOp::PaintSweepGradient {
                        x0,
                        y0,
                        start_angle,
                        end_angle,
                        color_line: self.decode_color_line(&grad.colorline)?,
                    };
                    self.ops.push(paint);
                }
                FT_COLR_PAINTFORMAT_GLYPH => {
                    // FIXME: harfbuzz, in COLR.hh, pushes the inverse of
                    // the root transform before emitting the glyph
                    // DrawOps, then pops it prior to recursing into
                    // the child paint
                    log::trace!("{level:>3} {:?}", paint.u.glyph.as_ref());

                    let glyph_index = paint.u.glyph.as_ref().glyphID;

                    let ops = self
                        .face
                        .load_glyph_outlines(glyph_index, self.load_flags)?;
                    log::trace!("{level:>3} -> {ops:?}");
                    self.ops.push(PaintOp::PushClip(ops));

                    self.walk_paint(paint.u.glyph.as_ref().paint, level + 1)?;

                    self.ops.push(PaintOp::PopClip);
                }
                FT_COLR_PAINTFORMAT_COLR_GLYPH => {
                    let g = paint.u.colr_glyph.as_ref();
                    log::trace!("{level:>3} {g:?}");
                    self.ops.push(PaintOp::PushGroup);
                    let paint = self.face.get_color_glyph_paint(
                        g.glyphID,
                        FT_Color_Root_Transform::FT_COLOR_NO_ROOT_TRANSFORM,
                    )?;

                    self.walk_paint(paint, level + 1)?;
                    self.ops.push(PaintOp::PopGroup(Operator::Over));
                }
                FT_COLR_PAINTFORMAT_TRANSFORM => {
                    let t = paint.u.transform.as_ref();
                    let matrix = affine2x3_to_matrix(t.affine);
                    log::trace!("{level:>3} {t:?} -> {matrix:?}");
                    self.ops.push(PaintOp::PushTransform(matrix));
                    self.walk_paint(t.paint, level + 1)?;
                    self.ops.push(PaintOp::PopTransform);
                }
                FT_COLR_PAINTFORMAT_TRANSLATE => {
                    let t = paint.u.translate.as_ref();
                    log::trace!("{level:>3} {t:?}");

                    let mut matrix = Matrix::identity();
                    matrix.translate(t.dx.to_num(), t.dy.to_num());
                    self.ops.push(PaintOp::PushTransform(matrix));
                    self.walk_paint(t.paint, level + 1)?;
                    self.ops.push(PaintOp::PopTransform);
                }
                FT_COLR_PAINTFORMAT_SCALE => {
                    let scale = paint.u.scale.as_ref();
                    log::trace!("{level:>3} {scale:?}");

                    // Scaling around a center coordinate
                    let center_x = scale.center_x.to_num();
                    let center_y = scale.center_x.to_num();

                    let mut p1 = Matrix::identity();
                    p1.translate(center_x, center_y);

                    let mut p2 = Matrix::identity();
                    p2.scale(scale.scale_x.to_num(), scale.scale_y.to_num());

                    let mut p3 = Matrix::identity();
                    p3.translate(-center_x, -center_y);

                    self.ops.push(PaintOp::PushTransform(p1));
                    self.ops.push(PaintOp::PushTransform(p2));
                    self.ops.push(PaintOp::PushTransform(p3));
                    self.walk_paint(scale.paint, level + 1)?;
                    self.ops.push(PaintOp::PopTransform);
                    self.ops.push(PaintOp::PopTransform);
                    self.ops.push(PaintOp::PopTransform);
                }
                FT_COLR_PAINTFORMAT_ROTATE => {
                    let rot = paint.u.rotate.as_ref();
                    log::trace!("{level:>3} {rot:?}");

                    // Rotating around a center coordinate
                    let center_x = rot.center_x.to_num();
                    let center_y = rot.center_x.to_num();

                    let mut p1 = Matrix::identity();
                    p1.translate(center_x, center_y);

                    let mut p2 = Matrix::identity();
                    p2.rotate(PI * rot.angle.to_num::<f64>());

                    let mut p3 = Matrix::identity();
                    p3.translate(-center_x, -center_y);

                    self.ops.push(PaintOp::PushTransform(p1));
                    self.ops.push(PaintOp::PushTransform(p2));
                    self.ops.push(PaintOp::PushTransform(p3));
                    self.walk_paint(rot.paint, level + 1)?;
                    self.ops.push(PaintOp::PopTransform);
                    self.ops.push(PaintOp::PopTransform);
                    self.ops.push(PaintOp::PopTransform);
                }
                FT_COLR_PAINTFORMAT_SKEW => {
                    let skew = paint.u.skew.as_ref();
                    log::trace!("{level:>3} {skew:?}");

                    // Skewing around a center coordinate
                    let center_x = skew.center_x.to_num();
                    let center_y = skew.center_x.to_num();

                    let mut p1 = Matrix::identity();
                    p1.translate(center_x, center_y);

                    let x_skew_angle: f64 = skew.x_skew_angle.to_num();
                    let y_skew_angle: f64 = skew.y_skew_angle.to_num();
                    let x = (PI * -x_skew_angle).tan();
                    let y = (PI * y_skew_angle).tan();

                    let p2 = Matrix::new(1., y, x, 1., 0., 0.);

                    let mut p3 = Matrix::identity();
                    p3.translate(-center_x, -center_y);

                    self.ops.push(PaintOp::PushTransform(p1));
                    self.ops.push(PaintOp::PushTransform(p2));
                    self.ops.push(PaintOp::PushTransform(p3));
                    self.walk_paint(skew.paint, level + 1)?;
                    self.ops.push(PaintOp::PopTransform);
                    self.ops.push(PaintOp::PopTransform);
                    self.ops.push(PaintOp::PopTransform);
                }
                FT_COLR_PAINTFORMAT_COMPOSITE => {
                    let comp = paint.u.composite.as_ref();
                    log::trace!("{level:>3} {comp:?}");

                    self.walk_paint(comp.backdrop_paint, level + 1)?;
                    self.ops.push(PaintOp::PushGroup);
                    self.walk_paint(comp.source_paint, level + 1)?;
                    self.ops.push(PaintOp::PopGroup(composite_mode_to_operator(
                        comp.composite_mode,
                    )));
                }
                wat => {
                    anyhow::bail!("unknown/unhandled FT_PaintFormat_ value {wat:?}");
                }
            }
        }

        Ok(())
    }

    fn decode_color_index(&mut self, c: &FT_ColorIndex) -> anyhow::Result<SrgbaPixel> {
        let alpha: f64 = c.alpha.to_num();
        let (r, g, b, a) = if c.palette_index == 0xffff {
            // Foreground color.
            // We use white here because the rendering stage will
            // tint this with the actual color in the correct context
            (0xff, 0xff, 0xff, 1.0)
        } else {
            let color = self.face.get_palette_entry(c.palette_index as _)?;
            (
                color.red,
                color.green,
                color.blue,
                color.alpha as f64 / 255.,
            )
        };

        let alpha = (a * alpha * 255.) as u8;
        Ok(SrgbaPixel::rgba(r, g, b, alpha))
    }

    fn decode_color_line(&mut self, line: &FT_ColorLine) -> anyhow::Result<ColorLine> {
        let mut iter = line.color_stop_iterator;
        let mut color_stops = vec![];
        loop {
            let mut stop = MaybeUninit::<FT_ColorStop>::zeroed();

            if unsafe { FT_Get_Colorline_Stops(self.face.face, stop.as_mut_ptr(), &mut iter) } == 0
            {
                break;
            }

            let stop = unsafe { stop.assume_init() };

            color_stops.push(ColorStop {
                offset: stop.stop_offset.to_num(),
                color: self.decode_color_index(&stop.color)?,
            });
        }

        Ok(ColorLine {
            extend: match line.extend {
                FT_PaintExtend::FT_COLR_PAINT_EXTEND_PAD => Extend::Pad,
                FT_PaintExtend::FT_COLR_PAINT_EXTEND_REPEAT => Extend::Repeat,
                FT_PaintExtend::FT_COLR_PAINT_EXTEND_REFLECT => Extend::Reflect,
            },
            color_stops,
        })
    }
}
