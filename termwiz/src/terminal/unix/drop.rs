impl Drop for UnixTerminal {
    fn drop(&mut self) {
        macro_rules! decreset {
            ($variant:ident) => {
                write!(
                    self.write,
                    "{}",
                    CSI::Mode(Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                        DecPrivateModeCode::$variant
                    )))
                )
                .unwrap();
            };
        }
        self.render(&[Change::CursorVisibility(
            crate::surface::CursorVisibility::Visible,
        )])
        .ok();
        if self.caps.bracketed_paste() {
            decreset!(BracketedPaste);
        }
        if self.caps.mouse_reporting() {
            decreset!(SGRMouse);
            decreset!(AnyEventMouse);
        }
        self.write.modify_other_keys(0).unwrap();
        self.exit_alternate_screen().unwrap();
        self.write.flush().unwrap();

        signal_hook::low_level::unregister(self.sigwinch_id);
        self.write
            .set_termios(&self.saved_termios, SetAttributeWhen::Now)
            .expect("failed to restore original termios state");
    }
}
