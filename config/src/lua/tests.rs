#[cfg(test)]
mod test {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn can_register_and_emit_multiple_events() -> anyhow::Result<()> {
        let _ = env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Trace)
            .try_init();

        let lua = make_lua_context(Path::new("testing"))?;

        let total = Arc::new(Mutex::new(0));

        let first = lua.create_function({
            let total = total.clone();
            move |_lua: &mlua::Lua, n: i32| {
                let mut l = total.lock().unwrap();
                *l += n;
                Ok(())
            }
        })?;

        let second = lua.create_function({
            let total = total.clone();
            move |_lua: &mlua::Lua, n: i32| {
                let mut l = total.lock().unwrap();
                *l += n * 2;
                // Prevent any later functions from being called
                Ok(false)
            }
        })?;

        let third = lua.create_function({
            let total = total.clone();
            move |_lua: &mlua::Lua, n: i32| {
                let mut l = total.lock().unwrap();
                *l += n * 3;
                Ok(())
            }
        })?;

        register_event(&lua, ("foo".to_string(), first))?;
        register_event(&lua, ("foo".to_string(), second))?;
        register_event(&lua, ("foo".to_string(), third))?;
        register_event(
            &lua,
            (
                "bar".to_string(),
                lua.create_function(|_: &mlua::Lua, (a, b): (i32, String)| {
                    eprintln!("a: {}, b: {}", a, b);
                    Ok(())
                })?,
            ),
        )?;

        smol::block_on(
            lua.load(
                r#"
local wezterm = require 'wezterm';

wezterm.on('foo', function (n)
    print("lua hook recording " .. n);
end);

-- one of the foo handlers returns false, so the emit
-- returns false overall, indicating that the default
-- action should not be taken
assert(wezterm.emit('foo', 2) == false)

wezterm.on('bar', function (n, str)
    print("bar says " .. n .. " " .. str)
end);

-- None of the bar handlers return anything, so the
-- emit returns true to indicate that the default
-- action should be performed
assert(wezterm.emit('bar', 42, 'woot') == true)
"#,
            )
            .exec_async(),
        )?;

        assert_eq!(*total.lock().unwrap(), 6);

        Ok(())
    }
}
