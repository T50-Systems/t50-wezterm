#[allow(unused_imports)]
pub use wezterm_gui_input::{human_key, ui_key, InputMap, OverlayKeyTables};

pub fn overlay_key_tables() -> OverlayKeyTables {
    wezterm_gui_input::default_overlay_key_tables()
}

pub fn new_input_map(config: &config::ConfigHandle) -> InputMap {
    let overlay_tables = overlay_key_tables();
    InputMap::new(config, &overlay_tables)
}

pub fn default_input_map() -> InputMap {
    let overlay_tables = overlay_key_tables();
    InputMap::default_input_map(&overlay_tables)
}
