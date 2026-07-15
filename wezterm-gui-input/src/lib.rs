mod overlay_keys;

pub mod commands;
pub mod inputmap;

pub use commands::{derive_command_from_key_assignment, ArgType, CommandDef, ExpandedCommand};
pub use inputmap::{human_key, ui_key, InputMap, OverlayKeyTables};

pub fn default_overlay_key_tables() -> OverlayKeyTables {
    OverlayKeyTables {
        copy_mode: overlay_keys::copy_key_table(),
        search_mode: overlay_keys::search_key_table(),
    }
}

#[cfg(test)]
mod tests;
