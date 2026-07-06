/// Set up a lua context for executing some code.
/// The path to the directory containing the configuration is
/// passed in and is used to pre-set some global values in
/// the environment.
///
/// The `package.path` is configured to search the user's
/// wezterm specific config paths for lua modules, should
/// they choose to `require` additional code from their config.
///
/// A `wezterm` module is registered so that the script can
/// `require "wezterm"` and call into functions provided by
/// wezterm.  The wezterm module contains:
/// * `executable_dir` - the directory containing the wezterm
///   executable.  This is potentially useful for portable
///   installs on Windows.
/// * `config_dir` - the directory containing the wezterm
///   configuration.
/// * `log_error` - a function that logs to stderr (or the server
///   log file for daemonized wezterm).
/// * `target_triple` - the rust compilation target triple.
/// * `version` - the version of the running wezterm instance.
/// * `home_dir` - the path to the user's home directory
///
/// In addition to this, the lua standard library, except for
/// the `debug` module, is also available to the script.
pub fn make_lua_context(config_file: &Path) -> anyhow::Result<Lua> {
    let lua = Lua::new();

    let config_dir = config_file.parent().unwrap_or_else(|| Path::new("/"));

    {
        let globals = lua.globals();
        // This table will be the `wezterm` module in the script
        let wezterm_mod = get_or_create_module(&lua, "wezterm")?;

        let package: Table = globals.get("package").context("get _G.package")?;
        let package_path: String = package.get("path").context("get package.path as String")?;
        let mut path_array: Vec<String> = package_path.split(";").map(|s| s.to_owned()).collect();

        fn prefix_path(array: &mut Vec<String>, path: &Path) {
            array.insert(0, format!("{}/?.lua", path.display()));
            array.insert(1, format!("{}/?/init.lua", path.display()));
        }

        prefix_path(&mut path_array, &crate::HOME_DIR.join(".wezterm"));
        for dir in crate::CONFIG_DIRS.iter() {
            prefix_path(&mut path_array, dir);
        }
        path_array.insert(
            2,
            format!("{}/plugins/?/plugin/init.lua", crate::DATA_DIR.display()),
        );

        if let Ok(exe) = std::env::current_exe() {
            if let Some(path) = exe.parent() {
                wezterm_mod
                    .set(
                        "executable_dir",
                        path.to_str()
                            .ok_or_else(|| anyhow!("current_exe path is not UTF-8"))?,
                    )
                    .context("set wezterm.executable_dir")?;
                if cfg!(windows) {
                    // For a portable windows install, force in this path ahead
                    // of the rest
                    prefix_path(&mut path_array, &path.join("wezterm_modules"));
                }
            }
        }
        let config_file_str = config_file
            .to_str()
            .ok_or_else(|| anyhow!("config file path is not UTF-8"))?;

        // Hook into loader and arrange to watch all require'd files.
        // <https://www.lua.org/manual/5.3/manual.html#pdf-package.searchers>
        // says that the second searcher function is the one that is responsible
        // for loading lua files, so we shim around that and speculatively
        // add the name of the file that it would find (as returned from
        // package.searchpath) to the watch list, then we just call the
        // original implementation.
        lua.load(
            r#"
local orig = package.searchers[2]
package.searchers[2] = function(module)
  local name, err = package.searchpath(module, package.path)
  if name then
    package.loaded.wezterm.add_to_config_reload_watch_list(name)
  end
  return orig(module)
end
        "#,
        )
        .set_name("=searcher")
        .eval::<()>()
        .context("replace package.searchers")?;

        wezterm_mod.set(
            "config_builder",
            lua.create_function(|lua, _: ()| {
                let config = lua.create_table()?;
                let mt = lua.create_table()?;

                mt.set("__index", lua.create_function(config_builder_index)?)?;
                mt.set("__newindex", lua.create_function(config_builder_new_index)?)?;
                mt.set(
                    "set_strict_mode",
                    lua.create_function(config_builder_set_strict_mode)?,
                )?;

                config.set_metatable(Some(mt));

                Ok(config)
            })?,
        )?;

        wezterm_mod.set(
            "reload_configuration",
            lua.create_function(|_, _: ()| {
                crate::reload();
                Ok(())
            })?,
        )?;
        wezterm_mod
            .set("config_file", config_file_str)
            .context("set wezterm.config_file")?;
        wezterm_mod
            .set(
                "config_dir",
                config_dir
                    .to_str()
                    .ok_or_else(|| anyhow!("config dir path is not UTF-8"))?,
            )
            .context("set wezterm.config_dir")?;

        lua.set_named_registry_value("wezterm-watch-paths", Vec::<String>::new())?;
        wezterm_mod.set(
            "add_to_config_reload_watch_list",
            lua.create_function(add_to_config_reload_watch_list)?,
        )?;

        wezterm_mod.set("target_triple", crate::wezterm_target_triple())?;
        wezterm_mod.set("version", crate::wezterm_version())?;
        wezterm_mod.set("home_dir", crate::HOME_DIR.to_str())?;
        wezterm_mod.set(
            "running_under_wsl",
            lua.create_function(|_, ()| Ok(crate::running_under_wsl()))?,
        )?;

        wezterm_mod.set(
            "default_wsl_domains",
            lua.create_function(|_, ()| Ok(crate::WslDomain::default_domains()))?,
        )?;

        wezterm_mod.set("font", lua.create_function(font)?)?;
        wezterm_mod.set(
            "font_with_fallback",
            lua.create_function(font_with_fallback)?,
        )?;
        wezterm_mod.set("hostname", lua.create_function(hostname)?)?;
        wezterm_mod.set("action", luahelper::enumctor::Enum::<KeyAssignment>::new())?;
        wezterm_mod.set(
            "has_action",
            lua.create_function(|_lua, name: String| {
                Ok(KeyAssignment::variants().contains(&name.as_str()))
            })?,
        )?;

        lua.set_named_registry_value(LUA_REGISTRY_USER_CALLBACK_COUNT, 0)?;
        wezterm_mod.set("action_callback", lua.create_function(action_callback)?)?;
        wezterm_mod.set("exec_domain", lua.create_function(exec_domain)?)?;

        wezterm_mod.set("utf16_to_utf8", lua.create_function(utf16_to_utf8)?)?;
        wezterm_mod.set("split_by_newlines", lua.create_function(split_by_newlines)?)?;
        wezterm_mod.set("on", lua.create_function(register_event)?)?;
        wezterm_mod.set("emit", lua.create_async_function(emit_event)?)?;
        wezterm_mod.set("shell_join_args", lua.create_function(shell_join_args)?)?;
        wezterm_mod.set("shell_quote_arg", lua.create_function(shell_quote_arg)?)?;
        wezterm_mod.set("shell_split", lua.create_function(shell_split)?)?;

        wezterm_mod.set(
            "default_hyperlink_rules",
            lua.create_function(move |lua, ()| {
                let rules = crate::config::default_hyperlink_rules();
                Ok(to_lua(lua, rules))
            })?,
        )?;

        // Define our own os.getenv function that knows how to resolve current
        // environment values from eg: the registry on Windows, or for
        // the current SHELL value on unix, even if the user has changed
        // those values since wezterm was started
        get_or_create_module(&lua, "os")?.set("getenv", lua.create_function(getenv)?)?;

        package
            .set("path", path_array.join(";"))
            .context("assign package.path")?;
    }

    for func in SETUP_FUNCS.lock().unwrap().iter() {
        func(&lua).context("calling SETUP_FUNCS")?;
    }

    Ok(lua)
}
