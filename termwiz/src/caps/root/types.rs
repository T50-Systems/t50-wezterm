builder! {
    /// Use the `ProbeHints` to configure an instance of
    /// the `ProbeHints` struct.  `ProbeHints` are passed to the `Capabilities`
    /// constructor to influence the effective set of terminal capabilities.
    #[derive(Debug, Default, Clone)]
    pub struct ProbeHints {
        /// The contents of the TERM environment variable
        term: Option<String>,

        /// The contents of the COLORTERM environment variable.
        /// <http://invisible-island.net/ncurses/ncurses-slang.html#env_COLORTERM>
        colorterm: Option<String>,

        /// The contents of the TERM_PROGRAM environment variable
        term_program: Option<String>,

        /// Override the choice of the number of colors
        color_level: Option<ColorLevel>,

        /// The contents of the TERM_PROGRAM_VERSION environment variable
        term_program_version: Option<String>,

        /// Definitively set whether hyperlinks are supported.
        /// The default is to assume yes as this is mostly harmless.
        hyperlinks: Option<bool>,

        /// Configure whether sixel graphics are supported.
        sixel: Option<bool>,

        /// Configure whether iTerm2 style graphics embedding is supported
        /// See <https://www.iterm2.com/documentation-images.html>
        iterm2_image: Option<bool>,

        /// Specify whether `bce`, background color erase, is supported.
        bce: Option<bool>,

        /// The contents of the COLORTERM_BCE environment variable
        /// <http://invisible-island.net/ncurses/ncurses-slang.html#env_COLORTERM_BCE>
        colorterm_bce: Option<String>,

        /// A loaded terminfo database entry
        terminfo_db: Option<terminfo::Database>,

        /// Whether bracketed paste mode is supported
        bracketed_paste: Option<bool>,

        /// Whether mouse support is present and should be used
        mouse_reporting: Option<bool>,

        /// When true, rather than using the terminfo `sgr` or `sgr0` entries,
        /// assume that the terminal is ANSI/ECMA-48 compliant for the
        /// common SGR attributes of bold, dim, reverse, underline, blink,
        /// invisible and reset, and directly emit those sequences.
        /// This can improve rendered text compatibility with pagers.
        force_terminfo_render_to_use_ansi_sgr: Option<bool>,
    }
}

impl ProbeHints {
    pub fn new_from_env() -> Self {
        let mut probe_hints = ProbeHints::default()
            .term(var("TERM").ok())
            .colorterm(var("COLORTERM").ok())
            .colorterm_bce(var("COLORTERM_BCE").ok())
            .term_program(var("TERM_PROGRAM").ok())
            .term_program_version(var("TERM_PROGRAM_VERSION").ok());

        if !std::env::var(NO_COLOR_ENV)
            .unwrap_or("".to_string())
            .is_empty()
        {
            probe_hints.color_level = Some(ColorLevel::MonoChrome);
        }

        probe_hints
    }
}

/// Describes the level of color support available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorLevel {
    /// Basic ANSI colors; 8 colors + bright versions
    Sixteen,
    /// In addition to the ANSI 16 colors, this has 24 levels of grey
    /// and 216 colors typically 6x6x6 color cube with 5 bits.  There
    /// is some variance in implementations: the precise color cube is
    /// different in different emulators.
    TwoFiftySix,
    /// Commonly accepted as 24-bit RGB color.  The implementation may
    /// display these exactly as specified or it may match to an internal
    /// palette with fewer than the theoretical maximum 16 million colors.
    /// What we care about here is whether the terminal supports the escape
    /// sequence to specify RGB values rather than a palette index.
    TrueColor,
    /// Describes monochrome (black and white) color support.
    /// Enabled via NO_COLOR environment variable.
    MonoChrome,
}

/// `Capabilities` holds information about the capabilities of a terminal.
/// On POSIX systems this is largely derived from an available terminfo
/// database, but there are some newish capabilities that are not yet
/// described by the majority of terminfo installations and thus have some
/// additional handling in this struct.
#[derive(Debug, Clone)]
pub struct Capabilities {
    color_level: ColorLevel,
    hyperlinks: bool,
    sixel: bool,
    iterm2_image: bool,
    bce: bool,
    terminfo_db: Option<terminfo::Database>,
    bracketed_paste: bool,
    mouse_reporting: bool,
    force_terminfo_render_to_use_ansi_sgr: bool,
}
