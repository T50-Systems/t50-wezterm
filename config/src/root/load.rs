fn default_config_with_overrides_applied() -> anyhow::Result<Config> {
    // Cause the default config to be re-evaluated with the overrides applied
    let lua = lua::make_lua_context(Path::new("override")).context("make_lua_context")?;
    let table = mlua::Value::Table(lua.create_table()?);
    let config = Config::apply_overrides_to(&lua, table).context("apply_overrides_to")?;

    let dyn_config = luahelper::lua_value_to_dynamic(config)?;

    let cfg: Config = Config::from_dynamic(
        &dyn_config,
        FromDynamicOptions {
            unknown_fields: UnknownFieldAction::Deny,
            deprecated_fields: UnknownFieldAction::Warn,
        },
    )
    .context("Error converting lua value from overrides to Config struct")?;
    // Compute but discard the key bindings here so that we raise any
    // problems earlier than we use them.
    let _ = cfg.key_bindings();

    cfg.check_consistency().context("check_consistency")?;

    Ok(cfg)
}

pub fn common_init(
    config_file: Option<&OsString>,
    overrides: &[(String, String)],
    skip_config: bool,
) -> anyhow::Result<()> {
    if let Some(config_file) = config_file {
        set_config_file_override(Path::new(config_file));
    } else if skip_config {
        CONFIG_SKIP.store(true, Ordering::Relaxed);
    }

    set_config_overrides(overrides).context("common_init: set_config_overrides")?;
    reload();
    Ok(())
}

pub fn assign_error_callback(cb: ErrorCallback) {
    let mut factory = SHOW_ERROR.lock().unwrap();
    factory.replace(cb);
}

pub fn show_error(err: &str) {
    let factory = SHOW_ERROR.lock().unwrap();
    if let Some(cb) = factory.as_ref() {
        cb(err)
    }
}

pub fn create_user_owned_dirs(p: &Path) -> anyhow::Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true);

    #[cfg(unix)]
    {
        builder.mode(0o700);
    }

    builder.create(p)?;
    Ok(())
}

fn xdg_config_home() -> PathBuf {
    match std::env::var_os("XDG_CONFIG_HOME").map(|s| PathBuf::from(s).join("wezterm")) {
        Some(p) => p,
        None => HOME_DIR.join(".config").join("wezterm"),
    }
}

fn config_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    dirs.push(xdg_config_home());

    #[cfg(unix)]
    if let Some(d) = std::env::var_os("XDG_CONFIG_DIRS") {
        dirs.extend(std::env::split_paths(&d).map(|s| PathBuf::from(s).join("wezterm")));
    }

    dirs
}

pub fn set_config_file_override(path: &Path) {
    CONFIG_FILE_OVERRIDE
        .lock()
        .unwrap()
        .replace(path.to_path_buf());
}

pub fn set_config_overrides(items: &[(String, String)]) -> anyhow::Result<()> {
    *CONFIG_OVERRIDES.lock().unwrap() = items.to_vec();

    let _ = default_config_with_overrides_applied()?;
    Ok(())
}

pub fn is_config_overridden() -> bool {
    CONFIG_SKIP.load(Ordering::Relaxed)
        || !CONFIG_OVERRIDES.lock().unwrap().is_empty()
        || CONFIG_FILE_OVERRIDE.lock().unwrap().is_some()
}

/// Discard the current configuration and replace it with
/// the default configuration
pub fn use_default_configuration() {
    CONFIG.use_defaults();
}

/// Use a config that doesn't depend on the user's
/// environment and is suitable for unit testing
pub fn use_test_configuration() {
    CONFIG.use_test();
}

pub fn use_this_configuration(config: Config) {
    CONFIG.use_this_config(config);
}

/// Returns a handle to the current configuration
pub fn configuration() -> ConfigHandle {
    CONFIG.get()
}

/// Returns a version of the config (loaded from the config file)
/// with some field overridden based on the supplied overrides object.
pub fn overridden_config(overrides: &wezterm_dynamic::Value) -> Result<ConfigHandle, Error> {
    CONFIG.overridden(overrides)
}

pub fn reload() {
    CONFIG.reload();
}

/// If there was an error loading the preferred configuration,
/// return it, otherwise return the current configuration
pub fn configuration_result() -> Result<ConfigHandle, Error> {
    if let Some(error) = CONFIG.get_error() {
        bail!("{}", error);
    }
    Ok(CONFIG.get())
}

/// Returns the combined set of errors + warnings encountered
/// while loading the preferred configuration
pub fn configuration_warnings_and_errors() -> Vec<String> {
    CONFIG.get_warnings_and_errors()
}
