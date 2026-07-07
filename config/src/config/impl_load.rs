use super::*;
use super::types::PathPossibility;

impl Config {
    pub fn load() -> LoadedConfig {
        Self::load_with_overrides(&wezterm_dynamic::Value::default())
    }

    /// It is relatively expensive to parse all the ssh config files,
    /// so we defer producing the default list until someone explicitly
    /// asks for it
    pub fn ssh_domains(&self) -> Vec<SshDomain> {
        if let Some(domains) = &self.ssh_domains {
            domains.clone()
        } else {
            SshDomain::default_domains()
        }
    }

    pub fn wsl_domains(&self) -> Vec<WslDomain> {
        if let Some(domains) = &self.wsl_domains {
            domains.clone()
        } else {
            WslDomain::default_domains()
        }
    }

    pub fn update_ulimit(&self) -> anyhow::Result<()> {
        #[cfg(unix)]
        {
            use nix::sys::resource::{getrlimit, rlim_t, setrlimit, Resource};
            use std::convert::TryInto;

            let (no_file_soft, no_file_hard) = getrlimit(Resource::RLIMIT_NOFILE)?;

            let ulimit_nofile: rlim_t = self.ulimit_nofile.try_into().with_context(|| {
                format!(
                    "ulimit_nofile value {} is out of range for this system",
                    self.ulimit_nofile
                )
            })?;

            if no_file_soft < ulimit_nofile {
                setrlimit(
                    Resource::RLIMIT_NOFILE,
                    ulimit_nofile.min(no_file_hard),
                    no_file_hard,
                )
                .with_context(|| {
                    format!(
                        "raise RLIMIT_NOFILE from {no_file_soft} to ulimit_nofile {}",
                        ulimit_nofile
                    )
                })?;
            }
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            use nix::sys::resource::{getrlimit, rlim_t, setrlimit, Resource};
            use std::convert::TryInto;

            let (nproc_soft, nproc_hard) = getrlimit(Resource::RLIMIT_NPROC)?;

            let ulimit_nproc: rlim_t = self.ulimit_nproc.try_into().with_context(|| {
                format!(
                    "ulimit_nproc value {} is out of range for this system",
                    self.ulimit_nproc
                )
            })?;

            if nproc_soft < ulimit_nproc {
                setrlimit(
                    Resource::RLIMIT_NPROC,
                    ulimit_nproc.min(nproc_hard),
                    nproc_hard,
                )
                .with_context(|| {
                    format!(
                        "raise RLIMIT_NPROC from {nproc_soft} to ulimit_nproc {}",
                        ulimit_nproc
                    )
                })?;
            }
        }

        Ok(())
    }

    pub fn load_with_overrides(overrides: &wezterm_dynamic::Value) -> LoadedConfig {
        // Note that the directories crate has methods for locating project
        // specific config directories, but only returns one of them, not
        // multiple.  In addition, it spawns a lot of subprocesses,
        // so we do this bit "by-hand"

        let mut paths = vec![PathPossibility::optional(HOME_DIR.join(".wezterm.lua"))];
        for dir in CONFIG_DIRS.iter() {
            paths.push(PathPossibility::optional(dir.join("wezterm.lua")))
        }

        if cfg!(windows) {
            // On Windows, a common use case is to maintain a thumb drive
            // with a set of portable tools that don't need to be installed
            // to run on a target system.  In that scenario, the user would
            // like to run with the config from their thumbdrive because
            // either the target system won't have any config, or will have
            // the config of another user.
            // So we prioritize that here: if there is a config in the same
            // dir as the executable that will take precedence.
            if let Ok(exe_name) = std::env::current_exe() {
                if let Some(exe_dir) = exe_name.parent() {
                    paths.insert(0, PathPossibility::optional(exe_dir.join("wezterm.lua")));
                }
            }
        }
        if let Some(path) = std::env::var_os("WEZTERM_CONFIG_FILE") {
            log::trace!("Note: WEZTERM_CONFIG_FILE is set in the environment");
            paths.insert(0, PathPossibility::required(path.into()));
        }

        if let Some(path) = CONFIG_FILE_OVERRIDE.lock().unwrap().as_ref() {
            log::trace!("Note: config file override is set");
            paths.insert(0, PathPossibility::required(path.clone()));
        }

        for path_item in &paths {
            if CONFIG_SKIP.load(Ordering::Relaxed) {
                break;
            }

            match Self::try_load(path_item, overrides) {
                Err(err) => {
                    return LoadedConfig {
                        config: Err(err),
                        file_name: Some(path_item.path.clone()),
                        lua: None,
                        warnings: vec![],
                    }
                }
                Ok(None) => continue,
                Ok(Some(loaded)) => return loaded,
            }
        }

        // We didn't find (or were asked to skip) a wezterm.lua file, so
        // update the environment to make it simpler to understand this
        // state.
        std::env::remove_var("WEZTERM_CONFIG_FILE");
        std::env::remove_var("WEZTERM_CONFIG_DIR");

        match Self::try_default() {
            Err(err) => LoadedConfig {
                config: Err(err),
                file_name: None,
                lua: None,
                warnings: vec![],
            },
            Ok(cfg) => cfg,
        }
    }

    pub fn try_default() -> anyhow::Result<LoadedConfig> {
        let (config, warnings) =
            wezterm_dynamic::Error::capture_warnings(|| -> anyhow::Result<Config> {
                Ok(default_config_with_overrides_applied()?.compute_extra_defaults(None))
            });

        Ok(LoadedConfig {
            config: Ok(config?),
            file_name: None,
            lua: Some(make_lua_context(Path::new(""))?),
            warnings,
        })
    }

    fn try_load(
        path_item: &PathPossibility,
        overrides: &wezterm_dynamic::Value,
    ) -> anyhow::Result<Option<LoadedConfig>> {
        let p = path_item.path.as_path();
        log::trace!("consider config: {}", p.display());
        let mut file = match std::fs::File::open(p) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound if !path_item.is_required => return Ok(None),
                _ => anyhow::bail!("Error opening {}: {}", p.display(), err),
            },
        };

        let mut s = String::new();
        file.read_to_string(&mut s)?;
        let lua = make_lua_context(p)?;

        let (config, warnings) =
            wezterm_dynamic::Error::capture_warnings(|| -> anyhow::Result<Config> {
                let cfg: Config;

                let config: mlua::Value = smol::block_on(
                    // Skip a potential BOM that Windows software may have placed in the
                    // file. Note that we can't catch this happening for files that are
                    // imported via the lua require function.
                    lua.load(s.trim_start_matches('\u{FEFF}'))
                        .set_name(p.to_string_lossy())
                        .eval_async(),
                )?;
                let config = Config::apply_overrides_to(&lua, config)?;
                let config = Config::apply_overrides_obj_to(&lua, config, overrides)?;
                cfg = Config::from_lua(config, &lua).with_context(|| {
                    format!(
                        "Error converting lua value returned by script {} to Config struct",
                        p.display()
                    )
                })?;
                cfg.check_consistency()?;

                // Compute but discard the key bindings here so that we raise any
                // problems earlier than we use them.
                let _ = cfg.key_bindings();

                std::env::set_var("WEZTERM_CONFIG_FILE", p);
                if let Some(dir) = p.parent() {
                    std::env::set_var("WEZTERM_CONFIG_DIR", dir);
                }
                Ok(cfg)
            });
        let cfg = config?;

        Ok(Some(LoadedConfig {
            config: Ok(cfg.compute_extra_defaults(Some(p))),
            file_name: Some(p.to_path_buf()),
            lua: Some(lua),
            warnings,
        }))
    }

    pub(crate) fn apply_overrides_obj_to<'l>(
        lua: &'l mlua::Lua,
        mut config: mlua::Value<'l>,
        overrides: &wezterm_dynamic::Value,
    ) -> anyhow::Result<mlua::Value<'l>> {
        // config may be a table, or it may be a config builder.
        // We'll leave it up to lua to call the appropriate
        // index function as managing that from Rust is a PITA.
        let setter: mlua::Function = lua
            .load(
                r#"
                    return function(config, key, value)
                        config[key] = value;
                        return config;
                    end
                    "#,
            )
            .eval()?;

        match overrides {
            wezterm_dynamic::Value::Object(obj) => {
                for (key, value) in obj {
                    let key = luahelper::dynamic_to_lua_value(lua, key.clone())?;
                    let value = luahelper::dynamic_to_lua_value(lua, value.clone())?;
                    config = setter.call((config, key, value))?;
                }
                Ok(config)
            }
            _ => Ok(config),
        }
    }

    pub(crate) fn apply_overrides_to<'l>(
        lua: &'l mlua::Lua,
        mut config: mlua::Value<'l>,
    ) -> anyhow::Result<mlua::Value<'l>> {
        let overrides = CONFIG_OVERRIDES.lock().unwrap();
        for (key, value) in &*overrides {
            if value == "nil" {
                // Literal nil as the value is the same as not specifying the value.
                // We special case this here as we want to explicitly check for
                // the value evaluating as nil, as can happen in the case where the
                // user specifies something like: `--config term=xterm`.
                // The RHS references a global that doesn't exist and evaluates as
                // nil. We want to raise this as an error.
                continue;
            }
            let literal = value.escape_debug();
            let code = format!(
                r#"
                local wezterm = require 'wezterm';
                local value = {value};
                if value == nil then
                    error("{literal} evaluated as nil. Check for missing quotes or other syntax issues")
                end
                config.{key} = value;
                return config;
                "#,
            );
            let chunk = lua.load(&code);
            let chunk = chunk.set_name(format!("--config {}={}", key, value));
            lua.globals().set("config", config.clone())?;
            log::debug!("Apply {}={} to config", key, value);
            config = chunk.eval()?;
        }
        Ok(config)
    }

    /// Check for logical conflicts in the config
    pub fn check_consistency(&self) -> anyhow::Result<()> {
        self.check_domain_consistency()?;
        Ok(())
    }

    fn check_domain_consistency(&self) -> anyhow::Result<()> {
        let mut domains = HashMap::new();

        let mut check_domain = |name: &str, kind: &str| {
            if let Some(exists) = domains.get(name) {
                anyhow::bail!(
                    "{kind} with name \"{name}\" conflicts with \
                     another existing {exists} with the same name"
                );
            }
            domains.insert(name.to_string(), kind.to_string());
            Ok(())
        };

        for d in &self.unix_domains {
            check_domain(&d.name, "unix domain")?;
        }
        if let Some(domains) = &self.ssh_domains {
            for d in domains {
                check_domain(&d.name, "ssh domain")?;
            }
        }
        for d in &self.exec_domains {
            check_domain(&d.name, "exec domain")?;
        }
        if let Some(domains) = &self.wsl_domains {
            for d in domains {
                check_domain(&d.name, "wsl domain")?;
            }
        }
        for d in &self.tls_clients {
            check_domain(&d.name, "tls domain")?;
        }
        Ok(())
    }
}
