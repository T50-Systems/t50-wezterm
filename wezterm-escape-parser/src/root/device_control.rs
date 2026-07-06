/// A fully parsed DCS sequence.
/// The parser emits these for byte/intermediate sequences that are
/// known to be relatively short and self contained (eg: DECRQSS)
/// as opposed to larger ones like Sixel (which is parsed separately),
/// or long lived terminal modes such as the TMUX CC protocol.
#[derive(Clone, PartialEq, Eq)]
pub struct ShortDeviceControl {
    /// Integer parameter values
    pub params: Vec<i64>,
    /// Intermediate bytes to refine the control
    pub intermediates: Vec<u8>,
    /// The final byte
    pub byte: u8,
    /// The data prior to the string terminator
    pub data: Vec<u8>,
}

impl core::fmt::Debug for ShortDeviceControl {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(
            fmt,
            "ShortDeviceControl(params: {:?}, intermediates: [",
            &self.params
        )?;
        for b in &self.intermediates {
            write!(fmt, "{:?} 0x{:x}, ", *b as char, *b)?;
        }
        write!(
            fmt,
            "], byte: {:?} 0x{:x}, data=[",
            self.byte as char, self.byte
        )?;

        for b in &self.data {
            write!(fmt, "{:?} 0x{:x}, ", *b as char, *b)?;
        }

        write!(fmt, ")")
    }
}

impl Display for ShortDeviceControl {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "\x1bP")?;
        for (idx, p) in self.params.iter().enumerate() {
            if idx > 0 {
                write!(f, ";")?;
            }
            write!(f, "{}", p)?;
        }
        for b in &self.intermediates {
            f.write_char(*b as char)?;
        }
        f.write_char(self.byte as char)?;
        for b in &self.data {
            f.write_char(*b as char)?;
        }
        write!(f, "\x1b\\")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct EnterDeviceControlMode {
    /// The final byte in the DCS mode
    pub byte: u8,
    pub params: Vec<i64>,
    pub intermediates: Vec<u8>,
    /// if true, more than two intermediates arrived and the
    /// remaining data was ignored
    pub ignored_extra_intermediates: bool,
}

impl core::fmt::Debug for EnterDeviceControlMode {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(
            fmt,
            "EnterDeviceControlMode(params: {:?}, intermediates: [",
            &self.params
        )?;
        for b in &self.intermediates {
            write!(fmt, "{:?} 0x{:x}, ", *b as char, *b)?;
        }
        write!(
            fmt,
            "], byte: {:?} 0x{:x}, ignored_extra_intermediates={})",
            self.byte as char, self.byte, self.ignored_extra_intermediates
        )
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum DeviceControlMode {
    /// Identify device control mode from the encoded parameters.
    /// This mode is activated and must remain active until
    /// `Exit` is observed.  While the mode is
    /// active, data is made available to the device mode via
    /// the `Data` variant.
    Enter(Box<EnterDeviceControlMode>),
    /// Exit the current device control mode
    Exit,
    /// Data for the device mode to consume
    Data(u8),
    /// A self contained (Enter, Data*, Exit) sequence
    ShortDeviceControl(Box<ShortDeviceControl>),
    /// Tmux parsed events
    #[cfg(feature = "tmux_cc")]
    TmuxEvents(Box<Vec<Event>>),
}

impl Display for DeviceControlMode {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Enter(mode) => {
                write!(f, "\x1bP")?;
                for (idx, p) in mode.params.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ";")?;
                    }
                    write!(f, "{}", p)?;
                }
                for b in &mode.intermediates {
                    f.write_char(*b as char)?;
                }
                f.write_char(mode.byte as char)
            }
            // We don't need to emit a sequence for the Exit, as we're
            // followed by eg: StringTerminator
            Self::Exit => Ok(()),
            Self::Data(c) => f.write_char(*c as char),
            Self::ShortDeviceControl(s) => s.fmt(f),
            #[cfg(feature = "tmux_cc")]
            Self::TmuxEvents(_) => write!(f, "tmux event"),
        }
    }
}

impl core::fmt::Debug for DeviceControlMode {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match self {
            Self::Enter(mode) => write!(fmt, "Enter({:?})", mode),
            Self::Exit => write!(fmt, "Exit"),
            Self::Data(b) => write!(fmt, "Data({:?} 0x{:x})", *b as char, *b),
            Self::ShortDeviceControl(s) => write!(fmt, "ShortDeviceControl({:?})", s),
            #[cfg(feature = "tmux_cc")]
            Self::TmuxEvents(_) => write!(fmt, "tmux event"),
        }
    }
}
