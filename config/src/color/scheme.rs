#[derive(Debug, Default, Clone, Eq, PartialEq, FromDynamic, ToDynamic)]
pub struct ColorSchemeMetaData {
    pub name: Option<String>,
    pub author: Option<String>,
    pub origin_url: Option<String>,
    pub wezterm_version: Option<String>,
    #[dynamic(default)]
    pub aliases: Vec<String>,
}
impl_lua_conversion_dynamic!(ColorSchemeMetaData);

#[derive(Debug, Clone, PartialEq, FromDynamic, ToDynamic)]
pub struct ColorSchemeFile {
    /// The color palette
    pub colors: Palette,
    /// Info about the scheme
    #[dynamic(default)]
    pub metadata: ColorSchemeMetaData,
}
impl_lua_conversion_dynamic!(ColorSchemeFile);

fn dynamic_to_toml(value: Value) -> anyhow::Result<toml::Value> {
    Ok(match value {
        Value::Null => anyhow::bail!("cannot map Null to toml"),
        Value::Bool(b) => toml::Value::Boolean(b),
        Value::String(s) => toml::Value::String(s),
        Value::Array(a) => {
            let mut arr = vec![];
            for v in a {
                arr.push(dynamic_to_toml(v)?);
            }
            toml::Value::Array(arr)
        }
        Value::Object(o) => {
            let mut map = toml::map::Map::new();
            for (k, v) in o {
                let k = match k {
                    Value::String(s) => s,
                    Value::U64(u) => u.to_string(),
                    Value::I64(u) => u.to_string(),
                    Value::F64(u) => u.to_string(),
                    _ => anyhow::bail!("toml keys must be strings {k:?}"),
                };
                let v = match v {
                    Value::Null => continue,
                    other => dynamic_to_toml(other)?,
                };
                map.insert(k, v);
            }
            toml::Value::Table(map)
        }
        Value::U64(i) => toml::Value::Integer(i.try_into()?),
        Value::I64(i) => toml::Value::Integer(i.try_into()?),
        Value::F64(f) => toml::Value::Float(*f),
    })
}

impl ColorSchemeFile {
    pub fn from_toml_value(value: &toml::Value) -> anyhow::Result<Self> {
        let scheme = Self::from_dynamic(&crate::toml_to_dynamic(value), Default::default())
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        anyhow::ensure!(
            scheme.colors.ansi.is_some(),
            "scheme is missing ANSI colors"
        );

        Ok(scheme)
    }

    pub fn from_toml_str(s: &str) -> anyhow::Result<Self> {
        let scheme: toml::Value = toml::from_str(s)?;
        Self::from_toml_value(&scheme)
    }

    pub fn to_toml_value(&self) -> anyhow::Result<toml::Value> {
        let value = self.to_dynamic();
        Ok(dynamic_to_toml(value)?)
    }

    pub fn from_json_value(value: &serde_json::Value) -> anyhow::Result<Self> {
        Self::from_dynamic(&crate::json_to_dynamic(value), Default::default())
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let value = self.to_toml_value()?;
        let text = toml::to_string_pretty(&value)?;
        std::fs::write(&path, text)
            .with_context(|| format!("writing toml to {}", path.as_ref().display()))
    }
}

#[cfg(test)]
#[test]
fn test_indexed_colors() {
    let scheme = r##"
[colors]
foreground = "#005661"
background = "#fef8ec"
cursor_bg = "#005661"
cursor_border = "#005661"
cursor_fg = "#ffffff"
selection_bg = "#cfe7f0"
selection_fg = "#005661"

ansi = [ "#8ca6a6" ,"#e64100" ,"#00b368" ,"#fa8900" ,"#0095a8" ,"#ff5792" ,"#00bdd6" ,"#005661" ]
brights = [ "#8ca6a6" ,"#e5164a" ,"#00b368" ,"#b3694d" ,"#0094f0" ,"#ff5792" ,"#00bdd6" ,"#004d57" ]

[colors.indexed]
52 = "#fbdada" # minus
88 = "#f6b6b6" # minus emph
22 = "#d6ffd6" # plus
28 = "#adffad" # plus emph
53 = "#feecf7" # purple
17 = "#e5dff6" # blue
23 = "#d8fdf6" # cyan
58 = "#f4ffe0" # yellow
"##;
    let scheme = ColorSchemeFile::from_toml_str(scheme).unwrap();
    assert_eq!(
        scheme.colors.indexed.get(&52),
        Some(&RgbColor::new_8bpc(0xfb, 0xda, 0xda).into())
    );
}
