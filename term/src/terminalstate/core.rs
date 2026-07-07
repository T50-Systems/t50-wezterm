use super::colors::default_color_map;
use super::threaded_writer::ThreadedWriter;
use super::*;

impl TerminalState {
    /// Constructs the terminal state.
    /// You generally want the `Terminal` struct rather than this one;
    /// Terminal contains and dereferences to `TerminalState`.
    pub fn new(
        size: TerminalSize,
        config: Arc<dyn TerminalConfiguration>,
        term_program: &str,
        term_version: &str,
        writer: Box<dyn std::io::Write + Send>,
    ) -> TerminalState {
        let writer = BufWriter::new(ThreadedWriter::new(writer));
        let seqno = 1;
        let screen = ScreenOrAlt::new(size, &config, seqno, config.bidi_mode());

        let color_map = default_color_map();

        let unicode_version = config.unicode_version();

        TerminalState {
            config,
            screen,
            pen: CellAttributes::default(),
            cursor: CursorPosition::default(),
            top_and_bottom_margins: 0..size.rows as VisibleRowIndex,
            left_and_right_margins: 0..size.cols,
            left_and_right_margin_mode: false,
            wrap_next: false,
            clear_semantic_attribute_on_newline: false,
            // We default auto wrap to true even though the default for
            // a dec terminal is false, because it is more useful this way.
            dec_auto_wrap: true,
            reverse_wraparound_mode: false,
            reverse_video_mode: false,
            dec_origin_mode: false,
            insert: false,
            application_cursor_keys: false,
            modify_other_keys: None,
            dec_ansi_mode: false,
            sixel_display_mode: false,
            use_private_color_registers_for_each_graphic: false,
            color_map,
            application_keypad: false,
            bracketed_paste: false,
            focus_tracking: false,
            mouse_encoding: MouseEncoding::X10,
            keyboard_encoding: KeyboardEncoding::Xterm,
            sixel_scrolls_right: false,
            any_event_mouse: false,
            button_event_mouse: false,
            mouse_tracking: false,
            last_mouse_move: None,
            cursor_visible: true,
            g0_charset: CharSet::Ascii,
            g1_charset: CharSet::Ascii,
            shift_out: false,
            newline_mode: false,
            current_mouse_buttons: vec![],
            tabs: TabStop::new(size.cols, 8),
            title: "wezterm".to_string(),
            icon_title: None,
            palette: None,
            pixel_height: size.pixel_height,
            pixel_width: size.pixel_width,
            dpi: size.dpi,
            clipboard: None,
            device_control_handler: None,
            alert_handler: None,
            download_handler: None,
            current_dir: None,
            term_program: term_program.to_string(),
            term_version: term_version.to_string(),
            writer,
            image_cache: lru::LruCache::new(NonZeroUsize::new(16).unwrap()),
            user_vars: HashMap::new(),
            kitty_img: Default::default(),
            seqno,
            unicode_version,
            unicode_version_stack: vec![],
            suppress_initial_title_change: false,
            enable_conpty_quirks: false,
            accumulating_title: None,
            lost_focus_seqno: seqno,
            lost_focus_alerted_seqno: seqno,
            focused: true,
            bidi_enabled: None,
            bidi_hint: None,
            progress: Progress::default(),
        }
    }

    pub fn enable_conpty_quirks(&mut self) {
        self.enable_conpty_quirks = true;
        self.suppress_initial_title_change = true;
    }

    pub fn current_seqno(&self) -> SequenceNo {
        self.seqno
    }

    pub fn increment_seqno(&mut self) {
        self.seqno += 1;
    }

    pub fn set_config(&mut self, config: Arc<dyn TerminalConfiguration>) {
        self.config = config;
    }

    pub fn get_config(&self) -> Arc<dyn TerminalConfiguration> {
        Arc::clone(&self.config)
    }

    pub fn set_clipboard(&mut self, clipboard: &Arc<dyn Clipboard>) {
        self.clipboard.replace(Arc::clone(clipboard));
    }

    pub fn set_device_control_handler(&mut self, handler: Box<dyn DeviceControlHandler>) {
        self.device_control_handler.replace(handler);
    }

    pub fn set_notification_handler(&mut self, handler: Box<dyn AlertHandler>) {
        self.alert_handler.replace(handler);
    }

    pub fn set_download_handler(&mut self, handler: &Arc<dyn DownloadHandler>) {
        self.download_handler.replace(handler.clone());
    }

    /// Returns the title text associated with the terminal session.
    /// The title can be changed by the application using a number
    /// of escape sequences:
    /// OSC 2 is used to set the window title.
    /// OSC 1 is used to set the "icon title", which some terminal
    /// emulators interpret as a shorter title string for use when
    /// showing the tab title.
    /// Here in wezterm the terminalstate is isolated from other
    /// tabs; we process escape sequences without knowledge of other
    /// tabs, so we maintain both title strings here.
    /// The gui layer doesn't currently have a concept of what the
    /// overall window title should be beyond the title for the
    /// active tab with some decoration about the number of tabs.
    /// Shell toolkits such as oh-my-zsh prefer OSC 1 titles for
    /// abbreviated information.
    /// What we do here is prefer to return the OSC 1 icon title
    /// if it is set, otherwise return the OSC 2 window title.
    pub fn get_title(&self) -> &str {
        self.icon_title.as_ref().unwrap_or(&self.title)
    }

    pub fn get_progress(&self) -> Progress {
        self.progress.clone()
    }

    /// Returns the current working directory associated with the
    /// terminal session.  The working directory can be changed by
    /// the applicaiton using the OSC 7 escape sequence.
    pub fn get_current_dir(&self) -> Option<&Url> {
        self.current_dir.as_ref()
    }

    /// Returns a copy of the palette.
    /// By default we don't keep a copy in the terminal state,
    /// preferring to take the config values from the users
    /// config file and updating to changes live.
    /// However, if they have used dynamic color scheme escape
    /// sequences we'll fork a copy of the palette at that time
    /// so that we can start tracking those changes.
    pub fn palette(&self) -> ColorPalette {
        self.palette
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.config.color_palette())
    }

    /// Called in response to dynamic color scheme escape sequences.
    /// Will make a copy of the palette from the config file if this
    /// is the first of these escapes we've seen.
    pub fn palette_mut(&mut self) -> &mut ColorPalette {
        if self.palette.is_none() {
            self.palette.replace(self.config.color_palette());
        }
        self.palette.as_mut().unwrap()
    }

    /// If the current overridden palette is effectively the same as
    /// the configured palette, remove the override and treat it as
    /// being the same as the configured state.
    /// This allows runtime changes to the configuration to take effect.
    pub fn implicit_palette_reset_if_same_as_configured(&mut self) {
        if self
            .palette
            .as_ref()
            .map(|p| *p == self.config.color_palette())
            .unwrap_or(false)
        {
            self.palette.take();
        }
    }

    /// Returns a reference to the active screen (either the primary or
    /// the alternate screen).
    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    /// Returns a mutable reference to the active screen (either the primary or
    /// the alternate screen).
    pub fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }

    pub(super) fn set_clipboard_contents(
        &self,
        selection: ClipboardSelection,
        text: Option<String>,
    ) -> anyhow::Result<()> {
        if let Some(clip) = self.clipboard.as_ref() {
            clip.set_contents(selection, text)?;
        }
        Ok(())
    }

    pub fn erase_scrollback_and_viewport(&mut self) {
        // Since we may be called outside of perform_actions,
        // we need to ensure that we increment the seqno in
        // order to correctly invalidate the display
        self.increment_seqno();
        self.erase_in_display(EraseInDisplay::EraseScrollback);

        let row_index = self.screen.phys_row(self.cursor.y);
        let rows = self.screen.lines_in_phys_range(row_index..row_index + 1);

        self.erase_in_display(EraseInDisplay::EraseDisplay);

        for (idx, row) in rows.into_iter().enumerate() {
            *self.screen.line_mut(idx) = row;
        }

        self.cursor.y = 0;
    }

    /// Discards the scrollback, leaving only the data that is present
    /// in the viewport.
    pub fn erase_scrollback(&mut self) {
        // Since we may be called outside of perform_actions,
        // we need to ensure that we increment the seqno in
        // order to correctly invalidate the display
        self.increment_seqno();
        self.screen_mut().erase_scrollback();
    }

    /// Returns true if the associated application has enabled any of the
    /// supported mouse reporting modes.
    /// This is useful for the hosting GUI application to decide how best
    /// to dispatch mouse events to the terminal.
    pub fn is_mouse_grabbed(&self) -> bool {
        self.mouse_tracking || self.button_event_mouse || self.any_event_mouse
    }

    pub fn is_alt_screen_active(&self) -> bool {
        self.screen.is_alt_screen_active()
    }

    /// Returns true if the associated application has enabled
    /// bracketed paste mode, which can be helpful to the hosting
    /// GUI application to decide about fragmenting a large paste.
    pub fn bracketed_paste_enabled(&self) -> bool {
        self.bracketed_paste
    }

    /// Advise the terminal about a change in its focus state
    pub fn focus_changed(&mut self, focused: bool) {
        if focused == self.focused {
            return;
        }
        if !focused {
            // notify app of release of buttons
            let buttons = self.current_mouse_buttons.clone();
            for b in buttons {
                self.mouse_event(MouseEvent {
                    kind: MouseEventKind::Release,
                    button: b,
                    modifiers: KeyModifiers::NONE,
                    x: 0,
                    y: 0,
                    x_pixel_offset: 0,
                    y_pixel_offset: 0,
                })
                .ok();
            }
        }
        if self.focus_tracking {
            write!(self.writer, "{}{}", CSI, if focused { "I" } else { "O" }).ok();
            self.writer.flush().ok();
        }
        self.focused = focused;
        if !focused {
            self.lost_focus_seqno = self.seqno;
        }
    }

    /// Returns true if there is new output since the terminal
    /// lost focus
    pub fn has_unseen_output(&self) -> bool {
        !self.focused && self.seqno > self.lost_focus_seqno
    }

    pub(crate) fn trigger_unseen_output_notif(&mut self) {
        if self.has_unseen_output() {
            // We want to avoid over-notifying about output events,
            // so here we gate the notification to the case where
            // we have lost the focus more recently than the last
            // time we notified about it
            if self.lost_focus_seqno > self.lost_focus_alerted_seqno {
                self.lost_focus_alerted_seqno = self.seqno;
                if let Some(handler) = self.alert_handler.as_mut() {
                    handler.alert(Alert::OutputSinceFocusLost);
                }
            }
        }
    }

    /// Send text to the terminal that is the result of pasting.
    /// If bracketed paste mode is enabled, the paste is enclosed
    /// in the bracketing, otherwise it is fed to the writer as-is.
    /// De-fang the text by removing any embedded bracketed paste
    /// sequence that may be present.
    pub fn send_paste(&mut self, text: &str) -> Result<(), Error> {
        let mut buf = String::new();
        if self.bracketed_paste {
            buf.push_str("\x1b[200~");
        }

        let canon = if self.bracketed_paste {
            NewlineCanon::None
        } else {
            self.config.canonicalize_pasted_newlines()
        };

        let canon = canon.canonicalize(text);
        let de_fanged = canon.replace("\x1b[200~", "").replace("\x1b[201~", "");
        buf.push_str(&de_fanged);

        if self.bracketed_paste {
            buf.push_str("\x1b[201~");
        }

        self.writer.write_all(buf.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }
}
