use crate::selection::{SelectionCoordinate, SelectionRange};
use crate::termwindow::{TermWindow, TermWindowNotif};
use config::keyassignment::{ClipboardCopyDestination, QuickSelectArguments, ScrollbackEraseMode};
use config::ConfigHandle;
use mux::domain::DomainId;
use mux::pane::{
    CachePolicy, ForEachPaneLogicalLine, LogicalLine, Pane, PaneId, Pattern, SearchResult,
    WithPaneLines,
};
use mux::renderable::*;
use parking_lot::{MappedMutexGuard, Mutex};
use rangeset::RangeSet;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;
use termwiz::cell::{Cell, CellAttributes};
use termwiz::color::AnsiColor;
use termwiz::surface::{SequenceNo, SEQ_ZERO};
use url::Url;
use wezterm_term::color::ColorPalette;
use wezterm_term::{
    Clipboard, Intensity, KeyCode, KeyModifiers, Line, MouseEvent, StableRowIndex, TerminalSize,
};
use window::WindowOps;

const PATTERNS: [&str; 14] = [
    // markdown_url
    r"\[[^]]*\]\(([^)]+)\)",
    // url
    r"(?:https?://|git@|git://|ssh://|ftp://|file://)\S+",
    // diff_a
    r"--- a/(\S+)",
    // diff_b
    r"\+\+\+ b/(\S+)",
    // docker
    r"sha256:([0-9a-f]{64})",
    // path
    r"(?:[.\w\-@~]+)?(?:/+[.\w\-@]+)+",
    // color
    r"#[0-9a-fA-F]{6}",
    // uuid
    r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",
    // ipfs
    r"Qm[0-9a-zA-Z]{44}",
    // sha
    r"[0-9a-f]{7,40}",
    // ip
    r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}",
    // ipv6
    r"[A-f0-9:]+:+[A-f0-9:]+[%\w\d]+",
    // address
    r"0x[0-9a-fA-F]+",
    // number
    r"[0-9]{4,}",
];

/// This function computes a set of labels for a given alphabet.
/// It is derived from https://github.com/fcsonline/tmux-thumbs/blob/master/src/alphabets.rs
/// which is Copyright (c) 2019 Ferran Basora and provided under the MIT license
pub fn compute_labels_for_alphabet(alphabet: &str, num_matches: usize) -> Vec<String> {
    compute_labels_for_alphabet_impl(alphabet, num_matches, true)
}

pub fn compute_labels_for_alphabet_with_preserved_case(
    alphabet: &str,
    num_matches: usize,
) -> Vec<String> {
    compute_labels_for_alphabet_impl(alphabet, num_matches, false)
}

fn compute_labels_for_alphabet_impl(
    alphabet: &str,
    num_matches: usize,
    make_lowercase: bool,
) -> Vec<String> {
    let alphabet = if make_lowercase {
        alphabet
            .chars()
            .map(|c| c.to_lowercase().to_string())
            .collect::<Vec<String>>()
    } else {
        alphabet
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
    };
    // Prefer to use single character matches to represent everything
    let mut primary = alphabet.clone();
    let mut secondary = vec![];

    loop {
        if primary.len() + secondary.len() >= num_matches {
            break;
        }

        // We have more matches than can be represented by alphabet,
        // so steal one of the single character options from the end
        // of the alphabet and use it to generate a two character
        // label
        let prefix = match primary.pop() {
            Some(p) => p,
            None => break,
        };

        // Generate a two character label for each of the alphabet
        // characters.  This ignores later alphabet characters;
        // since we popped our prefix from the end of alphabet,
        // length limiting this iteration ensures that we don't
        // end up with a duplicate letters in the result.
        let prefixed: Vec<String> = alphabet
            .iter()
            .take(num_matches - primary.len() - secondary.len())
            .map(|s| format!("{}{}", prefix, s))
            .collect();

        secondary.splice(0..0, prefixed);
    }

    let len = secondary.len();

    primary
        .drain(0..)
        .take(num_matches - len)
        .chain(secondary.drain(0..))
        .collect()
}

/// Returns true if a label should be displayed given a selection prefix.
fn label_matches_selection(label: &str, lowered_prefix: &str) -> bool {
    lowered_prefix.is_empty() || label.starts_with(lowered_prefix)
}

#[cfg(test)]
mod alphabet_test {
    use super::*;

    #[test]
    fn simple_alphabet() {
        assert_eq!(compute_labels_for_alphabet("abcd", 3), vec!["a", "b", "c"]);
    }

    #[test]
    fn more_matches_than_alphabet_can_represent() {
        assert_eq!(
            compute_labels_for_alphabet("asdfqwerzxcvjklmiuopghtybn", 792).len(),
            676
        );
    }

    #[test]
    fn composed_single() {
        assert_eq!(
            compute_labels_for_alphabet("abcd", 6),
            vec!["a", "b", "c", "da", "db", "dc"]
        );
    }

    #[test]
    fn composed_multiple() {
        assert_eq!(
            compute_labels_for_alphabet("abcd", 8),
            vec!["a", "b", "ca", "cb", "da", "db", "dc", "dd"]
        );
    }

    #[test]
    fn composed_max() {
        // The number of chars in the alphabet limits the potential matches to fewer
        // than the number of matches that we requested
        assert_eq!(
            compute_labels_for_alphabet("ab", 5),
            vec!["aa", "ab", "ba", "bb"]
        );
    }

    #[test]
    fn composed_capital() {
        assert_eq!(
            compute_labels_for_alphabet_with_preserved_case("AB", 4),
            vec!["AA", "AB", "BA", "BB"]
        );
    }

    #[test]
    fn composed_mixed() {
        assert_eq!(
            compute_labels_for_alphabet_with_preserved_case("aA", 4),
            vec!["aa", "aA", "Aa", "AA"]
        );
    }

    #[test]
    fn lowercase_alphabet_equal() {
        assert_eq!(
            compute_labels_for_alphabet_with_preserved_case("abc123", 12),
            compute_labels_for_alphabet("abc123", 12)
        );
    }
}

#[cfg(test)]
mod label_filter_test {
    use super::*;

    #[test]
    fn empty_prefix_matches_all() {
        let labels = ["a", "fq", "db", "av", "ac"];
        assert!(labels.iter().all(|l| label_matches_selection(l, "")));
    }

    #[test]
    fn single_char_prefix_filters_non_matching() {
        // Pressing 'a' should keep labels starting with 'a' and remove others
        let labels = ["ai", "fq", "db", "av", "ac"];
        let visible: Vec<&str> = labels
            .iter()
            .copied()
            .filter(|l| label_matches_selection(l, "a"))
            .collect();
        assert_eq!(visible, ["ai", "av", "ac"]);
    }

    #[test]
    fn full_prefix_matches_exact_label() {
        assert!(label_matches_selection("ai", "ai"));
        assert!(!label_matches_selection("ac", "ai"));
    }

    #[test]
    fn prefix_longer_than_label() {
        assert!(!label_matches_selection("a", "ab"));
        assert!(!label_matches_selection("", "a"));
    }

    #[test]
    fn two_char_labels_narrowed_by_first_char() {
        // With more matches than alphabet, two-char labels are generated
        let labels = compute_labels_for_alphabet("abcd", 6);
        assert_eq!(labels, ["a", "b", "c", "da", "db", "dc"]);

        // Typing 'd' should keep "da", "db", "dc" and remove "a", "b", "c"
        let visible: Vec<&str> = labels
            .iter()
            .map(String::as_str)
            .filter(|l| label_matches_selection(l, "d"))
            .collect();
        assert_eq!(visible, ["da", "db", "dc"]);
    }

    #[test]
    fn no_labels_match_unknown_prefix() {
        let labels = ["ai", "fq", "db"];
        assert!(!labels.iter().any(|l| label_matches_selection(l, "z")));
    }
}

pub struct QuickSelectOverlay {
    renderer: Mutex<QuickSelectRenderable>,
    delegate: Arc<dyn Pane>,
}

#[derive(Debug)]
struct MatchResult {
    range: Range<usize>,
    label: String,
}

struct QuickSelectRenderable {
    delegate: Arc<dyn Pane>,
    /// The text that the user entered
    pattern: Pattern,
    /// The most recently queried set of matches
    results: Vec<SearchResult>,
    by_line: HashMap<StableRowIndex, Vec<MatchResult>>,
    by_label: HashMap<String, usize>,
    selection: String,

    viewport: Option<StableRowIndex>,
    last_bar_pos: Option<StableRowIndex>,

    dirty_results: RangeSet<StableRowIndex>,
    result_pos: Option<usize>,
    width: usize,
    height: usize,

    /// We use this to cancel ourselves later
    window: ::window::Window,

    config: ConfigHandle,
    args: QuickSelectArguments,
}

impl QuickSelectOverlay {
    pub fn with_pane(
        term_window: &TermWindow,
        pane: &Arc<dyn Pane>,
        args: &QuickSelectArguments,
    ) -> Arc<dyn Pane> {
        let viewport = term_window.get_viewport(pane.pane_id());
        let dims = pane.get_dimensions();

        let config = term_window.config.clone();

        let mut pattern = "(?m)(".to_string();
        let mut have_patterns = false;
        if !args.patterns.is_empty() {
            for p in &args.patterns {
                if have_patterns {
                    pattern.push('|');
                }
                pattern.push_str(p);
                have_patterns = true;
            }
        } else {
            // User-provided patterns take precedence over built-ins
            for p in &config.quick_select_patterns {
                if have_patterns {
                    pattern.push('|');
                }
                pattern.push_str(p);
                have_patterns = true;
            }
            if !config.disable_default_quick_select_patterns {
                for p in &PATTERNS {
                    if have_patterns {
                        pattern.push('|');
                    }
                    pattern.push_str(p);
                    have_patterns = true;
                }
            }
        }
        pattern.push(')');

        let pattern = Pattern::Regex(pattern);

        let window = term_window.window.clone().unwrap();
        let mut renderer = QuickSelectRenderable {
            delegate: Arc::clone(pane),
            pattern,
            selection: "".to_string(),
            results: vec![],
            by_line: HashMap::new(),
            by_label: HashMap::new(),
            dirty_results: RangeSet::default(),
            viewport,
            last_bar_pos: None,
            window,
            result_pos: None,
            width: dims.cols,
            height: dims.viewport_rows,
            config,
            args: args.clone(),
        };

        let search_row = renderer.compute_search_row();
        renderer.dirty_results.add(search_row);
        renderer.update_search(true);

        Arc::new(QuickSelectOverlay {
            renderer: Mutex::new(renderer),
            delegate: Arc::clone(pane),
        })
    }

    pub fn viewport_changed(&self, viewport: Option<StableRowIndex>) {
        let mut render = self.renderer.lock();
        if render.viewport != viewport {
            if let Some(last) = render.last_bar_pos.take() {
                render.dirty_results.add(last);
            }
            if let Some(pos) = viewport.as_ref() {
                render.dirty_results.add(*pos);
            }
            render.viewport = viewport;
        }
    }
}
