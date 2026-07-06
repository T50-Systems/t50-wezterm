fn config_builder_set_strict_mode<'lua>(
    _lua: &'lua Lua,
    (myself, strict): (Table, bool),
) -> mlua::Result<()> {
    let mt = myself
        .get_metatable()
        .ok_or_else(|| mlua::Error::external("impossible that we have no metatable"))?;
    mt.set("__strict_mode", strict)
}

fn config_builder_index<'lua>(
    _lua: &'lua Lua,
    (myself, key): (Table<'lua>, mlua::Value<'lua>),
) -> mlua::Result<mlua::Value<'lua>> {
    let mt = myself
        .get_metatable()
        .ok_or_else(|| mlua::Error::external("impossible that we have no metatable"))?;
    match mt.get(key.clone()) {
        Ok(value) => Ok(value),
        _ => myself.raw_get(key),
    }
}

fn config_builder_new_index<'lua>(
    lua: &'lua Lua,
    (myself, key, value): (Table, String, Value),
) -> mlua::Result<()> {
    let stub_config = lua.create_table()?;
    stub_config.set(key.clone(), value.clone())?;

    let dvalue = lua_value_to_dynamic(Value::Table(stub_config)).map_err(|e| {
        mlua::Error::FromLuaConversionError {
            from: "table",
            to: "Config",
            message: Some(format!("lua_value_to_dynamic: {e}")),
        }
    })?;

    let mt = myself
        .get_metatable()
        .ok_or_else(|| mlua::Error::external("impossible that we have no metatable"))?;
    let strict = match mt.get("__strict_mode") {
        Ok(Value::Boolean(b)) => b,
        _ => true,
    };

    let options = FromDynamicOptions {
        unknown_fields: if strict {
            UnknownFieldAction::Deny
        } else {
            UnknownFieldAction::Warn
        },
        deprecated_fields: UnknownFieldAction::Warn,
    };

    let config_object = Config::from_dynamic(&dvalue, options).map_err(|e| {
        mlua::Error::FromLuaConversionError {
            from: "table",
            to: "Config",
            message: Some(format!("Config::from_dynamic: {e}")),
        }
    })?;

    match config_object.to_dynamic() {
        DynValue::Object(obj) => {
            match obj.get_by_str(&key) {
                None => {
                    // Show a stack trace to help them figure out where they made
                    // a mistake. This path is taken when they are not in strict
                    // mode, and we want to print some more context after the from_dynamic
                    // impl has logged a warning and suggested alternative field names.
                    let mut message =
                        format!("Attempted to set invalid config option `{key}` at:\n");
                    // Start at frame 1, our caller, as the frame for invoking this
                    // metamethod is not interesting
                    for i in 1.. {
                        if let Some(debug) = lua.inspect_stack(i) {
                            let names = debug.names();
                            let name = names.name;
                            let name_what = names.name_what;

                            let dbg_source = debug.source();
                            let source = dbg_source.source.unwrap_or_default();
                            let func_name = match (name, name_what) {
                                (Some(name), Some(name_what)) => {
                                    format!("{name_what} {name}")
                                }
                                (Some(name), None) => format!("{name}"),
                                _ => "".to_string(),
                            };

                            let line = debug.curr_line();
                            message.push_str(&format!("    [{i}] {source}:{line} {func_name}\n"));
                        } else {
                            break;
                        }
                    }
                    wezterm_dynamic::Error::warn(message);
                }
                Some(_dvalue) => {
                    myself.raw_set(key, value)?;
                }
            };
            Ok(())
        }
        _ => Err(mlua::Error::external(
            "computed config object is, impossibly, not an object",
        )),
    }
}
