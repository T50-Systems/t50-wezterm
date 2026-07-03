impl<'term> LineEditor<'term> {
    fn read_line_impl(
        &mut self,
        host: &mut dyn LineEditorHost,
        initial_value: Option<&str>,
    ) -> Result<Option<String>> {
        self.line.clear();
        if let Some(value) = initial_value {
            self.line.set_line_and_cursor(value, value.len());
        }
        self.history_pos = None;
        self.bottom_line = None;
        self.clear_completion();

        self.render(host)?;
        while let Some(event) = self.terminal.poll_input(None)? {
            if let Some(action) = self.resolve_action(&event, host) {
                self.apply_action(host, action)?;
                // Editor state might have changed. Re-render to clear
                // preview or highlight lines differently.
                self.render(host)?;
                match self.state {
                    EditorState::Searching { .. } | EditorState::Editing => {}
                    EditorState::Cancelled => return Ok(None),
                    EditorState::Accepted => return Ok(Some(self.line.get_line().to_string())),
                    EditorState::Inactive => bail!("editor is inactive during read line!?"),
                }
            } else {
                self.render(host)?;
            }
        }
        Ok(Some(self.line.get_line().to_string()))
    }
}
