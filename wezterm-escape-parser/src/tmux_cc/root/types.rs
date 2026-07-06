use crate::error::Context;
use crate::{bail, format_err, Result};
use parser::Rule;
use pest::iterators::{Pair, Pairs};
use pest::Parser as _;

pub type TmuxWindowId = u64;
pub type TmuxPaneId = u64;
pub type TmuxSessionId = u64;

pub mod parser {
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "tmux_cc/tmux.pest"]
    pub struct TmuxParser;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guarded {
    pub error: bool,
    pub timestamp: i64,
    pub number: u64,
    pub flags: i64,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    // Tmux generic events
    Begin {
        timestamp: i64,
        number: u64,
        flags: i64,
    },
    End {
        timestamp: i64,
        number: u64,
        flags: i64,
    },
    Error {
        timestamp: i64,
        number: u64,
        flags: i64,
    },
    Guarded(Guarded),

    // Tmux specific events
    ClientDetached {
        client_name: String,
    },
    ClientSessionChanged {
        client_name: String,
        session: TmuxSessionId,
        session_name: String,
    },
    ConfigError {
        error: String,
    },
    Continue {
        pane: TmuxPaneId,
    },
    ExtendedOutput {
        pane: TmuxPaneId,
        text: Vec<u8>,
    },
    Exit {
        reason: Option<String>,
    },
    LayoutChange {
        window: TmuxWindowId,
        layout: String,
        visible_layout: Option<String>,
        raw_flags: Option<String>,
    },
    Message {
        message: String,
    },
    Output {
        pane: TmuxPaneId,
        text: Vec<u8>,
    },
    PaneModeChanged {
        pane: TmuxPaneId,
    },
    PasteBufferChanged {
        buffer: String,
    },
    PasteBufferDeleted {
        buffer: String,
    },
    Pause {
        pane: TmuxPaneId,
    },
    SessionChanged {
        session: TmuxSessionId,
        name: String,
    },
    SessionRenamed {
        name: String,
    },
    SessionsChanged,
    SessionWindowChanged {
        session: TmuxSessionId,
        window: TmuxWindowId,
    },
    SubscriptionChanged,
    UnlinkedWindowAdd {
        window: TmuxWindowId,
    },
    UnlinkedWindowClose {
        window: TmuxWindowId,
    },
    UnlinkedWindowRenamed {
        window: TmuxWindowId,
    },
    WindowAdd {
        window: TmuxWindowId,
    },
    WindowClose {
        window: TmuxWindowId,
    },
    WindowPaneChanged {
        window: TmuxWindowId,
        pane: TmuxPaneId,
    },
    WindowRenamed {
        window: TmuxWindowId,
        name: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct PaneLayout {
    pub pane_id: TmuxPaneId,
    pub pane_width: u64,
    pub pane_height: u64,
    pub pane_left: u64,
    pub pane_top: u64,
}

#[derive(Debug)]
pub enum WindowLayout {
    SplitVertical(Vec<PaneLayout>),
    SplitHorizontal(Vec<PaneLayout>),
    SinglePane(PaneLayout),
}

fn parse_pane_id(pair: Pair<Rule>) -> Result<TmuxPaneId> {
    match pair.as_rule() {
        Rule::pane_id => {
            let mut pairs = pair.into_inner();
            pairs
                .next()
                .ok_or_else(|| format_err!("missing pane id"))?
                .as_str()
                .parse()
                .context("pane_id is somehow not digits")
        }
        _ => bail!("parse_pane_id can only parse Rule::pane_id, got {:?}", pair),
    }
}

fn parse_window_id(pair: Pair<Rule>) -> Result<TmuxWindowId> {
    match pair.as_rule() {
        Rule::window_id => {
            let mut pairs = pair.into_inner();
            pairs
                .next()
                .ok_or_else(|| format_err!("missing window id"))?
                .as_str()
                .parse()
                .context("window_id is somehow not digits")
        }
        _ => bail!(
            "parse_window_id can only parse Rule::window_id, got {:?}",
            pair
        ),
    }
}

fn parse_session_id(pair: Pair<Rule>) -> Result<TmuxSessionId> {
    match pair.as_rule() {
        Rule::session_id => {
            let mut pairs = pair.into_inner();
            pairs
                .next()
                .ok_or_else(|| format_err!("missing session id"))?
                .as_str()
                .parse()
                .context("session_id is somehow not digits")
        }
        _ => bail!(
            "parse_session_id can only parse Rule::session_id, got {:?}",
            pair
        ),
    }
}

/// Parses a %begin, %end, %error guard line tuple
fn parse_guard(mut pairs: Pairs<Rule>) -> Result<(i64, u64, i64)> {
    let timestamp = pairs
        .next()
        .ok_or_else(|| format_err!("missing timestamp"))?
        .as_str()
        .parse::<i64>()?;
    let number = pairs
        .next()
        .ok_or_else(|| format_err!("missing number"))?
        .as_str()
        .parse::<u64>()?;
    let flags = pairs
        .next()
        .ok_or_else(|| format_err!("missing flags"))?
        .as_str()
        .parse::<i64>()?;
    Ok((timestamp, number, flags))
}
