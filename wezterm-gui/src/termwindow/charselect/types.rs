struct MatchResults {
    selection: String,
    matches: Vec<usize>,
    group: CharSelectGroup,
}

pub struct CharSelector {
    group: RefCell<CharSelectGroup>,
    element: RefCell<Option<Vec<ComputedElement>>>,
    selection: RefCell<String>,
    aliases: Vec<Alias>,
    matches: RefCell<Option<MatchResults>>,
    selected_row: RefCell<usize>,
    top_row: RefCell<usize>,
    max_rows_on_screen: RefCell<usize>,
    copy_on_select: bool,
    copy_to: ClipboardCopyDestination,
}

enum Move {
    Up(usize),
    Down(usize),
    PageUp,
    PageDown,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Character {
    Unicode { name: &'static str, value: char },
    Emoji(&'static Emoji),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Alias {
    name: Cow<'static, str>,
    character: Character,
    group: CharSelectGroup,
}

impl Alias {
    fn name(&self) -> &str {
        &self.name
    }

    fn glyph(&self) -> String {
        match &self.character {
            Character::Unicode { value, .. } => value.to_string(),
            Character::Emoji(emoji) => emoji.as_str().to_string(),
        }
    }

    fn codepoints(&self) -> String {
        let mut res = String::new();
        for c in self.glyph().chars() {
            if !res.is_empty() {
                res.push(' ');
            }
            res.push_str(&format!("U+{:X}", c as u32));
        }
        res
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Recent {
    glyph: String,
    name: String,
    frecency: Frecency,
}

fn recent_file_name() -> PathBuf {
    config::DATA_DIR.join("recent-emoji.json")
}

fn load_recents() -> anyhow::Result<Vec<Recent>> {
    let file_name = recent_file_name();
    let f = std::fs::File::open(&file_name)?;
    let mut recents: Vec<Recent> = serde_json::from_reader(f)?;
    recents.sort_by(|a, b| b.frecency.score().partial_cmp(&a.frecency.score()).unwrap());
    Ok(recents)
}

fn save_recent(alias: &Alias) -> anyhow::Result<()> {
    let mut recents = load_recents().unwrap_or_else(|_| vec![]);
    let glyph = alias.glyph();
    if let Some(recent_idx) = recents.iter().position(|r| r.glyph == glyph) {
        let recent = recents.get_mut(recent_idx).unwrap();
        recent.frecency.register_access();
    } else {
        let mut frecency = Frecency::new();
        frecency.register_access();
        recents.push(Recent {
            glyph,
            name: alias.name().to_string(),
            frecency,
        });
    }

    let json = serde_json::to_string(&recents)?;
    let file_name = recent_file_name();
    std::fs::write(&file_name, json)?;
    Ok(())
}
