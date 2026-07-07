impl Drop for ConceptFrame {
    fn drop(&mut self) {
        for ptr in self.pointers.drain(..) {
            if ptr.as_ref().version() >= 3 {
                ptr.release();
            }
        }
    }
}

fn change_pointer(pointer: &ThemedPointer, inner: &Inner, location: Location, serial: Option<u32>) {
    // Prevent theming of the surface if it was requested.
    if !inner.theme_over_surface && location == Location::None {
        return;
    }

    let name = match location {
        // If we can't resize a frame we shouldn't show resize cursors.
        _ if !inner.resizable => "left_ptr",
        Location::Top => "top_side",
        Location::TopRight => "top_right_corner",
        Location::Right => "right_side",
        Location::BottomRight => "bottom_right_corner",
        Location::Bottom => "bottom_side",
        Location::BottomLeft => "bottom_left_corner",
        Location::Left => "left_side",
        Location::TopLeft => "top_left_corner",
        _ => "left_ptr",
    };

    if let Err(err) = pointer.set_cursor(name, serial) {
        log::error!("Unable to set cursor to {}: {:#}", name, err);
    }
}

fn request_for_location_on_lmb(
    pointer_data: &PointerUserData,
    maximized: bool,
    resizable: bool,
) -> Option<FrameRequest> {
    use wayland_protocols::xdg_shell::client::xdg_toplevel::ResizeEdge;
    match pointer_data.location {
        Location::Top if resizable => Some(FrameRequest::Resize(
            pointer_data.seat.clone(),
            ResizeEdge::Top,
        )),
        Location::TopLeft if resizable => Some(FrameRequest::Resize(
            pointer_data.seat.clone(),
            ResizeEdge::TopLeft,
        )),
        Location::Left if resizable => Some(FrameRequest::Resize(
            pointer_data.seat.clone(),
            ResizeEdge::Left,
        )),
        Location::BottomLeft if resizable => Some(FrameRequest::Resize(
            pointer_data.seat.clone(),
            ResizeEdge::BottomLeft,
        )),
        Location::Bottom if resizable => Some(FrameRequest::Resize(
            pointer_data.seat.clone(),
            ResizeEdge::Bottom,
        )),
        Location::BottomRight if resizable => Some(FrameRequest::Resize(
            pointer_data.seat.clone(),
            ResizeEdge::BottomRight,
        )),
        Location::Right if resizable => Some(FrameRequest::Resize(
            pointer_data.seat.clone(),
            ResizeEdge::Right,
        )),
        Location::TopRight if resizable => Some(FrameRequest::Resize(
            pointer_data.seat.clone(),
            ResizeEdge::TopRight,
        )),
        Location::Head => Some(FrameRequest::Move(pointer_data.seat.clone())),
        Location::Button(UIButton::Close) => Some(FrameRequest::Close),
        Location::Button(UIButton::Maximize) => {
            if maximized {
                Some(FrameRequest::UnMaximize)
            } else {
                Some(FrameRequest::Maximize)
            }
        }
        Location::Button(UIButton::Minimize) => Some(FrameRequest::Minimize),
        _ => None,
    }
}

fn request_for_location_on_rmb(pointer_data: &PointerUserData) -> Option<FrameRequest> {
    match pointer_data.location {
        Location::Head | Location::Button(_) => Some(FrameRequest::ShowMenu(
            pointer_data.seat.clone(),
            pointer_data.position.0 as i32,
            // We must offset it by header size for precise position.
            pointer_data.position.1 as i32 - HEADER_SIZE as i32,
        )),
        _ => None,
    }
}

// average of the two colors, approximately taking into account gamma correction
// result is as transparent as the most transparent color
fn mix_colors(x: RgbaColor, y: RgbaColor) -> RgbaColor {
    #[inline]
    fn gamma_mix(x: f32, y: f32) -> f32 {
        let z = ((x * x + y * y) / 2.0).sqrt();
        z
    }

    let x = x.to_tuple_rgba();
    let y = y.to_tuple_rgba();

    SrgbaTuple(
        gamma_mix(x.0, y.0),
        gamma_mix(x.1, y.1),
        gamma_mix(x.2, y.2),
        gamma_mix(x.3, y.3),
    )
    .into()
}

fn draw_buttons(
    pixmap: &mut PixmapMut,
    width: u32,
    scale: u32,
    maximizable: bool,
    state: WindowState,
    mouses: &[Location],
    config: &ConceptConfig,
) {
    let scale = scale as f32;

    let colors = config.colors();

    // Draw seperator between header and window contents
    let line_color = match state {
        WindowState::Active => colors.active_titlebar_border_bottom,
        WindowState::Inactive => colors.inactive_titlebar_border_bottom,
    };

    let mut sep_stroke = Stroke::default();
    sep_stroke.width = scale;

    let mut path = PathBuilder::new();
    let y = HEADER_SIZE as f32 * scale - sep_stroke.width;
    path.move_to(0., y as f32);
    path.line_to(width as f32 * scale as f32, y);
    let path = path.finish().unwrap();

    pixmap.stroke_path(
        &path,
        &color_to_paint(line_color),
        &Stroke::default(),
        Transform::identity(),
        None,
    );

    let mut drawn_buttons = 0;

    fn btn_colors(
        colors: &WindowFrameConfig,
        btn_state: ButtonState,
        state: WindowState,
    ) -> (RgbaColor, RgbaColor) {
        match (btn_state, state) {
            (ButtonState::Hovered, _) => (colors.button_hover_bg, colors.button_hover_fg),
            (_, WindowState::Inactive) => {
                (colors.inactive_titlebar_bg, colors.inactive_titlebar_fg)
            }
            _ => (colors.button_bg, colors.button_fg),
        }
    }

    if width >= HEADER_SIZE {
        // Draw the close button
        let btn_state = if mouses
            .iter()
            .any(|&l| l == Location::Button(UIButton::Close))
        {
            ButtonState::Hovered
        } else {
            ButtonState::Idle
        };

        let (button_color, icon_color) = btn_colors(colors, btn_state, state);

        draw_button(
            pixmap,
            0,
            scale,
            color_to_paint(button_color),
            color_to_paint(mix_colors(button_color, line_color)),
        );
        draw_icon(pixmap, 0, scale, color_to_paint(icon_color), Icon::Close);
        drawn_buttons += 1;
    }

    if width as usize >= (drawn_buttons + 1) * HEADER_SIZE as usize {
        let btn_state = if !maximizable {
            ButtonState::Disabled
        } else if mouses
            .iter()
            .any(|&l| l == Location::Button(UIButton::Maximize))
        {
            ButtonState::Hovered
        } else {
            ButtonState::Idle
        };

        let (button_color, icon_color) = btn_colors(colors, btn_state, state);
        draw_button(
            pixmap,
            drawn_buttons * HEADER_SIZE as usize,
            scale,
            color_to_paint(button_color),
            color_to_paint(mix_colors(button_color, line_color)),
        );
        draw_icon(
            pixmap,
            drawn_buttons * HEADER_SIZE as usize,
            scale,
            color_to_paint(icon_color),
            Icon::Maximize,
        );
        drawn_buttons += 1;
    }

    if width as usize >= (drawn_buttons + 1) * HEADER_SIZE as usize {
        let btn_state = if mouses
            .iter()
            .any(|&l| l == Location::Button(UIButton::Minimize))
        {
            ButtonState::Hovered
        } else {
            ButtonState::Idle
        };

        let (button_color, icon_color) = btn_colors(colors, btn_state, state);

        draw_button(
            pixmap,
            drawn_buttons * HEADER_SIZE as usize,
            scale,
            color_to_paint(button_color),
            color_to_paint(mix_colors(button_color, line_color)),
        );
        draw_icon(
            pixmap,
            drawn_buttons * HEADER_SIZE as usize,
            scale,
            color_to_paint(icon_color),
            Icon::Minimize,
        );
    }
}

enum Icon {
    Close,
    Maximize,
    Minimize,
}

fn draw_button(
    pixmap: &mut PixmapMut,
    x_offset: usize,
    scale: f32,
    btn_color: Paint,
    line_color: Paint,
) {
    let h = HEADER_SIZE as f32;
    let x_start = pixmap.width() as f32 / scale - h - x_offset as f32;
    // main square

    pixmap.fill_path(
        &PathBuilder::from_rect(
            Rect::from_xywh(x_start * scale, 0., h * scale, (h - 1.) * scale).unwrap(),
        ),
        &btn_color,
        FillRule::Winding,
        Transform::identity(),
        None,
    );

    // separation line

    let mut path = PathBuilder::new();
    path.move_to(x_start * scale, (h - 1.) * scale);
    path.line_to(x_start * scale, h * scale);
    let path = path.finish().unwrap();

    pixmap.stroke_path(
        &path,
        &line_color,
        &Stroke::default(),
        Transform::identity(),
        None,
    );
}

fn draw_icon(pixmap: &mut PixmapMut, x_offset: usize, scale: f32, icon_color: Paint, icon: Icon) {
    let h = HEADER_SIZE as f32;
    let cx = pixmap.width() as f32 / scale as f32 - h / 2. - x_offset as f32;
    let cy = h / 2.;
    let s = scale;

    let mut path = PathBuilder::new();
    let mut stroke = Stroke::default();
    stroke.width = 3.0;

    match icon {
        Icon::Close => {
            // Draw cross to represent the close button
            path.move_to((cx - 4.) * s, (cy - 4.) * s);
            path.line_to((cx + 4.) * s, (cy + 4.) * s);

            path.move_to((cx + 4.) * s, (cy - 4.) * s);
            path.line_to((cx - 4.) * s, (cy + 4.) * s);
        }
        Icon::Maximize => {
            path.move_to((cx - 4.) * s, (cy + 2.) * s);
            path.line_to(cx * s, (cy - 2.) * s);
            path.line_to((cx + 4.) * s, (cy + 2.) * s);
        }
        Icon::Minimize => {
            path.move_to((cx - 4.) * s, (cy - 3.) * s);
            path.line_to(cx * s, (cy + 1.) * s);
            path.line_to((cx + 4.) * s, (cy - 3.) * s);
        }
    }
    pixmap.stroke_path(
        &path.finish().unwrap(),
        &icon_color,
        &stroke,
        Transform::identity(),
        None,
    );
}
