use crate::commands::derive_command_from_key_assignment;
use crate::overlay::quickselect;
use crate::overlay::selector::{matcher_pattern, matcher_score};
use crate::termwindow::TermWindowNotif;
use config::configuration;
use config::keyassignment::{KeyAssignment, SpawnCommand, SpawnTabDomain};
use mux::domain::{DomainId, DomainState};
use mux::pane::PaneId;
use mux::termwiztermtab::TermWizTerminal;
use mux::window::WindowId;
use mux::Mux;
use rayon::prelude::*;
use std::collections::BTreeMap;
use termwiz::cell::{AttributeChange, CellAttributes};
use termwiz::color::ColorAttribute;
use termwiz::input::{InputEvent, KeyCode, KeyEvent, Modifiers, MouseButtons, MouseEvent};
use termwiz::surface::{Change, Position};
use termwiz::terminal::Terminal;
use termwiz_funcs::truncate_right;
use window::WindowOps;

pub use config::keyassignment::LauncherFlags;

#[derive(Clone)]
struct Entry {
    pub label: String,
    pub action: KeyAssignment,
}

pub struct LauncherTabEntry {
    pub title: String,
    pub tab_idx: usize,
    pub pane_count: Option<usize>,
}

#[derive(Debug)]
pub struct LauncherDomainEntry {
    pub domain_id: DomainId,
    pub name: String,
    pub state: DomainState,
    pub label: String,
}

pub struct LauncherArgs {
    flags: LauncherFlags,
    domains: Vec<LauncherDomainEntry>,
    tabs: Vec<LauncherTabEntry>,
    pane_id: PaneId,
    domain_id_of_current_tab: DomainId,
    title: String,
    active_workspace: String,
    workspaces: Vec<String>,
    help_text: String,
    fuzzy_help_text: String,
    alphabet: String,
}

impl LauncherArgs {
    /// Must be called on the Mux thread!
    pub async fn new(
        title: &str,
        flags: LauncherFlags,
        mux_window_id: WindowId,
        pane_id: PaneId,
        domain_id_of_current_tab: DomainId,
        help_text: &str,
        fuzzy_help_text: &str,
        alphabet: &str,
    ) -> Self {
        let mux = Mux::get();

        let active_workspace = mux.active_workspace();

        let workspaces = if flags.contains(LauncherFlags::WORKSPACES) {
            mux.iter_workspaces()
        } else {
            vec![]
        };

        let tabs = if flags.contains(LauncherFlags::TABS) {
            // Ideally we'd resolve the tabs on the fly once we've started the
            // overlay, but since the overlay runs in a different thread, accessing
            // the mux list is a bit awkward.  To get the ball rolling we capture
            // the list of tabs up front and live with a static list.
            let window = mux
                .get_window(mux_window_id)
                .expect("to resolve my own window_id");
            window
                .iter()
                .enumerate()
                .map(|(tab_idx, tab)| {
                    let tab_title = tab.get_title();
                    let title = if tab_title.is_empty() {
                        tab.get_active_pane()
                            .expect("tab to have a pane")
                            .get_title()
                    } else {
                        tab_title
                    };
                    LauncherTabEntry {
                        title,
                        tab_idx,
                        pane_count: tab.count_panes(),
                    }
                })
                .collect()
        } else {
            vec![]
        };

        let domains = if flags.contains(LauncherFlags::DOMAINS) {
            let mut domains = mux.iter_domains();
            domains.sort_by(|a, b| {
                let a_state = a.state();
                let b_state = b.state();
                if a_state != b_state {
                    use std::cmp::Ordering;
                    return if a_state == DomainState::Attached {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    };
                }
                a.domain_id().cmp(&b.domain_id())
            });
            domains.retain(|dom| dom.spawnable());
            let mut d = vec![];
            for dom in domains.into_iter() {
                let name = dom.domain_name();
                let label = dom.domain_label().await;
                let label = if name == label || label == "" {
                    format!("domain `{}`", name)
                } else {
                    format!("domain `{}` - {}", name, label)
                };
                d.push(LauncherDomainEntry {
                    domain_id: dom.domain_id(),
                    name: name.to_string(),
                    state: dom.state(),
                    label,
                });
            }
            d
        } else {
            vec![]
        };

        Self {
            flags,
            domains,
            tabs,
            pane_id,
            domain_id_of_current_tab,
            title: title.to_string(),
            workspaces,
            active_workspace,
            help_text: help_text.to_string(),
            fuzzy_help_text: fuzzy_help_text.to_string(),
            alphabet: alphabet.to_string(),
        }
    }
}

const ROW_OVERHEAD: usize = 3;

struct LauncherState {
    active_idx: usize,
    max_items: usize,
    top_row: usize,
    entries: Vec<Entry>,
    filter_term: String,
    filtered_entries: Vec<Entry>,
    pane_id: PaneId,
    window: ::window::Window,
    filtering: bool,
    help_text: String,
    fuzzy_help_text: String,
    labels: Vec<String>,
    alphabet: String,
    selection: String,
    always_fuzzy: bool,
}
