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
