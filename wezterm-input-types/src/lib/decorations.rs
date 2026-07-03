bitflags::bitflags! {
pub struct KittyKeyboardFlags: u16 {
    const NONE = 0;
    const DISAMBIGUATE_ESCAPE_CODES = 1;
    const REPORT_EVENT_TYPES = 2;
    const REPORT_ALTERNATE_KEYS = 4;
    const REPORT_ALL_KEYS_AS_ESCAPE_CODES = 8;
    const REPORT_ASSOCIATED_TEXT = 16;
}
}

bitflags! {
    #[derive(FromDynamic, ToDynamic)]
    #[cfg_attr(feature="serde", derive(Serialize, Deserialize), serde(try_from = "String"))]
    #[dynamic(try_from = "String", into = "String")]
    pub struct WindowDecorations: u8 {
        const TITLE = 1;
        const RESIZE = 2;
        const NONE = 0;
        // Reserve two bits for this enable/disable shadow,
        // so that we effective have Option<bool>
        const MACOS_FORCE_DISABLE_SHADOW = 4;
        const MACOS_FORCE_ENABLE_SHADOW = 4|8;
        const INTEGRATED_BUTTONS = 16;
        const MACOS_FORCE_SQUARE_CORNERS = 32;
        const MACOS_USE_BACKGROUND_COLOR_AS_TITLEBAR_COLOR = 64;
    }
}

impl Into<String> for &WindowDecorations {
    fn into(self) -> String {
        let mut s = vec![];
        if self.contains(WindowDecorations::TITLE) {
            s.push("TITLE");
        }
        if self.contains(WindowDecorations::RESIZE) {
            s.push("RESIZE");
        }
        if self.contains(WindowDecorations::INTEGRATED_BUTTONS) {
            s.push("INTEGRATED_BUTTONS");
        }
        if self.contains(WindowDecorations::MACOS_USE_BACKGROUND_COLOR_AS_TITLEBAR_COLOR) {
            s.push("MACOS_USE_BACKGROUND_COLOR_AS_TITLEBAR_COLOR")
        }
        if self.contains(WindowDecorations::MACOS_FORCE_ENABLE_SHADOW) {
            s.push("MACOS_FORCE_ENABLE_SHADOW");
        } else if self.contains(WindowDecorations::MACOS_FORCE_DISABLE_SHADOW) {
            s.push("MACOS_FORCE_DISABLE_SHADOW");
        } else if self.contains(WindowDecorations::MACOS_FORCE_SQUARE_CORNERS) {
            s.push("MACOS_FORCE_SQUARE_CORNERS");
        }
        if s.is_empty() {
            "NONE".to_string()
        } else {
            s.join("|")
        }
    }
}

impl TryFrom<String> for WindowDecorations {
    type Error = String;
    fn try_from(s: String) -> core::result::Result<WindowDecorations, String> {
        let mut flags = Self::NONE;
        for ele in s.split('|') {
            let ele = ele.trim();
            if ele == "TITLE" {
                flags |= Self::TITLE;
            } else if ele == "NONE" || ele == "None" {
                flags = Self::NONE;
            } else if ele == "RESIZE" {
                flags |= Self::RESIZE;
            } else if ele == "MACOS_USE_BACKGROUND_COLOR_AS_TITLEBAR_COLOR" {
                flags |= Self::MACOS_USE_BACKGROUND_COLOR_AS_TITLEBAR_COLOR;
            } else if ele == "MACOS_FORCE_DISABLE_SHADOW" {
                flags |= Self::MACOS_FORCE_DISABLE_SHADOW;
            } else if ele == "MACOS_FORCE_ENABLE_SHADOW" {
                flags |= Self::MACOS_FORCE_ENABLE_SHADOW;
            } else if ele == "MACOS_FORCE_SQUARE_CORNERS" {
                flags |= Self::MACOS_FORCE_SQUARE_CORNERS;
            } else if ele == "INTEGRATED_BUTTONS" {
                flags |= Self::INTEGRATED_BUTTONS;
            } else {
                return Err(format!("invalid WindowDecoration name {} in {}", ele, s));
            }
        }
        Ok(flags)
    }
}

impl Default for WindowDecorations {
    fn default() -> Self {
        WindowDecorations::TITLE | WindowDecorations::RESIZE
    }
}

#[derive(Debug, FromDynamic, ToDynamic, PartialEq, Eq, Clone, Copy)]
pub enum IntegratedTitleButton {
    Hide,
    Maximize,
    Close,
}

#[derive(Debug, Default, FromDynamic, ToDynamic, PartialEq, Eq, Clone, Copy)]
pub enum IntegratedTitleButtonAlignment {
    #[default]
    Right,
    Left,
}

#[derive(Debug, ToDynamic, PartialEq, Eq, Clone, Copy)]
pub enum IntegratedTitleButtonStyle {
    Windows,
    Gnome,
    MacOsNative,
}

impl Default for IntegratedTitleButtonStyle {
    fn default() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOsNative
        } else {
            Self::Windows
        }
    }
}

impl FromDynamic for IntegratedTitleButtonStyle {
    fn from_dynamic(
        value: &wezterm_dynamic::Value,
        _options: wezterm_dynamic::FromDynamicOptions,
    ) -> Result<Self, wezterm_dynamic::Error>
    where
        Self: Sized,
    {
        let type_name = "integrated_title_button_style";

        if let wezterm_dynamic::Value::String(string) = value {
            let style = match string.as_str() {
                "Windows" => Self::Windows,
                "Gnome" => Self::Gnome,
                "MacOsNative" if cfg!(target_os = "macos") => Self::MacOsNative,
                _ => {
                    return Err(wezterm_dynamic::Error::InvalidVariantForType {
                        variant_name: string.to_string(),
                        type_name,
                        possible: &["Windows", "Gnome", "MacOsNative"],
                    });
                }
            };
            Ok(style)
        } else {
            Err(wezterm_dynamic::Error::InvalidVariantForType {
                variant_name: value.variant_name().to_string(),
                type_name,
                possible: &["String"],
            })
        }
    }
}

/// Kitty wants us to report the un-shifted version of a key.
/// It's a PITA to obtain that from the OS-dependent keyboard
/// layout stuff. For the moment, we'll do the slightly gross
/// thing and make an assumption that a US ANSI layout is in
/// use; this function encodes that mapping.
fn us_layout_unshift(c: char) -> char {
    match c {
        '!' => '1',
        '@' => '2',
        '#' => '3',
        '$' => '4',
        '%' => '5',
        '^' => '6',
        '&' => '7',
        '*' => '8',
        '(' => '9',
        ')' => '0',
        '_' => '-',
        '+' => '=',
        '~' => '`',
        '{' => '[',
        '}' => ']',
        '|' => '\\',
        ':' => ';',
        '"' => '\'',
        '<' => ',',
        '>' => '.',
        '?' => '/',
        c => {
            let s: Vec<char> = c.to_lowercase().collect();
            if s.len() == 1 {
                s[0]
            } else {
                c
            }
        }
    }
}

/// Map c to its Ctrl equivalent.
/// In theory, this mapping is simply translating alpha characters
/// to upper case and then masking them by 0x1f, but xterm inherits
/// some built-in translation from legacy X11 so that are some
/// aliased mappings and a couple that might be technically tied
/// to US keyboard layout (particularly the punctuation characters
/// produced in combination with SHIFT) that may not be 100%
/// the right thing to do here for users with non-US layouts.
pub fn ctrl_mapping(c: char) -> Option<char> {
    Some(match c {
        '@' | '`' | ' ' | '2' => '\x00',
        'A' | 'a' => '\x01',
        'B' | 'b' => '\x02',
        'C' | 'c' => '\x03',
        'D' | 'd' => '\x04',
        'E' | 'e' => '\x05',
        'F' | 'f' => '\x06',
        'G' | 'g' => '\x07',
        'H' | 'h' => '\x08',
        'I' | 'i' => '\x09',
        'J' | 'j' => '\x0a',
        'K' | 'k' => '\x0b',
        'L' | 'l' => '\x0c',
        'M' | 'm' => '\x0d',
        'N' | 'n' => '\x0e',
        'O' | 'o' => '\x0f',
        'P' | 'p' => '\x10',
        'Q' | 'q' => '\x11',
        'R' | 'r' => '\x12',
        'S' | 's' => '\x13',
        'T' | 't' => '\x14',
        'U' | 'u' => '\x15',
        'V' | 'v' => '\x16',
        'W' | 'w' => '\x17',
        'X' | 'x' => '\x18',
        'Y' | 'y' => '\x19',
        'Z' | 'z' => '\x1a',
        '[' | '3' | '{' => '\x1b',
        '\\' | '4' | '|' => '\x1c',
        ']' | '5' | '}' => '\x1d',
        '^' | '6' | '~' => '\x1e',
        '_' | '7' | '/' => '\x1f',
        '8' | '?' => '\x7f', // `Delete`
        _ => return None,
    })
}
