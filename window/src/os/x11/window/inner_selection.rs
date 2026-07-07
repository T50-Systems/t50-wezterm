pub(crate) fn appearance_changed(&mut self, appearance: Appearance) {
    if appearance != self.appearance {
        self.appearance = appearance;
        self.events
            .dispatch(WindowEvent::AppearanceChanged(appearance));
    }
}

fn focus_changed(&mut self, focused: bool) {
    log::trace!("focus_changed {focused}, flagging geometry as unsure");
    self.sure_about_geometry = false;
    if self.has_focus != Some(focused) {
        self.has_focus.replace(focused);
        self.update_ime_position();
        log::trace!("Calling focus_change({focused})");
        self.events.dispatch(WindowEvent::FocusChanged(focused));
    }
}

pub fn dispatch_ime_compose_status(&mut self, status: DeadKeyStatus) {
    self.events
        .dispatch(WindowEvent::AdviseDeadKeyStatus(status));
}

pub fn dispatch_ime_text(&mut self, text: &str) {
    let key_event = KeyEvent {
        key: KeyCode::Composed(text.into()),
        leds: KeyboardLedStatus::empty(),
        modifiers: Modifiers::NONE,
        repeat_count: 1,
        key_is_down: true,
        raw: None,
    }
    .normalize_shift()
    .resurface_positional_modifier_key();
    self.events.dispatch(WindowEvent::KeyEvent(key_event));
    // Since we just composed, synthesize a cleared status, as we
    // are not guaranteed to receive an event notification to
    // trigger dispatch_ime_compose_status() above.
    // <https://github.com/wezterm/wezterm/issues/4841>
    self.events
        .dispatch(WindowEvent::AdviseDeadKeyStatus(DeadKeyStatus::None));
}

/// If we own the selection, make sure that the X server reflects
/// that and vice versa.
fn update_selection_owner(&mut self, clipboard: Clipboard) -> anyhow::Result<()> {
    let window_id = self.window_id;
    let conn = self.conn();
    let selection = match clipboard {
        Clipboard::PrimarySelection => xcb::x::ATOM_PRIMARY,
        Clipboard::Clipboard => conn.atom_clipboard,
    };
    let current_owner = conn
        .send_and_wait_request(&xcb::x::GetSelectionOwner { selection })
        .unwrap()
        .owner();

    let we_own_it = self.copy_and_paste.clipboard(clipboard).is_some();

    if !we_own_it && current_owner == window_id {
        log::trace!(
            "SEL: window_id={window_id:?} X thinks we own selection, \
                        but we don't: tell it to clear it"
        );
        // We don't have a selection but X thinks we do; disown it!
        conn.send_request_no_reply(&xcb::x::SetSelectionOwner {
            owner: xcb::x::Window::none(),
            selection,
            time: self.copy_and_paste.time,
        })?;
    } else if we_own_it {
        log::trace!(
            "SEL: window_id={window_id:?} currently owned by \
                 {current_owner:?}, tell X we now own it"
        );
        // We have the selection but X doesn't think we do; assert it!
        conn.send_request_no_reply(&xcb::x::SetSelectionOwner {
            owner: self.window_id,
            selection,
            time: self.copy_and_paste.time,
        })?;
    } else {
        log::trace!(
            "SEL: window_id={window_id:?} current_owner={current_owner:?} \
                owned={we_own_it}"
        );
    }
    conn.flush().context("flushing after updating selection")?;
    Ok(())
}

fn selection_atom_to_clipboard(&self, atom: Atom) -> Option<Clipboard> {
    if atom == xcb::x::ATOM_PRIMARY {
        Some(Clipboard::PrimarySelection)
    } else if atom == self.conn().atom_clipboard {
        Some(Clipboard::Clipboard)
    } else {
        None
    }
}

fn selection_clear(&mut self, request: &xcb::x::SelectionClearEvent) -> anyhow::Result<()> {
    let window_id = self.window_id;
    log::debug!("SEL: window_id={window_id:?} {:?}", request);
    if let Some(clipboard) = self.selection_atom_to_clipboard(request.selection()) {
        self.copy_and_paste.clipboard_mut(clipboard).take();
        self.copy_and_paste.request_mut(clipboard).take();
        self.update_selection_owner(clipboard)?;
    }

    Ok(())
}

/// A selection request is made to us after we've announced that we own the selection
/// and when another client wants to copy it.
fn selection_request(&mut self, request: &xcb::x::SelectionRequestEvent) -> anyhow::Result<()> {
    let conn = self.conn();
    let window_id = self.window_id;
    log::trace!("SEL: window_id={window_id:?} {:?}", request);
    log::trace!(
        "XSEL={:?}, UTF8={:?} PRIMARY={:?} clip={:?}",
        conn.atom_xsel_data,
        conn.atom_utf8_string,
        xcb::x::ATOM_PRIMARY,
        conn.atom_clipboard,
    );

    let selprop = if request.target() == conn.atom_targets {
        // They want to know which targets we support
        let atoms: [Atom; 1] = [conn.atom_utf8_string];
        log::trace!("SEL: window_id={window_id:?} requestor wants supported targets");
        conn.send_request_no_reply(&xcb::x::ChangeProperty {
            mode: PropMode::Replace,
            window: request.requestor(),
            property: request.property(),
            r#type: xcb::x::ATOM_ATOM,
            data: &atoms,
        })?;

        // let the requestor know that we set their property
        request.property()
    } else if request.target() == conn.atom_utf8_string || request.target() == xcb::x::ATOM_STRING {
        log::trace!("SEL: window_id={window_id:?} requestor wants string data");
        if let Some(clipboard) = self.selection_atom_to_clipboard(request.selection()) {
            // We'll accept requests for UTF-8 or STRING data.
            // We don't and won't do any conversion from UTF-8 to
            // whatever STRING represents; let's just assume that
            // the other end is going to handle it correctly.
            if let Some(text) = self.copy_and_paste.clipboard(clipboard) {
                conn.send_request_no_reply(&xcb::x::ChangeProperty {
                    mode: PropMode::Replace,
                    window: request.requestor(),
                    property: request.property(),
                    r#type: request.target(),
                    data: text.as_bytes(),
                })?;
                // let the requestor know that we set their property
                request.property()
            } else {
                // We have no clipboard so there is nothing to report
                xcb::x::ATOM_NONE
            }
        } else {
            xcb::x::ATOM_NONE
        }
    } else {
        // We didn't support their request, so there is nothing
        // we can report back to them.
        xcb::x::ATOM_NONE
    };
    log::trace!(
        "SEL: window_id={window_id:?} responding with selprop={:?}",
        selprop
    );

    conn.send_request_no_reply(&xcb::x::SendEvent {
        propagate: true,
        destination: xcb::x::SendEventDest::Window(request.requestor()),
        event_mask: xcb::x::EventMask::empty(),
        event: &xcb::x::SelectionNotifyEvent::new(
            request.time(),
            request.requestor(),
            request.selection(),
            request.target(),
            selprop, // the disposition from the operation above
        ),
    })?;

    Ok(())
}

fn selection_notify(&mut self, selection: &xcb::x::SelectionNotifyEvent) -> anyhow::Result<()> {
    let conn = self.conn();
    let window_id = self.window_id;
    let selection_name = conn.atom_name(selection.selection());
    let target_name = conn.atom_name(selection.target());

    log::trace!(
        "SEL: window_id={window_id:?} SELECTION_NOTIFY received {selection:?} \
            selection.selection={selection_name} selection.target={target_name}"
    );

    if let Some(clipboard) = self.selection_atom_to_clipboard(selection.selection()) {
        if selection.property() == xcb::x::ATOM_NONE {
            if selection.target() == conn.atom_utf8_string {
                log::trace!(
                    "SEL: window_id={window_id:?} -> UTF-8 selection data \
                         available, requesting STRING instead"
                );
                conn.send_request_no_reply_log(&xcb::x::ConvertSelection {
                    requestor: window_id,
                    selection: selection.selection(),
                    target: xcb::x::ATOM_STRING,
                    property: conn.atom_xsel_data,
                    time: self.copy_and_paste.time,
                });
                return Ok(());
            }

            if let Some(mut promise) = self.copy_and_paste.request_mut(clipboard).take() {
                log::trace!(
                    "SEL: window_id={window_id:?} -> no compatible selection data \
                         available, fulfil promise with empty string"
                );
                promise.ok("".to_owned());
                return Ok(());
            }
            log::trace!(
                "SEL: window_id={window_id:?} -> no compatible selection data \
                     available, and no promise. weird!"
            );

            return Ok(());
        }

        match conn.send_and_wait_request(&xcb::x::GetProperty {
            delete: false,
            window: selection.requestor(),
            property: selection.property(),
            r#type: selection.target(),
            long_offset: 0,
            long_length: u32::max_value(),
        }) {
            Ok(prop) => {
                if let Some(mut promise) = self.copy_and_paste.request_mut(clipboard).take() {
                    fn latin1_to_string(s: &[u8]) -> String {
                        s.iter().map(|&c| c as char).collect()
                    }

                    let data = if selection.target() == xcb::x::ATOM_STRING {
                        latin1_to_string(prop.value())
                    } else {
                        // selection.target() is probably == conn.atom_utf8_string,
                        // because we only ever ask for either STRING or UTF8_STRING.
                        // If it isn't, we'll just try to convert it anyway.
                        String::from_utf8_lossy(prop.value()).to_string()
                    };

                    promise.ok(data);
                }

                conn.send_request_no_reply(&xcb::x::DeleteProperty {
                    window: self.window_id,
                    property: conn.atom_xsel_data,
                })?;
            }
            Err(err) => {
                log::error!("clipboard: err while getting clipboard property: {:?}", err);
                if let Some(mut promise) = self.copy_and_paste.request_mut(clipboard).take() {
                    promise.ok("".to_owned());
                }
            }
        }
    } else if selection.selection() == conn.atom_xdndselection
        && selection.property() == conn.atom_xsel_data
    {
        if let Some(srcwin) = self.drag_and_drop.src_window {
            match conn.send_and_wait_request(&xcb::x::GetProperty {
                delete: true,
                window: selection.requestor(),
                property: selection.property(),
                r#type: selection.target(),
                long_offset: 0,
                long_length: u32::max_value(),
            }) {
                Ok(prop) => {
                    if selection.target() == conn.atom_utf8_string {
                        let text = String::from_utf8_lossy(prop.value()).to_string();
                        self.events.dispatch(WindowEvent::DroppedString(text));
                    } else if selection.target() == conn.atom_xmozurl {
                        let data = decode_dropped_url_string(prop.value());
                        let urls = parse_xmozurl_list(&data);
                        self.events.dispatch(WindowEvent::DroppedUrl(urls));
                    } else if selection.target() == conn.atom_texturilist {
                        let paths = parse_texturi_list(prop.value());
                        self.events.dispatch(WindowEvent::DroppedFile(paths));
                    }
                }
                Err(err) => {
                    log::error!("clipboard: err while getting clipboard property: {err:#}");
                }
            }
            conn.send_request_no_reply_log(&xcb::x::SendEvent {
                propagate: false,
                destination: xcb::x::SendEventDest::Window(srcwin),
                event_mask: xcb::x::EventMask::empty(),
                event: &xcb::x::ClientMessageEvent::new(
                    srcwin,
                    conn.atom_xdndfinished,
                    xcb::x::ClientMessageData::Data32([
                        window_id.resource_id(),
                        1,
                        self.drag_and_drop.target_action.resource_id(),
                        0,
                        0,
                    ]),
                ),
            });
        } else {
            log::warn!("No Xdnd in progress, but received Xdnd selection. Ignoring.");
        }
    } else {
        log::trace!("SEL: window_id={window_id:?} unknown selection {selection_name}");
    }
    Ok(())
}

fn get_window_state(&self) -> anyhow::Result<WindowState> {
    let conn = self.conn();

    let reply = conn.send_and_wait_request(&xcb::x::GetProperty {
        delete: false,
        window: self.window_id,
        property: conn.atom_net_wm_state,
        r#type: xcb::x::ATOM_ATOM,
        long_offset: 0,
        long_length: 1024,
    })?;

    let state = reply.value::<u32>();
    let mut window_state = WindowState::default();

    for &s in state {
        if s == conn.atom_state_fullscreen.resource_id() {
            window_state |= WindowState::FULL_SCREEN;
        } else if s == conn.atom_state_maximized_vert.resource_id()
            || s == conn.atom_state_maximized_horz.resource_id()
        {
            window_state |= WindowState::MAXIMIZED;
        } else if s == conn.atom_state_hidden.resource_id() {
            window_state |= WindowState::HIDDEN;
        }
    }

    Ok(window_state)
}

fn set_wm_state(
    &mut self,
    action: NetWmStateAction,
    atom: Atom,
    atom2: Option<Atom>,
) -> anyhow::Result<()> {
    let conn = self.conn();
    let data: [u32; 5] = [
        action as u32,
        atom.resource_id(),
        atom2.map(|a| a.resource_id()).unwrap_or(0),
        0,
        0,
    ];

    // Ask window manager to change our fullscreen state
    conn.send_request_no_reply(&xcb::x::SendEvent {
        propagate: true,
        destination: xcb::x::SendEventDest::Window(conn.root),
        event_mask: xcb::x::EventMask::SUBSTRUCTURE_REDIRECT
            | xcb::x::EventMask::SUBSTRUCTURE_NOTIFY,
        event: &xcb::x::ClientMessageEvent::new(
            self.window_id,
            conn.atom_net_wm_state,
            xcb::x::ClientMessageData::Data32(data),
        ),
    })?;
    conn.flush()?;
    self.adjust_decorations(self.config.window_decorations)?;

    Ok(())
}

fn set_maximized_hint(&mut self, enable: bool) -> anyhow::Result<()> {
    self.set_wm_state(
        NetWmStateAction::with_bool(enable),
        self.conn().atom_state_maximized_vert,
        Some(self.conn().atom_state_maximized_horz),
    )
}

fn set_fullscreen_hint(&mut self, enable: bool) -> anyhow::Result<()> {
    self.set_wm_state(
        NetWmStateAction::with_bool(enable),
        self.conn().atom_state_fullscreen,
        None,
    )
}

#[allow(clippy::identity_op)]
fn adjust_decorations(&mut self, decorations: WindowDecorations) -> anyhow::Result<()> {
    // Set the motif hints to disable decorations.
    // See https://stackoverflow.com/a/1909708
    #[repr(C)]
    struct MwmHints {
        flags: u32,
        functions: u32,
        decorations: u32,
        input_mode: i32,
        status: u32,
    }

    const HINTS_DECORATIONS: u32 = 1 << 1;
    const FUNC_ALL: u32 = 1 << 0;
    const FUNC_RESIZE: u32 = 1 << 1;
    // const HINTS_FUNCTIONS: u32 = 1 << 0;
    const FUNC_MOVE: u32 = 1 << 2;
    const FUNC_MINIMIZE: u32 = 1 << 3;
    const FUNC_MAXIMIZE: u32 = 1 << 4;
    const FUNC_CLOSE: u32 = 1 << 5;

    let decorations = if decorations == WindowDecorations::TITLE | WindowDecorations::RESIZE {
        FUNC_ALL
    } else if decorations == WindowDecorations::RESIZE
        || decorations == WindowDecorations::INTEGRATED_BUTTONS
        || decorations == WindowDecorations::INTEGRATED_BUTTONS | WindowDecorations::RESIZE
    {
        FUNC_RESIZE
    } else if decorations == WindowDecorations::TITLE {
        FUNC_MOVE | FUNC_MINIMIZE | FUNC_MAXIMIZE | FUNC_CLOSE
    } else if decorations == WindowDecorations::NONE {
        0
    } else {
        FUNC_ALL
    };

    let hints = MwmHints {
        flags: HINTS_DECORATIONS,
        functions: 0,
        decorations,
        input_mode: 0,
        status: 0,
    };

    let conn = self.conn();

    let hints_slice = unsafe { std::slice::from_raw_parts(&hints as *const _ as *const u32, 5) };

    conn.send_request_no_reply(&xcb::x::ChangeProperty {
        mode: PropMode::Replace,
        window: self.window_id,
        property: conn.atom_motif_wm_hints,
        r#type: conn.atom_motif_wm_hints,
        data: hints_slice,
    })?;
    Ok(())
}

fn conn(&self) -> Rc<XConnection> {
    self.conn.upgrade().expect("XConnection to be alive")
}
