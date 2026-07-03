impl VTParser {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let param_indices = [0usize; MAX_OSC];

        Self {
            state: State::Ground,
            utf8_return_state: State::Ground,

            intermediates: [0, 0],
            num_intermediates: 0,
            ignored_excess_intermediates: false,

            osc: OscState {
                buffer: Vec::new(),
                param_indices,
                num_params: 0,
                full: false,
            },

            params: [CsiParam::default(); MAX_PARAMS],
            num_params: 0,
            params_full: false,
            current_param: None,

            utf8_parser: Utf8Parser::new(),
            #[cfg(any(feature = "std", feature = "alloc"))]
            apc_data: Vec::new(),
        }
    }

    /// Returns if the state machine is in the ground state,
    /// i.e. there is no pending state held by the state machine.
    pub fn is_ground(&self) -> bool {
        self.state == State::Ground
    }

    fn as_integer_params(&self) -> [i64; MAX_PARAMS] {
        let mut res = [0i64; MAX_PARAMS];
        let mut i = 0;
        for src in &self.params[0..self.num_params] {
            if let CsiParam::Integer(value) = src {
                res[i] = *value;
            } else if let CsiParam::P(b';') = src {
                i += 1;
            }
        }
        res
    }

    fn finish_param(&mut self) {
        if let Some(val) = self.current_param.take() {
            if self.num_params < MAX_PARAMS {
                self.params[self.num_params] = val;
                self.num_params += 1;
            }
        }
    }

    /// Promote early intermediates to parameters.
    /// This is handle sequences such as DECSET that use `?`
    /// prior to other numeric parameters.
    /// `?` is technically in the intermediate range and shouldn't
    /// appear in the parameter position according to ECMA 48
    fn promote_intermediates_to_params(&mut self) {
        if self.num_intermediates > 0 {
            for &p in &self.intermediates[..self.num_intermediates] {
                if self.num_params >= MAX_PARAMS {
                    self.ignored_excess_intermediates = true;
                    break;
                }
                self.params[self.num_params] = CsiParam::P(p);
                self.num_params += 1;
            }
            self.num_intermediates = 0;
        }
    }

    fn action(&mut self, action: Action, param: u8, actor: &mut dyn VTActor) {
        match action {
            Action::None | Action::Ignore => {}
            Action::Print => actor.print(param as char),
            Action::Execute => actor.execute_c0_or_c1(param),
            Action::Clear => {
                self.num_intermediates = 0;
                self.ignored_excess_intermediates = false;
                self.osc.num_params = 0;
                self.osc.full = false;
                self.num_params = 0;
                self.params_full = false;
                self.current_param.take();
                #[cfg(any(feature = "std", feature = "alloc"))]
                {
                    self.apc_data.clear();
                    self.apc_data.shrink_to_fit();
                    self.osc.buffer.clear();
                    self.osc.buffer.shrink_to_fit();
                }
            }
            Action::Collect => {
                if self.num_intermediates < MAX_INTERMEDIATES {
                    self.intermediates[self.num_intermediates] = param;
                    self.num_intermediates += 1;
                } else {
                    self.ignored_excess_intermediates = true;
                }
            }
            Action::Param => {
                if self.params_full {
                    return;
                }

                self.promote_intermediates_to_params();

                match param {
                    b'0'..=b'9' => match self.current_param.take() {
                        Some(CsiParam::Integer(i)) => {
                            self.current_param.replace(CsiParam::Integer(
                                i.saturating_mul(10).saturating_add((param - b'0') as i64),
                            ));
                        }
                        Some(_) => unreachable!(),
                        None => {
                            self.current_param
                                .replace(CsiParam::Integer((param - b'0') as i64));
                        }
                    },
                    p => {
                        self.finish_param();

                        if self.num_params + 1 > MAX_PARAMS {
                            self.params_full = true;
                        } else {
                            self.params[self.num_params] = CsiParam::P(p);
                            self.num_params += 1;
                        }
                    }
                }
            }
            Action::Hook => {
                self.finish_param();
                actor.dcs_hook(
                    param,
                    &self.as_integer_params()[0..self.num_params],
                    &self.intermediates[0..self.num_intermediates],
                    self.ignored_excess_intermediates,
                );
            }
            Action::Put => actor.dcs_put(param),
            Action::EscDispatch => {
                self.finish_param();
                actor.esc_dispatch(
                    &self.as_integer_params()[0..self.num_params],
                    &self.intermediates[0..self.num_intermediates],
                    self.ignored_excess_intermediates,
                    param,
                );
            }
            Action::CsiDispatch => {
                self.finish_param();
                self.promote_intermediates_to_params();
                actor.csi_dispatch(
                    &self.params[0..self.num_params],
                    self.ignored_excess_intermediates,
                    param,
                );
            }
            Action::Unhook => actor.dcs_unhook(),
            Action::OscStart => {
                self.osc.buffer.clear();
                #[cfg(any(feature = "std", feature = "alloc"))]
                self.osc.buffer.shrink_to_fit();
                self.osc.num_params = 0;
                self.osc.full = false;
            }
            Action::OscPut => self.osc.put(param as char),

            Action::OscEnd => {
                if self.osc.num_params == 0 {
                    actor.osc_dispatch(&[]);
                } else {
                    let mut params: [&[u8]; MAX_OSC] = [b""; MAX_OSC];
                    let mut offset = 0usize;
                    let mut slice = self.osc.buffer.as_slice();
                    let limit = self.osc.num_params.min(MAX_OSC);
                    #[allow(clippy::needless_range_loop)]
                    for i in 0..limit - 1 {
                        let (a, b) = slice.split_at(self.osc.param_indices[i] - offset);
                        params[i] = a;
                        slice = b;
                        offset = self.osc.param_indices[i];
                    }
                    params[limit - 1] = slice;
                    actor.osc_dispatch(&params[0..limit]);
                }
            }

            Action::ApcStart => {
                #[cfg(any(feature = "std", feature = "alloc"))]
                {
                    self.apc_data.clear();
                    self.apc_data.shrink_to_fit();
                }
            }
            Action::ApcPut => {
                #[cfg(any(feature = "std", feature = "alloc"))]
                self.apc_data.push(param);
            }
            Action::ApcEnd => {
                #[cfg(any(feature = "std", feature = "alloc"))]
                actor.apc_dispatch(core::mem::take(&mut self.apc_data));
            }

            Action::Utf8 => self.next_utf8(actor, param),
        }
    }

    // Process a utf-8 multi-byte sequence.
    // The state tables emit Action::Utf8 to initiate a multi-byte
    // sequence, and once we're in the utf-8 state we'll defer to
    // this method for each byte until the Decode struct is signalled
    // that we're done.
    // We use the REPLACEMENT_CHARACTER for invalid sequences.
    // We return to the ground state after each codepoint, successful
    // or otherwise.
    fn next_utf8(&mut self, actor: &mut dyn VTActor, byte: u8) {
        struct Decoder {
            codepoint: Option<char>,
        }

        impl utf8parse::Receiver for Decoder {
            fn codepoint(&mut self, c: char) {
                self.codepoint.replace(c);
            }

            fn invalid_sequence(&mut self) {
                self.codepoint(char::REPLACEMENT_CHARACTER);
            }
        }

        let mut decoder = Decoder { codepoint: None };

        self.utf8_parser.advance(&mut decoder, byte);
        if let Some(c) = decoder.codepoint {
            // Slightly gross special cases C1 controls that were
            // encoded as UTF-8 rather than emitted as raw 8-bit.
            // If the decoded value is in the byte range, and that
            // value would cause a state transition, then we process
            // that state transition rather than performing the default
            // string accumulation.
            if c as u32 <= 0xff {
                let byte = ((c as u32) & 0xff) as u8;

                let (action, state) = lookup(self.utf8_return_state, byte);
                if action == Action::Execute
                    || (state != self.utf8_return_state && state != State::Utf8Sequence)
                {
                    self.action(lookup_exit(self.utf8_return_state), 0, actor);
                    self.action(action, byte, actor);
                    self.action(lookup_entry(state), 0, actor);
                    self.utf8_return_state = self.state;
                    self.state = state;
                    return;
                }
            }

            match self.utf8_return_state {
                State::Ground => actor.print(c),
                State::OscString => self.osc.put(c),
                state => panic!("unreachable state {:?}", state),
            };
            self.state = self.utf8_return_state;
        }
    }

    /// Parse a single byte.  This may result in a call to one of the
    /// methods on the provided `actor`.
    #[inline(always)]
    pub fn parse_byte(&mut self, byte: u8, actor: &mut dyn VTActor) {
        // While in utf-8 parsing mode, co-opt the vt state
        // table and instead use the utf-8 state table from the
        // parser.  It will drop us back into the Ground state
        // after each recognized (or invalid) codepoint.
        if self.state == State::Utf8Sequence {
            self.next_utf8(actor, byte);
            return;
        }

        let (action, state) = lookup(self.state, byte);

        if state != self.state {
            if state != State::Utf8Sequence {
                self.action(lookup_exit(self.state), 0, actor);
            }
            self.action(action, byte, actor);
            self.action(lookup_entry(state), byte, actor);
            self.utf8_return_state = self.state;
            self.state = state;
        } else {
            self.action(action, byte, actor);
        }
    }

    /// Parse a sequence of bytes.  The sequence need not be complete.
    /// This may result in some number of calls to the methods on the
    /// provided `actor`.
    pub fn parse(&mut self, bytes: &[u8], actor: &mut dyn VTActor) {
        for b in bytes {
            self.parse_byte(*b, actor);
        }
    }
}
