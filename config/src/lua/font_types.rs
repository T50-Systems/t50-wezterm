/// Resolve an environment variable.
/// Lean on CommandBuilder's ability to update to current values of certain
/// environment variables that may be adjusted via the registry or implicitly
/// via eg: chsh (SHELL).
fn getenv<'lua>(_: &'lua Lua, env: String) -> mlua::Result<Option<String>> {
    let cmd = CommandBuilder::new_default_prog();
    match cmd.get_env(&env) {
        Some(s) => match s.to_str() {
            Some(s) => Ok(Some(s.to_string())),
            None => Err(mlua::Error::external(format!(
                "env var {env} is not representable as UTF-8"
            ))),
        },
        None => Ok(None),
    }
}

fn shell_split<'lua>(_: &'lua Lua, line: String) -> mlua::Result<Vec<String>> {
    shlex::split(&line).ok_or_else(|| {
        mlua::Error::external(format!("cannot tokenize `{line}` using posix shell rules"))
    })
}

fn shell_join_args<'lua>(_: &'lua Lua, args: Vec<String>) -> mlua::Result<String> {
    Ok(shlex::try_join(args.iter().map(|arg| arg.as_ref())).map_err(mlua::Error::external)?)
}

fn shell_quote_arg<'lua>(_: &'lua Lua, arg: String) -> mlua::Result<String> {
    Ok(shlex::try_quote(&arg)
        .map_err(mlua::Error::external)?
        .into_owned())
}

/// Returns the system hostname.
/// Errors may occur while retrieving the hostname from the system,
/// or if the hostname isn't a UTF-8 string.
fn hostname<'lua>(_: &'lua Lua, _: ()) -> mlua::Result<String> {
    let hostname = hostname::get().map_err(mlua::Error::external)?;
    match hostname.to_str() {
        Some(hostname) => Ok(hostname.to_owned()),
        None => Err(mlua::Error::external(anyhow!("hostname isn't UTF-8"))),
    }
}

#[derive(Debug, Default, FromDynamic, ToDynamic, Clone, PartialEq, Eq, Hash)]
struct TextStyleAttributes {
    /// Whether the font should be a bold variant
    #[dynamic(default)]
    pub bold: Option<bool>,
    #[dynamic(default)]
    pub weight: Option<FontWeight>,
    #[dynamic(default)]
    pub stretch: FontStretch,
    /// Whether the font should be an italic variant
    #[dynamic(default)]
    pub style: FontStyle,
    // Ideally we'd simply use serde's aliasing functionality on the `style`
    // field to support backwards compatibility, but aliases are invisible
    // to serde_lua, so we do a little fixup here ourselves in our from_lua impl.
    italic: Option<bool>,
    /// If set, when rendering text that is set to the default
    /// foreground color, use this color instead.  This is most
    /// useful in a `[[font_rules]]` section to implement changing
    /// the text color for eg: bold text.
    pub foreground: Option<RgbaColor>,
}
impl<'lua> FromLua<'lua> for TextStyleAttributes {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> Result<Self, mlua::Error> {
        let mut attr: TextStyleAttributes = from_lua_value_dynamic(value)?;
        if let Some(italic) = attr.italic.take() {
            attr.style = if italic {
                FontStyle::Italic
            } else {
                FontStyle::Normal
            };
        }
        Ok(attr)
    }
}

#[derive(Debug, Default, FromDynamic, ToDynamic, Clone, PartialEq, Eq, Hash)]
struct LuaFontAttributes {
    /// The font family name
    pub family: String,
    /// Whether the font should be a bold variant
    #[dynamic(default)]
    pub weight: FontWeight,
    #[dynamic(default)]
    pub stretch: FontStretch,
    /// Whether the font should be an italic variant
    #[dynamic(default)]
    pub style: FontStyle,
    // Ideally we'd simply use serde's aliasing functionality on the `style`
    // field to support backwards compatibility, but aliases are invisible
    // to serde_lua, so we do a little fixup here ourselves in our from_lua impl.
    #[dynamic(default)]
    italic: Option<bool>,

    #[dynamic(default)]
    pub harfbuzz_features: Option<Vec<String>>,
    #[dynamic(default)]
    pub freetype_load_target: Option<FreeTypeLoadTarget>,
    #[dynamic(default)]
    pub freetype_render_target: Option<FreeTypeLoadTarget>,
    #[dynamic(default)]
    pub freetype_load_flags: Option<String>,
    #[dynamic(default)]
    pub scale: Option<NotNan<f64>>,
    #[dynamic(default)]
    pub assume_emoji_presentation: Option<bool>,
}
impl<'lua> FromLua<'lua> for LuaFontAttributes {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> Result<Self, mlua::Error> {
        match value {
            Value::String(s) => {
                let mut attr = LuaFontAttributes::default();
                attr.family = s.to_str()?.to_string();
                Ok(attr)
            }
            v => {
                let mut attr: LuaFontAttributes = from_lua_value_dynamic(v)?;
                if let Some(italic) = attr.italic.take() {
                    attr.style = if italic {
                        FontStyle::Italic
                    } else {
                        FontStyle::Normal
                    };
                }
                Ok(attr)
            }
        }
    }
}
