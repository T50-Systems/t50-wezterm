impl<'a> CSIParser<'a> {
    /// Parse extended mouse reports known as SGR 1006 mode
    fn mouse_sgr1006(&mut self, params: &'a [CsiParam]) -> Result<MouseReport, ()> {
        let (p0, p1, p2) = match params {
            [
                CsiParam::P(b'<'),
                CsiParam::Integer(p0),
                CsiParam::P(b';'),
                CsiParam::Integer(p1),
                CsiParam::P(b';'),
                CsiParam::Integer(p2),
            ] => (*p0, *p1, *p2),
            _ => return Err(()),
        };

        // 'M' encodes a press, 'm' a release.
        let button = match (self.control, p0 & 0b110_0011) {
            ('M', 0) => MouseButton::Button1Press,
            ('m', 0) => MouseButton::Button1Release,
            ('M', 1) => MouseButton::Button2Press,
            ('m', 1) => MouseButton::Button2Release,
            ('M', 2) => MouseButton::Button3Press,
            ('m', 2) => MouseButton::Button3Release,
            ('M', 64) => MouseButton::Button4Press,
            ('m', 64) => MouseButton::Button4Release,
            ('M', 65) => MouseButton::Button5Press,
            ('m', 65) => MouseButton::Button5Release,
            ('M', 66) => MouseButton::Button6Press,
            ('m', 66) => MouseButton::Button6Release,
            ('M', 67) => MouseButton::Button7Press,
            ('m', 67) => MouseButton::Button7Release,
            ('M', 32) => MouseButton::Button1Drag,
            ('M', 33) => MouseButton::Button2Drag,
            ('M', 34) => MouseButton::Button3Drag,
            // Note that there is some theoretical ambiguity with these None values.
            // The ambiguity stems from alternative encodings of the mouse protocol;
            // when set to SGR1006 mode the variants with the `3` parameter do not
            // occur.  They included here as a reminder for when support for those
            // other encodings is added and this block is likely copied and pasted
            // or refactored for re-use with them.
            ('M', 35) => MouseButton::None, // mouse motion with no buttons
            ('m', 35) => MouseButton::None, // mouse motion with no buttons (in Windows Terminal)
            ('M', 3) => MouseButton::None,  // legacy notification about button release
            ('m', 3) => MouseButton::None,  // release+press doesn't make sense
            _ => {
                return Err(());
            }
        };

        let mut modifiers = Modifiers::NONE;
        if p0 & 4 != 0 {
            modifiers |= Modifiers::SHIFT;
        }
        if p0 & 8 != 0 {
            modifiers |= Modifiers::ALT;
        }
        if p0 & 16 != 0 {
            modifiers |= Modifiers::CTRL;
        }

        Ok(self.advance_by(
            6,
            params,
            MouseReport::SGR1006 {
                x: p1 as u16,
                y: p2 as u16,
                button,
                modifiers,
            },
        ))
    }

    fn decrqm(&mut self, params: &'a [CsiParam]) -> Result<CSI, ()> {
        Ok(CSI::Mode(match params {
            [CsiParam::Integer(p), CsiParam::P(b'$')] => {
                Mode::QueryMode(match FromPrimitive::from_i64(*p) {
                    None => TerminalMode::Unspecified(p.to_u16().ok_or(())?),
                    Some(mode) => TerminalMode::Code(mode),
                })
            }
            [CsiParam::P(b'?'), CsiParam::Integer(p), CsiParam::P(b'$')] => {
                Mode::QueryDecPrivateMode(match FromPrimitive::from_i64(*p) {
                    None => DecPrivateMode::Unspecified(p.to_u16().ok_or(())?),
                    Some(mode) => DecPrivateMode::Code(mode),
                })
            }
            _ => return Err(()),
        }))
    }

    fn dec(&mut self, params: &'a [CsiParam]) -> Result<DecPrivateMode, ()> {
        match params {
            [CsiParam::Integer(p0), ..] => match FromPrimitive::from_i64(*p0) {
                None => Ok(self.advance_by(
                    1,
                    params,
                    DecPrivateMode::Unspecified(p0.to_u16().ok_or(())?),
                )),
                Some(mode) => Ok(self.advance_by(1, params, DecPrivateMode::Code(mode))),
            },
            _ => Err(()),
        }
    }

    fn terminal_mode(&mut self, params: &'a [CsiParam]) -> Result<TerminalMode, ()> {
        let p0 = params
            .get(0)
            .and_then(CsiParam::as_integer)
            .ok_or_else(|| ())?;
        match FromPrimitive::from_i64(p0) {
            None => {
                Ok(self.advance_by(1, params, TerminalMode::Unspecified(p0.to_u16().ok_or(())?)))
            }
            Some(mode) => Ok(self.advance_by(1, params, TerminalMode::Code(mode))),
        }
    }

    fn parse_sgr_color(&mut self, params: &'a [CsiParam]) -> Result<ColorSpec, ()> {
        match params {
            // wezterm extension to support an optional alpha channel in the `:` form only
            [
                _,
                CsiParam::P(b':'),
                CsiParam::Integer(6),
                CsiParam::P(b':'),
                CsiParam::Integer(_colorspace),
                CsiParam::P(b':'),
                red,
                CsiParam::P(b':'),
                green,
                CsiParam::P(b':'),
                blue,
                CsiParam::P(b':'),
                alpha,
                ..,
            ] => {
                let res: SrgbaTuple =
                    (to_u8(red)?, to_u8(green)?, to_u8(blue)?, to_u8(alpha)?).into();
                Ok(self.advance_by(13, params, res.into()))
            }
            [
                _,
                CsiParam::P(b':'),
                CsiParam::Integer(6),
                CsiParam::P(b':'),
                /* empty colorspace */ CsiParam::P(b':'),
                red,
                CsiParam::P(b':'),
                green,
                CsiParam::P(b':'),
                blue,
                CsiParam::P(b':'),
                alpha,
                ..,
            ] => {
                let res: SrgbaTuple =
                    (to_u8(red)?, to_u8(green)?, to_u8(blue)?, to_u8(alpha)?).into();
                Ok(self.advance_by(12, params, res.into()))
            }
            [
                _,
                CsiParam::P(b':'),
                CsiParam::Integer(6),
                CsiParam::P(b':'),
                red,
                CsiParam::P(b':'),
                green,
                CsiParam::P(b':'),
                blue,
                CsiParam::P(b':'),
                alpha,
                ..,
            ] => {
                let res: SrgbaTuple =
                    (to_u8(red)?, to_u8(green)?, to_u8(blue)?, to_u8(alpha)?).into();
                Ok(self.advance_by(11, params, res.into()))
            }

            // standard sgr colors
            [
                _,
                CsiParam::P(b':'),
                CsiParam::Integer(2),
                CsiParam::P(b':'),
                CsiParam::Integer(_colorspace),
                CsiParam::P(b':'),
                red,
                CsiParam::P(b':'),
                green,
                CsiParam::P(b':'),
                blue,
                ..,
            ] => {
                let res = RgbColor::new_8bpc(to_u8(red)?, to_u8(green)?, to_u8(blue)?).into();
                Ok(self.advance_by(11, params, res))
            }

            [
                _,
                CsiParam::P(b':'),
                CsiParam::Integer(2),
                CsiParam::P(b':'),
                /* empty colorspace */ CsiParam::P(b':'),
                red,
                CsiParam::P(b':'),
                green,
                CsiParam::P(b':'),
                blue,
                ..,
            ] => {
                let res = RgbColor::new_8bpc(to_u8(red)?, to_u8(green)?, to_u8(blue)?).into();
                Ok(self.advance_by(10, params, res))
            }

            [
                _,
                CsiParam::P(b';'),
                CsiParam::Integer(2),
                CsiParam::P(b';'),
                red,
                CsiParam::P(b';'),
                green,
                CsiParam::P(b';'),
                blue,
                ..,
            ]
            | [
                _,
                CsiParam::P(b':'),
                CsiParam::Integer(2),
                CsiParam::P(b':'),
                red,
                CsiParam::P(b':'),
                green,
                CsiParam::P(b':'),
                blue,
                ..,
            ] => {
                let res = RgbColor::new_8bpc(to_u8(red)?, to_u8(green)?, to_u8(blue)?).into();
                Ok(self.advance_by(9, params, res))
            }

            [
                _,
                CsiParam::P(b';'),
                CsiParam::Integer(5),
                CsiParam::P(b';'),
                idx,
                ..,
            ]
            | [
                _,
                CsiParam::P(b':'),
                CsiParam::Integer(5),
                CsiParam::P(b':'),
                idx,
                ..,
            ] => Ok(self.advance_by(5, params, ColorSpec::PaletteIndex(to_u8(idx)?))),
            _ => Err(()),
        }
    }

    fn window(&mut self, params: &'a [CsiParam]) -> Result<Window, ()> {
        let params = Cracked::parse(params)?;

        let p = params.int(0)?;
        let arg1 = params.opt_int(1);
        let arg2 = params.opt_int(2);

        match p {
            1 => Ok(Window::DeIconify),
            2 => Ok(Window::Iconify),
            3 => Ok(Window::MoveWindow {
                x: arg1.unwrap_or(0),
                y: arg2.unwrap_or(0),
            }),
            4 => Ok(Window::ResizeWindowPixels {
                height: arg1,
                width: arg2,
            }),
            5 => Ok(Window::RaiseWindow),
            6 => match params.len() {
                1 => Ok(Window::LowerWindow),
                _ => Ok(Window::ReportCellSizePixelsResponse {
                    height: arg1,
                    width: arg2,
                }),
            },
            7 => Ok(Window::RefreshWindow),
            8 => Ok(Window::ResizeWindowCells {
                height: arg1,
                width: arg2,
            }),
            9 => match arg1 {
                Some(0) => Ok(Window::RestoreMaximizedWindow),
                Some(1) => Ok(Window::MaximizeWindow),
                Some(2) => Ok(Window::MaximizeWindowVertically),
                Some(3) => Ok(Window::MaximizeWindowHorizontally),
                _ => Err(()),
            },
            10 => match arg1 {
                Some(0) => Ok(Window::UndoFullScreenMode),
                Some(1) => Ok(Window::ChangeToFullScreenMode),
                Some(2) => Ok(Window::ToggleFullScreen),
                _ => Err(()),
            },
            11 => Ok(Window::ReportWindowState),
            13 => match arg1 {
                None => Ok(Window::ReportWindowPosition),
                Some(2) => Ok(Window::ReportTextAreaPosition),
                _ => Err(()),
            },
            14 => match arg1 {
                None => Ok(Window::ReportTextAreaSizePixels),
                Some(2) => Ok(Window::ReportWindowSizePixels),
                _ => Err(()),
            },
            15 => Ok(Window::ReportScreenSizePixels),
            16 => Ok(Window::ReportCellSizePixels),
            18 => Ok(Window::ReportTextAreaSizeCells),
            19 => Ok(Window::ReportScreenSizeCells),
            20 => Ok(Window::ReportIconLabel),
            21 => Ok(Window::ReportWindowTitle),
            22 => match arg1 {
                Some(0) => Ok(Window::PushIconAndWindowTitle),
                Some(1) => Ok(Window::PushIconTitle),
                Some(2) => Ok(Window::PushWindowTitle),
                _ => Err(()),
            },
            23 => match arg1 {
                Some(0) => Ok(Window::PopIconAndWindowTitle),
                Some(1) => Ok(Window::PopIconTitle),
                Some(2) => Ok(Window::PopWindowTitle),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }

    fn underline(&mut self, params: &'a [CsiParam]) -> Result<Sgr, ()> {
        let (sgr, n) = match params {
            [_, CsiParam::P(b':'), CsiParam::Integer(0), ..] => {
                (Sgr::Underline(Underline::None), 3)
            }
            [_, CsiParam::P(b':'), CsiParam::Integer(1), ..] => {
                (Sgr::Underline(Underline::Single), 3)
            }
            [_, CsiParam::P(b':'), CsiParam::Integer(2), ..] => {
                (Sgr::Underline(Underline::Double), 3)
            }
            [_, CsiParam::P(b':'), CsiParam::Integer(3), ..] => {
                (Sgr::Underline(Underline::Curly), 3)
            }
            [_, CsiParam::P(b':'), CsiParam::Integer(4), ..] => {
                (Sgr::Underline(Underline::Dotted), 3)
            }
            [_, CsiParam::P(b':'), CsiParam::Integer(5), ..] => {
                (Sgr::Underline(Underline::Dashed), 3)
            }
            _ => (Sgr::Underline(Underline::Single), 1),
        };

        Ok(self.advance_by(n, params, sgr))
    }
}
