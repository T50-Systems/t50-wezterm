impl<'term> LineEditor<'term> {
    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.to_owned();
    }

    /// Enter line editing mode.
    /// Control is not returned to the caller until a line has been
    /// accepted, or until an error is detected.
    /// Returns Ok(None) if the editor was cancelled eg: via CTRL-C.
    pub fn read_line(&mut self, host: &mut dyn LineEditorHost) -> Result<Option<String>> {
        self.read_line_with_optional_initial_value(host, None)
    }

    pub fn read_line_with_optional_initial_value(
        &mut self,
        host: &mut dyn LineEditorHost,
        initial_value: Option<&str>,
    ) -> Result<Option<String>> {
        ensure!(
            self.state == EditorState::Inactive,
            "recursive call to read_line!"
        );

        // Clear out the last render info so that we don't over-compensate
        // on the first call to render().
        self.move_to_editor_end.take();
        self.move_to_editor_start.take();

        self.terminal.set_raw_mode()?;
        self.state = EditorState::Editing;
        let res = self.read_line_impl(host, initial_value);
        self.state = EditorState::Inactive;

        if let Some(move_end) = self.move_to_editor_end.take() {
            self.terminal
                .render(&[move_end, Change::ClearToEndOfScreen(Default::default())])?;
        }

        self.terminal.flush()?;
        self.terminal.set_cooked_mode()?;
        res
    }
}
