fn extract_inherited_theme_name(p: PathBuf) -> Option<String> {
    let data = std::fs::read_to_string(&p).ok()?;
    log::trace!("Parsing {p:?} to determine inheritance");
    for line in data.lines() {
        let fields: Vec<&str> = line.splitn(2, '=').collect();
        if fields.len() == 2 {
            let key = fields[0].trim();
            if key == "Inherits" {
                fn separator(c: char) -> bool {
                    c.is_whitespace() || c == ';' || c == ','
                }

                return Some(
                    fields[1]
                        .trim()
                        .chars()
                        .skip_while(|&c| separator(c))
                        .take_while(|&c| !separator(c))
                        .collect(),
                );
            }
        }
    }
    None
}
