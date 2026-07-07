impl<'a> CSIParser<'a> {
    /// Consume some number of elements from params and update it.
    /// Take care to avoid setting params back to an empty slice
    /// as this would trigger returning a default value and/or
    /// an unterminated parse loop.
    fn advance_by<T>(&mut self, n: usize, params: &'a [CsiParam], result: T) -> T {
        let n = if matches!(params.get(n), Some(CsiParam::P(b';'))) {
            n + 1
        } else {
            n
        };

        let (_, next) = params.split_at(n);
        if !next.is_empty() {
            self.params = Some(next);
        }
        result
    }

    fn focus(&self, params: &'a [CsiParam], from_start: usize, from_end: usize) -> &'a [CsiParam] {
        if params == self.orig_params {
            let len = params.len();
            &params[from_start..len - from_end]
        } else {
            params
        }
    }

    fn select_character_path(&mut self, params: &'a [CsiParam]) -> Result<CSI, ()> {
        fn path(n: i64) -> Result<CharacterPath, ()> {
            Ok(match n {
                0 => CharacterPath::ImplementationDefault,
                1 => CharacterPath::LeftToRightOrTopToBottom,
                2 => CharacterPath::RightToLeftOrBottomToTop,
                _ => return Err(()),
            })
        }

        match params {
            [CsiParam::P(b' ')] => Ok(self.advance_by(
                1,
                params,
                CSI::SelectCharacterPath(CharacterPath::ImplementationDefault, 0),
            )),
            [CsiParam::Integer(a), CsiParam::P(b' ')] => {
                Ok(self.advance_by(2, params, CSI::SelectCharacterPath(path(*a)?, 0)))
            }
            [
                CsiParam::Integer(a),
                CsiParam::P(b';'),
                CsiParam::Integer(b),
                CsiParam::P(b' '),
            ] => Ok(self.advance_by(4, params, CSI::SelectCharacterPath(path(*a)?, *b))),
            _ => Err(()),
        }
    }

    fn cursor_style(&mut self, params: &'a [CsiParam]) -> Result<CSI, ()> {
        match params {
            [CsiParam::Integer(p), CsiParam::P(b' ')] => match FromPrimitive::from_i64(*p) {
                None => Err(()),
                Some(style) => {
                    Ok(self.advance_by(2, params, CSI::Cursor(Cursor::CursorStyle(style))))
                }
            },
            _ => Err(()),
        }
    }

    fn checksum_area(&mut self, params: &'a [CsiParam]) -> Result<CSI, ()> {
        let params = Cracked::parse(&params[..params.len() - 1])?;

        let request_id = params.int(0)?;
        let page_number = params.int(1)?;
        let top = OneBased::from_optional_esc_param(params.get(2))?;
        let left = OneBased::from_optional_esc_param(params.get(3))?;
        let bottom = OneBased::from_optional_esc_param(params.get(4))?;
        let right = OneBased::from_optional_esc_param(params.get(5))?;
        Ok(CSI::Window(Box::new(Window::ChecksumRectangularArea {
            request_id,
            page_number,
            top,
            left,
            bottom,
            right,
        })))
    }

    fn dsr(&mut self, params: &'a [CsiParam]) -> Result<CSI, ()> {
        match params {
            [CsiParam::Integer(5)] => {
                Ok(self.advance_by(1, params, CSI::Device(Box::new(Device::StatusReport))))
            }

            [CsiParam::Integer(6)] => {
                Ok(self.advance_by(1, params, CSI::Cursor(Cursor::RequestActivePositionReport)))
            }
            _ => Err(()),
        }
    }

    fn decstbm(&mut self, params: &'a [CsiParam]) -> Result<CSI, ()> {
        match params {
            [] => Ok(CSI::Cursor(Cursor::SetTopAndBottomMargins {
                top: OneBased::new(1),
                bottom: OneBased::new(u32::max_value()),
            })),
            [p] => Ok(self.advance_by(
                1,
                params,
                CSI::Cursor(Cursor::SetTopAndBottomMargins {
                    top: OneBased::from_esc_param(p)?,
                    bottom: OneBased::new(u32::max_value()),
                }),
            )),
            [a, CsiParam::P(b';'), b] => Ok(self.advance_by(
                3,
                params,
                CSI::Cursor(Cursor::SetTopAndBottomMargins {
                    top: OneBased::from_esc_param(a)?,
                    bottom: OneBased::from_esc_param_with_big_default(b)?,
                }),
            )),
            [CsiParam::P(b';'), b] => Ok(self.advance_by(
                2,
                params,
                CSI::Cursor(Cursor::SetTopAndBottomMargins {
                    top: OneBased::new(1),
                    bottom: OneBased::from_esc_param_with_big_default(b)?,
                }),
            )),
            _ => Err(()),
        }
    }

    fn xterm_key_modifier(&mut self, params: &'a [CsiParam]) -> Result<CSI, ()> {
        match params {
            [CsiParam::P(b'>'), a, CsiParam::P(b';'), b] => {
                let resource = XtermKeyModifierResource::parse(a.as_integer().ok_or_else(|| ())?)
                    .ok_or_else(|| ())?;
                Ok(self.advance_by(
                    4,
                    params,
                    CSI::Mode(Mode::XtermKeyMode {
                        resource,
                        value: Some(b.as_integer().ok_or_else(|| ())?),
                    }),
                ))
            }
            [CsiParam::P(b'>'), a, CsiParam::P(b';')] => {
                let resource = XtermKeyModifierResource::parse(a.as_integer().ok_or_else(|| ())?)
                    .ok_or_else(|| ())?;
                Ok(self.advance_by(
                    3,
                    params,
                    CSI::Mode(Mode::XtermKeyMode {
                        resource,
                        value: None,
                    }),
                ))
            }
            [CsiParam::P(b'>'), p] => {
                let resource = XtermKeyModifierResource::parse(p.as_integer().ok_or_else(|| ())?)
                    .ok_or_else(|| ())?;
                Ok(self.advance_by(
                    2,
                    params,
                    CSI::Mode(Mode::XtermKeyMode {
                        resource,
                        value: None,
                    }),
                ))
            }
            _ => Err(()),
        }
    }

    fn decslrm(&mut self, params: &'a [CsiParam]) -> Result<CSI, ()> {
        match params {
            [] => {
                // with no params this is a request to save the cursor
                // and is technically in conflict with SetLeftAndRightMargins.
                // The emulator needs to decide based on DECSLRM mode
                // whether this saves the cursor or is SetLeftAndRightMargins
                // with default parameters!
                Ok(CSI::Cursor(Cursor::SaveCursor))
            }
            [p] => Ok(self.advance_by(
                1,
                params,
                CSI::Cursor(Cursor::SetLeftAndRightMargins {
                    left: OneBased::from_esc_param(p)?,
                    right: OneBased::new(u32::max_value()),
                }),
            )),
            [a, CsiParam::P(b';'), b] => Ok(self.advance_by(
                3,
                params,
                CSI::Cursor(Cursor::SetLeftAndRightMargins {
                    left: OneBased::from_esc_param(a)?,
                    right: OneBased::from_esc_param(b)?,
                }),
            )),
            [CsiParam::P(b';'), b] => Ok(self.advance_by(
                2,
                params,
                CSI::Cursor(Cursor::SetLeftAndRightMargins {
                    left: OneBased::new(1),
                    right: OneBased::from_esc_param(b)?,
                }),
            )),
            _ => Err(()),
        }
    }

    fn req_primary_device_attributes(&mut self, params: &'a [CsiParam]) -> Result<Device, ()> {
        match params {
            [] => Ok(Device::RequestPrimaryDeviceAttributes),
            [CsiParam::Integer(0)] => {
                Ok(self.advance_by(1, params, Device::RequestPrimaryDeviceAttributes))
            }
            _ => Err(()),
        }
    }

    fn req_terminal_name_and_version(&mut self, params: &'a [CsiParam]) -> Result<Device, ()> {
        match params {
            [_] => Ok(Device::RequestTerminalNameAndVersion),

            [_, CsiParam::Integer(0)] => {
                Ok(self.advance_by(2, params, Device::RequestTerminalNameAndVersion))
            }
            _ => Err(()),
        }
    }

    fn req_secondary_device_attributes(&mut self, params: &'a [CsiParam]) -> Result<Device, ()> {
        match params {
            [CsiParam::P(b'>')] => Ok(Device::RequestSecondaryDeviceAttributes),
            [CsiParam::P(b'>'), CsiParam::Integer(0)] => {
                Ok(self.advance_by(2, params, Device::RequestSecondaryDeviceAttributes))
            }
            _ => Err(()),
        }
    }

    fn req_tertiary_device_attributes(&mut self, params: &'a [CsiParam]) -> Result<Device, ()> {
        match params {
            [CsiParam::P(b'=')] => Ok(Device::RequestTertiaryDeviceAttributes),
            [CsiParam::P(b'='), CsiParam::Integer(0)] => {
                Ok(self.advance_by(2, params, Device::RequestTertiaryDeviceAttributes))
            }
            _ => Err(()),
        }
    }

    fn secondary_device_attributes(&mut self, params: &'a [CsiParam]) -> Result<Device, ()> {
        match params {
            [
                _,
                CsiParam::Integer(1),
                CsiParam::P(b';'),
                CsiParam::Integer(0),
            ] => Ok(self.advance_by(
                4,
                params,
                Device::DeviceAttributes(DeviceAttributes::Vt101WithNoOptions),
            )),
            [_, CsiParam::Integer(6)] => {
                Ok(self.advance_by(2, params, Device::DeviceAttributes(DeviceAttributes::Vt102)))
            }
            [
                _,
                CsiParam::Integer(1),
                CsiParam::P(b';'),
                CsiParam::Integer(2),
            ] => Ok(self.advance_by(
                4,
                params,
                Device::DeviceAttributes(DeviceAttributes::Vt100WithAdvancedVideoOption),
            )),
            [_, CsiParam::Integer(62), ..] => Ok(self.advance_by(
                params.len(),
                params,
                Device::DeviceAttributes(DeviceAttributes::Vt220(
                    DeviceAttributeFlags::from_params(&params[2..]),
                )),
            )),
            [_, CsiParam::Integer(63), ..] => Ok(self.advance_by(
                params.len(),
                params,
                Device::DeviceAttributes(DeviceAttributes::Vt320(
                    DeviceAttributeFlags::from_params(&params[2..]),
                )),
            )),
            [_, CsiParam::Integer(64), ..] => Ok(self.advance_by(
                params.len(),
                params,
                Device::DeviceAttributes(DeviceAttributes::Vt420(
                    DeviceAttributeFlags::from_params(&params[2..]),
                )),
            )),
            _ => Err(()),
        }
    }

    fn req_terminal_parameters(&mut self, params: &'a [CsiParam]) -> Result<Device, ()> {
        match params {
            [] | [CsiParam::Integer(0)] => Ok(Device::RequestTerminalParameters(0)),
            [CsiParam::Integer(1)] => Ok(Device::RequestTerminalParameters(1)),
            _ => Err(()),
        }
    }
}
