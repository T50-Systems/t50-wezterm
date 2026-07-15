mod palette;

pub use palette::{actions_for_palette_and_menubar, recreate_menubar};
#[allow(unused_imports)]
pub use wezterm_gui_input::{
    ArgType, CommandDef, ExpandedCommand, derive_command_from_key_assignment,
};
