/// Describes the semantic "type" of the cell.
/// This categorizes cells into Output (from the actions the user is
/// taking; this is the default if left unspecified),
/// Input (that the user typed) and Prompt (effectively, "chrome" provided
/// by the shell or application that the user is interacting with.
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromDynamic, ToDynamic)]
#[repr(u8)]
pub enum SemanticType {
    Output = 0,
    Input = 1,
    Prompt = 2,
}

impl Default for SemanticType {
    fn default() -> Self {
        Self::Output
    }
}

pub use wezterm_escape_parser::csi::{Blink, Intensity, Underline, VerticalAlign};
