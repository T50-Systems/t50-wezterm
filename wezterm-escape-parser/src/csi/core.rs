use super::OneBased;
use crate::color::{AnsiColor, ColorSpec, RgbColor, SrgbaTuple};
use bitflags::bitflags;
use core::convert::TryInto;
use core::fmt::{Display, Error as FmtError, Formatter};
use num_derive::*;
use num_traits::{FromPrimitive, ToPrimitive};
#[cfg(feature = "use_serde")]
use serde::{Deserialize, Serialize};
use wezterm_dynamic::{FromDynamic, ToDynamic};
use wezterm_input_types::Modifiers;

use crate::allocate::*;

pub use vtparse::CsiParam;

/// Specify whether you want to slowly or rapidly annoy your users
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromDynamic, ToDynamic)]
#[repr(u8)]
pub enum Blink {
    None = 0,
    Slow = 1,
    Rapid = 2,
}

/// Allow converting to boolean; true means some kind of
/// blink, false means none.  This is used in some
/// generic code to determine whether to enable blink.
impl Into<bool> for Blink {
    fn into(self) -> bool {
        self != Blink::None
    }
}

/// The `Intensity` of a cell describes its boldness.  Most terminals
/// implement `Intensity::Bold` by either using a bold font or by simply
/// using an alternative color.  Some terminals implement `Intensity::Half`
/// as a dimmer color variant.
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromDynamic, ToDynamic)]
#[repr(u8)]
pub enum Intensity {
    Normal = 0,
    Bold = 1,
    Half = 2,
}

impl Default for Intensity {
    fn default() -> Self {
        Self::Normal
    }
}

/// Specify just how underlined you want your `Cell` to be
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromDynamic, ToDynamic)]
#[repr(u8)]
pub enum Underline {
    /// The cell is not underlined
    None = 0,
    /// The cell is underlined with a single line
    Single = 1,
    /// The cell is underlined with two lines
    Double = 2,
    /// Curly underline
    Curly = 3,
    /// Dotted underline
    Dotted = 4,
    /// Dashed underline
    Dashed = 5,
}

impl Default for Underline {
    fn default() -> Self {
        Self::None
    }
}

/// Allow converting to boolean; true means some kind of
/// underline, false means none.  This is used in some
/// generic code to determine whether to enable underline.
impl Into<bool> for Underline {
    fn into(self) -> bool {
        self != Underline::None
    }
}

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromDynamic, ToDynamic)]
#[repr(u8)]
pub enum VerticalAlign {
    BaseLine = 0,
    SuperScript = 1,
    SubScript = 2,
}

bitflags! {
    #[cfg_attr(feature="use_serde", derive(Serialize, Deserialize))]
    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    pub struct MouseButtons: u8 {
        const NONE = 0;
        const LEFT = 1<<1;
        const RIGHT = 1<<2;
        const MIDDLE = 1<<3;
        const VERT_WHEEL = 1<<4;
        const HORZ_WHEEL = 1<<5;
        /// if set then the wheel movement was in the positive
        /// direction, else the negative direction
        const WHEEL_POSITIVE = 1<<6;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CSI {
    /// SGR: Set Graphics Rendition.
    /// These values affect how the character is rendered.
    Sgr(Sgr),

    /// CSI codes that relate to the cursor
    Cursor(Cursor),

    Edit(Edit),

    Mode(Mode),

    Device(Box<Device>),

    Mouse(MouseReport),

    Window(Box<Window>),

    Keyboard(Keyboard),

    /// ECMA-48 SCP
    SelectCharacterPath(CharacterPath, i64),

    /// Unknown or unspecified; should be rare and is rather
    /// large, so it is boxed and kept outside of the enum
    /// body to help reduce space usage in the common cases.
    Unspecified(Box<Unspecified>),
}

#[cfg(all(test, target_pointer_width = "64"))]
#[test]
fn csi_size() {
    assert_eq!(core::mem::size_of::<Sgr>(), 24);
    assert_eq!(core::mem::size_of::<Cursor>(), 12);
    assert_eq!(core::mem::size_of::<Edit>(), 8);
    assert_eq!(core::mem::size_of::<Mode>(), 24);
    assert_eq!(core::mem::size_of::<MouseReport>(), 8);
    assert_eq!(core::mem::size_of::<Window>(), 40);
    assert_eq!(core::mem::size_of::<Keyboard>(), 8);
    assert_eq!(core::mem::size_of::<CSI>(), 32);
}

pub use wezterm_input_types::KittyKeyboardFlags;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum KittyKeyboardMode {
    AssignAll = 1,
    SetSpecified = 2,
    ClearSpecified = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Keyboard {
    SetKittyState {
        flags: KittyKeyboardFlags,
        mode: KittyKeyboardMode,
    },
    PushKittyState {
        flags: KittyKeyboardFlags,
        mode: KittyKeyboardMode,
    },
    PopKittyState(u32),
    QueryKittySupport,
    ReportKittyState(KittyKeyboardFlags),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterPath {
    /// 0
    ImplementationDefault,
    /// 1
    LeftToRightOrTopToBottom,
    /// 2
    RightToLeftOrBottomToTop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unspecified {
    pub params: Vec<CsiParam>,
    /// if true, more than two intermediates arrived and the
    /// remaining data was ignored
    pub parameters_truncated: bool,
    /// The final character in the CSI sequence; this typically
    /// defines how to interpret the other parameters.
    pub control: char,
}

impl Display for Unspecified {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        for p in &self.params {
            write!(f, "{}", p)?;
        }
        write!(f, "{}", self.control)
    }
}

impl Display for CSI {
    // TODO: data size optimization opportunity: if we could somehow know that we
    // had a run of CSI instances being encoded in sequence, we could
    // potentially collapse them together.  This is a few bytes difference in
    // practice so it may not be worthwhile with modern networks.
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "\x1b[")?;
        match self {
            CSI::Sgr(sgr) => sgr.fmt(f)?,
            CSI::Cursor(c) => c.fmt(f)?,
            CSI::Edit(e) => e.fmt(f)?,
            CSI::Mode(mode) => mode.fmt(f)?,
            CSI::Unspecified(unspec) => unspec.fmt(f)?,
            CSI::Mouse(mouse) => mouse.fmt(f)?,
            CSI::Device(dev) => dev.fmt(f)?,
            CSI::Window(window) => window.fmt(f)?,
            CSI::Keyboard(Keyboard::SetKittyState { flags, mode }) => {
                write!(f, "={};{}u", flags.bits(), *mode as u16)?
            }
            CSI::Keyboard(Keyboard::PushKittyState { flags, mode }) => {
                write!(f, ">{};{}u", flags.bits(), *mode as u16)?
            }
            CSI::Keyboard(Keyboard::PopKittyState(n)) => write!(f, "<{}u", *n)?,
            CSI::Keyboard(Keyboard::QueryKittySupport) => write!(f, "?u")?,
            CSI::Keyboard(Keyboard::ReportKittyState(flags)) => write!(f, "?{}u", flags.bits())?,
            CSI::SelectCharacterPath(path, n) => {
                let a = match path {
                    CharacterPath::ImplementationDefault => 0,
                    CharacterPath::LeftToRightOrTopToBottom => 1,
                    CharacterPath::RightToLeftOrBottomToTop => 2,
                };
                match (a, n) {
                    (0, 0) => write!(f, " k")?,
                    (a, 0) => write!(f, "{} k", a)?,
                    (a, n) => write!(f, "{};{} k", a, n)?,
                }
            }
        };
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum CursorStyle {
    Default = 0,
    BlinkingBlock = 1,
    SteadyBlock = 2,
    BlinkingUnderline = 3,
    SteadyUnderline = 4,
    BlinkingBar = 5,
    SteadyBar = 6,
}

impl Default for CursorStyle {
    fn default() -> CursorStyle {
        CursorStyle::Default
    }
}

#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum DeviceAttributeCodes {
    Columns132 = 1,
    Printer = 2,
    RegisGraphics = 3,
    SixelGraphics = 4,
    SelectiveErase = 6,
    UserDefinedKeys = 8,
    NationalReplacementCharsets = 9,
    TechnicalCharacters = 15,
    UserWindows = 18,
    HorizontalScrolling = 21,
    AnsiColor = 22,
    AnsiTextLocator = 29,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceAttribute {
    Code(DeviceAttributeCodes),
    Unspecified(CsiParam),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceAttributeFlags {
    pub attributes: Vec<DeviceAttribute>,
}

impl DeviceAttributeFlags {
    fn emit(&self, f: &mut Formatter, leader: &str) -> Result<(), FmtError> {
        write!(f, "{}", leader)?;
        for item in &self.attributes {
            match item {
                DeviceAttribute::Code(c) => write!(f, ";{}", c.to_u16().ok_or_else(|| FmtError)?)?,
                DeviceAttribute::Unspecified(param) => write!(f, ";{}", param)?,
            }
        }
        write!(f, "c")?;
        Ok(())
    }

    pub fn new(attributes: Vec<DeviceAttribute>) -> Self {
        Self { attributes }
    }

    fn from_params(params: &[CsiParam]) -> Self {
        let mut attributes = Vec::new();
        for i in params {
            match i {
                CsiParam::Integer(p) => match FromPrimitive::from_i64(*p) {
                    Some(c) => attributes.push(DeviceAttribute::Code(c)),
                    None => attributes.push(DeviceAttribute::Unspecified(i.clone())),
                },
                CsiParam::P(b';') => {}
                _ => attributes.push(DeviceAttribute::Unspecified(i.clone())),
            }
        }
        Self { attributes }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceAttributes {
    Vt100WithAdvancedVideoOption,
    Vt101WithNoOptions,
    Vt102,
    Vt220(DeviceAttributeFlags),
    Vt320(DeviceAttributeFlags),
    Vt420(DeviceAttributeFlags),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XtSmGraphicsItem {
    NumberOfColorRegisters,
    SixelGraphicsGeometry,
    RegisGraphicsGeometry,
    Unspecified(i64),
}

impl Display for XtSmGraphicsItem {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Self::NumberOfColorRegisters => write!(f, "1"),
            Self::SixelGraphicsGeometry => write!(f, "2"),
            Self::RegisGraphicsGeometry => write!(f, "3"),
            Self::Unspecified(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XtSmGraphicsAction {
    ReadAttribute,
    ResetToDefault,
    SetToValue,
    ReadMaximumAllowedValue,
}

impl XtSmGraphicsAction {
    pub fn to_i64(&self) -> i64 {
        match self {
            Self::ReadAttribute => 1,
            Self::ResetToDefault => 2,
            Self::SetToValue => 3,
            Self::ReadMaximumAllowedValue => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XtSmGraphicsStatus {
    Success,
    InvalidItem,
    InvalidAction,
    Failure,
}

impl XtSmGraphicsStatus {
    pub fn to_i64(&self) -> i64 {
        match self {
            Self::Success => 0,
            Self::InvalidItem => 1,
            Self::InvalidAction => 2,
            Self::Failure => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XtSmGraphics {
    pub item: XtSmGraphicsItem,
    pub action_or_status: i64,
    pub value: Vec<i64>,
}

impl XtSmGraphics {
    pub fn action(&self) -> Option<XtSmGraphicsAction> {
        match self.action_or_status {
            1 => Some(XtSmGraphicsAction::ReadAttribute),
            2 => Some(XtSmGraphicsAction::ResetToDefault),
            3 => Some(XtSmGraphicsAction::SetToValue),
            4 => Some(XtSmGraphicsAction::ReadMaximumAllowedValue),
            _ => None,
        }
    }

    pub fn status(&self) -> Option<XtSmGraphicsStatus> {
        match self.action_or_status {
            0 => Some(XtSmGraphicsStatus::Success),
            1 => Some(XtSmGraphicsStatus::InvalidItem),
            2 => Some(XtSmGraphicsStatus::InvalidAction),
            3 => Some(XtSmGraphicsStatus::Failure),
            _ => None,
        }
    }

    pub fn parse(params: &[CsiParam]) -> Result<CSI, ()> {
        let params = Cracked::parse(&params[1..])?;
        Ok(CSI::Device(Box::new(Device::XtSmGraphics(XtSmGraphics {
            item: match params.get(0).ok_or(())? {
                CsiParam::Integer(1) => XtSmGraphicsItem::NumberOfColorRegisters,
                CsiParam::Integer(2) => XtSmGraphicsItem::SixelGraphicsGeometry,
                CsiParam::Integer(3) => XtSmGraphicsItem::RegisGraphicsGeometry,
                CsiParam::Integer(n) => XtSmGraphicsItem::Unspecified(*n),
                _ => return Err(()),
            },
            action_or_status: match params.get(1).ok_or(())? {
                CsiParam::Integer(n) => *n,
                _ => return Err(()),
            },
            value: params.params[2..]
                .iter()
                .filter_map(|p| match p {
                    Some(CsiParam::Integer(n)) => Some(*n),
                    _ => None,
                })
                .collect(),
        }))))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Device {
    DeviceAttributes(DeviceAttributes),
    /// DECSTR - https://vt100.net/docs/vt510-rm/DECSTR.html
    SoftReset,
    RequestPrimaryDeviceAttributes,
    RequestSecondaryDeviceAttributes,
    RequestTertiaryDeviceAttributes,
    StatusReport,
    /// https://github.com/mintty/mintty/issues/881
    /// https://gitlab.gnome.org/GNOME/vte/-/issues/235
    RequestTerminalNameAndVersion,
    RequestTerminalParameters(i64),
    XtSmGraphics(XtSmGraphics),
}

impl Display for Device {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Device::DeviceAttributes(DeviceAttributes::Vt100WithAdvancedVideoOption) => {
                write!(f, "?1;2c")?
            }
            Device::DeviceAttributes(DeviceAttributes::Vt101WithNoOptions) => write!(f, "?1;0c")?,
            Device::DeviceAttributes(DeviceAttributes::Vt102) => write!(f, "?6c")?,
            Device::DeviceAttributes(DeviceAttributes::Vt220(attr)) => attr.emit(f, "?62")?,
            Device::DeviceAttributes(DeviceAttributes::Vt320(attr)) => attr.emit(f, "?63")?,
            Device::DeviceAttributes(DeviceAttributes::Vt420(attr)) => attr.emit(f, "?64")?,
            Device::SoftReset => write!(f, "!p")?,
            Device::RequestPrimaryDeviceAttributes => write!(f, "c")?,
            Device::RequestSecondaryDeviceAttributes => write!(f, ">c")?,
            Device::RequestTertiaryDeviceAttributes => write!(f, "=c")?,
            Device::RequestTerminalNameAndVersion => write!(f, ">q")?,
            Device::RequestTerminalParameters(n) => write!(f, "{};1;1;128;128;1;0x", n + 2)?,
            Device::StatusReport => write!(f, "5n")?,
            Device::XtSmGraphics(g) => {
                write!(f, "?{};{}", g.item, g.action_or_status)?;
                for v in &g.value {
                    write!(f, ";{}", v)?;
                }
                write!(f, "S")?;
            }
        };
        Ok(())
    }
}
