#[cfg(feature = "std")]
static NAMED_COLORS: LazyLock<HashMap<String, SrgbaTuple>> = LazyLock::new(build_colors);

const RGB_TXT: &str = core::include_str!("../rgb.txt");

fn iter_rgb_txt(mut func: impl FnMut(&str, SrgbaTuple) -> bool) {
    let transparent = SrgbaTuple(0., 0., 0., 0.);
    for name in &["transparent", "none", "clear"] {
        if (func)(name, transparent) {
            return;
        }
    }

    for line in RGB_TXT.lines() {
        let mut fields = line.split_ascii_whitespace();
        let red = fields.next().unwrap();
        let green = fields.next().unwrap();
        let blue = fields.next().unwrap();
        let name = fields.collect::<Vec<&str>>().join(" ");

        let name = name.to_ascii_lowercase();
        let color = SrgbaTuple(
            red.parse::<f32>().unwrap() / 255.,
            green.parse::<f32>().unwrap() / 255.,
            blue.parse::<f32>().unwrap() / 255.,
            1.0,
        );

        if (func)(&name, color) {
            return;
        }
    }
}

#[cfg(feature = "std")]
fn build_colors() -> HashMap<String, SrgbaTuple> {
    let mut map = HashMap::new();

    iter_rgb_txt(|name, color| {
        map.insert(name.to_string(), color);
        false
    });
    map
}
