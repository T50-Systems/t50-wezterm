#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Send a single printable character to the display
    Print(char),
    /// Send a string of printable characters to the display.
    PrintString(String),
    /// A C0 or C1 control code
    Control(ControlCode),
    /// Device control.  This is uncommon wrt. terminal emulation.
    DeviceControl(DeviceControlMode),
    /// A command that typically doesn't change the contents of the
    /// terminal, but rather influences how it displays or otherwise
    /// interacts with the rest of the system
    OperatingSystemCommand(Box<OperatingSystemCommand>),
    CSI(CSI),
    Esc(Esc),
    Sixel(Box<Sixel>),
    /// A list of termcap, terminfo names for which the application
    /// wants information
    XtGetTcap(Vec<String>),
    KittyImage(Box<KittyImage>),
}

impl Action {
    /// Append this `Action` to a `Vec<Action>`.
    /// If this `Action` is `Print` and the last element is `Print` or
    /// `PrintString` then the elements are combined into `PrintString`
    /// to reduce heap utilization.
    pub fn append_to(self, dest: &mut Vec<Self>) {
        if let Action::Print(c) = &self {
            match dest.last_mut() {
                Some(Action::PrintString(s)) => {
                    s.push(*c);
                    return;
                }
                Some(Action::Print(prior)) => {
                    let mut s = prior.to_string();
                    dest.pop();
                    s.push(*c);
                    dest.push(Action::PrintString(s));
                    return;
                }
                _ => {}
            }
        }
        dest.push(self);
    }
}

#[cfg(all(test, target_pointer_width = "64"))]
#[test]
fn action_size() {
    assert_eq!(core::mem::size_of::<Action>(), 32);
    assert_eq!(core::mem::size_of::<DeviceControlMode>(), 16);
    assert_eq!(core::mem::size_of::<ControlCode>(), 1);
    assert_eq!(core::mem::size_of::<CSI>(), 32);
    assert_eq!(core::mem::size_of::<Esc>(), 4);
}

/// Encode self as an escape sequence.  The escape sequence may potentially
/// be clear text with no actual escape sequences.
impl Display for Action {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Action::Print(c) => write!(f, "{}", c),
            Action::PrintString(s) => write!(f, "{}", s),
            Action::Control(c) => f.write_char(*c as u8 as char),
            Action::DeviceControl(c) => c.fmt(f),
            Action::OperatingSystemCommand(osc) => osc.fmt(f),
            Action::CSI(csi) => csi.fmt(f),
            Action::Esc(esc) => esc.fmt(f),
            Action::Sixel(sixel) => sixel.fmt(f),
            Action::XtGetTcap(names) => {
                write!(f, "\x1bP+q")?;
                for (i, name) in names.iter().enumerate() {
                    if i > 0 {
                        write!(f, ";")?;
                    }
                    for &b in name.as_bytes() {
                        write!(f, "{:x}", b)?;
                    }
                }

                Ok(())
            }
            Action::KittyImage(img) => img.fmt(f),
        }
    }
}
