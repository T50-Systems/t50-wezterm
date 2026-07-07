use super::*;

impl TerminalState {
    pub(super) fn perform_csi_mode_part2(&mut self, mode: Mode) -> Option<Mode> {
        match mode {
            Mode::SetMode(TerminalMode::Code(TerminalModeCode::BiDirectionalSupportMode)) => {
                self.bidi_enabled.replace(true);
            }
            Mode::ResetMode(TerminalMode::Code(TerminalModeCode::BiDirectionalSupportMode)) => {
                self.bidi_enabled.replace(false);
            }
            Mode::QueryMode(TerminalMode::Code(TerminalModeCode::BiDirectionalSupportMode)) => {
                self.decqrm_response(
                    mode,
                    true,
                    self.bidi_enabled
                        .unwrap_or_else(|| self.config.bidi_mode().enabled),
                );
            }

            Mode::SetMode(TerminalMode::Code(TerminalModeCode::Insert)) => {
                self.insert = true;
            }
            Mode::ResetMode(TerminalMode::Code(TerminalModeCode::Insert)) => {
                self.insert = false;
            }
            Mode::QueryMode(TerminalMode::Code(TerminalModeCode::Insert)) => {
                self.decqrm_response(mode, true, self.insert);
            }

            Mode::SetMode(TerminalMode::Code(TerminalModeCode::AutomaticNewline)) => {
                self.newline_mode = true;
            }
            Mode::ResetMode(TerminalMode::Code(TerminalModeCode::AutomaticNewline)) => {
                self.newline_mode = false;
            }
            Mode::QueryMode(TerminalMode::Code(TerminalModeCode::AutomaticNewline)) => {
                self.decqrm_response(mode, true, self.newline_mode);
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::BracketedPaste)) => {
                self.bracketed_paste = true;
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::BracketedPaste)) => {
                self.bracketed_paste = false;
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::BracketedPaste)) => {
                self.decqrm_response(mode, true, self.bracketed_paste);
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::OptEnableAlternateScreen,
            ))
            | Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::EnableAlternateScreen,
            )) => {
                if !self.screen.is_alt_screen_active() {
                    self.screen.activate_alt_screen(self.seqno);
                    self.pen = CellAttributes::default();
                }
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::OptEnableAlternateScreen,
            )) => {
                if self.screen.is_alt_screen_active() {
                    self.pen = CellAttributes::default();
                    self.erase_in_display(EraseInDisplay::EraseDisplay);
                    self.screen.activate_primary_screen(self.seqno);
                }
            }

            Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::EnableAlternateScreen,
            )) => {
                if self.screen.is_alt_screen_active() {
                    self.screen.activate_primary_screen(self.seqno);
                    self.pen = CellAttributes::default();
                }
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::ApplicationCursorKeys,
            )) => {
                self.application_cursor_keys = true;
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::ApplicationCursorKeys,
            )) => {
                self.application_cursor_keys = false;
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::ApplicationCursorKeys,
            )) => {
                self.decqrm_response(mode, true, self.application_cursor_keys);
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::SixelDisplayMode)) => {
                self.sixel_display_mode = true;
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::SixelDisplayMode,
            )) => {
                self.sixel_display_mode = false;
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::SixelDisplayMode,
            )) => {
                self.decqrm_response(mode, true, self.sixel_display_mode);
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::DecAnsiMode)) => {
                self.dec_ansi_mode = true;
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::DecAnsiMode)) => {
                self.dec_ansi_mode = false;
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::DecAnsiMode)) => {
                self.decqrm_response(mode, true, self.dec_ansi_mode);
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::ShowCursor)) => {
                self.cursor_visible = true;
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::ShowCursor)) => {
                self.cursor_visible = false;
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::ShowCursor)) => {
                self.decqrm_response(mode, true, self.cursor_visible);
            }
            Mode::SetMode(TerminalMode::Code(TerminalModeCode::ShowCursor)) => {
                self.cursor_visible = true;
            }
            Mode::ResetMode(TerminalMode::Code(TerminalModeCode::ShowCursor)) => {
                self.cursor_visible = false;
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::MouseTracking)) => {
                self.mouse_tracking = true;
                self.last_mouse_move.take();
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::MouseTracking)) => {
                self.mouse_tracking = false;
                self.last_mouse_move.take();
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::MouseTracking)) => {
                self.decqrm_response(mode, true, self.mouse_tracking);
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::HighlightMouseTracking,
            ))
            | Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::HighlightMouseTracking,
            )) => {}

            Mode::SetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::ButtonEventMouse)) => {
                self.button_event_mouse = true;
                self.last_mouse_move.take();
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::ButtonEventMouse,
            )) => {
                self.button_event_mouse = false;
                self.last_mouse_move.take();
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::ButtonEventMouse,
            )) => {
                self.decqrm_response(mode, true, self.button_event_mouse);
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::AnyEventMouse)) => {
                self.any_event_mouse = true;
                self.last_mouse_move.take();
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::AnyEventMouse)) => {
                self.any_event_mouse = false;
                self.last_mouse_move.take();
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::AnyEventMouse)) => {
                self.decqrm_response(mode, true, self.any_event_mouse);
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::FocusTracking)) => {
                self.focus_tracking = true;
                self.last_mouse_move.take();
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::FocusTracking)) => {
                self.focus_tracking = false;
                self.last_mouse_move.take();
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::FocusTracking)) => {
                self.decqrm_response(mode, true, self.focus_tracking);
            }

            mode => return Some(mode),
        }
        None
    }
}
