lazy_static! {
    pub static ref HOME_DIR: PathBuf = dirs_next::home_dir().expect("can't find HOME dir");
    pub static ref CONFIG_DIRS: Vec<PathBuf> = config_dirs();
    pub static ref RUNTIME_DIR: PathBuf = compute_runtime_dir().unwrap();
    pub static ref DATA_DIR: PathBuf = compute_data_dir().unwrap();
    pub static ref CACHE_DIR: PathBuf = compute_cache_dir().unwrap();
    static ref CONFIG: Configuration = Configuration::new();
    static ref CONFIG_FILE_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);
    static ref CONFIG_SKIP: AtomicBool = AtomicBool::new(false);
    static ref CONFIG_OVERRIDES: Mutex<Vec<(String, String)>> = Mutex::new(vec![]);
    static ref SHOW_ERROR: Mutex<Option<ErrorCallback>> =
        Mutex::new(Some(|e| log::error!("{}", e)));
    static ref LUA_PIPE: LuaPipe = LuaPipe::new();
    pub static ref COLOR_SCHEMES: HashMap<String, Palette> = build_default_schemes();
}

thread_local! {
    static LUA_CONFIG: RefCell<Option<LuaConfigState>> = RefCell::new(None);
}

fn toml_table_has_numeric_keys(t: &toml::value::Table) -> bool {
    t.keys().all(|k| k.parse::<isize>().is_ok())
}

fn json_object_has_numeric_keys(t: &serde_json::Map<String, serde_json::Value>) -> bool {
    t.keys().all(|k| k.parse::<isize>().is_ok())
}

fn toml_to_dynamic(value: &toml::Value) -> Value {
    match value {
        toml::Value::String(s) => s.to_dynamic(),
        toml::Value::Integer(n) => n.to_dynamic(),
        toml::Value::Float(n) => n.to_dynamic(),
        toml::Value::Boolean(b) => b.to_dynamic(),
        toml::Value::Datetime(d) => d.to_string().to_dynamic(),
        toml::Value::Array(a) => a
            .iter()
            .map(toml_to_dynamic)
            .collect::<Vec<_>>()
            .to_dynamic(),
        // Allow `colors.indexed` to be passed through with actual integer keys
        toml::Value::Table(t) if toml_table_has_numeric_keys(t) => Value::Object(
            t.iter()
                .map(|(k, v)| (k.parse::<isize>().unwrap().to_dynamic(), toml_to_dynamic(v)))
                .collect::<BTreeMap<_, _>>()
                .into(),
        ),
        toml::Value::Table(t) => Value::Object(
            t.iter()
                .map(|(k, v)| (Value::String(k.to_string()), toml_to_dynamic(v)))
                .collect::<BTreeMap<_, _>>()
                .into(),
        ),
    }
}

fn json_to_dynamic(value: &serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => b.to_dynamic(),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_dynamic()
            } else if let Some(i) = n.as_u64() {
                i.to_dynamic()
            } else if let Some(f) = n.as_f64() {
                f.to_dynamic()
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => s.to_dynamic(),
        serde_json::Value::Array(a) => a
            .iter()
            .map(json_to_dynamic)
            .collect::<Vec<_>>()
            .to_dynamic(),
        // Allow `colors.indexed` to be passed through with actual integer keys
        serde_json::Value::Object(t) if json_object_has_numeric_keys(t) => Value::Object(
            t.iter()
                .map(|(k, v)| (k.parse::<isize>().unwrap().to_dynamic(), json_to_dynamic(v)))
                .collect::<BTreeMap<_, _>>()
                .into(),
        ),
        serde_json::Value::Object(t) => Value::Object(
            t.iter()
                .map(|(k, v)| (Value::String(k.to_string()), json_to_dynamic(v)))
                .collect::<BTreeMap<_, _>>()
                .into(),
        ),
    }
}

pub fn build_default_schemes() -> HashMap<String, Palette> {
    let mut color_schemes = HashMap::new();
    for (scheme_name, data) in scheme_data::SCHEMES.iter() {
        let scheme_name = scheme_name.to_string();
        let scheme = ColorSchemeFile::from_toml_str(data).unwrap();
        color_schemes.insert(scheme_name, scheme.colors.clone());
        for alias in scheme.metadata.aliases {
            color_schemes.insert(alias, scheme.colors.clone());
        }
    }
    color_schemes
}
