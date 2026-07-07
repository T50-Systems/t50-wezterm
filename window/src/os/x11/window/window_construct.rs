/// A Window!
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XWindow(xcb::x::Window);

impl PartialOrd for XWindow {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.resource_id().partial_cmp(&other.0.resource_id())
    }
}

impl Ord for XWindow {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.resource_id().cmp(&other.0.resource_id())
    }
}

impl XWindow {
    pub(crate) fn from_id(id: xcb::x::Window) -> Self {
        Self(id)
    }

    /// Create a new window on the specified screen with the specified
    /// dimensions
    pub async fn new_window<F>(
        class_name: &str,
        name: &str,
        geometry: RequestedWindowGeometry,
        config: Option<&ConfigHandle>,
        _font_config: Rc<FontConfiguration>,
        event_handler: F,
    ) -> anyhow::Result<Window>
    where
        F: 'static + FnMut(WindowEvent, &Window),
    {
        let config = match config {
            Some(c) => c.clone(),
            None => config::configuration(),
        };
        let conn = Connection::get()
            .ok_or_else(|| {
                anyhow!(
                "new_window must be called on the gui thread after Connection::init has succeeded",
            )
            })?
            .x11();

        let ResolvedGeometry {
            x,
            y,
            width,
            height,
        } = conn.resolve_geometry(geometry);

        let mut events = WindowEventSender::new(event_handler);

        let window_id;
        let child_id;
        let window = {
            let setup = conn.conn().get_setup();
            let screen = setup
                .roots()
                .nth(conn.screen_num() as usize)
                .ok_or_else(|| anyhow!("no screen?"))?;

            window_id = conn.conn().generate_id();
            child_id = conn.conn().generate_id();

            let color_map_id = conn.conn().generate_id();
            conn.send_request_no_reply(&xcb::x::CreateColormap {
                alloc: xcb::x::ColormapAlloc::None,
                mid: color_map_id,
                window: screen.root(),
                visual: conn.visual.visual_id(),
            })
            .context("create_colormap_checked")?;

            conn.send_request_no_reply(&xcb::x::CreateWindow {
                depth: conn.depth,
                wid: window_id,
                parent: screen.root(),
                x: x.unwrap_or(0).try_into()?,
                y: y.unwrap_or(0).try_into()?,
                width: width.try_into()?,
                height: height.try_into()?,
                border_width: 0,
                class: xcb::x::WindowClass::InputOutput,
                visual: conn.visual.visual_id(),
                value_list: &[
                    // We have to specify both a border pixel color and a colormap
                    // when specifying a depth that doesn't match the root window in
                    // order to avoid a BadMatch
                    xcb::x::Cw::BackPixel(0), // transparent background
                    xcb::x::Cw::BorderPixel(screen.black_pixel()),
                    xcb::x::Cw::EventMask(
                        xcb::x::EventMask::FOCUS_CHANGE
                            | xcb::x::EventMask::KEY_PRESS
                            | xcb::x::EventMask::BUTTON_PRESS
                            | xcb::x::EventMask::BUTTON_RELEASE
                            | xcb::x::EventMask::POINTER_MOTION
                            | xcb::x::EventMask::LEAVE_WINDOW
                            | xcb::x::EventMask::BUTTON_MOTION
                            | xcb::x::EventMask::KEY_RELEASE
                            | xcb::x::EventMask::PROPERTY_CHANGE
                            | xcb::x::EventMask::STRUCTURE_NOTIFY,
                    ),
                    xcb::x::Cw::Colormap(color_map_id),
                ],
            })
            .context("xcb::create_window_checked")?;

            conn.send_request_no_reply(&xcb::x::CreateWindow {
                depth: conn.depth,
                wid: child_id,
                parent: window_id,
                x: 0,
                y: 0,
                width: width.try_into()?,
                height: height.try_into()?,
                border_width: 0,
                class: xcb::x::WindowClass::InputOutput,
                visual: conn.visual.visual_id(),
                value_list: &[
                    // We have to specify both a border pixel color and a colormap
                    // when specifying a depth that doesn't match the root window in
                    // order to avoid a BadMatch
                    xcb::x::Cw::BackPixel(0), // transparent background
                    xcb::x::Cw::BorderPixel(screen.black_pixel()),
                    xcb::x::Cw::BitGravity(xcb::x::Gravity::NorthWest),
                    xcb::x::Cw::EventMask(xcb::x::EventMask::EXPOSURE),
                    xcb::x::Cw::Colormap(color_map_id),
                ],
            })
            .context("xcb::create_window_checked")?;

            conn.send_request_no_reply(&xcb::x::MapWindow { window: child_id })
                .context("xcb::map_window")?;

            events.assign_window(Window::X11(XWindow::from_id(window_id)));

            let appearance = conn.get_appearance();

            Arc::new(Mutex::new(XWindowInner {
                title: String::new(),
                appearance,
                window_id,
                child_id,
                conn: Rc::downgrade(&conn),
                events,
                width: width.try_into()?,
                height: height.try_into()?,
                dpi: conn.default_dpi(),
                copy_and_paste: CopyAndPaste::default(),
                drag_and_drop: DragAndDrop::default(),
                cursors: CursorInfo::new(&config, &conn),
                config: config.clone(),
                has_focus: None,
                verify_focus: true,
                last_cursor_position: Rect::default(),
                paint_throttled: false,
                last_wm_state: WindowState::default(),
                invalidated: false,
                pending: vec![],
                sure_about_geometry: false,
                current_mouse_event: None,
                window_drag_position: None,
                dragging: false,
                outstanding_configure_requests: 0,
                pending_finished_resizes: 0,
            }))
        };

        // WM_CLASS is encoded as the class and instance name,
        // null terminated
        let mut class_string = class_name.as_bytes().to_vec();
        class_string.push(0);
        class_string.extend_from_slice(class_name.as_bytes());
        class_string.push(0);

        conn.send_request_no_reply(&xcb::x::ChangeProperty {
            mode: PropMode::Replace,
            window: window_id,
            property: xcb::x::ATOM_WM_CLASS,
            r#type: xcb::x::ATOM_STRING,
            data: &class_string,
        })?;

        conn.send_request_no_reply(&xcb::x::ChangeProperty {
            mode: PropMode::Replace,
            window: window_id,
            property: conn.atom_net_wm_pid,
            r#type: xcb::x::ATOM_CARDINAL,
            data: &[unsafe { libc::getpid() as u32 }],
        })?;

        conn.send_request_no_reply(&xcb::x::ChangeProperty {
            mode: PropMode::Replace,
            window: window_id,
            property: conn.atom_protocols,
            r#type: xcb::x::ATOM_ATOM,
            data: &[conn.atom_delete],
        })?;

        conn.send_request_no_reply(&xcb::x::ChangeProperty {
            mode: PropMode::Replace,
            window: window_id,
            property: conn.atom_xdndaware,
            r#type: xcb::x::ATOM_ATOM,
            data: &[5u32],
        })?;

        window
            .lock()
            .unwrap()
            .adjust_decorations(config.window_decorations)?;

        let window_handle = Window::X11(XWindow::from_id(window_id));

        conn.windows.borrow_mut().insert(window_id, window);
        conn.child_to_parent_id
            .borrow_mut()
            .insert(child_id, window_id);

        window_handle.set_title(name);
        // Before we map the window, flush to ensure that all of the other properties
        // have been applied to it.
        // This is a speculative fix for this race condition issue:
        // <https://github.com/wezterm/wezterm/issues/2155>
        conn.flush().context("flushing before mapping window")?;
        window_handle.show();

        // Some window managers will ignore the x,y that we set during window
        // creation, so we ask them again once the window is mapped
        if let (Some(x), Some(y)) = (x, y) {
            window_handle.set_window_position(ScreenPoint::new(x.try_into()?, y.try_into()?));
        }

        if conn
            .active_extensions()
            .any(|e| e == xcb::Extension::Present)
        {
            let event_id = conn.generate_id();
            conn.send_request_no_reply(&xcb::present::SelectInput {
                eid: event_id,
                window: window_id,
                event_mask: xcb::present::EventMask::CONFIGURE_NOTIFY,
            })
            .context("Present::SelectInput")?;
        }

        Ok(window_handle)
    }
}
