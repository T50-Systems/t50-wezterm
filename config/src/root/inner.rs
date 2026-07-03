struct ConfigInner {
    config: Arc<Config>,
    error: Option<String>,
    warnings: Vec<String>,
    generation: usize,
    watcher: Option<notify::RecommendedWatcher>,
    subscribers: HashMap<usize, Box<dyn Fn() -> bool + Send>>,
}

impl ConfigInner {
    fn new() -> Self {
        Self {
            config: Arc::new(Config::default_config()),
            error: None,
            warnings: vec![],
            generation: 0,
            watcher: None,
            subscribers: HashMap::new(),
        }
    }

    fn subscribe<F>(&mut self, subscriber: F) -> usize
    where
        F: Fn() -> bool + 'static + Send,
    {
        static SUB_ID: AtomicUsize = AtomicUsize::new(0);
        let sub_id = SUB_ID.fetch_add(1, Ordering::Relaxed);
        self.subscribers.insert(sub_id, Box::new(subscriber));
        sub_id
    }

    fn unsub(&mut self, sub_id: usize) {
        self.subscribers.remove(&sub_id);
    }

    fn notify(&mut self) {
        self.subscribers.retain(|_, notify| notify());
    }

    fn watch_path(&mut self, path: PathBuf) {
        if self.watcher.is_none() {
            let (tx, rx) = std::sync::mpsc::channel();
            const DELAY: Duration = Duration::from_millis(200);
            let watcher = notify::recommended_watcher(tx).unwrap();
            let path = path.clone();

            std::thread::spawn(move || {
                // block until we get an event
                use notify::EventKind;

                fn extract_path(event: notify::Event) -> Vec<PathBuf> {
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                            event.paths
                        }
                        _ => vec![],
                    }
                }

                while let Ok(event) = rx.recv() {
                    log::debug!("event:{:?}", event);
                    match event {
                        Ok(event) => {
                            let mut paths = extract_path(event);
                            if !paths.is_empty() {
                                // Grace period to allow events to settle
                                std::thread::sleep(DELAY);
                                // Drain any other immediately ready events
                                while let Ok(Ok(event)) = rx.try_recv() {
                                    paths.append(&mut extract_path(event));
                                }
                                paths.sort();
                                paths.dedup();
                                log::debug!("paths {:?} changed, reload config", path);
                                reload();
                            }
                        }
                        Err(_) => {
                            reload();
                        }
                    }
                }
            });
            self.watcher.replace(watcher);
        }
        if let Some(watcher) = self.watcher.as_mut() {
            use notify::Watcher;
            watcher
                .watch(&path, notify::RecursiveMode::NonRecursive)
                .ok();
        }
    }

    fn accumulate_watch_paths(lua: &Lua, watch_paths: &mut Vec<PathBuf>) {
        if let Ok(mlua::Value::Table(tbl)) = lua.named_registry_value("wezterm-watch-paths") {
            for path in tbl.sequence_values::<String>() {
                if let Ok(path) = path {
                    watch_paths.push(PathBuf::from(path));
                }
            }
        }
    }

    /// Attempt to load the user's configuration.
    /// On success, clear any error and replace the current
    /// configuration.
    /// On failure, retain the existing configuration but
    /// replace any captured error message.
    fn reload(&mut self) {
        let LoadedConfig {
            config,
            file_name,
            lua,
            warnings,
        } = Config::load();

        self.warnings = warnings;

        // Before we process the success/failure, extract and update
        // any paths that we should be watching
        let mut watch_paths = vec![];
        if let Some(path) = file_name {
            // Let's also watch the parent directory for folks that do
            // things with symlinks:
            if let Some(parent) = path.parent() {
                // But avoid watching the home dir itself, so that we
                // don't keep reloading every time something in the
                // home dir changes!
                // <https://github.com/wezterm/wezterm/issues/1895>
                if parent != &*HOME_DIR {
                    watch_paths.push(parent.to_path_buf());
                }
            }
            watch_paths.push(path);
        }
        if let Some(lua) = &lua {
            ConfigInner::accumulate_watch_paths(lua, &mut watch_paths);
        }

        match config {
            Ok(config) => {
                self.config = Arc::new(config);
                self.error.take();
                self.generation += 1;

                // If we loaded a user config, publish this latest version of
                // the lua state to the LUA_PIPE.  This allows a subsequent
                // call to `with_lua_config` to reference this lua context
                // even though we are (probably) resolving this from a background
                // reloading thread.
                if let Some(lua) = lua {
                    LUA_PIPE.sender.try_send(lua).ok();
                }
                log::debug!("Reloaded configuration! generation={}", self.generation);
            }
            Err(err) => {
                let err = format!("{:#}", err);
                if self.generation > 0 {
                    // Only generate the message for an actual reload
                    show_error(&err);
                }
                self.error.replace(err);
            }
        }

        self.notify();
        if self.config.automatically_reload_config {
            for path in watch_paths {
                self.watch_path(path);
            }
        }
    }

    /// Discard the current configuration and any recorded
    /// error message; replace them with the default
    /// configuration
    fn use_defaults(&mut self) {
        self.config = Arc::new(Config::default_config());
        self.error.take();
        self.generation += 1;
    }

    fn use_this_config(&mut self, cfg: Config) {
        self.config = Arc::new(cfg);
        self.error.take();
        self.generation += 1;
    }

    fn overridden(&mut self, overrides: &wezterm_dynamic::Value) -> Result<ConfigHandle, Error> {
        let config = Config::load_with_overrides(overrides);
        Ok(ConfigHandle {
            config: Arc::new(config.config?),
            generation: self.generation,
        })
    }

    fn use_test(&mut self) {
        let mut config = Config::default_config();
        config.font_locator = FontLocatorSelection::ConfigDirsOnly;
        let exe_name = std::env::current_exe().unwrap();
        let exe_dir = exe_name.parent().unwrap();
        config.font_dirs.push(exe_dir.join("../../../assets/fonts"));
        // If we're building for a specific target, the dir
        // level is one deeper.
        #[cfg(target_os = "macos")]
        config
            .font_dirs
            .push(exe_dir.join("../../../../assets/fonts"));
        // Specify the same DPI used on non-mac systems so
        // that we have consistent values regardless of the
        // operating system that we're running tests on
        config.dpi.replace(96.0);
        self.config = Arc::new(config);
        self.error.take();
        self.generation += 1;
    }
}
