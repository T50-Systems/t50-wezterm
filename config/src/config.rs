use crate::background::{BackgroundLayer, Gradient};
use crate::bell::{AudibleBell, EasingFunction, VisualBell};
use crate::color::{ColorSchemeFile, HsbTransform, Palette, SrgbaTuple, TabBarStyle, WindowFrameConfig};
use crate::daemon::DaemonOptions;
use crate::exec_domain::ExecDomain;
use crate::font::{AllowSquareGlyphOverflow, DisplayPixelGeometry, FontLocatorSelection, FontRasterizerSelection, FontShaperSelection, FreeTypeLoadFlags, FreeTypeLoadTarget, StyleRule, TextStyle};
use crate::frontend::FrontEndSelection;
use crate::keyassignment::{KeyAssignment, KeyTable, KeyTableEntry, KeyTables, MouseEventTrigger, SpawnCommand};
use crate::keys::{Key, LeaderKey, Mouse};
use crate::lua::make_lua_context;
use crate::ssh::{SshBackend, SshDomain};
use crate::tls::{TlsDomainClient, TlsDomainServer};
use crate::units::Dimension;
use crate::unix::UnixDomain;
use crate::wsl::WslDomain;
use crate::{default_config_with_overrides_applied, default_one_point_oh, default_one_point_oh_f64, default_true, default_win32_acrylic_accent_color, CellWidth, GpuInfo, IntegratedTitleButtonColor, KeyMapPreference, LoadedConfig, MouseEventTriggerMods, RgbaColor, SerialDomain, SystemBackdrop, WebGpuPowerPreference, CONFIG_DIRS, CONFIG_FILE_OVERRIDE, CONFIG_OVERRIDES, CONFIG_SKIP, HOME_DIR};
use anyhow::Context;
use luahelper::impl_lua_conversion_dynamic;
use mlua::FromLua;
use portable_pty::CommandBuilder;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Duration;
use termwiz::hyperlink;
use termwiz::surface::CursorShape;
use wezterm_bidi::ParagraphDirectionHint;
use wezterm_config_derive::ConfigMeta;
use wezterm_dynamic::{FromDynamic, ToDynamic};
use wezterm_input_types::{IntegratedTitleButton, IntegratedTitleButtonAlignment, IntegratedTitleButtonStyle, Modifiers, UIKeyCapRendering, WindowDecorations};
use wezterm_term::TerminalSize;
mod defaults;
mod impl_load;
mod impl_runtime;
mod types;
pub use defaults::*;
pub use types::*;
#[allow(unused_imports)] use defaults::*;
#[derive(Debug, Clone, FromDynamic, ToDynamic, ConfigMeta)] pub struct Config {
    /// The font size, measured in points
    #[dynamic(default = "default_font_size")] pub font_size: f64,
    #[dynamic(default = "default_one_point_oh_f64", validate = "validate_line_height")] pub line_height: f64,
    #[dynamic(default = "default_one_point_oh_f64")] pub cell_width: f64,
    #[dynamic(try_from = "crate::units::OptPixelUnit", default)] pub cursor_thickness: Option<Dimension>,
    #[dynamic(try_from = "crate::units::OptPixelUnit", default)] pub underline_thickness: Option<Dimension>,
    #[dynamic(try_from = "crate::units::OptPixelUnit", default)] pub underline_position: Option<Dimension>,
    #[dynamic(try_from = "crate::units::OptPixelUnit", default)] pub strikethrough_position: Option<Dimension>,
    #[dynamic(default)] pub allow_square_glyphs_to_overflow_width: AllowSquareGlyphOverflow,
    #[dynamic(default)] pub window_decorations: WindowDecorations,
    #[dynamic(default = "default_integrated_title_buttons")] pub integrated_title_buttons: Vec<IntegratedTitleButton>,
    #[dynamic(default)] pub log_unknown_escape_sequences: bool,
    #[dynamic(default)] pub integrated_title_button_alignment: IntegratedTitleButtonAlignment,
    #[dynamic(default)] pub integrated_title_button_style: IntegratedTitleButtonStyle,
    #[dynamic(default)] pub integrated_title_button_color: IntegratedTitleButtonColor,
    /// When using FontKitXXX font systems, a set of directories to
    /// search ahead of the standard font locations for fonts.
    /// Relative paths are taken to be relative to the directory
    /// from which the config was loaded.
    #[dynamic(default)] pub font_dirs: Vec<PathBuf>,
    #[dynamic(default)] pub color_scheme_dirs: Vec<PathBuf>,
    /// The DPI to assume
    pub dpi: Option<f64>,
    #[dynamic(default)] pub dpi_by_screen: HashMap<String, f64>,
    /// The baseline font to use
    #[dynamic(default)] pub font: TextStyle,
    /// An optional set of style rules to select the font based
    /// on the cell attributes
    #[dynamic(default)] pub font_rules: Vec<StyleRule>,
    /// When true (the default), PaletteIndex 0-7 are shifted to
    /// bright when the font intensity is bold.  The brightening
    /// doesn't apply to text that is the default color.
    #[dynamic(default)] pub bold_brightens_ansi_colors: BoldBrightening,
    /// The color palette
    pub colors: Option<Palette>,
    #[dynamic(default)] pub switch_to_last_active_tab_when_closing_tab: bool,
    /// When true, launching a new wezterm instance will prefer
    /// to spawn a new tab into an existing instance.
    /// Otherwise, it will spawn a new window.
    #[dynamic(default)] pub prefer_to_spawn_tabs: bool,
    #[dynamic(default)] pub window_frame: WindowFrameConfig,
    /// Font to use for CharSelect
    #[dynamic(default)] pub char_select_font: Option<TextStyle>,
    #[dynamic(default = "default_char_select_font_size")] pub char_select_font_size: f64,
    #[dynamic(default = "default_char_select_fg_color")] pub char_select_fg_color: RgbaColor,
    #[dynamic(default = "default_char_select_bg_color")] pub char_select_bg_color: RgbaColor,
    /// Font to use for ActivateCommandPalette
    #[dynamic(default)] pub command_palette_font: Option<TextStyle>,
    #[dynamic(default = "default_command_palette_font_size")] pub command_palette_font_size: f64,
    pub command_palette_rows: Option<usize>,
    #[dynamic(default = "default_command_palette_fg_color")] pub command_palette_fg_color: RgbaColor,
    #[dynamic(default = "default_command_palette_bg_color")] pub command_palette_bg_color: RgbaColor,
    /// Font to use for PaneSelect
    #[dynamic(default)] pub pane_select_font: Option<TextStyle>,
    #[dynamic(default = "default_pane_select_font_size")] pub pane_select_font_size: f64,
    #[dynamic(default = "default_pane_select_fg_color")] pub pane_select_fg_color: RgbaColor,
    #[dynamic(default = "default_pane_select_bg_color")] pub pane_select_bg_color: RgbaColor,
    #[dynamic(default)] pub tab_bar_style: TabBarStyle,
    #[dynamic(default)] pub resolved_palette: Palette,
    /// Use a named color scheme rather than the palette specified
    /// by the colors setting.
    pub color_scheme: Option<String>,
    /// Named color schemes
    #[dynamic(default)] pub color_schemes: HashMap<String, Palette>,
    /// How many lines of scrollback you want to retain
    #[dynamic(default = "default_scrollback_lines", validate = "validate_scrollback_lines")] pub scrollback_lines: usize,
    /// If no `prog` is specified on the command line, use this
    /// instead of running the user's shell.
    /// For example, to have `wezterm` always run `top` by default,
    /// you'd use this:
    ///
    /// ```toml
    /// default_prog = ["top"]
    /// ```
    ///
    /// `default_prog` is implemented as an array where the 0th element
    /// is the command to run and the rest of the elements are passed
    /// as the positional arguments to that command.
    pub default_prog: Option<Vec<String>>,
    #[dynamic(default = "default_gui_startup_args")] pub default_gui_startup_args: Vec<String>,
    /// Specifies the default current working directory if none is specified
    /// through configuration or OSC 7 (see docs for `default_cwd` for more
    /// info!)
    pub default_cwd: Option<PathBuf>,
    #[dynamic(default)] pub exit_behavior: ExitBehavior,
    #[dynamic(default)] pub exit_behavior_messaging: ExitBehaviorMessaging,
    #[dynamic(default = "default_clean_exits")] pub clean_exit_codes: Vec<u32>,
    #[dynamic(default = "default_true")] pub detect_password_input: bool,
    /// Specifies a map of environment variables that should be set
    /// when spawning commands in the local domain.
    /// This is not used when working with remote domains.
    #[dynamic(default)] pub set_environment_variables: HashMap<String, String>,
    /// Specifies the height of a new window, expressed in character cells.
    #[dynamic(default = "default_initial_rows", validate = "validate_row_or_col")] pub initial_rows: u16,
    #[dynamic(default = "default_true")] pub enable_kitty_graphics: bool,
    #[dynamic(default)] pub enable_kitty_keyboard: bool,
    /// Whether the terminal should respond to requests to read the
    /// title string.
    /// Disabled by default for security concerns with shells that might
    /// otherwise attempt to execute the response.
    /// <https://marc.info/?l=bugtraq&m=104612710031920&w=2>
    #[dynamic(default)] pub enable_title_reporting: bool,
    /// Whether the terminal should respond to DECRQCRA checksum requests.
    /// Disabled by default as it allows programs to read screen contents.
    /// <https://vt100.net/docs/vt510-rm/DECRQCRA.html>
    #[dynamic(default)] pub enable_checksum_rectangular_area: bool,
    /// Specifies the width of a new window, expressed in character cells
    #[dynamic(default = "default_initial_cols", validate = "validate_row_or_col")] pub initial_cols: u16,
    #[dynamic(default = "default_hyperlink_rules")] pub hyperlink_rules: Vec<hyperlink::Rule>,
    /// What to set the TERM variable to
    #[dynamic(default = "default_term")] pub term: String,
    #[dynamic(default)] pub font_locator: FontLocatorSelection,
    #[dynamic(default)] pub font_rasterizer: FontRasterizerSelection,
    #[dynamic(default = "default_colr_rasterizer")] pub font_colr_rasterizer: FontRasterizerSelection,
    #[dynamic(default)] pub font_shaper: FontShaperSelection,
    #[dynamic(default)] pub display_pixel_geometry: DisplayPixelGeometry,
    #[dynamic(default)] pub freetype_load_target: FreeTypeLoadTarget,
    #[dynamic(default)] pub freetype_render_target: Option<FreeTypeLoadTarget>,
    #[dynamic(default)] pub freetype_load_flags: Option<FreeTypeLoadFlags>,
    /// Selects the freetype interpret version to use.
    /// Likely values are 35, 38 and 40 which have different
    /// characteristics with respective to subpixel hinting.
    /// See https://freetype.org/freetype2/docs/subpixel-hinting.html
    pub freetype_interpreter_version: Option<u32>,
    #[dynamic(default)] pub freetype_pcf_long_family_names: bool,
    /// Specify the features to enable when using harfbuzz for font shaping.
    /// There is some light documentation here:
    /// <https://harfbuzz.github.io/shaping-opentype-features.html>
    /// but it boils down to allowing opentype feature names to be specified
    /// using syntax similar to the CSS font-feature-settings options:
    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/font-feature-settings>.
    /// The OpenType spec lists a number of features here:
    /// <https://docs.microsoft.com/en-us/typography/opentype/spec/featurelist>
    ///
    /// Options of likely interest will be:
    ///
    /// * `calt` - <https://docs.microsoft.com/en-us/typography/opentype/spec/features_ae#tag-calt>
    /// * `clig` - <https://docs.microsoft.com/en-us/typography/opentype/spec/features_ae#tag-clig>
    ///
    /// If you want to disable ligatures in most fonts, then you may want to
    /// use a setting like this:
    ///
    /// ```toml
    /// harfbuzz_features = ["calt=0", "clig=0", "liga=0"]
    /// ```
    ///
    /// Some fonts make available extended options via stylistic sets.
    /// If you use the [Fira Code font](https://github.com/tonsky/FiraCode),
    /// it lists available stylistic sets here:
    /// <https://github.com/tonsky/FiraCode/wiki/How-to-enable-stylistic-sets>
    ///
    /// and you can set them in wezterm:
    ///
    /// ```toml
    /// # Use this for a zero with a dot rather than a line through it
    /// # when using the Fira Code font
    /// harfbuzz_features = ["zero"]
    /// ```
    #[dynamic(default = "default_harfbuzz_features")] pub harfbuzz_features: Vec<String>,
    #[dynamic(default)] pub front_end: FrontEndSelection,
    /// Whether to select the higher powered discrete GPU when
    /// the system has a choice of integrated or discrete.
    /// Defaults to low power.
    #[dynamic(default)] pub webgpu_power_preference: WebGpuPowerPreference,
    #[dynamic(default)] pub webgpu_force_fallback_adapter: bool,
    #[dynamic(default)] pub webgpu_preferred_adapter: Option<GpuInfo>,
    #[dynamic(default)] pub wsl_domains: Option<Vec<WslDomain>>,
    #[dynamic(default)] pub exec_domains: Vec<ExecDomain>,
    #[dynamic(default)] pub serial_ports: Vec<SerialDomain>,
    /// The set of unix domains
    #[dynamic(default = "UnixDomain::default_unix_domains")] pub unix_domains: Vec<UnixDomain>,
    #[dynamic(default)] pub ssh_domains: Option<Vec<SshDomain>>,
    #[dynamic(default)] pub ssh_backend: SshBackend,
    /// When running in server mode, defines configuration for
    /// each of the endpoints that we'll listen for connections
    #[dynamic(default)] pub tls_servers: Vec<TlsDomainServer>,
    /// The set of tls domains that we can connect to as a client
    #[dynamic(default)] pub tls_clients: Vec<TlsDomainClient>,
    /// Constrains the rate at which the multiplexer client will
    /// speculatively fetch line data.
    /// This helps to avoid saturating the link between the client
    /// and server if the server is dumping a large amount of output
    /// to the client.
    #[dynamic(default = "default_ratelimit_line_prefetches_per_second")] pub ratelimit_mux_line_prefetches_per_second: u32,
    /// The buffer size used by parse_buffered_data in the mux module.
    /// This should not be too large, otherwise the processing cost
    /// of applying a batch of actions to the terminal will be too
    /// high and the user experience will be laggy and less responsive.
    #[dynamic(default = "default_mux_output_parser_buffer_size")] pub mux_output_parser_buffer_size: usize,
    #[dynamic(default = "default_true")] pub mux_enable_ssh_agent: bool,
    #[dynamic(default)] pub default_ssh_auth_sock: Option<String>,
    /// How many ms to delay after reading a chunk of output
    /// in order to try to coalesce fragmented writes into
    /// a single bigger chunk of output and reduce the chances
    /// observing "screen tearing" with un-synchronized output
    #[dynamic(default = "default_mux_output_parser_coalesce_delay_ms")] pub mux_output_parser_coalesce_delay_ms: u64,
    #[dynamic(default = "default_mux_env_remove")] pub mux_env_remove: Vec<String>,
    #[dynamic(default)] pub keys: Vec<Key>,
    #[dynamic(default)] pub key_tables: HashMap<String, Vec<Key>>,
    #[dynamic(default = "default_bypass_mouse_reporting_modifiers")] pub bypass_mouse_reporting_modifiers: Modifiers,
    #[dynamic(default)] pub debug_key_events: bool,
    #[dynamic(default)] pub normalize_output_to_unicode_nfc: bool,
    #[dynamic(default)] pub disable_default_key_bindings: bool,
    pub leader: Option<LeaderKey>,
    #[dynamic(default = "default_num_alphabet")] pub launcher_alphabet: String,
    #[dynamic(default)] pub disable_default_quick_select_patterns: bool,
    #[dynamic(default)] pub quick_select_patterns: Vec<String>,
    #[dynamic(default = "default_alphabet")] pub quick_select_alphabet: String,
    #[dynamic(default)] pub quick_select_remove_styling: bool,
    #[dynamic(default)] pub mouse_bindings: Vec<Mouse>,
    #[dynamic(default)] pub disable_default_mouse_bindings: bool,
    #[dynamic(default)] pub daemon_options: DaemonOptions,
    #[dynamic(default)] pub send_composed_key_when_left_alt_is_pressed: bool,
    #[dynamic(default = "default_true")] pub send_composed_key_when_right_alt_is_pressed: bool,
    #[dynamic(default = "default_macos_forward_mods")] pub macos_forward_to_ime_modifier_mask: Modifiers,
    #[dynamic(default)] pub treat_left_ctrlalt_as_altgr: bool,
    /// If true, the `Backspace` and `Delete` keys generate `Delete` and `Backspace`
    /// keypresses, respectively, rather than their normal keycodes.
    /// On macOS the default for this is true because its Backspace key
    /// is labeled as Delete and things are backwards.
    #[dynamic(default = "default_swap_backspace_and_delete")] pub swap_backspace_and_delete: bool,
    /// If true, display the tab bar UI at the top of the window.
    /// The tab bar shows the titles of the tabs and which is the
    /// active tab.  Clicking on a tab activates it.
    #[dynamic(default = "default_true")] pub enable_tab_bar: bool,
    #[dynamic(default = "default_true")] pub use_fancy_tab_bar: bool,
    #[dynamic(default)] pub tab_bar_at_bottom: bool,
    #[dynamic(default)] pub enable_secondary_bar: bool,
    #[dynamic(default = "default_true")] pub mouse_wheel_scrolls_tabs: bool,
    /// If true, tab bar titles are prefixed with the tab index
    #[dynamic(default = "default_true")] pub show_tab_index_in_tab_bar: bool,
    #[dynamic(default = "default_true")] pub show_tabs_in_tab_bar: bool,
    #[dynamic(default = "default_true")] pub show_new_tab_button_in_tab_bar: bool,
    #[dynamic(default = "default_true")] pub show_close_tab_button_in_tabs: bool,
    /// If true, show_tab_index_in_tab_bar uses a zero-based index.
    /// The default is false and the tab shows a one-based index.
    #[dynamic(default)] pub tab_and_split_indices_are_zero_based: bool,
    /// Specifies the maximum width that a tab can have in the
    /// tab bar.  Defaults to 16 glyphs in width.
    #[dynamic(default = "default_tab_max_width")] pub tab_max_width: usize,
    /// If true, hide the tab bar if the window only has a single tab.
    #[dynamic(default)] pub hide_tab_bar_if_only_one_tab: bool,
    #[dynamic(default)] pub enable_scroll_bar: bool,
    #[dynamic(try_from = "crate::units::PixelUnit", default = "default_half_cell")] pub min_scroll_bar_height: Dimension,
    /// If false, do not try to use a Wayland protocol connection
    /// when starting the gui frontend, and instead use X11.
    /// This option is only considered on X11/Wayland systems and
    /// has no effect on macOS or Windows.
    /// The default is true.
    #[dynamic(default = "default_true")] pub enable_wayland: bool,
    #[dynamic(default)] pub enable_zwlr_output_manager: bool,
    /// Whether to prefer EGL over other GL implementations.
    /// EGL on Windows has jankier resize behavior than WGL (which
    /// is used if EGL is unavailable), but EGL survives graphics
    /// driver updates without breaking and losing your work.
    #[dynamic(default = "default_prefer_egl")] pub prefer_egl: bool,
    #[dynamic(default = "default_true")] pub custom_block_glyphs: bool,
    #[dynamic(default = "default_true")] pub anti_alias_custom_block_glyphs: bool,
    /// Controls the amount of padding to use around the terminal cell area
    #[dynamic(default)] pub window_padding: WindowPadding,
    #[dynamic(default)] pub window_content_alignment: WindowContentAlignment,
    /// Specifies the path to a background image attachment file.
    /// The file can be any image format that the rust `image`
    /// crate is able to identify and load.
    /// A window background image is rendered into the background
    /// of the window before any other content.
    ///
    /// The image will be scaled to fit the window.
    #[dynamic(default)] pub window_background_image: Option<PathBuf>,
    #[dynamic(default)] pub window_background_gradient: Option<Gradient>,
    #[dynamic(default)] pub window_background_image_hsb: Option<HsbTransform>,
    #[dynamic(default)] pub foreground_text_hsb: HsbTransform,
    #[dynamic(default)] pub background: Vec<BackgroundLayer>,
    /// Only works on MacOS
    #[dynamic(default)] pub macos_window_background_blur: i64,
    /// Only works on KDE Wayland
    #[dynamic(default)] pub kde_window_background_blur: bool,
    /// Only works on Windows
    #[dynamic(default)] pub win32_system_backdrop: SystemBackdrop,
    #[dynamic(default = "default_win32_acrylic_accent_color")] pub win32_acrylic_accent_color: RgbaColor,
    /// Specifies the alpha value to use when rendering the background
    /// of the window.  The background is taken either from the
    /// window_background_image, or if there is none, the background
    /// color of the cell in the current position.
    /// The default is 1.0 which is 100% opaque.  Setting it to a number
    /// between 0.0 and 1.0 will allow for the screen behind the window
    /// to "shine through" to varying degrees.
    /// This only works on systems with a compositing window manager.
    /// Setting opacity to a value other than 1.0 can impact render
    /// performance.
    #[dynamic(default = "default_one_point_oh")] pub window_background_opacity: f32,
    /// inactive_pane_hue, inactive_pane_saturation and
    /// inactive_pane_brightness allow for transforming the color
    /// of inactive panes.
    /// The pane colors are converted to HSV values and multiplied
    /// by these values before being converted back to RGB to
    /// use in the display.
    ///
    /// The default is 1.0 which leaves the values as-is.
    ///
    /// Modifying the hue changes the hue of the color by rotating
    /// it through the color wheel.  It is not as useful as the
    /// other components, but is available "for free" as part of
    /// the colorspace conversion.
    ///
    /// Modifying the saturation can add or reduce the amount of
    /// "colorfulness".  Making the value smaller can make it appear
    /// more washed out.
    ///
    /// Modifying the brightness can be used to dim or increase
    /// the perceived amount of light.
    ///
    /// The range of these values is 0.0 and up; they are used to
    /// multiply the existing values, so the default of 1.0
    /// preserves the existing component, whilst 0.5 will reduce
    /// it by half, and 2.0 will double the value.
    ///
    /// A subtle dimming effect can be achieved by setting:
    /// inactive_pane_saturation = 0.9
    /// inactive_pane_brightness = 0.8
    #[dynamic(default = "default_inactive_pane_hsb")] pub inactive_pane_hsb: HsbTransform,
    #[dynamic(default = "default_one_point_oh")] pub text_background_opacity: f32,
    /// Specifies how often a blinking cursor transitions between visible
    /// and invisible, expressed in milliseconds.
    /// Setting this to 0 disables blinking.
    /// Note that this value is approximate due to the way that the system
    /// event loop schedulers manage timers; non-zero values will be at
    /// least the interval specified with some degree of slop.
    #[dynamic(default = "default_cursor_blink_rate")] pub cursor_blink_rate: u64,
    #[dynamic(default = "linear_ease")] pub cursor_blink_ease_in: EasingFunction,
    #[dynamic(default = "linear_ease")] pub cursor_blink_ease_out: EasingFunction,
    #[dynamic(default = "default_anim_fps")] pub animation_fps: u8,
    #[dynamic(default)] pub text_min_contrast_ratio: Option<f32>,
    #[dynamic(default)] pub force_reverse_video_cursor: bool,
    #[dynamic(default = "default_reverse_video_cursor_min_contrast")] pub reverse_video_cursor_min_contrast: f32,
    /// Specifies the default cursor style.  various escape sequences
    /// can override the default style in different situations (eg:
    /// an editor can change it depending on the mode), but this value
    /// controls how the cursor appears when it is reset to default.
    /// The default is `SteadyBlock`.
    /// Acceptable values are `SteadyBlock`, `BlinkingBlock`,
    /// `SteadyUnderline`, `BlinkingUnderline`, `SteadyBar`,
    /// and `BlinkingBar`.
    #[dynamic(default)] pub default_cursor_style: DefaultCursorStyle,
    /// Specifies how often blinking text (normal speed) transitions
    /// between visible and invisible, expressed in milliseconds.
    /// Setting this to 0 disables slow text blinking.  Note that this
    /// value is approximate due to the way that the system event loop
    /// schedulers manage timers; non-zero values will be at least the
    /// interval specified with some degree of slop.
    #[dynamic(default = "default_text_blink_rate")] pub text_blink_rate: u64,
    #[dynamic(default = "linear_ease")] pub text_blink_ease_in: EasingFunction,
    #[dynamic(default = "linear_ease")] pub text_blink_ease_out: EasingFunction,
    /// Specifies how often blinking text (rapid speed) transitions
    /// between visible and invisible, expressed in milliseconds.
    /// Setting this to 0 disables rapid text blinking.  Note that this
    /// value is approximate due to the way that the system event loop
    /// schedulers manage timers; non-zero values will be at least the
    /// interval specified with some degree of slop.
    #[dynamic(default = "default_text_blink_rate_rapid")] pub text_blink_rate_rapid: u64,
    #[dynamic(default = "linear_ease")] pub text_blink_rapid_ease_in: EasingFunction,
    #[dynamic(default = "linear_ease")] pub text_blink_rapid_ease_out: EasingFunction,
    /// If true, the mouse cursor will be hidden while typing.
    /// This option is true by default.
    #[dynamic(default = "default_true")] pub hide_mouse_cursor_when_typing: bool,
    /// If non-zero, specifies the period (in seconds) at which various
    /// statistics are logged.  Note that there is a minimum period of
    /// 10 seconds.
    #[dynamic(default)] pub periodic_stat_logging: u64,
    /// If false, do not scroll to the bottom of the terminal when
    /// you send input to the terminal.
    /// The default is to scroll to the bottom when you send input
    /// to the terminal.
    #[dynamic(default = "default_true")] pub scroll_to_bottom_on_input: bool,
    #[dynamic(default = "default_true")] pub use_ime: bool,
    #[dynamic(default)] pub xim_im_name: Option<String>,
    #[dynamic(default)] pub ime_preedit_rendering: ImePreeditRendering,
    #[dynamic(default)] pub notification_handling: NotificationHandling,
    #[dynamic(default = "default_true")] pub use_dead_keys: bool,
    #[dynamic(default)] pub launch_menu: Vec<SpawnCommand>,
    #[dynamic(default)] pub use_box_model_render: bool,
    /// When true, watch the config file and reload it automatically
    /// when it is detected as changing.
    #[dynamic(default = "default_true")] pub automatically_reload_config: bool,
    #[dynamic(default = "default_check_for_updates")] pub check_for_updates: bool,
    #[dynamic(default, deprecated = "this option no longer does anything and will be removed in a future release")] pub show_update_window: bool,
    #[dynamic(default = "default_update_interval")] pub check_for_updates_interval_seconds: u64,
    /// When set to true, use the CSI-U encoding scheme as described
    /// in http://www.leonerd.org.uk/hacks/fixterms/
    /// This is off by default because @wez and @jsgf find the shift-space
    /// mapping annoying in vim :-p
    #[dynamic(default)] pub enable_csi_u_key_encoding: bool,
    #[dynamic(default)] pub window_close_confirmation: WindowCloseConfirmation,
    #[dynamic(default)] pub native_macos_fullscreen_mode: bool,
    #[dynamic(default)] pub macos_fullscreen_extend_behind_notch: bool,
    #[dynamic(default = "default_word_boundary")] pub selection_word_boundary: String,
    #[dynamic(default = "default_enq_answerback")] pub enq_answerback: String,
    #[dynamic(default)] pub adjust_window_size_when_changing_font_size: Option<bool>,
    #[dynamic(default = "default_tiling_desktop_environments")] pub tiling_desktop_environments: Vec<String>,
    #[dynamic(default)] pub use_resize_increments: bool,
    #[dynamic(default = "default_alternate_buffer_wheel_scroll_speed")] pub alternate_buffer_wheel_scroll_speed: u8,
    #[dynamic(default = "default_status_update_interval")] pub status_update_interval: u64,
    #[dynamic(default)] pub experimental_pixel_positioning: bool,
    #[dynamic(default)] pub ignore_svg_fonts: bool,
    #[dynamic(default)] pub bidi_enabled: bool,
    #[dynamic(default)] pub bidi_direction: ParagraphDirectionHint,
    #[dynamic(default = "default_stateless_process_list")] pub skip_close_confirmation_for_processes_named: Vec<String>,
    #[dynamic(default = "default_true")] pub quit_when_all_windows_are_closed: bool,
    #[dynamic(default = "default_true")] pub warn_about_missing_glyphs: bool,
    #[dynamic(default)] pub sort_fallback_fonts_by_coverage: bool,
    #[dynamic(default)] pub search_font_dirs_for_fallback: bool,
    #[dynamic(default)] pub use_cap_height_to_scale_fallback_fonts: bool,
    #[dynamic(default)] pub swallow_mouse_click_on_pane_focus: bool,
    #[dynamic(default = "default_swallow_mouse_click_on_window_focus")] pub swallow_mouse_click_on_window_focus: bool,
    #[dynamic(default)] pub pane_focus_follows_mouse: bool,
    #[dynamic(default = "default_true")] pub unzoom_on_switch_pane: bool,
    #[dynamic(default = "default_max_fps")] pub max_fps: u64,
    #[dynamic(default = "default_shape_cache_size")] pub shape_cache_size: usize,
    #[dynamic(default = "default_line_state_cache_size")] pub line_state_cache_size: usize,
    #[dynamic(default = "default_line_quad_cache_size")] pub line_quad_cache_size: usize,
    #[dynamic(default = "default_line_to_ele_shape_cache_size")] pub line_to_ele_shape_cache_size: usize,
    #[dynamic(default = "default_glyph_cache_image_cache_size")] pub glyph_cache_image_cache_size: usize,
    #[dynamic(default)] pub visual_bell: VisualBell,
    #[dynamic(default)] pub audible_bell: AudibleBell,
    #[dynamic(default)] pub canonicalize_pasted_newlines: Option<NewlineCanon>,
    #[dynamic(default = "default_unicode_version")] pub unicode_version: u8,
    #[dynamic(default)] pub treat_east_asian_ambiguous_width_as_wide: bool,
    #[dynamic(default)] pub cell_widths: Option<Vec<CellWidth>>,
    #[dynamic(default = "default_true")] pub allow_download_protocols: bool,
    #[dynamic(default = "default_true")] pub allow_win32_input_mode: bool,
    #[dynamic(default)] pub default_domain: Option<String>,
    #[dynamic(default)] pub default_mux_server_domain: Option<String>,
    #[dynamic(default)] pub default_workspace: Option<String>,
    #[dynamic(default)] pub xcursor_theme: Option<String>,
    #[dynamic(default)] pub xcursor_size: Option<u32>,
    #[dynamic(default)] pub key_map_preference: KeyMapPreference,
    #[dynamic(default)] pub quote_dropped_files: DroppedFileQuoting,
    #[dynamic(default)] pub ui_key_cap_rendering: UIKeyCapRendering,
    #[dynamic(default = "default_one")] pub palette_max_key_assigments_for_action: usize,
    #[dynamic(default = "default_ulimit_nofile")] pub ulimit_nofile: u64,
    #[dynamic(default = "default_ulimit_nproc")] pub ulimit_nproc: u64,
}
impl_lua_conversion_dynamic!(Config);

impl Default for Config {
    fn default() -> Self {
        // Ask FromDynamic to provide the defaults based on the attributes
        // specified in the struct so that we don't have to repeat
        // the same thing in a different form down here
        Config::from_dynamic(
            &wezterm_dynamic::Value::Object(Default::default()),
            Default::default(),
        )
        .unwrap()
    }
}
