fn affine2x3_to_matrix(t: FT_Affine23) -> Matrix {
    Matrix::new(
        t.xx.to_num(),
        t.yx.to_num(),
        t.xy.to_num(),
        t.yy.to_num(),
        t.dy.to_num(),
        t.dx.to_num(),
    )
}

fn record_to_cairo_surface(
    paint_ops: Vec<PaintOp>,
    scale_x: f64,
    scale_y: f64,
) -> anyhow::Result<(RecordingSurface, bool)> {
    let mut has_color = false;
    let surface = RecordingSurface::create(Content::ColorAlpha, None)?;
    let context = Context::new(&surface)?;
    context.scale(scale_x, scale_y);
    context.set_antialias(cairo::Antialias::Best);

    for pop in paint_ops {
        match pop {
            PaintOp::PushTransform(matrix) => {
                context.save()?;
                context.transform(matrix);
            }
            PaintOp::PopTransform => {
                context.restore()?;
            }
            PaintOp::PushClip(draw) => {
                context.save()?;
                apply_draw_ops_to_context(&draw, &context)?;
                context.clip();
            }
            PaintOp::PopClip => {
                context.restore()?;
            }
            PaintOp::PushGroup => {
                context.save()?;
                context.push_group();
            }
            PaintOp::PopGroup(operator) => {
                context.pop_group_to_source()?;
                context.set_operator(operator);
                context.paint()?;
                context.restore()?;
            }
            PaintOp::PaintSolid(color) => {
                if color.as_srgba32() != 0xffffffff {
                    has_color = true;
                }
                let (r, g, b, a) = color.as_srgba_tuple();
                context.set_source_rgba(r.into(), g.into(), b.into(), a.into());
                context.paint()?;
            }
            PaintOp::PaintLinearGradient {
                x0,
                y0,
                x1,
                y1,
                x2,
                y2,
                color_line,
            } => {
                has_color = true;
                paint_linear_gradient(
                    &context,
                    x0.into(),
                    y0.into(),
                    x1.into(),
                    y1.into(),
                    x2.into(),
                    y2.into(),
                    color_line,
                )?;
            }
            PaintOp::PaintRadialGradient {
                x0,
                y0,
                r0,
                x1,
                y1,
                r1,
                color_line,
            } => {
                has_color = true;
                paint_radial_gradient(
                    &context,
                    x0.into(),
                    y0.into(),
                    r0.into(),
                    x1.into(),
                    y1.into(),
                    r1.into(),
                    color_line,
                )?;
            }
            PaintOp::PaintSweepGradient {
                x0,
                y0,
                start_angle,
                end_angle,
                color_line,
            } => {
                has_color = true;
                paint_sweep_gradient(
                    &context,
                    x0.into(),
                    y0.into(),
                    start_angle.into(),
                    end_angle.into(),
                    color_line,
                )?;
            }
        }
    }

    Ok((surface, has_color))
}
