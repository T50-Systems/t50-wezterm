/// On macOS, both Menlo and Monaco fonts have ligatures for `fi` that
/// take effect for words like `find` and which are a source of
/// confusion/annoyance and issues filed on Github.
/// Let's default to disabling ligatures for these fonts unless
/// the user has explicitly specified harfbuzz_features.
/// <https://github.com/wezterm/wezterm/issues/1736>
/// <https://github.com/wezterm/wezterm/issues/1786>
fn disable_ligatures_for_menlo_or_monaco(mut attrs: FontAttributes) -> FontAttributes {
    if attrs.harfbuzz_features.is_none() && (attrs.family == "Menlo" || attrs.family == "Monaco") {
        attrs.harfbuzz_features = Some(vec![
            "kern".to_string(),
            "clig".to_string(),
            "liga=0".to_string(),
        ]);
    }
    attrs
}

/// Given a simple font family name, returns a text style instance.
/// The second optional argument is a list of the other TextStyle
/// fields, which at the time of writing includes only the
/// `foreground` color that can be used to force a particular
/// color to be used for this text style.
///
/// `wezterm.font("foo", {foreground="tomato"})`
/// yields:
/// `{ font = {{ family = "foo" }}, foreground="tomato"}`
fn font<'lua>(
    _lua: &'lua Lua,
    (mut attrs, map_defaults): (LuaFontAttributes, Option<TextStyleAttributes>),
) -> mlua::Result<TextStyle> {
    let mut text_style = TextStyle::default();
    text_style.font.clear();

    if let Some(map_defaults) = map_defaults {
        attrs.weight = match map_defaults.bold {
            Some(true) => FontWeight::BOLD,
            Some(false) => FontWeight::REGULAR,
            None => map_defaults.weight.unwrap_or(FontWeight::REGULAR),
        };
        attrs.stretch = map_defaults.stretch;
        attrs.style = map_defaults.style;
        text_style.foreground = map_defaults.foreground;
    }

    text_style
        .font
        .push(disable_ligatures_for_menlo_or_monaco(FontAttributes {
            family: attrs.family,
            stretch: attrs.stretch,
            weight: attrs.weight,
            style: attrs.style,
            is_fallback: false,
            is_synthetic: false,
            harfbuzz_features: attrs.harfbuzz_features,
            freetype_load_target: attrs.freetype_load_target,
            freetype_render_target: attrs.freetype_render_target,
            freetype_load_flags: match attrs.freetype_load_flags {
                Some(flags) => Some(TryFrom::try_from(flags).map_err(mlua::Error::external)?),
                None => None,
            },
            scale: attrs.scale,
            assume_emoji_presentation: attrs.assume_emoji_presentation,
        }));

    Ok(text_style)
}

/// Given a list of font family names in order of preference, return a
/// text style instance for that font configuration.
///
/// `wezterm.font_with_fallback({"Operator Mono", "DengXian"})`
///
/// The second optional argument is a list of other TextStyle fields,
/// as described by the `wezterm.font` documentation.
fn font_with_fallback<'lua>(
    _lua: &'lua Lua,
    (fallback, map_defaults): (Vec<LuaFontAttributes>, Option<TextStyleAttributes>),
) -> mlua::Result<TextStyle> {
    let mut text_style = TextStyle::default();
    text_style.font.clear();

    for (idx, mut attrs) in fallback.into_iter().enumerate() {
        if let Some(map_defaults) = &map_defaults {
            attrs.weight = match map_defaults.bold {
                Some(true) => FontWeight::BOLD,
                Some(false) => FontWeight::REGULAR,
                None => map_defaults.weight.unwrap_or(FontWeight::REGULAR),
            };
            attrs.stretch = map_defaults.stretch;
            attrs.style = map_defaults.style;
            text_style.foreground = map_defaults.foreground;
        }

        text_style
            .font
            .push(disable_ligatures_for_menlo_or_monaco(FontAttributes {
                family: attrs.family,
                stretch: attrs.stretch,
                weight: attrs.weight,
                style: attrs.style,
                is_fallback: idx != 0,
                is_synthetic: false,
                harfbuzz_features: attrs.harfbuzz_features,
                freetype_load_target: attrs.freetype_load_target,
                freetype_render_target: attrs.freetype_render_target,
                freetype_load_flags: match attrs.freetype_load_flags {
                    Some(flags) => Some(TryFrom::try_from(flags).map_err(mlua::Error::external)?),
                    None => None,
                },
                scale: attrs.scale,
                assume_emoji_presentation: attrs.assume_emoji_presentation,
            }));
    }

    Ok(text_style)
}

pub fn wrap_callback<'lua>(lua: &'lua Lua, callback: mlua::Function) -> mlua::Result<String> {
    let callback_count: i32 = lua.named_registry_value(LUA_REGISTRY_USER_CALLBACK_COUNT)?;
    let user_event_id = format!("user-defined-{}", callback_count);
    lua.set_named_registry_value(LUA_REGISTRY_USER_CALLBACK_COUNT, callback_count + 1)?;
    register_event(lua, (user_event_id.clone(), callback))?;
    Ok(user_event_id)
}

fn action_callback<'lua>(lua: &'lua Lua, callback: mlua::Function) -> mlua::Result<KeyAssignment> {
    let user_event_id = wrap_callback(lua, callback)?;
    Ok(KeyAssignment::EmitEvent(user_event_id))
}

fn exec_domain<'lua>(
    lua: &'lua Lua,
    (name, fixup_command, label): (String, mlua::Function, Option<mlua::Value>),
) -> mlua::Result<ExecDomain> {
    let fixup_command = {
        let event_name = format!("exec-domain-{name}");
        register_event(lua, (event_name.clone(), fixup_command))?;
        event_name
    };

    let label = match label {
        Some(Value::Function(callback)) => {
            let event_name = format!("exec-domain-{name}-label");
            register_event(lua, (event_name.clone(), callback))?;
            Some(ValueOrFunc::Func(event_name))
        }
        Some(Value::String(value)) => Some(ValueOrFunc::Value(lua_value_to_dynamic(
            Value::String(value),
        )?)),
        Some(_) => {
            return Err(mlua::Error::external(
                "label function parameter must be either a string or a lua function",
            ))
        }
        None => None,
    };
    Ok(ExecDomain {
        name,
        fixup_command,
        label,
    })
}
