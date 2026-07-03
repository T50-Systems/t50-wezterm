fn section_header(title: &str) {
    let dash = "-".repeat(title.len());
    println!("{title}");
    println!("{dash}");
    println!();
}

pub fn ui_key(key: &KeyCode, ui_key_cap_rendering: UIKeyCapRendering) -> String {
    match key {
        KeyCode::Char('\x1b') | KeyCode::Char('\x7f')
            if ui_key_cap_rendering == UIKeyCapRendering::AppleSymbols =>
        {
            "\u{238b}".to_string()
        }
        KeyCode::Char('\x1b') | KeyCode::Char('\x7f') => "Esc".to_string(),
        KeyCode::Char('\x08') if ui_key_cap_rendering == UIKeyCapRendering::AppleSymbols => {
            "\u{232b}".to_string()
        }
        KeyCode::Char('\x08') => "Del".to_string(),
        KeyCode::Char('\r') if ui_key_cap_rendering == UIKeyCapRendering::AppleSymbols => {
            "\u{21b5}".to_string()
        }
        KeyCode::Char('\r') => "Enter".to_string(),
        KeyCode::Physical(PhysKeyCode::Space) | KeyCode::Char(' ')
            if ui_key_cap_rendering == UIKeyCapRendering::AppleSymbols =>
        {
            "\u{2423}".to_string()
        }
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char('\t') if ui_key_cap_rendering == UIKeyCapRendering::AppleSymbols => {
            "\u{21e5}".to_string()
        }
        KeyCode::Char('\t') => "Tab".to_string(),
        KeyCode::Char(c) if c.is_ascii_control() => c.escape_debug().to_string(),
        KeyCode::Char(c) => c.to_uppercase().to_string(),

        KeyCode::Physical(PhysKeyCode::PageUp) | KeyCode::PageUp
            if ui_key_cap_rendering == UIKeyCapRendering::AppleSymbols =>
        {
            "\u{21de}".to_string()
        }
        KeyCode::Physical(PhysKeyCode::PageDown) | KeyCode::PageDown
            if ui_key_cap_rendering == UIKeyCapRendering::AppleSymbols =>
        {
            "\u{21df}".to_string()
        }
        KeyCode::Physical(PhysKeyCode::LeftArrow) | KeyCode::LeftArrow => "\u{2190}".to_string(),
        KeyCode::Physical(PhysKeyCode::UpArrow) | KeyCode::UpArrow => "\u{2191}".to_string(),
        KeyCode::Physical(PhysKeyCode::RightArrow) | KeyCode::RightArrow => "\u{2192}".to_string(),
        KeyCode::Physical(PhysKeyCode::DownArrow) | KeyCode::DownArrow => "\u{2193}".to_string(),
        KeyCode::Function(n) => format!("F{n}"),
        KeyCode::Numpad(n) => format!("Numpad{n}"),
        KeyCode::Physical(phys) => phys.to_string(),
        _ => format!("{key:?}"),
    }
}

pub fn human_key(key: &KeyCode) -> String {
    match key {
        KeyCode::Char('\x1b') => "Escape".to_string(),
        KeyCode::Char('\x7f') => "Escape".to_string(),
        KeyCode::Char('\x08') => "Backspace".to_string(),
        KeyCode::Char('\r') => "Enter".to_string(),
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char('\t') => "Tab".to_string(),
        KeyCode::Char(c) if c.is_ascii_control() => c.escape_debug().to_string(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Function(n) => format!("F{n}"),
        KeyCode::Numpad(n) => format!("Numpad{n}"),
        KeyCode::Physical(phys) => format!("{} (Physical)", phys.to_string()),
        _ => format!("{key:?}"),
    }
}

fn lua_key_code(key: &KeyCode) -> String {
    match key {
        KeyCode::Char('\x1b') => "Escape".to_string(),
        KeyCode::Char('\x7f') => "Escape".to_string(),
        KeyCode::Char('\x08') => "Backspace".to_string(),
        KeyCode::Char('\r') => "Enter".to_string(),
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char('\t') => "Tab".to_string(),
        KeyCode::Char(c) if c.is_ascii_control() => c.escape_debug().to_string(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Function(n) => format!("F{n}"),
        KeyCode::Numpad(n) => format!("Numpad{n}"),
        KeyCode::Physical(phys) => format!("phys:{}", phys.to_string()),
        _ => format!("{key:?}"),
    }
}

fn luaify(value: Value, is_top: bool) -> String {
    match value {
        Value::String(s) if is_top => format!("act.{s}"),
        Value::String(s) => quote_lua_string(&s),
        Value::Bool(true) => "true".to_string(),
        Value::Bool(false) => "false".to_string(),
        Value::Null => "nil".to_string(),
        Value::U64(u) => u.to_string(),
        Value::F64(u) => u.to_string(),
        Value::I64(u) => u.to_string(),
        Value::Array(a) => {
            format!("wat {a:?}")
        }
        Value::Object(o) if is_top => {
            for (k, v) in o {
                let k = match k {
                    Value::String(s) => s,
                    _ => unreachable!(),
                };
                let arg = match v {
                    Value::String(_) => format!(" {}", luaify(v, false)),
                    Value::Array(a) => {
                        let b: Vec<String> = a.into_iter().map(|v| luaify(v, false)).collect();
                        format!("{{ {} }}", b.join(", "))
                    }
                    Value::I64(i) => format!("({i})"),
                    Value::U64(i) => format!("({i})"),
                    Value::F64(i) => format!("({i})"),
                    _ => luaify(v, false),
                };
                return format!("act.{k}{arg}");
            }
            unreachable!()
        }
        Value::Object(o) => {
            let mut fields = vec![];
            for (k, v) in o {
                let k = match k {
                    Value::String(s) => s,
                    _ => unreachable!(),
                };
                let arg = match v {
                    Value::Null => continue,
                    Value::String(_) => format!(" {}", luaify(v, false)),
                    Value::Array(a) => {
                        let b: Vec<String> = a.into_iter().map(|v| luaify(v, false)).collect();
                        format!("{{ {} }}", b.join(", "))
                    }
                    Value::I64(i) => format!("({i})"),
                    Value::U64(i) => format!("({i})"),
                    Value::F64(i) => format!("({i})"),
                    Value::Object(o) if o.is_empty() => continue,
                    _ => luaify(v, false),
                };
                fields.push(format!("{k} = {arg}"));
            }
            format!("{{ {} }}", fields.join(", "))
        }
    }
}

fn quote_lua_string(s: &str) -> String {
    let mut result = String::new();
    result.push('\'');
    for c in s.chars() {
        match c {
            '\u{07}' => {
                result.push_str("\\a");
            }
            '\u{08}' => {
                result.push_str("\\b");
            }
            '\u{0c}' => {
                result.push_str("\\f");
            }
            '\n' => {
                result.push_str("\\n");
            }
            '\r' => {
                result.push_str("\\r");
            }
            '\t' => {
                result.push_str("\\t");
            }
            '\u{0b}' => {
                result.push_str("\\v");
            }
            '\\' => {
                result.push_str("\\\\");
            }
            '"' => {
                result.push_str("\\\"");
            }
            '\'' => {
                result.push_str("\\'");
            }
            c if c.is_alphanumeric() || c.is_ascii_punctuation() => {
                result.push(c);
            }
            _ => {
                let b = c as u32;
                result.push_str(&format!("\\u{{{b:x}}}"));
            }
        }
    }
    result.push('\'');
    result
}

fn lua_key(key: &KeyCode, mods: Modifiers, action: &KeyAssignment) -> String {
    let dyn_action = action.to_dynamic();
    // println!(" -- {dyn_action:?}");
    let action = luaify(dyn_action, true);
    let key = lua_key_code(key);
    let key = quote_lua_string(&key);

    let mods = format!("{mods:?}").replace(" ", "");

    format!("{{ key = {key}, mods = '{mods}', action = {action} }}")
}

fn show_key_table(table: &config::keyassignment::KeyTable) {
    let ordered = table.iter().collect::<BTreeMap<_, _>>();

    let mut key_width = 0;
    let mut mod_width = 0;
    for (key, mods) in ordered.keys() {
        mod_width = mod_width.max(format!("{mods:?}").len());
        key_width = key_width.max(human_key(key).len());
    }

    for ((key, mods), entry) in ordered {
        let action = &entry.action;
        let mods = if *mods == Modifiers::NONE {
            String::new()
        } else {
            format!("{mods:?}")
        };
        let key = human_key(key);
        println!("\t{mods:mod_width$}   {key:key_width$}   ->   {action:?}");
    }
}

fn show_key_table_as_lua(table: &config::keyassignment::KeyTable, indent: usize) {
    let ordered = table.iter().collect::<BTreeMap<_, _>>();

    let pad = " ".repeat(indent);
    for ((key, mods), entry) in ordered {
        let action = &entry.action;
        println!("{pad}{},", lua_key(key, *mods, action));
    }
}
