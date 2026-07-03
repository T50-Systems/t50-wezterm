pub struct Configuration {
    inner: Mutex<ConfigInner>,
}

impl Configuration {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(ConfigInner::new()),
        }
    }

    /// Returns the effective configuration.
    pub fn get(&self) -> ConfigHandle {
        let inner = self.inner.lock().unwrap();
        ConfigHandle {
            config: Arc::clone(&inner.config),
            generation: inner.generation,
        }
    }

    /// Subscribe to config reload events
    fn subscribe<F>(&self, subscriber: F) -> usize
    where
        F: Fn() -> bool + 'static + Send,
    {
        let mut inner = self.inner.lock().unwrap();
        inner.subscribe(subscriber)
    }

    fn unsub(&self, sub_id: usize) {
        let mut inner = self.inner.lock().unwrap();
        inner.unsub(sub_id);
    }

    /// Reset the configuration to defaults
    pub fn use_defaults(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.use_defaults();
    }

    fn use_this_config(&self, cfg: Config) {
        let mut inner = self.inner.lock().unwrap();
        inner.use_this_config(cfg);
    }

    fn overridden(&self, overrides: &wezterm_dynamic::Value) -> Result<ConfigHandle, Error> {
        let mut inner = self.inner.lock().unwrap();
        inner.overridden(overrides)
    }

    /// Use a config that doesn't depend on the user's
    /// environment and is suitable for unit testing
    pub fn use_test(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.use_test();
    }

    /// Reload the configuration
    pub fn reload(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.reload();
    }

    /// Returns a copy of any captured error message.
    /// The error message is not cleared.
    pub fn get_error(&self) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        inner.error.as_ref().cloned()
    }

    pub fn get_warnings_and_errors(&self) -> Vec<String> {
        let mut result = vec![];
        let inner = self.inner.lock().unwrap();
        if let Some(error) = &inner.error {
            result.push(error.clone());
        }
        for warning in &inner.warnings {
            result.push(warning.clone());
        }
        result
    }

    /// Returns any captured error message, and clears
    /// it from the config state.
    #[allow(dead_code)]
    pub fn clear_error(&self) -> Option<String> {
        let mut inner = self.inner.lock().unwrap();
        inner.error.take()
    }
}

#[derive(Clone, Debug)]
pub struct ConfigHandle {
    config: Arc<Config>,
    generation: usize,
}

impl ConfigHandle {
    /// Returns the generation number for the configuration,
    /// allowing consuming code to know whether the config
    /// has been reloading since they last derived some
    /// information from the configuration
    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn default_config() -> Self {
        Self {
            config: Arc::new(Config::default_config()),
            generation: 0,
        }
    }

    pub fn unicode_version(&self) -> UnicodeVersion {
        UnicodeVersion {
            version: self.config.unicode_version,
            ambiguous_are_wide: self.config.treat_east_asian_ambiguous_width_as_wide,
            cell_widths: CellWidth::compile_to_map(self.config.cell_widths.clone()),
        }
    }
}

impl std::ops::Deref for ConfigHandle {
    type Target = Config;
    fn deref(&self) -> &Config {
        &*self.config
    }
}

pub struct LoadedConfig {
    pub config: anyhow::Result<Config>,
    pub file_name: Option<PathBuf>,
    pub lua: Option<mlua::Lua>,
    pub warnings: Vec<String>,
}

fn default_one_point_oh_f64() -> f64 {
    1.0
}

fn default_one_point_oh() -> f32 {
    1.0
}

fn default_true() -> bool {
    true
}
