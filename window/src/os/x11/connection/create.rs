impl XConnection {

    pub(crate) fn create_new() -> anyhow::Result<Rc<XConnection>> {
        let (conn, screen_num) = xcb::Connection::connect_with_xlib_display_and_extensions(
            &[xcb::Extension::Xkb],
            &[
                xcb::Extension::Present,
                xcb::Extension::RandR,
                xcb::Extension::Render,
                xcb::Extension::Dri2,
            ],
        )?;
        conn.set_event_queue_owner(xcb::EventQueueOwner::Xcb);

        let atom_protocols = Self::intern_atom(&conn, "WM_PROTOCOLS")?;
        let atom_delete = Self::intern_atom(&conn, "WM_DELETE_WINDOW")?;
        let atom_utf8_string = Self::intern_atom(&conn, "UTF8_STRING")?;
        let atom_xsel_data = Self::intern_atom(&conn, "XSEL_DATA")?;
        let atom_targets = Self::intern_atom(&conn, "TARGETS")?;
        let atom_clipboard = Self::intern_atom(&conn, "CLIPBOARD")?;
        let atom_texturilist = Self::intern_atom(&conn, "text/uri-list")?;
        let atom_xmozurl = Self::intern_atom(&conn, "text/x-moz-url")?;
        let atom_xdndaware = Self::intern_atom(&conn, "XdndAware")?;
        let atom_xdndtypelist = Self::intern_atom(&conn, "XdndTypeList")?;
        let atom_xdndselection = Self::intern_atom(&conn, "XdndSelection")?;
        let atom_xdndenter = Self::intern_atom(&conn, "XdndEnter")?;
        let atom_xdndposition = Self::intern_atom(&conn, "XdndPosition")?;
        let atom_xdndstatus = Self::intern_atom(&conn, "XdndStatus")?;
        let atom_xdndleave = Self::intern_atom(&conn, "XdndLeave")?;
        let atom_xdnddrop = Self::intern_atom(&conn, "XdndDrop")?;
        let atom_xdndfinished = Self::intern_atom(&conn, "XdndFinished")?;
        let atom_xdndactioncopy = Self::intern_atom(&conn, "XdndActionCopy")?;
        let atom_xdndactionmove = Self::intern_atom(&conn, "XdndActionMove")?;
        let atom_xdndactionlink = Self::intern_atom(&conn, "XdndActionLink")?;
        let atom_xdndactionask = Self::intern_atom(&conn, "XdndActionAsk")?;
        let atom_xdndactionprivate = Self::intern_atom(&conn, "XdndActionPrivate")?;
        let atom_gtk_edge_constraints = Self::intern_atom(&conn, "_GTK_EDGE_CONSTRAINTS")?;
        let atom_xsettings_selection =
            Self::intern_atom(&conn, &format!("_XSETTINGS_S{}", screen_num))?;
        let atom_xsettings_settings = Self::intern_atom(&conn, "_XSETTINGS_SETTINGS")?;
        let atom_manager = Self::intern_atom(&conn, "MANAGER")?;
        let atom_state_maximized_vert = Self::intern_atom(&conn, "_NET_WM_STATE_MAXIMIZED_VERT")?;
        let atom_state_maximized_horz = Self::intern_atom(&conn, "_NET_WM_STATE_MAXIMIZED_HORZ")?;
        let atom_state_hidden = Self::intern_atom(&conn, "_NET_WM_STATE_HIDDEN")?;
        let atom_state_fullscreen = Self::intern_atom(&conn, "_NET_WM_STATE_FULLSCREEN")?;
        let atom_net_wm_state = Self::intern_atom(&conn, "_NET_WM_STATE")?;
        let atom_motif_wm_hints = Self::intern_atom(&conn, "_MOTIF_WM_HINTS")?;
        let atom_net_wm_pid = Self::intern_atom(&conn, "_NET_WM_PID")?;
        let atom_net_wm_name = Self::intern_atom(&conn, "_NET_WM_NAME")?;
        let atom_net_wm_icon = Self::intern_atom(&conn, "_NET_WM_ICON")?;
        let atom_net_move_resize_window = Self::intern_atom(&conn, "_NET_MOVERESIZE_WINDOW")?;
        let atom_net_wm_moveresize = Self::intern_atom(&conn, "_NET_WM_MOVERESIZE")?;
        let atom_net_supported = Self::intern_atom(&conn, "_NET_SUPPORTED")?;
        let atom_net_supporting_wm_check = Self::intern_atom(&conn, "_NET_SUPPORTING_WM_CHECK")?;
        let atom_net_active_window = Self::intern_atom(&conn, "_NET_ACTIVE_WINDOW")?;

        let has_randr = conn.active_extensions().any(|e| e == xcb::Extension::RandR);

        let screen = conn
            .get_setup()
            .roots()
            .nth(screen_num as usize)
            .ok_or_else(|| anyhow!("no screen?"))?;

        let mut visuals = vec![];
        for depth in screen.allowed_depths() {
            let depth_bpp = depth.depth();
            if depth_bpp == 24 || depth_bpp == 32 {
                for vis in depth.visuals() {
                    if vis.class() == xcb::x::VisualClass::TrueColor
                        && vis.bits_per_rgb_value() >= 8
                    {
                        visuals.push((depth_bpp, vis));
                    }
                }
            }
        }
        if visuals.is_empty() {
            bail!("no suitable visuals of depth 24 or 32 are available");
        }
        visuals.sort_by(|(a_depth, _), (b_depth, _)| b_depth.cmp(&a_depth));
        let (depth, visual) = visuals[0];
        let visual = *visual;

        log::trace!(
            "picked depth {} visual id:0x{:x}, class:{:?}, bits_per_rgb_value:{}, \
                    colormap entries:{}, masks: r=0x{:x},g=0x{:x},b=0x{:x}",
            depth,
            visual.visual_id(),
            visual.class(),
            visual.bits_per_rgb_value(),
            visual.colormap_entries(),
            visual.red_mask(),
            visual.green_mask(),
            visual.blue_mask()
        );
        let (keyboard, kbd_ev) = Keyboard::new(&conn)?;
        let keyboard = KeyboardWithFallback::new(keyboard)?;

        let cursor_font_id = conn.generate_id();
        let cursor_font_name = "cursor";
        conn.check_request(conn.send_request_checked(&xcb::x::OpenFont {
            fid: cursor_font_id,
            name: cursor_font_name.as_bytes(),
        }))
        .context("OpenFont")?;

        let root = screen.root();

        if has_randr {
            conn.check_request(conn.send_request_checked(&xcb::randr::SelectInput {
                window: root,
                enable: xcb::randr::NotifyMask::SCREEN_CHANGE
                    | xcb::randr::NotifyMask::PROVIDER_CHANGE
                    | xcb::randr::NotifyMask::RESOURCE_CHANGE,
            }))
            .context("XRANDR::SelectInput")?;
        }

        let xrm =
            crate::x11::xrm::parse_root_resource_manager(&conn, root).unwrap_or(HashMap::new());

        let xsettings = read_xsettings(&conn, atom_xsettings_selection, atom_xsettings_settings)
            .unwrap_or_else(|err| {
                log::trace!("Failed to read xsettings: {:#}", err);
                Default::default()
            });
        log::trace!("xsettings are {:?}", xsettings);

        let default_dpi = RefCell::new(compute_default_dpi(&xrm, &xsettings));
        log::trace!("computed initial dpi: {:?}", default_dpi);

        let input_style = match config::configuration().ime_preedit_rendering {
            config::ImePreeditRendering::Builtin => xcb_imdkit::InputStyle::PREEDIT_CALLBACKS,
            config::ImePreeditRendering::System => xcb_imdkit::InputStyle::DEFAULT,
        };

        xcb_imdkit::ImeClient::set_logger(|msg| log::debug!("Ime: {}", msg));
        let ime = unsafe {
            xcb_imdkit::ImeClient::unsafe_new(
                &conn,
                screen_num,
                input_style,
                config::configuration().xim_im_name.as_deref(),
            )
        };

        let conn = Rc::new(XConnection {
            conn,
            default_dpi,
            xsettings: RefCell::new(xsettings),
            cursor_font_id,
            screen_num,
            root,
            xrm: RefCell::new(xrm),
            atom_protocols,
            atom_clipboard,
            atom_texturilist,
            atom_xmozurl,
            atom_xdndaware,
            atom_xdndtypelist,
            atom_xdndselection,
            atom_xdndenter,
            atom_xdndposition,
            atom_xdndstatus,
            atom_xdndleave,
            atom_xdnddrop,
            atom_xdndfinished,
            atom_xdndactioncopy,
            atom_xdndactionmove,
            atom_xdndactionlink,
            atom_xdndactionask,
            atom_xdndactionprivate,
            atom_gtk_edge_constraints,
            atom_xsettings_selection,
            atom_xsettings_settings,
            atom_manager,
            atom_delete,
            atom_state_maximized_vert,
            atom_state_maximized_horz,
            atom_state_hidden,
            atom_state_fullscreen,
            atom_net_wm_state,
            atom_motif_wm_hints,
            atom_net_wm_pid,
            atom_net_wm_name,
            atom_net_move_resize_window,
            atom_net_wm_moveresize,
            atom_net_supported,
            atom_net_supporting_wm_check,
            atom_net_active_window,
            atom_net_wm_icon,
            keyboard,
            kbd_ev,
            atom_utf8_string,
            atom_xsel_data,
            atom_targets,
            windows: RefCell::new(HashMap::new()),
            child_to_parent_id: RefCell::new(HashMap::new()),
            should_terminate: RefCell::new(false),
            depth,
            visual,
            gl_connection: RefCell::new(None),
            ime: RefCell::new(ime),
            ime_process_event_result: RefCell::new(Ok(())),
            has_randr,
            atom_names: RefCell::new(HashMap::new()),
            supported: RefCell::new(HashSet::new()),
            screens: RefCell::new(None),
        });

        {
            let conn = conn.clone();
            conn.clone()
                .ime
                .borrow_mut()
                .set_commit_string_cb(move |window_id, input| {
                    if let Some(window) = conn.window_by_id(window_id) {
                        let mut inner = window.lock().unwrap();
                        inner.dispatch_ime_text(input);
                    }
                });
        }
        if config::configuration().ime_preedit_rendering == config::ImePreeditRendering::Builtin {
            let conn = conn.clone();
            conn.clone()
                .ime
                .borrow_mut()
                .set_preedit_draw_cb(move |window_id, info| {
                    if let Some(window) = conn.window_by_id(window_id) {
                        let mut inner = window.lock().unwrap();

                        let text = info.text();
                        let status = DeadKeyStatus::Composing(text);
                        inner.dispatch_ime_compose_status(status);
                    }
                });
        }
        if config::configuration().ime_preedit_rendering == config::ImePreeditRendering::Builtin {
            let conn = conn.clone();
            conn.clone()
                .ime
                .borrow_mut()
                .set_preedit_done_cb(move |window_id| {
                    if let Some(window) = conn.window_by_id(window_id) {
                        let mut inner = window.lock().unwrap();
                        inner.dispatch_ime_compose_status(DeadKeyStatus::None);
                    }
                });
        }
        {
            let conn = conn.clone();
            conn.clone()
                .ime
                .borrow_mut()
                .set_forward_event_cb(move |_win, e| {
                    if let err @ Err(_) = conn.process_xcb_event(e) {
                        if let Err(err) = conn.ime_process_event_result.replace(err) {
                            log::warn!("IME process event error dropped: {}", err);
                        }
                    }
                });
        }

        conn.update_net_supported();

        Ok(conn)
}
