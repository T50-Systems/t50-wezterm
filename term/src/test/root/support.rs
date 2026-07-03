#[derive(Debug)]
struct LocalClip {
    clip: Mutex<Option<String>>,
}

impl LocalClip {
    fn new() -> Self {
        Self {
            clip: Mutex::new(None),
        }
    }
}

impl Clipboard for LocalClip {
    fn set_contents(
        &self,
        _selection: ClipboardSelection,
        clip: Option<String>,
    ) -> anyhow::Result<()> {
        *self.clip.lock().unwrap() = clip;
        Ok(())
    }
}

struct TestTerm {
    term: Terminal,
}

#[derive(Debug)]
struct TestTermConfig {
    scrollback: usize,
}
impl TerminalConfiguration for TestTermConfig {
    fn scrollback_size(&self) -> usize {
        self.scrollback
    }

    fn color_palette(&self) -> ColorPalette {
        ColorPalette::default()
    }

    fn enable_kitty_graphics(&self) -> bool {
        true
    }
}

impl TestTerm {
    fn new(height: usize, width: usize, scrollback: usize) -> Self {
        let _ = env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Trace)
            .try_init();

        let mut term = Terminal::new(
            TerminalSize {
                rows: height,
                cols: width,
                pixel_width: width * 8,
                pixel_height: height * 16,
                dpi: 0,
            },
            Arc::new(TestTermConfig { scrollback }),
            "WezTerm",
            "O_o",
            Box::new(Vec::new()),
        );
        let clip: Arc<dyn Clipboard> = Arc::new(LocalClip::new());
        term.set_clipboard(&clip);

        let mut term = Self { term };

        term.set_auto_wrap(true);

        term
    }

    fn print<B: AsRef<[u8]>>(&mut self, bytes: B) {
        self.term.advance_bytes(bytes);
    }

    fn set_mode(&mut self, mode: &str, enable: bool) {
        self.print(CSI);
        self.print(mode);
        self.print(if enable { b"h" } else { b"l" });
    }

    fn set_auto_wrap(&mut self, enable: bool) {
        self.set_mode("?7", enable);
    }

    fn set_left_and_right_margins(&mut self, left: usize, right: usize) {
        self.print(CSI);
        self.print(format!("{};{}s", left + 1, right + 1));
    }

    fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        self.print(CSI);
        self.print(format!("{};{}r", top + 1, bottom + 1));
    }

    fn delete_lines(&mut self, n: isize) {
        self.print(CSI);
        self.print(format!("{}M", n));
    }

    fn cup(&mut self, col: isize, row: isize) {
        self.print(CSI);
        self.print(format!("{};{}H", row + 1, col + 1));
    }

    fn hvp(&mut self, col: isize, row: isize) {
        self.print(CSI);
        self.print(format!("{};{}f", row + 1, col + 1));
    }

    fn erase_in_display(&mut self, erase: EraseInDisplay) {
        let csi = CSI::Edit(Edit::EraseInDisplay(erase));
        self.print(format!("{}", csi));
    }

    fn erase_in_line(&mut self, erase: EraseInLine) {
        let csi = CSI::Edit(Edit::EraseInLine(erase));
        self.print(format!("{}", csi));
    }

    fn hyperlink(&mut self, link: &Arc<Hyperlink>) {
        let osc = OperatingSystemCommand::SetHyperlink(Some(link.as_ref().clone()));
        self.print(format!("{}", osc));
    }

    fn hyperlink_off(&mut self) {
        self.print("\x1b]8;;\x1b\\");
    }

    fn soft_reset(&mut self) {
        self.print(CSI);
        self.print("!p");
    }

    fn assert_cursor_pos(&self, x: usize, y: i64, reason: Option<&str>, seqno: Option<SequenceNo>) {
        let cursor = self.cursor_pos();
        let expect = CursorPosition {
            x,
            y,
            shape: CursorShape::Default,
            visibility: CursorVisibility::Visible,
            seqno: seqno.unwrap_or_else(|| self.current_seqno()),
        };
        assert_eq!(
            cursor, expect,
            "actual cursor (left) didn't match expected cursor (right) reason={:?}",
            reason
        );
    }

    fn assert_dirty_lines(&self, seqno: SequenceNo, expected: &[usize], reason: Option<&str>) {
        let mut seqs = vec![];
        let mut dirty_indices = vec![];

        self.screen().for_each_phys_line(|i, line| {
            seqs.push(line.current_seqno());
            if line.changed_since(seqno) {
                dirty_indices.push(i);
            }
        });
        assert_eq!(
            &dirty_indices, &expected,
            "actual dirty lines (left) didn't match expected dirty \
             lines (right) reason={:?}. threshold seq: {} seqs: {:?}",
            reason, seqno, seqs
        );
    }
}

impl Deref for TestTerm {
    type Target = Terminal;

    fn deref(&self) -> &Terminal {
        &self.term
    }
}

impl DerefMut for TestTerm {
    fn deref_mut(&mut self) -> &mut Terminal {
        &mut self.term
    }
}

/// Asserts that both line slices match according to the
/// selected flags.
fn assert_lines_equal(
    file: &str,
    line_no: u32,
    lines: &[Line],
    expect_lines: &[Line],
    compare: Compare,
) {
    let mut expect_iter = expect_lines.iter();

    println!("actual_lines:");
    for line in lines {
        println!("[{}]", line.as_str());
    }
    println!("expect_lines");
    for line in expect_lines {
        println!("[{}]", line.as_str());
    }

    for (idx, line) in lines.iter().enumerate() {
        let expect = match expect_iter.next() {
            Some(e) => e,
            None => break,
        };

        if compare.contains(Compare::ATTRS) {
            let line_attrs: Vec<_> = line.visible_cells().map(|c| c.attrs().clone()).collect();
            let expect_attrs: Vec<_> = expect.visible_cells().map(|c| c.attrs().clone()).collect();
            assert_eq!(
                expect_attrs,
                line_attrs,
                "{}:{}: line {} `{}` attrs didn't match (left=expected, right=actual)",
                file,
                line_no,
                idx,
                line.as_str()
            );
        }
        if compare.contains(Compare::TEXT) {
            let line_str = line.as_str();
            let expect_str = expect.as_str();
            assert_eq!(
                line_str,
                expect_str,
                "{}:{}: line {} text didn't match '{}' vs '{}'",
                file,
                line_no,
                idx,
                line_str.escape_default(),
                expect_str.escape_default()
            );
        }
    }

    assert_eq!(
        lines.len(),
        expect_lines.len(),
        "{}:{}: expectation has wrong number of lines",
        file,
        line_no
    );
}

bitflags! {
    struct Compare : u8{
        const TEXT = 1;
        const ATTRS = 2;
        const DIRTY = 4;
    }
}

fn print_all_lines(term: &Terminal) {
    let screen = term.screen();

    println!("whole screen contents are:");
    screen.for_each_phys_line(|_, line| {
        println!("[{}]", line.as_str());
    });
}

fn print_visible_lines(term: &Terminal) {
    let screen = term.screen();

    println!("screen contents are:");
    for line in screen.visible_lines().iter() {
        println!("[{}]", line.as_str());
    }
}

/// Asserts that the visible lines of the terminal have the
/// same character contents as the expected lines.
/// The other cell attributes are not compared; this is
/// a convenience for writing visually understandable tests.
fn assert_visible_contents(term: &Terminal, file: &str, line: u32, expect_lines: &[&str]) {
    print_visible_lines(&term);
    let screen = term.screen();

    let expect: Vec<Line> = expect_lines.iter().map(|s| (*s).into()).collect();

    assert_lines_equal(file, line, &screen.visible_lines(), &expect, Compare::TEXT);
}

fn assert_all_contents(term: &Terminal, file: &str, line: u32, expect_lines: &[&str]) {
    print_all_lines(&term);
    let screen = term.screen();

    let expect: Vec<Line> = expect_lines.iter().map(|s| (*s).into()).collect();

    assert_lines_equal(file, line, &screen.all_lines(), &expect, Compare::TEXT);
}
