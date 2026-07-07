use super::*;
pub(super) fn default_one() -> usize {
    1
}

pub(super) fn default_ulimit_nofile() -> u64 {
    2048
}

pub(super) fn default_ulimit_nproc() -> u64 {
    2048
}
pub(super) fn default_check_for_updates() -> bool {
    cfg!(not(feature = "distro-defaults"))
}

pub(super) fn default_pane_select_fg_color() -> RgbaColor {
    SrgbaTuple(0.75, 0.75, 0.75, 1.0).into()
}

pub(super) fn default_pane_select_bg_color() -> RgbaColor {
    SrgbaTuple(0., 0., 0., 0.5).into()
}

pub(super) fn default_pane_select_font_size() -> f64 {
    36.0
}

pub(super) fn default_integrated_title_buttons() -> Vec<IntegratedTitleButton> {
    use IntegratedTitleButton::*;
    vec![Hide, Maximize, Close]
}

pub(super) fn default_char_select_font_size() -> f64 {
    18.0
}

pub(super) fn default_char_select_fg_color() -> RgbaColor {
    SrgbaTuple(0.75, 0.75, 0.75, 1.0).into()
}

pub(super) fn default_char_select_bg_color() -> RgbaColor {
    (0x33, 0x33, 0x33).into()
}

pub(super) fn default_command_palette_font_size() -> f64 {
    14.0
}

pub(super) fn default_command_palette_fg_color() -> RgbaColor {
    SrgbaTuple(0.75, 0.75, 0.75, 1.0).into()
}

pub(super) fn default_command_palette_bg_color() -> RgbaColor {
    (0x33, 0x33, 0x33).into()
}

pub(super) fn default_swallow_mouse_click_on_window_focus() -> bool {
    cfg!(target_os = "macos")
}

pub(super) fn default_mux_output_parser_coalesce_delay_ms() -> u64 {
    3
}

pub(super) fn default_mux_output_parser_buffer_size() -> usize {
    128 * 1024
}

pub(super) fn default_ratelimit_line_prefetches_per_second() -> u32 {
    50
}

pub(super) fn default_cursor_blink_rate() -> u64 {
    800
}

pub(super) fn default_text_blink_rate() -> u64 {
    500
}

pub(super) fn default_text_blink_rate_rapid() -> u64 {
    250
}

pub(super) fn default_swap_backspace_and_delete() -> bool {
    // cfg!(target_os = "macos")
    // See: https://github.com/wezterm/wezterm/issues/88
    false
}

pub(super) fn default_scrollback_lines() -> usize {
    3500
}

const MAX_SCROLLBACK_LINES: usize = 999_999_999;
pub(super) fn validate_scrollback_lines(value: &usize) -> Result<(), String> {
    if *value > MAX_SCROLLBACK_LINES {
        return Err(format!(
            "Illegal value {value} for scrollback_lines; it must be <= {MAX_SCROLLBACK_LINES}!"
        ));
    }
    Ok(())
}

pub(super) fn default_initial_rows() -> u16 {
    24
}

pub(super) fn default_initial_cols() -> u16 {
    80
}

pub fn default_hyperlink_rules() -> Vec<hyperlink::Rule> {
    vec![
        // First handle URLs wrapped with punctuation (i.e. brackets)
        // e.g. [http://foo] (http://foo) <http://foo>
        hyperlink::Rule::with_highlight(r"\((\w+://\S+)\)", "$1", 1).unwrap(),
        hyperlink::Rule::with_highlight(r"\[(\w+://\S+)\]", "$1", 1).unwrap(),
        hyperlink::Rule::with_highlight(r"<(\w+://\S+)>", "$1", 1).unwrap(),
        // Then handle URLs not wrapped in brackets that
        // 1) have a balanced ending parenthesis or
        hyperlink::Rule::new(hyperlink::CLOSING_PARENTHESIS_HYPERLINK_PATTERN, "$0").unwrap(),
        // 2) include terminating _, / or - characters, if any
        hyperlink::Rule::new(hyperlink::GENERIC_HYPERLINK_PATTERN, "$0").unwrap(),
        // implicit mailto link
        hyperlink::Rule::new(r"\b\w+@[\w-]+(\.[\w-]+)+\b", "mailto:$0").unwrap(),
    ]
}

pub(super) fn default_harfbuzz_features() -> Vec<String> {
    ["kern", "liga", "clig"]
        .iter()
        .map(|&s| s.to_string())
        .collect()
}

pub(super) fn default_term() -> String {
    "xterm-256color".into()
}

pub(super) fn default_font_size() -> f64 {
    12.0
}

pub(crate) fn compute_cache_dir() -> anyhow::Result<PathBuf> {
    if let Some(runtime) = dirs_next::cache_dir() {
        return Ok(runtime.join("wezterm"));
    }

    Ok(crate::HOME_DIR.join(".local/share/wezterm"))
}

pub(crate) fn compute_data_dir() -> anyhow::Result<PathBuf> {
    if let Some(runtime) = dirs_next::data_dir() {
        return Ok(runtime.join("wezterm"));
    }

    Ok(crate::HOME_DIR.join(".local/share/wezterm"))
}

pub(crate) fn compute_runtime_dir() -> anyhow::Result<PathBuf> {
    if let Some(runtime) = dirs_next::runtime_dir() {
        return Ok(runtime.join("wezterm"));
    }

    Ok(crate::HOME_DIR.join(".local/share/wezterm"))
}

pub fn pki_dir() -> anyhow::Result<PathBuf> {
    compute_runtime_dir().map(|d| d.join("pki"))
}

pub fn default_read_timeout() -> Duration {
    Duration::from_secs(60)
}

pub fn default_write_timeout() -> Duration {
    Duration::from_secs(60)
}

pub fn default_local_echo_threshold_ms() -> Option<u64> {
    Some(100)
}

pub(super) fn default_bypass_mouse_reporting_modifiers() -> Modifiers {
    Modifiers::SHIFT
}

pub(super) fn default_gui_startup_args() -> Vec<String> {
    vec!["start".to_string()]
}

// Coupled with term/src/config.rs:TerminalConfiguration::unicode_version
pub(super) fn default_unicode_version() -> u8 {
    9
}

pub(super) fn default_mux_env_remove() -> Vec<String> {
    vec![
        "SSH_AUTH_SOCK".to_string(),
        "SSH_CLIENT".to_string(),
        "SSH_CONNECTION".to_string(),
    ]
}

pub(super) fn default_anim_fps() -> u8 {
    10
}

pub(super) fn default_max_fps() -> u64 {
    60
}

pub(super) fn default_tiling_desktop_environments() -> Vec<String> {
    [
        "X11 LG3D",
        "X11 Qtile",
        "X11 awesome",
        "X11 bspwm",
        "X11 dwm",
        "X11 i3",
        "X11 xmonad",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

pub(super) fn default_stateless_process_list() -> Vec<String> {
    [
        "bash",
        "sh",
        "zsh",
        "fish",
        "tmux",
        "nu",
        "nu.exe",
        "cmd.exe",
        "pwsh.exe",
        "powershell.exe",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

pub(super) fn default_status_update_interval() -> u64 {
    1_000
}

pub(super) fn default_alternate_buffer_wheel_scroll_speed() -> u8 {
    3
}

pub(super) fn default_num_alphabet() -> String {
    // Note: vi motion keys are intentionally excluded from this alphabet
    "1234567890abcdefghilmnopqrstuvwxyz".to_string()
}

pub(super) fn default_alphabet() -> String {
    "asdfqwerzxcvjklmiuopghtybn".to_string()
}

pub(super) fn default_word_boundary() -> String {
    " \t\n{[}]()\"'`".to_string()
}

pub(super) fn default_enq_answerback() -> String {
    "".to_string()
}

pub(super) fn default_tab_max_width() -> usize {
    16
}

pub(super) fn default_update_interval() -> u64 {
    86400
}

pub(super) fn default_prefer_egl() -> bool {
    !cfg!(windows)
}

pub(super) fn default_clean_exits() -> Vec<u32> {
    vec![]
}

pub(super) fn default_inactive_pane_hsb() -> HsbTransform {
    HsbTransform {
        brightness: 0.8,
        saturation: 0.9,
        hue: 1.0,
    }
}
pub(super) const fn linear_ease() -> EasingFunction {
    EasingFunction::Linear
}

pub(super) const fn default_half_cell() -> Dimension {
    Dimension::Cells(0.5)
}

pub(super) const fn default_reverse_video_cursor_min_contrast() -> f32 {
    2.5
}
pub(super) fn default_glyph_cache_image_cache_size() -> usize {
    256
}

pub(super) fn default_shape_cache_size() -> usize {
    1024
}

pub(super) fn default_line_state_cache_size() -> usize {
    1024
}

pub(super) fn default_line_quad_cache_size() -> usize {
    1024
}

pub(super) fn default_line_to_ele_shape_cache_size() -> usize {
    1024
}

pub(super) fn validate_row_or_col(value: &u16) -> Result<(), String> {
    if *value < 1 {
        Err("initial_cols and initial_rows must be non-zero".to_string())
    } else {
        Ok(())
    }
}

pub(super) fn validate_line_height(value: &f64) -> Result<(), String> {
    if *value <= 0.0 {
        Err(format!(
            "Illegal value {value} for line_height; it must be positive and greater than zero!"
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn validate_domain_name(name: &str) -> Result<(), String> {
    if name == "local" {
        Err(format!(
            "\"{name}\" is a built-in domain and cannot be redefined"
        ))
    } else if name == "" {
        Err("the empty string is an invalid domain name".to_string())
    } else {
        Ok(())
    }
}

/// <https://github.com/wezterm/wezterm/pull/2435>
/// <https://github.com/wezterm/wezterm/issues/2771>
/// <https://github.com/wezterm/wezterm/issues/2630>
pub(super) fn default_macos_forward_mods() -> Modifiers {
    Modifiers::SHIFT
}

pub(super) fn default_colr_rasterizer() -> FontRasterizerSelection {
    FontRasterizerSelection::Harfbuzz
}
