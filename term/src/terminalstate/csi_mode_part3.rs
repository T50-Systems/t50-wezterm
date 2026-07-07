use super::*;

impl TerminalState {
    pub(super) fn perform_csi_mode_part3(&mut self, mode: Mode) {
        match mode {
            Mode::SetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::SGRMouse)) => {
                self.mouse_encoding = MouseEncoding::SGR;
                self.last_mouse_move.take();
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::SGRMouse)) => {
                self.mouse_encoding = MouseEncoding::X10;
                self.last_mouse_move.take();
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::SGRMouse)) => {
                self.decqrm_response(
                    mode,
                    true,
                    match self.mouse_encoding {
                        MouseEncoding::SGR => true,
                        _ => false,
                    },
                );
            }
            Mode::SetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::SGRPixelsMouse)) => {
                self.mouse_encoding = MouseEncoding::SgrPixels;
                self.last_mouse_move.take();
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::SGRPixelsMouse)) => {
                self.mouse_encoding = MouseEncoding::X10;
                self.last_mouse_move.take();
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::SGRPixelsMouse)) => {
                self.decqrm_response(
                    mode,
                    true,
                    match self.mouse_encoding {
                        MouseEncoding::SgrPixels => true,
                        _ => false,
                    },
                );
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::Utf8Mouse)) => {
                self.mouse_encoding = MouseEncoding::Utf8;
                self.last_mouse_move.take();
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::Utf8Mouse)) => {
                self.mouse_encoding = MouseEncoding::X10;
                self.last_mouse_move.take();
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(DecPrivateModeCode::Utf8Mouse)) => {
                self.decqrm_response(
                    mode,
                    true,
                    match self.mouse_encoding {
                        MouseEncoding::Utf8 => true,
                        _ => false,
                    },
                );
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::SixelScrollsRight,
            )) => {
                self.sixel_scrolls_right = true;
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::SixelScrollsRight,
            )) => {
                self.sixel_scrolls_right = false;
            }
            Mode::QueryDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::SixelScrollsRight,
            )) => {
                self.decqrm_response(mode, true, self.sixel_scrolls_right);
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::ClearAndEnableAlternateScreen,
            )) => {
                if !self.screen.is_alt_screen_active() {
                    self.dec_save_cursor();
                    self.screen.activate_alt_screen(self.seqno);
                    self.set_cursor_pos(&Position::Absolute(0), &Position::Absolute(0));
                    self.pen = CellAttributes::default();
                    self.erase_in_display(EraseInDisplay::EraseDisplay);
                }
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::ClearAndEnableAlternateScreen,
            )) => {
                if self.screen.is_alt_screen_active() {
                    self.screen.activate_primary_screen(self.seqno);
                    self.dec_restore_cursor();
                }
            }
            Mode::SaveDecPrivateMode(DecPrivateMode::Code(n))
            | Mode::RestoreDecPrivateMode(DecPrivateMode::Code(n)) => {
                log::warn!("save/restore dec mode {:?} unimplemented", n)
            }

            Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::MinTTYApplicationEscapeKeyMode,
            ))
            | Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::MinTTYApplicationEscapeKeyMode,
            )) => {}

            Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::XTermMetaSendsEscape,
            ))
            | Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::XTermMetaSendsEscape,
            )) => {}

            Mode::SetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::XTermAltSendsEscape,
            ))
            | Mode::ResetDecPrivateMode(DecPrivateMode::Code(
                DecPrivateModeCode::XTermAltSendsEscape,
            )) => {}

            Mode::SetDecPrivateMode(DecPrivateMode::Unspecified(_))
            | Mode::ResetDecPrivateMode(DecPrivateMode::Unspecified(_))
            | Mode::SaveDecPrivateMode(DecPrivateMode::Unspecified(_))
            | Mode::RestoreDecPrivateMode(DecPrivateMode::Unspecified(_)) => {
                if self.config.log_unknown_escape_sequences() {
                    log::warn!("unhandled DecPrivateMode {:?}", mode);
                }
            }

            mode @ Mode::SetMode(_) | mode @ Mode::ResetMode(_) => {
                if self.config.log_unknown_escape_sequences() {
                    log::warn!("unhandled {:?}", mode);
                }
            }

            Mode::XtermKeyMode {
                resource: XtermKeyModifierResource::OtherKeys,
                value,
            } => {
                self.modify_other_keys = match value {
                    Some(0) => None,
                    _ => value,
                };
                log::debug!("XtermKeyMode OtherKeys -> {:?}", self.modify_other_keys);
            }

            Mode::XtermKeyMode { resource, value } => {
                if self.config.log_unknown_escape_sequences() {
                    log::warn!("unhandled XtermKeyMode {:?} {:?}", resource, value);
                }
            }

            Mode::QueryDecPrivateMode(_) | Mode::QueryMode(_) => {
                self.decqrm_response(mode, false, false);
            }
            mode => {
                if self.config.log_unknown_escape_sequences() {
                    log::warn!("unhandled {:?}", mode);
                }
            }
        }
    }
}
