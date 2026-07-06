pub struct TermWizTerminalPane {
    pane_id: PaneId,
    domain_id: DomainId,
    terminal: Mutex<wezterm_term::Terminal>,
    input_tx: Sender<InputEvent>,
    dead: Mutex<bool>,
    writer: Mutex<Vec<u8>>,
    render_rx: FileDescriptor,
}

impl TermWizTerminalPane {
    fn new(
        domain_id: DomainId,
        size: TerminalSize,
        input_tx: Sender<InputEvent>,
        render_rx: FileDescriptor,
        term_config: Option<Arc<dyn TerminalConfiguration + Send + Sync>>,
    ) -> Self {
        let pane_id = alloc_pane_id();

        let terminal = Mutex::new(wezterm_term::Terminal::new(
            size,
            term_config.unwrap_or_else(|| Arc::new(config::TermConfig::new())),
            "WezTerm",
            config::wezterm_version(),
            Box::new(Vec::new()), // FIXME: connect to something?
        ));

        Self {
            pane_id,
            domain_id,
            terminal,
            writer: Mutex::new(Vec::new()),
            render_rx,
            input_tx,
            dead: Mutex::new(false),
        }
    }
}

impl Pane for TermWizTerminalPane {
    fn pane_id(&self) -> PaneId {
        self.pane_id
    }

    fn get_cursor_position(&self) -> StableCursorPosition {
        terminal_get_cursor_position(&mut self.terminal.lock())
    }

    fn get_current_seqno(&self) -> SequenceNo {
        self.terminal.lock().current_seqno()
    }

    fn get_changed_since(
        &self,
        lines: Range<StableRowIndex>,
        seqno: SequenceNo,
    ) -> RangeSet<StableRowIndex> {
        terminal_get_dirty_lines(&mut self.terminal.lock(), lines, seqno)
    }

    fn for_each_logical_line_in_stable_range_mut(
        &self,
        lines: Range<StableRowIndex>,
        for_line: &mut dyn ForEachPaneLogicalLine,
    ) {
        terminal_for_each_logical_line_in_stable_range_mut(
            &mut self.terminal.lock(),
            lines,
            for_line,
        );
    }

    fn get_logical_lines(&self, lines: Range<StableRowIndex>) -> Vec<LogicalLine> {
        crate::pane::impl_get_logical_lines_via_get_lines(self, lines)
    }

    fn with_lines_mut(&self, lines: Range<StableRowIndex>, with_lines: &mut dyn WithPaneLines) {
        terminal_with_lines_mut(&mut self.terminal.lock(), lines, with_lines)
    }

    fn get_lines(&self, lines: Range<StableRowIndex>) -> (StableRowIndex, Vec<Line>) {
        terminal_get_lines(&mut self.terminal.lock(), lines)
    }

    fn get_dimensions(&self) -> RenderableDimensions {
        terminal_get_dimensions(&mut self.terminal.lock())
    }

    fn get_title(&self) -> String {
        self.terminal.lock().get_title().to_string()
    }

    fn can_close_without_prompting(&self, _reason: CloseReason) -> bool {
        true
    }

    fn send_paste(&self, text: &str) -> anyhow::Result<()> {
        let paste = InputEvent::Paste(text.to_string());
        self.input_tx.send(paste)?;
        Ok(())
    }

    fn reader(&self) -> anyhow::Result<Option<Box<dyn std::io::Read + Send>>> {
        Ok(Some(Box::new(self.render_rx.try_clone()?)))
    }

    fn writer(&self) -> MappedMutexGuard<'_, dyn std::io::Write> {
        MutexGuard::map(self.writer.lock(), |writer| {
            let w: &mut dyn std::io::Write = writer;
            w
        })
    }

    fn resize(&self, size: TerminalSize) -> anyhow::Result<()> {
        self.input_tx.send(InputEvent::Resized {
            rows: size.rows as usize,
            cols: size.cols as usize,
        })?;

        self.terminal.lock().resize(size);

        Ok(())
    }

    fn key_down(&self, key: KeyCode, modifiers: KeyModifiers) -> anyhow::Result<()> {
        let event = InputEvent::Key(KeyEvent {
            key,
            modifiers: modifiers.remove_positional_mods(),
        });
        if let Err(e) = self.input_tx.send(event) {
            *self.dead.lock() = true;
            return Err(e.into());
        }
        Ok(())
    }

    fn key_up(&self, _key: KeyCode, _modifiers: KeyModifiers) -> anyhow::Result<()> {
        Ok(())
    }

    fn mouse_event(&self, event: MouseEvent) -> anyhow::Result<()> {
        use termwiz::input::MouseButtons as Buttons;
        use wezterm_term::input::MouseButton;

        let mouse_buttons = match event.button {
            MouseButton::Left => Buttons::LEFT,
            MouseButton::Middle => Buttons::MIDDLE,
            MouseButton::Right => Buttons::RIGHT,
            MouseButton::WheelUp(_) => Buttons::VERT_WHEEL | Buttons::WHEEL_POSITIVE,
            MouseButton::WheelDown(_) => Buttons::VERT_WHEEL,
            MouseButton::WheelLeft(_) => Buttons::HORZ_WHEEL | Buttons::WHEEL_POSITIVE,
            MouseButton::WheelRight(_) => Buttons::HORZ_WHEEL,
            MouseButton::None => Buttons::NONE,
        };

        let event = InputEvent::Mouse(TermWizMouseEvent {
            x: event.x as u16,
            y: event.y as u16,
            mouse_buttons,
            modifiers: event.modifiers,
        });
        if let Err(e) = self.input_tx.send(event) {
            *self.dead.lock() = true;
            return Err(e.into());
        }
        Ok(())
    }

    fn set_config(&self, config: Arc<dyn TerminalConfiguration>) {
        self.terminal.lock().set_config(config);
    }

    fn get_config(&self) -> Option<Arc<dyn TerminalConfiguration>> {
        Some(self.terminal.lock().get_config())
    }

    fn perform_actions(&self, actions: Vec<termwiz::escape::Action>) {
        self.terminal.lock().perform_actions(actions)
    }

    fn kill(&self) {
        *self.dead.lock() = true;
    }

    fn is_dead(&self) -> bool {
        *self.dead.lock()
    }

    fn palette(&self) -> ColorPalette {
        self.terminal.lock().palette()
    }

    fn domain_id(&self) -> DomainId {
        self.domain_id
    }

    fn is_mouse_grabbed(&self) -> bool {
        self.terminal.lock().is_mouse_grabbed()
    }

    fn is_alt_screen_active(&self) -> bool {
        self.terminal.lock().is_alt_screen_active()
    }

    fn get_current_working_dir(&self, _policy: CachePolicy) -> Option<Url> {
        self.terminal.lock().get_current_dir().cloned()
    }

    fn erase_scrollback(&self, erase_mode: ScrollbackEraseMode) {
        match erase_mode {
            ScrollbackEraseMode::ScrollbackOnly => {
                self.terminal.lock().erase_scrollback();
            }
            ScrollbackEraseMode::ScrollbackAndViewport => {
                self.terminal.lock().erase_scrollback_and_viewport();
            }
        }
    }
}
