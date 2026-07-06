fn split_by_newlines<'lua>(_: &'lua Lua, text: String) -> mlua::Result<Vec<String>> {
    Ok(text
        .lines()
        .map(|s| {
            // Ungh, `str.lines()` is supposed to split by `\n` or `\r\n`, but I've
            // found that it is necessary to have an additional trim here in order
            // to actually remove the `\r`.
            s.trim_end_matches('\r').to_string()
        })
        .collect())
}

/// This implements `wezterm.on`, whose goal is to register an event handler
/// callback.
/// The callback function may return `false` to prevent other handlers from
/// triggering.  The `false` return means "prevent the default action",
/// and thus, depending on the semantics of the emitted event, can be used
/// to override rather augment built-in behavior.
///
/// To allow the default action you can omit a return statement, or
/// explicitly return `true`.
///
/// The arguments to the handler are passed through from the corresponding
/// `wezterm.emit` call.
///
/// ```lua
/// wezterm.on("event-name", function(arg1, arg2)
///   -- do something
///   return false -- if you want to prevent other handlers running
/// end);
///
/// wezterm.emit("event-name", "foo", "bar");
/// ```
pub fn register_event<'lua>(
    lua: &'lua Lua,
    (name, func): (String, mlua::Function),
) -> mlua::Result<()> {
    let decorated_name = format!("wezterm-event-{}", name);
    let tbl: mlua::Value = lua.named_registry_value(&decorated_name)?;
    match tbl {
        mlua::Value::Nil => {
            let tbl = lua.create_table()?;
            tbl.set(1, func)?;
            lua.set_named_registry_value(&decorated_name, tbl)?;
            Ok(())
        }
        mlua::Value::Table(tbl) => {
            let len = tbl.raw_len();
            tbl.set(len + 1, func)?;
            Ok(())
        }
        _ => Err(mlua::Error::external(anyhow!(
            "registry key for {} has invalid type",
            decorated_name
        ))),
    }
}

const IS_EVENT: &str = "wezterm-is-event-emission";

/// Returns true if the current lua context is being called as part
/// of an emit_event call.
pub fn is_event_emission<'lua>(lua: &'lua Lua) -> mlua::Result<bool> {
    lua.named_registry_value(IS_EVENT)
}

/// This implements `wezterm.emit`.
/// The first parameter to emit is the name of a signal that may or may not
/// have previously been registered via `wezterm.on`.
/// `wezterm.emit` will call each of the registered handlers in the order
/// that they were registered and pass the remainder of the `emit` arguments
/// to those handler functions.
/// If a handler returns `false` then `wezterm.emit` will stop calling
/// any additional handlers and then return `false`.
/// Otherwise, once all handlers have been called and none of them returned
/// `false`, `wezterm.emit` will return `true`.
/// The return value indicates to the caller whether the default action
/// should take place.
pub async fn emit_event<'lua>(
    lua: &'lua Lua,
    (name, args): (String, mlua::MultiValue<'lua>),
) -> mlua::Result<bool> {
    lua.set_named_registry_value(IS_EVENT, true)?;

    let decorated_name = format!("wezterm-event-{}", name);
    let tbl: mlua::Value = lua.named_registry_value(&decorated_name)?;
    match tbl {
        mlua::Value::Table(tbl) => {
            for func in tbl.sequence_values::<mlua::Function>() {
                let func = func?;
                match func.call_async(args.clone()).await? {
                    mlua::Value::Boolean(b) if !b => {
                        // Default action prevented
                        return Ok(false);
                    }
                    _ => {
                        // Continue with other handlers
                    }
                }
            }
            Ok(true)
        }
        _ => Ok(true),
    }
}

pub fn emit_sync_callback<'lua, A>(
    lua: &'lua Lua,
    (name, args): (String, A),
) -> mlua::Result<mlua::Value<'lua>>
where
    A: IntoLuaMulti<'lua>,
{
    let decorated_name = format!("wezterm-event-{}", name);
    let tbl: mlua::Value = lua.named_registry_value(&decorated_name)?;
    match tbl {
        mlua::Value::Table(tbl) => {
            for func in tbl.sequence_values::<mlua::Function>() {
                let func = func?;
                return func.call(args);
            }
            Ok(mlua::Value::Nil)
        }
        _ => Ok(mlua::Value::Nil),
    }
}

pub async fn emit_async_callback<'lua, A>(
    lua: &'lua Lua,
    (name, args): (String, A),
) -> mlua::Result<mlua::Value<'lua>>
where
    A: IntoLuaMulti<'lua>,
{
    let decorated_name = format!("wezterm-event-{}", name);
    let tbl: mlua::Value = lua.named_registry_value(&decorated_name)?;
    match tbl {
        mlua::Value::Table(tbl) => {
            for func in tbl.sequence_values::<mlua::Function>() {
                let func = func?;
                return func.call_async(args).await;
            }
            Ok(mlua::Value::Nil)
        }
        _ => Ok(mlua::Value::Nil),
    }
}

/// Ungh: https://github.com/microsoft/WSL/issues/4456
fn utf16_to_utf8<'lua>(_: &'lua Lua, text: mlua::String) -> mlua::Result<String> {
    let bytes = text.as_bytes();

    if bytes.len() % 2 != 0 {
        return Err(mlua::Error::external(anyhow!(
            "input data has odd length, cannot be utf16"
        )));
    }

    // This is "safe" because we checked that the length seems reasonable,
    // and our new slice is within those same bounds.
    let wide: &[u16] =
        unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u16, bytes.len() / 2) };

    String::from_utf16(wide).map_err(mlua::Error::external)
}

pub fn add_to_config_reload_watch_list<'lua>(
    lua: &'lua Lua,
    args: Variadic<String>,
) -> mlua::Result<()> {
    let mut watch_paths: Vec<String> = lua.named_registry_value("wezterm-watch-paths")?;
    watch_paths.extend_from_slice(&args);
    lua.set_named_registry_value("wezterm-watch-paths", watch_paths)?;
    Ok(())
}
