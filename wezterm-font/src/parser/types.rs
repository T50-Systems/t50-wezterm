use crate::locator::{FontDataHandle, FontDataSource, FontOrigin};
use crate::shaper::GlyphInfo;
use config::{FontAttributes, FontStyle, FreeTypeLoadFlags, FreeTypeLoadTarget};
pub use config::{FontStretch, FontWeight};
use rangeset::RangeSet;
use std::cmp::Ordering;
use std::sync::Mutex;

#[derive(Debug)]
pub enum MaybeShaped {
    Resolved(GlyphInfo),
    Unresolved { raw: String, slice_start: usize },
}

#[derive(Debug, Clone)]
pub struct FontPaletteInfo {
    pub name: String,
    pub palette_index: usize,
    pub usable_with_light_bg: bool,
    pub usable_with_dark_bg: bool,
}

/// Represents a parsed font
pub struct ParsedFont {
    names: Names,
    weight: FontWeight,
    stretch: FontStretch,
    style: FontStyle,
    cap_height: Option<f64>,
    pub handle: FontDataHandle,
    coverage: Mutex<RangeSet<u32>>,
    pub synthesize_italic: bool,
    pub synthesize_bold: bool,
    pub synthesize_dim: bool,
    pub assume_emoji_presentation: bool,
    pub pixel_sizes: Vec<u16>,
    pub is_built_in_fallback: bool,
    pub palettes: Vec<FontPaletteInfo>,

    pub harfbuzz_features: Option<Vec<String>>,
    pub freetype_load_target: Option<FreeTypeLoadTarget>,
    pub freetype_render_target: Option<FreeTypeLoadTarget>,
    pub freetype_load_flags: Option<FreeTypeLoadFlags>,
    pub scale: Option<f64>,
}

impl std::fmt::Debug for ParsedFont {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("ParsedFont")
            .field("names", &self.names)
            .field("weight", &self.weight)
            .field("stretch", &self.stretch)
            .field("style", &self.style)
            .field("handle", &self.handle)
            .field("cap_height", &self.cap_height)
            .field("synthesize_italic", &self.synthesize_italic)
            .field("synthesize_bold", &self.synthesize_bold)
            .field("synthesize_dim", &self.synthesize_dim)
            .field("assume_emoji_presentation", &self.assume_emoji_presentation)
            .field("pixel_sizes", &self.pixel_sizes)
            .field("harfbuzz_features", &self.harfbuzz_features)
            .field("freetype_load_target", &self.freetype_load_target)
            .field("freetype_render_target", &self.freetype_render_target)
            .field("freetype_load_flags", &self.freetype_load_flags)
            .field("scale", &self.scale)
            .finish()
    }
}

impl Clone for ParsedFont {
    fn clone(&self) -> Self {
        Self {
            names: self.names.clone(),
            weight: self.weight,
            stretch: self.stretch,
            style: self.style,
            synthesize_italic: self.synthesize_italic,
            synthesize_bold: self.synthesize_bold,
            synthesize_dim: self.synthesize_dim,
            assume_emoji_presentation: self.assume_emoji_presentation,
            handle: self.handle.clone(),
            cap_height: self.cap_height,
            coverage: Mutex::new(self.coverage.lock().unwrap().clone()),
            pixel_sizes: self.pixel_sizes.clone(),
            harfbuzz_features: self.harfbuzz_features.clone(),
            freetype_load_target: self.freetype_load_target,
            freetype_render_target: self.freetype_render_target,
            freetype_load_flags: self.freetype_load_flags,
            is_built_in_fallback: self.is_built_in_fallback,
            scale: self.scale,
            palettes: self.palettes.clone(),
        }
    }
}

impl Eq for ParsedFont {}

impl PartialEq for ParsedFont {
    fn eq(&self, rhs: &Self) -> bool {
        self.stretch == rhs.stretch
            && self.weight == rhs.weight
            && self.style == rhs.style
            && self.names == rhs.names
    }
}

impl Ord for ParsedFont {
    fn cmp(&self, rhs: &Self) -> Ordering {
        match self.names.family.cmp(&rhs.names.family) {
            o @ Ordering::Less | o @ Ordering::Greater => o,
            Ordering::Equal => match self.stretch.cmp(&rhs.stretch) {
                o @ Ordering::Less | o @ Ordering::Greater => o,
                Ordering::Equal => match self.weight.cmp(&rhs.weight) {
                    o @ Ordering::Less | o @ Ordering::Greater => o,
                    Ordering::Equal => match self.style.cmp(&rhs.style) {
                        o @ Ordering::Less | o @ Ordering::Greater => o,
                        Ordering::Equal => self.handle.cmp(&rhs.handle),
                    },
                },
            },
        }
    }
}

impl PartialOrd for ParsedFont {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Names {
    pub full_name: String,
    pub family: String,
    pub sub_family: Option<String>,
    pub postscript_name: Option<String>,
    pub aliases: Vec<String>,
}

/// Returns the "best" name from a set of records.
/// Best is English from a MS entry if available, as freetype's
/// source claims that a number of Mac entries have somewhat
/// broken encodings.
fn best_name(records: &[crate::ftwrap::NameRecord]) -> String {
    let mut win = None;
    let mut uni = None;
    let mut apple = None;

    for rec in records {
        match rec.platform_id as u32 {
            freetype::TT_PLATFORM_APPLE_UNICODE | freetype::TT_PLATFORM_ISO => {
                uni.replace(rec);
            }
            freetype::TT_PLATFORM_MACINTOSH => {
                apple.replace(rec);
            }
            freetype::TT_PLATFORM_MICROSOFT => {
                let is_english = (rec.language_id & 0x3ff) == 0x9;
                if is_english {
                    return rec.name.clone();
                }
                win.replace(rec);
            }
            _ => {}
        }
    }

    if let Some(rec) = apple {
        return rec.name.clone();
    }
    if let Some(rec) = win {
        return rec.name.clone();
    }
    if let Some(rec) = uni {
        return rec.name.clone();
    }
    records[0].name.clone()
}

/// Return a single name from a table.
/// The list of ids are tried in order: the first id with corresponding
/// names is taken, and the "best" of those names is returned.
fn name_from_table(
    names: &std::collections::HashMap<u32, Vec<crate::ftwrap::NameRecord>>,
    ids: &[u32],
) -> Option<String> {
    for id in ids {
        if let Some(name_list) = names.get(id) {
            return Some(best_name(name_list));
        }
    }
    None
}

/// Returns the sorted, deduplicated set of names across the list of ids
fn names_from_table(
    names: &std::collections::HashMap<u32, Vec<crate::ftwrap::NameRecord>>,
    ids: &[u32],
) -> Vec<String> {
    let mut result = vec![];

    for id in ids {
        if let Some(name_list) = names.get(id) {
            for rec in name_list {
                result.push(rec.name.clone());
            }
        }
    }
    result.sort();
    result.dedup();
    result
}

impl Names {
    pub fn from_ft_face(face: &crate::ftwrap::Face) -> Names {
        // We don't simply use the freetype functions to retrieve names,
        // as freetype has a limited set of encodings that it supports.
        // We process the name table for ourselves to increase our chances
        // of returning a good version of the name.
        // See <https://github.com/wezterm/wezterm/issues/1761#issuecomment-1079150560>
        // for a case where freetype returns `?????` for a name.
        let names = face.get_sfnt_names();

        let family = name_from_table(
            &names,
            &[
                freetype::TT_NAME_ID_TYPOGRAPHIC_FAMILY,
                freetype::TT_NAME_ID_FONT_FAMILY,
            ],
        )
        .unwrap_or_else(|| face.family_name());

        let sub_family = name_from_table(
            &names,
            &[
                freetype::TT_NAME_ID_TYPOGRAPHIC_SUBFAMILY,
                freetype::TT_NAME_ID_FONT_SUBFAMILY,
            ],
        )
        .unwrap_or_else(|| face.style_name());

        let postscript_name = name_from_table(&names, &[freetype::TT_NAME_ID_PS_NAME])
            .unwrap_or_else(|| face.postscript_name());

        let full_name = if sub_family.is_empty() {
            family.to_string()
        } else {
            format!("{} {}", family, sub_family)
        };

        let mut aliases = names_from_table(
            &names,
            &[
                freetype::TT_NAME_ID_TYPOGRAPHIC_FAMILY,
                freetype::TT_NAME_ID_FONT_FAMILY,
            ],
        );
        aliases.retain(|n| *n != full_name && *n != family);

        Names {
            full_name,
            family,
            sub_family: Some(sub_family),
            postscript_name: Some(postscript_name),
            aliases,
        }
    }
}
