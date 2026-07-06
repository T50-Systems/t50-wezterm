impl Capabilities {
    /// Detect the capabilities of the terminal and return the
    /// Capability object holding the outcome.
    /// This function inspects the environment variables to build
    /// up configuration hints.
    pub fn new_from_env() -> Result<Self> {
        Self::new_with_hints(ProbeHints::new_from_env())
    }

    /// Return modified capabilities with the assumption that we're
    /// using an xterm compatible terminal and the built-in xterm
    /// terminfo database.  This is used on Windows when the TERM
    /// is set to xterm-256color and we didn't find an equivalent
    /// terminfo on the local filesystem.  We're using this as a
    /// way to opt in to using terminal escapes rather than the
    /// legacy win32 console API.
    #[cfg(windows)]
    pub(crate) fn apply_builtin_terminfo(mut self) -> Self {
        let data = include_bytes!("../../../data/xterm-256color");
        let db = terminfo::Database::from_buffer(data.as_ref()).unwrap();
        self.terminfo_db = Some(db);
        self.color_level = ColorLevel::TrueColor;
        self
    }

    /// Build a `Capabilities` object based on the provided `ProbeHints` object.
    pub fn new_with_hints(hints: ProbeHints) -> Result<Self> {
        let terminfo_db = hints.terminfo_db.as_ref().cloned();
        let terminfo_db = if cfg!(test) {
            // Don't load from the system terminfo in tests, as it is unpredictable
            terminfo_db
        } else {
            terminfo_db.or_else(|| match hints.term.as_ref() {
                Some(t) => terminfo::Database::from_name(t).ok(),
                None => terminfo::Database::from_env().ok(),
            })
        };

        let color_level = hints.color_level.unwrap_or_else(|| {
            // If set, COLORTERM overrides any other source of information
            match hints.colorterm.as_ref().map(String::as_ref) {
                Some("truecolor") | Some("24bit") => ColorLevel::TrueColor,
                Some(_) => ColorLevel::TwoFiftySix,
                _ => {
                    // COLORTERM isn't set, so look at the terminfo.
                    if let Some(ref db) = terminfo_db.as_ref() {
                        let has_true_color = db
                            .get::<cap::TrueColor>()
                            .unwrap_or(cap::TrueColor(false))
                            .0;
                        if has_true_color {
                            ColorLevel::TrueColor
                        } else if let Some(cap::MaxColors(n)) = db.get::<cap::MaxColors>() {
                            if n >= 16777216 {
                                ColorLevel::TrueColor
                            } else if n >= 256 {
                                ColorLevel::TwoFiftySix
                            } else {
                                ColorLevel::Sixteen
                            }
                        } else {
                            ColorLevel::Sixteen
                        }
                    } else if let Some(ref term) = hints.term {
                        // if we don't have TERMINFO, use a somewhat awful
                        // substring test against the TERM name.
                        if term.contains("256color") {
                            ColorLevel::TwoFiftySix
                        } else {
                            ColorLevel::Sixteen
                        }
                    } else {
                        ColorLevel::Sixteen
                    }
                }
            }
        });

        // I don't know of a way to detect SIXEL support, so we
        // assume no by default.
        let sixel = hints.sixel.unwrap_or(false);

        // The use of OSC 8 for hyperlinks means that it is generally
        // safe to assume yes: if the terminal doesn't support it,
        // the text will look "OK", although some versions of VTE based
        // terminals had a bug where it look like garbage.
        let hyperlinks = hints.hyperlinks.unwrap_or(true);

        let bce = hints.bce.unwrap_or_else(|| {
            // Use the COLORTERM_BCE variable to override any terminfo
            match hints.colorterm_bce.as_ref().map(String::as_ref) {
                Some("1") => true,
                _ => {
                    // Look it up from terminfo
                    terminfo_db
                        .as_ref()
                        .map(|db| {
                            db.get::<cap::BackColorErase>()
                                .unwrap_or(cap::BackColorErase(false))
                                .0
                        })
                        .unwrap_or(false)
                }
            }
        });

        let iterm2_image = hints.iterm2_image.unwrap_or_else(|| {
            match hints.term_program.as_ref().map(String::as_ref) {
                Some("iTerm.app") => {
                    // We're testing whether it has animated gif support
                    // here because the iTerm2 docs don't say when the
                    // image protocol was first implemented, but do mention
                    // the gif version.
                    version_ge(
                        hints
                            .term_program_version
                            .as_ref()
                            .unwrap_or(&"0.0.0".to_owned()),
                        "2.9.20150512",
                    )
                }
                Some("WezTerm") => true,
                _ => false,
            }
        });

        let bracketed_paste = hints.bracketed_paste.unwrap_or(true);
        let mouse_reporting = hints.mouse_reporting.unwrap_or(true);

        let force_terminfo_render_to_use_ansi_sgr =
            hints.force_terminfo_render_to_use_ansi_sgr.unwrap_or(false);

        Ok(Self {
            color_level,
            sixel,
            hyperlinks,
            iterm2_image,
            bce,
            terminfo_db,
            bracketed_paste,
            mouse_reporting,
            force_terminfo_render_to_use_ansi_sgr,
        })
    }

    /// Indicates how many colors are supported
    pub fn color_level(&self) -> ColorLevel {
        self.color_level
    }

    /// Does the terminal support SIXEL graphics?
    pub fn sixel(&self) -> bool {
        self.sixel
    }

    /// Does the terminal support hyperlinks?
    /// See <https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda>
    pub fn hyperlinks(&self) -> bool {
        self.hyperlinks
    }

    /// Does the terminal support the iTerm2 image protocol?
    /// See <https://www.iterm2.com/documentation-images.html>
    pub fn iterm2_image(&self) -> bool {
        self.iterm2_image
    }

    /// Is `bce`, background color erase supported?
    /// <http://invisible-island.net/ncurses/ncurses-slang.html#env_COLORTERM_BCE>
    pub fn bce(&self) -> bool {
        self.bce
    }

    /// Returns a reference to the loaded terminfo, if any.
    pub fn terminfo_db(&self) -> Option<&terminfo::Database> {
        self.terminfo_db.as_ref()
    }

    /// Whether bracketed paste is supported
    pub fn bracketed_paste(&self) -> bool {
        self.bracketed_paste
    }

    /// Whether mouse reporting is supported
    pub fn mouse_reporting(&self) -> bool {
        self.mouse_reporting
    }

    /// Whether to emit standard ANSI/ECMA-48 codes, overriding any
    /// SGR terminfo capabilities.
    pub fn force_terminfo_render_to_use_ansi_sgr(&self) -> bool {
        self.force_terminfo_render_to_use_ansi_sgr
    }
}
