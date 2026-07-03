/// The `LineEditor` struct provides line editing facilities similar
/// to those in the unix shell.
/// ```no_run
/// use termwiz::lineedit::{line_editor_terminal, NopLineEditorHost, LineEditor};
///
/// fn main() -> termwiz::Result<()> {
///     let mut terminal = line_editor_terminal()?;
///     let mut editor = LineEditor::new(&mut terminal);
///     let mut host = NopLineEditorHost::default();
///
///     let line = editor.read_line(&mut host)?;
///     println!("read line: {:?}", line);
///
///     Ok(())
/// }
/// ```
pub struct LineEditor<'term> {
    terminal: &'term mut dyn Terminal,
    prompt: String,
    line: LineEditBuffer,

    history_pos: Option<usize>,
    bottom_line: Option<String>,

    completion: Option<CompletionState>,

    move_to_editor_start: Option<Change>,
    move_to_editor_end: Option<Change>,

    state: EditorState,
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum EditorState {
    Inactive,
    Editing,
    Cancelled,
    Accepted,
    Searching {
        style: SearchStyle,
        direction: SearchDirection,
        matching_line: String,
        cursor: usize,
    },
}

struct CompletionState {
    candidates: Vec<CompletionCandidate>,
    index: usize,
    original_line: String,
    original_cursor: usize,
}

impl CompletionState {
    fn next(&mut self) {
        self.index += 1;
        if self.index >= self.candidates.len() {
            self.index = 0;
        }
    }

    fn current(&self) -> (usize, String) {
        let mut line = self.original_line.clone();
        let candidate = &self.candidates[self.index];
        line.replace_range(candidate.range.clone(), &candidate.text);

        // To figure the new cursor position do a little math:
        // "he<TAB>" when the completion is "hello" will set the completion
        // candidate to replace "he" with "hello", so the difference in the
        // lengths of these two is how far the cursor needs to move.
        let range_len = candidate.range.end - candidate.range.start;
        let new_cursor = self.original_cursor + candidate.text.len() - range_len;

        (new_cursor, line)
    }
}
