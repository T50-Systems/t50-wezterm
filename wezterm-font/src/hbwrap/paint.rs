#[derive(Debug, Clone)]
pub enum PaintOp {
    PushTransform {
        xx: f32,
        yx: f32,
        xy: f32,
        yy: f32,
        dx: f32,
        dy: f32,
    },
    PopTransform,
    PushGlyphClip {
        #[allow(unused)]
        glyph: hb_codepoint_t,
        draw: Vec<DrawOp>,
    },
    PushRectClip {
        xmin: f32,
        ymin: f32,
        xmax: f32,
        ymax: f32,
    },
    PopClip,
    PaintSolid {
        #[allow(unused)]
        is_foreground: bool,
        color: hb_color_t,
    },
    PaintImage {
        image: Blob,
        #[allow(unused)]
        width: u32,
        #[allow(unused)]
        height: u32,
        format: hb_tag_t,
        slant: f32,
        extents: Option<hb_glyph_extents_t>,
    },
    PaintLinearGradient {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color_line: ColorLine,
    },
    PaintRadialGradient {
        x0: f32,
        y0: f32,
        r0: f32,
        x1: f32,
        y1: f32,
        r1: f32,
        color_line: ColorLine,
    },
    PaintSweepGradient {
        x0: f32,
        y0: f32,
        start_angle: f32,
        end_angle: f32,
        color_line: ColorLine,
    },
    PushGroup,
    PopGroup {
        mode: hb_paint_composite_mode_t,
    },
}

impl PaintOp {
    unsafe fn paint_data(data: *mut ::std::os::raw::c_void) -> &'static mut Vec<PaintOp> {
        &mut *(data as *mut Vec<PaintOp>)
    }

    unsafe extern "C" fn push_transform(
        _funcs: *mut hb_paint_funcs_t,
        paint_data: *mut ::std::os::raw::c_void,
        xx: f32,
        yx: f32,
        xy: f32,
        yy: f32,
        dx: f32,
        dy: f32,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::paint_data(paint_data);
        ops.push(Self::PushTransform {
            xx,
            yx,
            xy,
            yy,
            dx,
            dy,
        });
    }

    unsafe extern "C" fn pop_transform(
        _funcs: *mut hb_paint_funcs_t,
        paint_data: *mut ::std::os::raw::c_void,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::paint_data(paint_data);
        ops.push(Self::PopTransform);
    }

    unsafe extern "C" fn push_clip_rect(
        _funcs: *mut hb_paint_funcs_t,
        paint_data: *mut ::std::os::raw::c_void,
        xmin: f32,
        ymin: f32,
        xmax: f32,
        ymax: f32,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::paint_data(paint_data);
        ops.push(Self::PushRectClip {
            xmin,
            ymin,
            xmax,
            ymax,
        });
    }

    unsafe extern "C" fn pop_clip(
        _funcs: *mut hb_paint_funcs_t,
        paint_data: *mut ::std::os::raw::c_void,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::paint_data(paint_data);
        ops.push(Self::PopClip);
    }

    unsafe extern "C" fn paint_solid(
        _funcs: *mut hb_paint_funcs_t,
        paint_data: *mut ::std::os::raw::c_void,
        is_foreground: hb_bool_t,
        color: hb_color_t,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::paint_data(paint_data);
        ops.push(Self::PaintSolid {
            is_foreground: is_foreground != 0,
            color,
        });
    }

    unsafe extern "C" fn paint_linear_gradient(
        _funcs: *mut hb_paint_funcs_t,
        paint_data: *mut ::std::os::raw::c_void,
        color_line: *mut hb_color_line_t,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::paint_data(paint_data);
        let color_line = ColorLine::new_from_hb(color_line);
        ops.push(Self::PaintLinearGradient {
            color_line,
            x0,
            y0,
            x1,
            y1,
            x2,
            y2,
        });
    }

    unsafe extern "C" fn paint_radial_gradient(
        _funcs: *mut hb_paint_funcs_t,
        paint_data: *mut ::std::os::raw::c_void,
        color_line: *mut hb_color_line_t,
        x0: f32,
        y0: f32,
        r0: f32,
        x1: f32,
        y1: f32,
        r1: f32,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::paint_data(paint_data);
        let color_line = ColorLine::new_from_hb(color_line);
        ops.push(Self::PaintRadialGradient {
            color_line,
            x0,
            y0,
            r0,
            x1,
            y1,
            r1,
        });
    }

    unsafe extern "C" fn paint_sweep_gradient(
        _funcs: *mut hb_paint_funcs_t,
        paint_data: *mut ::std::os::raw::c_void,
        color_line: *mut hb_color_line_t,
        x0: f32,
        y0: f32,
        start_angle: f32,
        end_angle: f32,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::paint_data(paint_data);
        let color_line = ColorLine::new_from_hb(color_line);
        ops.push(Self::PaintSweepGradient {
            color_line,
            x0,
            y0,
            start_angle,
            end_angle,
        });
    }

    unsafe extern "C" fn paint_image(
        _funcs: *mut hb_paint_funcs_t,
        paint_data: *mut ::std::os::raw::c_void,
        image: *mut hb_blob_t,
        width: ::std::os::raw::c_uint,
        height: ::std::os::raw::c_uint,
        format: hb_tag_t,
        slant: f32,
        extents: *mut hb_glyph_extents_t,
        _user_data: *mut ::std::os::raw::c_void,
    ) -> hb_bool_t {
        if format != IS_PNG && format != IS_BGRA {
            // We only support PNG and BGRA
            return 0;
        }

        let ops = Self::paint_data(paint_data);
        let image = Blob::with_reference(image);
        let extents = if extents.is_null() {
            None
        } else {
            Some(*extents)
        };
        ops.push(Self::PaintImage {
            image,
            extents,
            width,
            height,
            format,
            slant,
        });

        1
    }

    unsafe extern "C" fn push_group(
        _funcs: *mut hb_paint_funcs_t,
        paint_data: *mut ::std::os::raw::c_void,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::paint_data(paint_data);
        ops.push(Self::PushGroup);
    }

    unsafe extern "C" fn pop_group(
        _funcs: *mut hb_paint_funcs_t,
        paint_data: *mut ::std::os::raw::c_void,
        mode: hb_paint_composite_mode_t,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::paint_data(paint_data);
        ops.push(Self::PopGroup { mode });
    }

    unsafe extern "C" fn push_clip_glyph(
        _funcs: *mut hb_paint_funcs_t,
        paint_data: *mut ::std::os::raw::c_void,
        glyph: hb_codepoint_t,
        font: *mut hb_font_t,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::paint_data(paint_data);

        let mut draw = vec![];

        let funcs = DrawFuncs::new().unwrap();
        macro_rules! func {
            ($hbfunc:ident, $method:ident) => {
                $hbfunc(
                    funcs.funcs,
                    Some(DrawOp::$method),
                    std::ptr::null_mut(),
                    None,
                );
            };
        }
        func!(hb_draw_funcs_set_move_to_func, move_to);
        func!(hb_draw_funcs_set_line_to_func, line_to);
        func!(hb_draw_funcs_set_quadratic_to_func, quad_to);
        func!(hb_draw_funcs_set_cubic_to_func, cubic_to);
        func!(hb_draw_funcs_set_close_path_func, close_path);

        hb_font_draw_glyph(
            font,
            glyph,
            funcs.funcs,
            &mut draw as *mut Vec<DrawOp> as *mut _,
        );

        ops.push(Self::PushGlyphClip { glyph, draw });
    }
}

impl DrawOp {
    unsafe fn draw_data(data: *mut ::std::os::raw::c_void) -> &'static mut Vec<DrawOp> {
        &mut *(data as *mut Vec<DrawOp>)
    }

    unsafe extern "C" fn move_to(
        _dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut ::std::os::raw::c_void,
        _st: *mut hb_draw_state_t,
        to_x: f32,
        to_y: f32,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::draw_data(draw_data);
        ops.push(Self::MoveTo { to_x, to_y });
    }

    unsafe extern "C" fn line_to(
        _dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut ::std::os::raw::c_void,
        _st: *mut hb_draw_state_t,
        to_x: f32,
        to_y: f32,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::draw_data(draw_data);
        ops.push(Self::LineTo { to_x, to_y });
    }

    unsafe extern "C" fn quad_to(
        _dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut ::std::os::raw::c_void,
        _st: *mut hb_draw_state_t,
        control_x: f32,
        control_y: f32,
        to_x: f32,
        to_y: f32,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::draw_data(draw_data);
        ops.push(Self::QuadTo {
            control_x,
            control_y,
            to_x,
            to_y,
        });
    }

    unsafe extern "C" fn cubic_to(
        _dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut ::std::os::raw::c_void,
        _st: *mut hb_draw_state_t,
        control1_x: f32,
        control1_y: f32,
        control2_x: f32,
        control2_y: f32,
        to_x: f32,
        to_y: f32,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::draw_data(draw_data);
        ops.push(Self::CubicTo {
            control1_x,
            control1_y,
            control2_x,
            control2_y,
            to_x,
            to_y,
        });
    }

    unsafe extern "C" fn close_path(
        _dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut ::std::os::raw::c_void,
        _st: *mut hb_draw_state_t,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        let ops = Self::draw_data(draw_data);
        ops.push(Self::ClosePath);
    }
}

impl ColorLine {
    pub fn new_from_hb(line: *mut hb_color_line_t) -> Self {
        let num_stops = unsafe {
            hb_color_line_get_color_stops(line, 0, std::ptr::null_mut(), std::ptr::null_mut())
        };
        let mut color_stops = Vec::with_capacity(num_stops as usize);
        color_stops.resize(
            num_stops as usize,
            hb_color_stop_t {
                offset: 0.,
                is_foreground: 0,
                color: 0,
            },
        );

        unsafe {
            let mut count = num_stops;
            hb_color_line_get_color_stops(line, 0, &mut count, color_stops.as_mut_ptr());
        }

        let extend = unsafe { hb_color_line_get_extend(line) };

        Self {
            color_stops: color_stops
                .into_iter()
                .map(|stop| ColorStop {
                    offset: stop.offset.into(),
                    color: if stop.is_foreground != 0 {
                        SrgbaPixel::rgba(0xff, 0xff, 0xff, 0xff)
                    } else {
                        hb_color_to_srgba_pixel(stop.color)
                    },
                })
                .collect(),
            extend: hb_extend_to_cairo(extend),
        }
    }
}

fn hb_color_to_srgba_pixel(color: hb_color_t) -> SrgbaPixel {
    let red = unsafe { hb_color_get_red(color) };
    let green = unsafe { hb_color_get_green(color) };
    let blue = unsafe { hb_color_get_blue(color) };
    let alpha = unsafe { hb_color_get_alpha(color) };
    SrgbaPixel::rgba(red, green, blue, alpha)
}

fn hb_extend_to_cairo(extend: hb_paint_extend_t) -> Extend {
    match extend {
        hb_paint_extend_t::HB_PAINT_EXTEND_PAD => Extend::Pad,
        hb_paint_extend_t::HB_PAINT_EXTEND_REPEAT => Extend::Repeat,
        hb_paint_extend_t::HB_PAINT_EXTEND_REFLECT => Extend::Reflect,
    }
}
