/// Create a `Terminal` with the recommended settings for use with
/// a `LineEditor`.
pub fn line_editor_terminal() -> Result<impl Terminal> {
    let hints = ProbeHints::new_from_env().mouse_reporting(Some(false));
    let caps = Capabilities::new_with_hints(hints)?;
    new_terminal(caps)
}
