impl<'term> LineEditor<'term> {
    /// Create a new line editor.
    /// In most cases, you'll want to use the `line_editor` function,
    /// because it creates a `Terminal` instance with the recommended
    /// settings, but if you need to decompose that for some reason,
    /// this snippet shows the recommended way to create a line
    /// editor:
    ///
    /// ```no_run
    /// use termwiz::caps::{Capabilities, ProbeHints};
    /// use termwiz::terminal::new_terminal;
    /// use termwiz::Error;
    /// // Disable mouse input in the line editor
    /// let hints = ProbeHints::new_from_env()
    ///     .mouse_reporting(Some(false));
    /// let caps = Capabilities::new_with_hints(hints)?;
    /// let terminal = new_terminal(caps)?;
    /// # Ok::<(), Error>(())
    /// ```
    pub fn new(terminal: &'term mut dyn Terminal) -> Self {
        Self {
            terminal,
            prompt: "> ".to_owned(),
            line: LineEditBuffer::default(),
            history_pos: None,
            bottom_line: None,
            completion: None,
            move_to_editor_start: None,
            move_to_editor_end: None,
            state: EditorState::Inactive,
        }
    }

    fn render(&mut self, host: &mut dyn LineEditorHost) -> Result<()> {
        let screen_size = self.terminal.get_screen_size()?;

        let mut changes = ChangeSequence::new(screen_size.rows, screen_size.cols);

        changes.add(Change::ClearToEndOfScreen(Default::default()));
        changes.add(Change::AllAttributes(Default::default()));
        for ele in host.render_prompt(&self.prompt) {
            changes.add(ele);
        }
        changes.add(Change::AllAttributes(Default::default()));

        // If we're searching, the input area shows the match rather than the input,
        // and the cursor moves to the first matching character
        let (line_to_display, cursor) = match &self.state {
            EditorState::Searching {
                matching_line,
                cursor,
                ..
            } => (matching_line.as_str(), *cursor),
            _ => (self.line.get_line(), self.line.get_cursor()),
        };

        let cursor_position_after_printing_prompt = changes.current_cursor_position();

        let (elements, cursor_x_pos) = host.highlight_line(line_to_display, cursor);

        // Calculate what the cursor position would be after printing X columns
        // of text from the specified location.
        // Returns (x, y) of the resultant cursor position.
        fn compute_cursor_after_printing_x_columns(
            cursor_x: usize,
            cursor_y: isize,
            delta: usize,
            screen_cols: usize,
        ) -> (usize, isize) {
            let y = (cursor_x + delta) / screen_cols;
            let x = (cursor_x + delta) % screen_cols;

            let row = cursor_y + y as isize;
            let col = x.max(0) as usize;

            (col, row)
        }
        let cursor_position = compute_cursor_after_printing_x_columns(
            cursor_position_after_printing_prompt.0,
            cursor_position_after_printing_prompt.1,
            cursor_x_pos,
            screen_size.cols,
        );

        for ele in elements {
            changes.add(ele);
        }

        let cursor_after_line_render = changes.current_cursor_position();
        if cursor_after_line_render.0 == screen_size.cols {
            // If the cursor position remains in the first column
            // then the renderer may still consider itself to be on
            // the prior line; force out an additional character to force
            // it to apply wrapping/flush.
            changes.add(" ");
        }

        if let EditorState::Editing = &self.state {
            let preview_elements = host.render_preview(line_to_display);
            if !preview_elements.is_empty() {
                // Preview starts from a new line.
                changes.add("\r\n");
                // Do not be affected by attributes set by highlight_line.
                changes.add(Change::AllAttributes(Default::default()));
                for ele in preview_elements {
                    changes.add(ele);
                }
            }
        }

        if let EditorState::Searching {
            style, direction, ..
        } = &self.state
        {
            // We want to draw the search state below the input area
            let label = match (style, direction) {
                (SearchStyle::Substring, SearchDirection::Backwards) => "bck-i-search",
                (SearchStyle::Substring, SearchDirection::Forwards) => "fwd-i-search",
            };
            // Do not be affected by attributes set by previous lines.
            changes.add(Change::AllAttributes(Default::default()));
            // We position the actual cursor on the matching portion of
            // the text in the line editing area, but since the input
            // is drawn here, we render an `_` to indicate where the input
            // position really is.
            changes.add(format!("\r\n{}: {}_", label, self.line.get_line()));
        }

        // Add some debugging status at the bottom
        /*
        changes.add(format!(
            "\r\n{:?} {:?}",
            cursor_position,
            (changes.cursor_x, changes.cursor_y)
        ));
        */

        let render_height = changes.render_height();

        changes.move_to(cursor_position);

        let mut changes = changes.consume();
        if let Some(start) = self.move_to_editor_start.take() {
            changes.insert(0, start);
        }
        self.terminal.render(&changes)?;

        self.move_to_editor_start.replace(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Relative(-1 * cursor_position.1),
        });

        self.move_to_editor_end.replace(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Relative(1 + render_height as isize - cursor_position.1),
        });

        Ok(())
    }
}
