use crate::exec_domain::{ExecDomain, ValueOrFunc};
use crate::keyassignment::KeyAssignment;
use crate::{
    Config, FontAttributes, FontStretch, FontStyle, FontWeight, FreeTypeLoadTarget, RgbaColor,
    TextStyle,
};
use anyhow::{anyhow, Context};
use luahelper::{from_lua_value_dynamic, lua_value_to_dynamic, to_lua};
use mlua::{FromLua, IntoLuaMulti, Lua, Table, Value, Variadic};
use ordered_float::NotNan;
use portable_pty::CommandBuilder;
use std::convert::TryFrom;
use std::path::Path;
use std::sync::Mutex;
use wezterm_dynamic::{
    FromDynamic, FromDynamicOptions, ToDynamic, UnknownFieldAction, Value as DynValue,
};

pub use mlua;

static LUA_REGISTRY_USER_CALLBACK_COUNT: &str = "wezterm-user-callback-count";

pub type SetupFunc = fn(&Lua) -> anyhow::Result<()>;

lazy_static::lazy_static! {
    static ref SETUP_FUNCS: Mutex<Vec<SetupFunc>> = Mutex::new(vec![]);
}

pub fn add_context_setup_func(func: SetupFunc) {
    SETUP_FUNCS.lock().unwrap().push(func);
}

pub fn get_or_create_module<'lua>(lua: &'lua Lua, name: &str) -> anyhow::Result<mlua::Table<'lua>> {
    let globals = lua.globals();
    let package: Table = globals.get("package")?;
    let loaded: Table = package.get("loaded")?;

    let module = loaded.get(name)?;
    match module {
        Value::Nil => {
            let module = lua.create_table()?;
            loaded.set(name, module.clone())?;
            Ok(module)
        }
        Value::Table(table) => Ok(table),
        wat => anyhow::bail!(
            "cannot register module {} as package.loaded.{} is already set to a value of type {}",
            name,
            name,
            wat.type_name()
        ),
    }
}

pub fn get_or_create_sub_module<'lua>(
    lua: &'lua Lua,
    name: &str,
) -> anyhow::Result<mlua::Table<'lua>> {
    let wezterm_mod = get_or_create_module(lua, "wezterm")?;
    let sub = wezterm_mod.get(name)?;
    match sub {
        Value::Nil => {
            let sub = lua.create_table()?;
            wezterm_mod.set(name, sub.clone())?;
            Ok(sub)
        }
        Value::Table(sub) => Ok(sub),
        wat => anyhow::bail!(
            "cannot register module wezterm.{name} as it is already set to a value of type {}",
            wat.type_name()
        ),
    }
}
