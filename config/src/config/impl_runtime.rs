use super::*;

impl Config {
    pub fn default_config() -> Self {
        Self::default().compute_extra_defaults(None)
    }

    pub fn key_bindings(&self) -> KeyTables {
        let mut tables = KeyTables::default();

        for k in &self.keys {
            let (key, mods) = k
                .key
                .key
                .resolve(self.key_map_preference)
                .normalize_shift(k.key.mods);
            tables.default.insert(
                (key, mods),
                KeyTableEntry {
                    action: k.action.clone(),
                },
            );
        }

        for (name, keys) in &self.key_tables {
            let mut table = KeyTable::default();
            for k in keys {
                let (key, mods) = k
                    .key
                    .key
                    .resolve(self.key_map_preference)
                    .normalize_shift(k.key.mods);
                table.insert(
                    (key, mods),
                    KeyTableEntry {
                        action: k.action.clone(),
                    },
                );
            }
            tables.by_name.insert(name.to_string(), table);
        }

        tables
    }

    pub fn mouse_bindings(
        &self,
    ) -> HashMap<(MouseEventTrigger, MouseEventTriggerMods), KeyAssignment> {
        let mut map = HashMap::new();

        for m in &self.mouse_bindings {
            map.insert((m.event.clone(), m.mods), m.action.clone());
        }

        map
    }

    /// In some cases we need to compute expanded values based
    /// on those provided by the user.  This is where we do that.
    pub fn compute_extra_defaults(&self, config_path: Option<&Path>) -> Self {
        let mut cfg = self.clone();

        // Convert any relative font dirs to their config file relative locations
        if let Some(config_dir) = config_path.as_ref().and_then(|p| p.parent()) {
            for font_dir in &mut cfg.font_dirs {
                if !font_dir.is_absolute() {
                    let dir = config_dir.join(&font_dir);
                    *font_dir = dir;
                }
            }

            if let Some(path) = &self.window_background_image {
                if !path.is_absolute() {
                    cfg.window_background_image.replace(config_dir.join(path));
                }
            }
        }

        // Add some reasonable default font rules
        let reduced = self.font.reduce_first_font_to_family();

        let italic = reduced.make_italic();

        let bold = reduced.make_bold();
        let bold_italic = bold.make_italic();

        let half_bright = reduced.make_half_bright();
        let half_bright_italic = half_bright.make_italic();

        cfg.font_rules.push(StyleRule {
            italic: Some(true),
            intensity: Some(wezterm_term_api::Intensity::Half),
            font: half_bright_italic,
            ..Default::default()
        });

        cfg.font_rules.push(StyleRule {
            italic: Some(false),
            intensity: Some(wezterm_term_api::Intensity::Half),
            font: half_bright,
            ..Default::default()
        });

        cfg.font_rules.push(StyleRule {
            italic: Some(false),
            intensity: Some(wezterm_term_api::Intensity::Bold),
            font: bold,
            ..Default::default()
        });

        cfg.font_rules.push(StyleRule {
            italic: Some(true),
            intensity: Some(wezterm_term_api::Intensity::Bold),
            font: bold_italic,
            ..Default::default()
        });

        cfg.font_rules.push(StyleRule {
            italic: Some(true),
            intensity: Some(wezterm_term_api::Intensity::Normal),
            font: italic,
            ..Default::default()
        });

        // Load any additional color schemes into the color_schemes map
        cfg.load_color_schemes(&cfg.compute_color_scheme_dirs())
            .ok();

        if let Some(scheme) = cfg.color_scheme.as_ref() {
            match cfg.resolve_color_scheme() {
                None => {
                    log::error!(
                        "Your configuration specifies color_scheme=\"{}\" \
                        but that scheme was not found",
                        scheme
                    );
                }
                Some(p) => {
                    cfg.resolved_palette = p.clone();
                }
            }
        }

        if let Some(colors) = &cfg.colors {
            cfg.resolved_palette = cfg.resolved_palette.overlay_with(colors);
        }

        if let Some(bg) = BackgroundLayer::with_legacy(self) {
            cfg.background.insert(0, bg);
        }

        cfg
    }

    fn compute_color_scheme_dirs(&self) -> Vec<PathBuf> {
        let mut paths = self.color_scheme_dirs.clone();
        for dir in CONFIG_DIRS.iter() {
            paths.push(dir.join("colors"));
        }
        if cfg!(windows) {
            // See commentary re: portable tools above!
            if let Ok(exe_name) = std::env::current_exe() {
                if let Some(exe_dir) = exe_name.parent() {
                    paths.insert(0, exe_dir.join("colors"));
                }
            }
        }
        paths
    }

    fn load_color_schemes(&mut self, paths: &[PathBuf]) -> anyhow::Result<()> {
        fn extract_scheme_name(name: &str) -> Option<&str> {
            if name.ends_with(".toml") {
                let len = name.len();
                Some(&name[..len - 5])
            } else {
                None
            }
        }

        fn load_scheme(path: &Path) -> anyhow::Result<ColorSchemeFile> {
            let s = std::fs::read_to_string(path)?;
            ColorSchemeFile::from_toml_str(&s).context("parsing TOML")
        }

        for colors_dir in paths {
            if let Ok(dir) = std::fs::read_dir(colors_dir) {
                for entry in dir {
                    if let Ok(entry) = entry {
                        if let Some(name) = entry.file_name().to_str() {
                            if let Some(scheme_name) = extract_scheme_name(name) {
                                if self.color_schemes.contains_key(scheme_name) {
                                    // This scheme has already been defined
                                    continue;
                                }

                                let path = entry.path();
                                match load_scheme(&path) {
                                    Ok(scheme) => {
                                        let name = scheme
                                            .metadata
                                            .name
                                            .unwrap_or_else(|| scheme_name.to_string());
                                        log::trace!(
                                            "Loaded color scheme `{}` from {}",
                                            name,
                                            path.display()
                                        );
                                        self.color_schemes.insert(name, scheme.colors);
                                    }
                                    Err(err) => {
                                        log::error!(
                                            "Color scheme in `{}` failed to load: {:#}",
                                            path.display(),
                                            err
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn resolve_color_scheme(&self) -> Option<&Palette> {
        let scheme_name = self.color_scheme.as_ref()?;

        if let Some(palette) = self.color_schemes.get(scheme_name) {
            Some(palette)
        } else {
            crate::COLOR_SCHEMES.get(scheme_name)
        }
    }

    pub fn initial_size(&self, dpi: u32, cell_pixel_dims: Option<(usize, usize)>) -> TerminalSize {
        // If we aren't passed the actual values, guess at a plausible
        // default set of pixel dimensions.
        // This is based on "typical" 10 point font at "normal"
        // pixel density.
        // This will get filled in by the gui layer, but there is
        // an edge case where we emit an iTerm image escape in
        // the software update banner through the mux layer before
        // the GUI has had a chance to update the pixel dimensions
        // when running under X11.
        // This is a bit gross.
        let (cell_pixel_width, cell_pixel_height) = cell_pixel_dims.unwrap_or((8, 16));

        TerminalSize {
            rows: self.initial_rows as usize,
            cols: self.initial_cols as usize,
            pixel_width: cell_pixel_width * self.initial_cols as usize,
            pixel_height: cell_pixel_height * self.initial_rows as usize,
            dpi,
        }
    }

    pub fn build_prog(
        &self,
        prog: Option<Vec<&OsStr>>,
        default_prog: Option<&Vec<String>>,
        default_cwd: Option<&PathBuf>,
    ) -> anyhow::Result<CommandBuilder> {
        let mut cmd = match prog {
            Some(args) => {
                let mut args = args.iter();
                let mut cmd = CommandBuilder::new(args.next().expect("executable name"));
                cmd.args(args);
                cmd
            }
            None => {
                if let Some(prog) = default_prog {
                    let mut args = prog.iter();
                    let mut cmd = CommandBuilder::new(args.next().expect("executable name"));
                    cmd.args(args);
                    cmd
                } else {
                    CommandBuilder::new_default_prog()
                }
            }
        };

        self.apply_cmd_defaults(&mut cmd, None, default_cwd);

        Ok(cmd)
    }

    pub fn apply_cmd_defaults(
        &self,
        cmd: &mut CommandBuilder,
        default_prog: Option<&Vec<String>>,
        default_cwd: Option<&PathBuf>,
    ) {
        // Apply `default_cwd` only if `cwd` is not already set, allows `--cwd`
        // option to take precedence
        if let (None, Some(cwd)) = (cmd.get_cwd(), default_cwd) {
            cmd.cwd(cwd);
        }

        if let Some(default_prog) = default_prog {
            if cmd.is_default_prog() {
                cmd.replace_default_prog(default_prog);
            }
        }

        // Augment WSLENV so that TERM related environment propagates
        // across the win32/wsl boundary
        let mut wsl_env = std::env::var("WSLENV").ok();

        // If we are running as an appimage, we will have "$APPIMAGE"
        // and "$APPDIR" set in the wezterm process. These will be
        // propagated to the child processes. Since some apps (including
        // wezterm) use these variables to detect if they are running in
        // an appimage, those child processes will be misconfigured.
        // Ensure that they are unset.
        // https://docs.appimage.org/packaging-guide/environment-variables.html#id2
        cmd.env_remove("APPIMAGE");
        cmd.env_remove("APPDIR");
        cmd.env_remove("OWD");

        for (k, v) in &self.set_environment_variables {
            if k == "WSLENV" {
                wsl_env.replace(v.clone());
            } else {
                cmd.env(k, v);
            }
        }

        if wsl_env.is_some() || cfg!(windows) || crate::version::running_under_wsl() {
            let mut wsl_env = wsl_env.unwrap_or_default();
            if !wsl_env.is_empty() {
                wsl_env.push(':');
            }
            wsl_env.push_str("TERM:COLORTERM:TERM_PROGRAM:TERM_PROGRAM_VERSION");
            cmd.env("WSLENV", wsl_env);
        }

        #[cfg(unix)]
        cmd.umask(umask::UmaskSaver::saved_umask());
        cmd.env("TERM", &self.term);
        cmd.env("COLORTERM", "truecolor");
        // TERM_PROGRAM and TERM_PROGRAM_VERSION are an emerging
        // de-facto standard for identifying the terminal.
        cmd.env("TERM_PROGRAM", "WezTerm");
        cmd.env("TERM_PROGRAM_VERSION", crate::wezterm_version());
    }
}
