use crate::termwindow::{PaneInformation, TabInformation, UIItem, UIItemType};
use config::{ConfigHandle, TabBarColors};
use finl_unicode::grapheme_clusters::Graphemes;
use mlua::FromLua;
use mux::pane::PaneId;
use termwiz::cell::{Cell, CellAttributes, unicode_column_width};
use termwiz::color::{AnsiColor, ColorSpec};
use termwiz::escape::csi::Sgr;
use termwiz::escape::parser::Parser;
use termwiz::escape::{Action, CSI, ControlCode};
use termwiz::surface::SEQ_ZERO;
use termwiz_funcs::{FormatColor, FormatItem, format_as_escapes};
use wezterm_term::{Line, Progress};
use window::{IntegratedTitleButton, IntegratedTitleButtonAlignment, IntegratedTitleButtonStyle};

#[derive(Clone, Debug, PartialEq)]
pub struct TabBarState {
    line: Line,
    items: Vec<TabEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabBarItem {
    None,
    LeftStatus,
    CenterStatus,
    RightStatus,
    PaneStatus { pane_id: PaneId, active: bool },
    Tab { tab_idx: usize, active: bool },
    NewTabButton,
    WindowButton(IntegratedTitleButton),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabBarZone {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabBarContentMode {
    Full,
    StatusOnly,
}

impl TabBarItem {
    pub fn zone(self) -> TabBarZone {
        match self {
            Self::LeftStatus => TabBarZone::Left,
            Self::RightStatus => TabBarZone::Right,
            // Center is everything that is neither left nor right status.
            // It includes the main tabbar payload and any center overlays.
            Self::None
            | Self::CenterStatus
            | Self::PaneStatus { .. }
            | Self::Tab { .. }
            | Self::NewTabButton
            | Self::WindowButton(_) => TabBarZone::Center,
        }
    }

    pub fn is_center(self) -> bool {
        self.zone() == TabBarZone::Center
    }

    pub fn is_left_or_right(self) -> bool {
        let zone = self.zone();
        zone == TabBarZone::Left || zone == TabBarZone::Right
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TabEntry {
    pub item: TabBarItem,
    pub title: Line,
    x: usize,
    width: usize,
}

impl TabEntry {
    pub fn x(&self) -> usize {
        self.x
    }

    pub fn width(&self) -> usize {
        self.width
    }
}

#[derive(Clone, Debug)]
struct TitleText {
    items: Vec<FormatItem>,
    len: usize,
}

fn call_format_tab_title(
    tab: &TabInformation,
    tab_info: &[TabInformation],
    pane_info: &[PaneInformation],
    config: &ConfigHandle,
    hover: bool,
    tab_max_width: usize,
) -> Option<TitleText> {
    match config::run_immediate_with_lua_config(|lua| {
        if let Some(lua) = lua {
            let tabs = lua.create_sequence_from(tab_info.iter().cloned())?;
            let panes = lua.create_sequence_from(pane_info.iter().cloned())?;

            let v = config::lua::emit_sync_callback(
                &*lua,
                (
                    "format-tab-title".to_string(),
                    (
                        tab.clone(),
                        tabs,
                        panes,
                        (**config).clone(),
                        hover,
                        tab_max_width,
                    ),
                ),
            )?;
            match &v {
                mlua::Value::Nil => Ok(None),
                mlua::Value::Table(_) => {
                    let items = <Vec<FormatItem>>::from_lua(v, &*lua)?;

                    let esc = format_as_escapes(items.clone())?;
                    let line = parse_status_text(&esc, CellAttributes::default());

                    Ok(Some(TitleText {
                        items,
                        len: line.len(),
                    }))
                }
                _ => {
                    let s = String::from_lua(v, &*lua)?;
                    let line = parse_status_text(&s, CellAttributes::default());
                    Ok(Some(TitleText {
                        len: line.len(),
                        items: vec![FormatItem::Text(s)],
                    }))
                }
            }
        } else {
            Ok(None)
        }
    }) {
        Ok(s) => s,
        Err(err) => {
            log::warn!("format-tab-title: {}", err);
            None
        }
    }
}

/// pct is a percentage in the range 0-100.
/// We want to map it to one of the nerdfonts:
///
/// * `md-checkbox_blank_circle_outline` (0xf0130) for an empty circle
/// * `md_circle_slice_1..=7` (0xf0a9e ..= 0xf0aa4) for a partly filled
///   circle
/// * `md_circle_slice_8` (0xf0aa5) for a filled circle
///
/// We use an empty circle for values close to 0%, a filled circle for values
/// close to 100%, and a partly filled circle for the rest (roughly evenly
/// distributed).
fn pct_to_glyph(pct: u8) -> char {
    match pct {
        0..=5 => '\u{f0130}',    // empty circle
        6..=18 => '\u{f0a9e}',   // centered at 12 (slightly smaller than 12.5)
        19..=31 => '\u{f0a9f}',  // centered at 25
        32..=43 => '\u{f0aa0}',  // centered at 37.5
        44..=56 => '\u{f0aa1}',  // half-filled circle, centered at 50
        57..=68 => '\u{f0aa2}',  // centered at 62.5
        69..=81 => '\u{f0aa3}',  // centered at 75
        82..=94 => '\u{f0aa4}',  // centered at 88 (slightly larger than 87.5)
        95..=100 => '\u{f0aa5}', // filled circle
        // Any other value is mapped to a filled circle.
        _ => '\u{f0aa5}',
    }
}

fn compute_tab_title(
    tab: &TabInformation,
    tab_info: &[TabInformation],
    pane_info: &[PaneInformation],
    config: &ConfigHandle,
    hover: bool,
    tab_max_width: usize,
) -> TitleText {
    let title = call_format_tab_title(tab, tab_info, pane_info, config, hover, tab_max_width);

    match title {
        Some(title) => title,
        Option::None => {
            let mut items = vec![];
            let mut len = 0;

            if let Some(pane) = &tab.active_pane {
                let mut title = if tab.tab_title.is_empty() {
                    pane.title.clone()
                } else {
                    tab.tab_title.clone()
                };

                let classic_spacing = if config.use_fancy_tab_bar { "" } else { " " };
                if config.show_tab_index_in_tab_bar {
                    let index = format!(
                        "{classic_spacing}{}: ",
                        tab.tab_index
                            + if config.tab_and_split_indices_are_zero_based {
                                0
                            } else {
                                1
                            }
                    );
                    len += unicode_column_width(&index, None);
                    items.push(FormatItem::Text(index));

                    title = format!("{}{classic_spacing}", title);
                }

                match pane.progress {
                    Progress::None => {}
                    Progress::Percentage(pct) | Progress::Error(pct) => {
                        let graphic = format!("{} ", pct_to_glyph(pct));
                        len += unicode_column_width(&graphic, None);
                        let color = if matches!(pane.progress, Progress::Percentage(_)) {
                            FormatItem::Foreground(FormatColor::AnsiColor(AnsiColor::Green))
                        } else {
                            FormatItem::Foreground(FormatColor::AnsiColor(AnsiColor::Red))
                        };
                        items.push(color);
                        items.push(FormatItem::Text(graphic));
                        items.push(FormatItem::Foreground(FormatColor::Default));
                    }
                    Progress::Indeterminate => {
                        // TODO: Decide what to do here to indicate this
                    }
                }

                // We have a preferred soft minimum on tab width to make it
                // easier to click on tab titles, but we'll still go below
                // this if there are too many tabs to fit the window at
                // this width.
                if !config.use_fancy_tab_bar {
                    while len + unicode_column_width(&title, None) < 5 {
                        title.push(' ');
                    }
                }

                len += unicode_column_width(&title, None);
                items.push(FormatItem::Text(title));
            } else {
                let title = " no pane ".to_string();
                len += unicode_column_width(&title, None);
                items.push(FormatItem::Text(title));
            };

            TitleText { len, items }
        }
    }
}

fn is_tab_hover(mouse_x: Option<usize>, x: usize, tab_title_len: usize) -> bool {
    return mouse_x
        .map(|mouse_x| mouse_x >= x && mouse_x < x + tab_title_len)
        .unwrap_or(false);
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PaneLabelFormatter {
    zero_based: bool,
    max_title_cell_width: usize,
}

impl PaneLabelFormatter {
    fn new(zero_based: bool) -> Self {
        Self {
            zero_based,
            max_title_cell_width: 12,
        }
    }

    fn display_index(&self, pane_index: usize) -> usize {
        pane_index + usize::from(!self.zero_based)
    }

    fn title_for(&self, title: &str) -> String {
        let title = title.rsplit(['/', '\\']).next().unwrap_or(title);
        let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
        let title = if title.is_empty() {
            "shell".to_string()
        } else {
            title
        };

        truncate_to_cell_width(&title, self.max_title_cell_width)
    }

    fn label_for(&self, pane_index: usize, title: &str) -> String {
        format!(
            " {}:{} ",
            self.display_index(pane_index),
            self.title_for(title)
        )
    }
}

fn truncate_to_cell_width(text: &str, max_width: usize) -> String {
    if unicode_column_width(text, None) <= max_width {
        return text.to_string();
    }

    if max_width == 0 {
        return String::new();
    }

    let ellipsis = "…";
    let ellipsis_width = unicode_column_width(ellipsis, None);
    let content_width = max_width.saturating_sub(ellipsis_width);
    let mut result = String::new();
    let mut width = 0;

    for grapheme in Graphemes::new(text) {
        let grapheme_width = unicode_column_width(grapheme, None);
        if width + grapheme_width > content_width {
            break;
        }
        result.push_str(grapheme);
        width += grapheme_width;
    }

    result.push_str(ellipsis);
    result
}

impl TabBarState {
    pub fn default() -> Self {
        Self {
            line: Line::with_width(1, SEQ_ZERO),
            items: vec![TabEntry {
                item: TabBarItem::None,
                title: Line::from_text(" ", &CellAttributes::blank(), 1, None),
                x: 1,
                width: 1,
            }],
        }
    }

    pub fn line(&self) -> &Line {
        &self.line
    }

    pub fn items(&self) -> &[TabEntry] {
        &self.items
    }

    fn integrated_title_buttons(
        mouse_x: Option<usize>,
        x: &mut usize,
        config: &ConfigHandle,
        items: &mut Vec<TabEntry>,
        line: &mut Line,
        colors: &TabBarColors,
    ) {
        let default_cell = if config.use_fancy_tab_bar {
            CellAttributes::default()
        } else {
            colors.new_tab().as_cell_attributes()
        };

        let default_cell_hover = if config.use_fancy_tab_bar {
            CellAttributes::default()
        } else {
            colors.new_tab_hover().as_cell_attributes()
        };

        let window_hide =
            parse_status_text(&config.tab_bar_style.window_hide, default_cell.clone());
        let window_hide_hover = parse_status_text(
            &config.tab_bar_style.window_hide_hover,
            default_cell_hover.clone(),
        );

        let window_maximize =
            parse_status_text(&config.tab_bar_style.window_maximize, default_cell.clone());
        let window_maximize_hover = parse_status_text(
            &config.tab_bar_style.window_maximize_hover,
            default_cell_hover.clone(),
        );

        let window_close =
            parse_status_text(&config.tab_bar_style.window_close, default_cell.clone());
        let window_close_hover = parse_status_text(
            &config.tab_bar_style.window_close_hover,
            default_cell_hover.clone(),
        );

        for button in &config.integrated_title_buttons {
            use IntegratedTitleButton as Button;
            let title = match button {
                Button::Hide => {
                    let hover = is_tab_hover(mouse_x, *x, window_hide_hover.len());

                    if hover {
                        &window_hide_hover
                    } else {
                        &window_hide
                    }
                }
                Button::Maximize => {
                    let hover = is_tab_hover(mouse_x, *x, window_maximize_hover.len());

                    if hover {
                        &window_maximize_hover
                    } else {
                        &window_maximize
                    }
                }
                Button::Close => {
                    let hover = is_tab_hover(mouse_x, *x, window_close_hover.len());

                    if hover {
                        &window_close_hover
                    } else {
                        &window_close
                    }
                }
            };

            line.append_line(title.to_owned(), SEQ_ZERO);

            let width = title.len();
            items.push(TabEntry {
                item: TabBarItem::WindowButton(*button),
                title: title.to_owned(),
                x: *x,
                width,
            });

            *x += width;
        }
    }

    fn append_center_pane_status(
        x: &mut usize,
        pane_info: &[PaneInformation],
        items: &mut Vec<TabEntry>,
        line: &mut Line,
        default_attrs: &CellAttributes,
        zero_based: bool,
    ) {
        if pane_info.len() <= 1 {
            return;
        }

        for (idx, pane) in pane_info.iter().enumerate() {
            let formatter = PaneLabelFormatter::new(zero_based);
            let label = formatter.label_for(pane.pane_index, &pane.title);
            let pane_line = parse_status_text(&label, default_attrs.clone());
            let width = pane_line.len();
            items.push(TabEntry {
                item: TabBarItem::PaneStatus {
                    pane_id: pane.pane_id,
                    active: pane.is_active,
                },
                title: pane_line.clone(),
                x: *x,
                width,
            });
            line.append_line(pane_line, SEQ_ZERO);
            *x += width;

            if idx + 1 < pane_info.len() {
                let sep = parse_status_text(" ", default_attrs.clone());
                line.append_line(sep, SEQ_ZERO);
                *x += 1;
            }
        }
    }

    /// Build a new tab bar from the current state
    /// mouse_x is some if the mouse is on the same row as the tab bar.
    /// title_width is the total number of cell columns in the window.
    /// window allows access to the tabs associated with the window.
    pub fn new_primary(
        title_width: usize,
        mouse_x: Option<usize>,
        tab_info: &[TabInformation],
        pane_info: &[PaneInformation],
        colors: Option<&TabBarColors>,
        config: &ConfigHandle,
        left_status: &str,
        right_status: &str,
    ) -> Self {
        Self::build(
            title_width,
            mouse_x,
            tab_info,
            pane_info,
            colors,
            config,
            left_status,
            "",
            right_status,
            TabBarContentMode::Full,
        )
    }

    pub fn new_status_bar(
        title_width: usize,
        tab_info: &[TabInformation],
        pane_info: &[PaneInformation],
        colors: Option<&TabBarColors>,
        config: &ConfigHandle,
        left_status: &str,
        center_status: &str,
        right_status: &str,
    ) -> Self {
        Self::build(
            title_width,
            None,
            tab_info,
            pane_info,
            colors,
            config,
            left_status,
            center_status,
            right_status,
            TabBarContentMode::StatusOnly,
        )
    }

    fn build(
        title_width: usize,
        mouse_x: Option<usize>,
        tab_info: &[TabInformation],
        pane_info: &[PaneInformation],
        colors: Option<&TabBarColors>,
        config: &ConfigHandle,
        left_status: &str,
        center_status: &str,
        right_status: &str,
        content_mode: TabBarContentMode,
    ) -> Self {
        let colors = colors.cloned().unwrap_or_else(TabBarColors::default);

        let active_cell_attrs = colors.active_tab().as_cell_attributes();
        let inactive_hover_attrs = colors.inactive_tab_hover().as_cell_attributes();
        let inactive_cell_attrs = colors.inactive_tab().as_cell_attributes();
        let new_tab_hover_attrs = colors.new_tab_hover().as_cell_attributes();
        let new_tab_attrs = colors.new_tab().as_cell_attributes();

        let new_tab = parse_status_text(
            &config.tab_bar_style.new_tab,
            if config.use_fancy_tab_bar {
                CellAttributes::default()
            } else {
                new_tab_attrs.clone()
            },
        );
        let new_tab_hover = parse_status_text(
            &config.tab_bar_style.new_tab_hover,
            if config.use_fancy_tab_bar {
                CellAttributes::default()
            } else {
                new_tab_hover_attrs.clone()
            },
        );

        let use_integrated_title_buttons = config
            .window_decorations
            .contains(window::WindowDecorations::INTEGRATED_BUTTONS);

        // We ultimately want to produce a line looking like this:
        // ` | tab1-title x | tab2-title x |  +      . - X `
        // Where the `+` sign will spawn a new tab (or show a context
        // menu with tab creation options) and the other three chars
        // are symbols representing minimize, maximize and close.

        let mut active_tab_no = 0;

        let tab_titles: Vec<TitleText> =
            if matches!(content_mode, TabBarContentMode::Full) && config.show_tabs_in_tab_bar {
                tab_info
                    .iter()
                    .map(|tab| {
                        if tab.is_active {
                            active_tab_no = tab.tab_index;
                        }
                        compute_tab_title(
                            tab,
                            tab_info,
                            pane_info,
                            config,
                            false,
                            config.tab_max_width,
                        )
                    })
                    .collect()
            } else {
                vec![]
            };
        let titles_len: usize = tab_titles.iter().map(|s| s.len).sum();
        let number_of_tabs = tab_titles.len();

        let new_tab_len = if matches!(content_mode, TabBarContentMode::Full)
            && config.show_new_tab_button_in_tab_bar
        {
            new_tab.len()
        } else {
            0
        };
        let available_cells =
            title_width.saturating_sub(number_of_tabs.saturating_sub(1) + new_tab_len);
        let tab_width_max = if config.use_fancy_tab_bar || available_cells >= titles_len {
            // We can render each title with its full width
            usize::max_value()
        } else {
            // We need to clamp the length to balance them out
            available_cells / number_of_tabs
        }
        .min(config.tab_max_width);

        let mut line = Line::with_width(0, SEQ_ZERO);

        let mut x = 0;
        let mut items = vec![];

        let black_cell = Cell::blank_with_attrs(
            CellAttributes::default()
                .set_background(ColorSpec::TrueColor(*colors.background()))
                .clone(),
        );

        if matches!(content_mode, TabBarContentMode::Full)
            && use_integrated_title_buttons
            && config.integrated_title_button_style == IntegratedTitleButtonStyle::MacOsNative
            && config.use_fancy_tab_bar == false
            && config.tab_bar_at_bottom == false
        {
            for _ in 0..10 as usize {
                line.insert_cell(0, black_cell.clone(), title_width, SEQ_ZERO);
                x += 1;
            }
        }

        if matches!(content_mode, TabBarContentMode::Full)
            && use_integrated_title_buttons
            && config.integrated_title_button_style != IntegratedTitleButtonStyle::MacOsNative
            && config.integrated_title_button_alignment == IntegratedTitleButtonAlignment::Left
        {
            Self::integrated_title_buttons(mouse_x, &mut x, config, &mut items, &mut line, &colors);
        }

        let left_status_line = parse_status_text(left_status, black_cell.attrs().clone());
        if left_status_line.len() > 0 {
            items.push(TabEntry {
                item: TabBarItem::LeftStatus,
                title: left_status_line.clone(),
                x,
                width: left_status_line.len(),
            });
            x += left_status_line.len();
            line.append_line(left_status_line, SEQ_ZERO);
        }

        let mut deferred_center_status = None;
        if center_status.is_empty() {
            // nothing
        } else if matches!(content_mode, TabBarContentMode::StatusOnly) {
            deferred_center_status = Some(center_status.to_string());
        } else {
            let center_status_line = parse_status_text(center_status, black_cell.attrs().clone());
            if center_status_line.len() > 0 {
                items.push(TabEntry {
                    item: TabBarItem::CenterStatus,
                    title: center_status_line.clone(),
                    x,
                    width: center_status_line.len(),
                });
                x += center_status_line.len();
                line.append_line(center_status_line, SEQ_ZERO);
            }
        }

        for (tab_idx, tab_title) in tab_titles.iter().enumerate() {
            let tab_title_len = tab_title.len.min(tab_width_max);
            let active = tab_idx == active_tab_no;
            let hover = !active && is_tab_hover(mouse_x, x, tab_title_len);

            // Recompute the title so that it factors in both the hover state
            // and the adjusted maximum tab width based on available space.
            let tab_title = compute_tab_title(
                &tab_info[tab_idx],
                tab_info,
                pane_info,
                config,
                hover,
                tab_title_len,
            );

            let cell_attrs = if active {
                &active_cell_attrs
            } else if hover {
                &inactive_hover_attrs
            } else {
                &inactive_cell_attrs
            };

            let tab_start_idx = x;

            let esc = format_as_escapes(tab_title.items.clone()).expect("already parsed ok above");
            let mut tab_line = parse_status_text(
                &esc,
                if config.use_fancy_tab_bar {
                    CellAttributes::default()
                } else {
                    cell_attrs.clone()
                },
            );

            let title = tab_line.clone();
            if tab_line.len() > tab_width_max {
                tab_line.resize(tab_width_max, SEQ_ZERO);
            }

            let width = tab_line.len();

            items.push(TabEntry {
                item: TabBarItem::Tab { tab_idx, active },
                title,
                x: tab_start_idx,
                width,
            });

            line.append_line(tab_line, SEQ_ZERO);
            x += width;
        }

        if matches!(content_mode, TabBarContentMode::Full) {
            Self::append_center_pane_status(
                &mut x,
                pane_info,
                &mut items,
                &mut line,
                black_cell.attrs(),
                config.tab_and_split_indices_are_zero_based,
            );

            // New tab button
            if config.show_new_tab_button_in_tab_bar {
                let hover = is_tab_hover(mouse_x, x, new_tab_hover.len());

                let new_tab_button = if hover { &new_tab_hover } else { &new_tab };

                let button_start = x;
                let width = new_tab_button.len();

                line.append_line(new_tab_button.clone(), SEQ_ZERO);

                items.push(TabEntry {
                    item: TabBarItem::NewTabButton,
                    title: new_tab_button.clone(),
                    x: button_start,
                    width,
                });

                x += width;
            }
        }

        // Reserve place for integrated title buttons
        let title_width = if matches!(content_mode, TabBarContentMode::Full)
            && use_integrated_title_buttons
            && config.integrated_title_button_style != IntegratedTitleButtonStyle::MacOsNative
            && config.integrated_title_button_alignment == IntegratedTitleButtonAlignment::Right
        {
            let window_hide =
                parse_status_text(&config.tab_bar_style.window_hide, CellAttributes::default());
            let window_hide_hover = parse_status_text(
                &config.tab_bar_style.window_hide_hover,
                CellAttributes::default(),
            );

            let window_maximize = parse_status_text(
                &config.tab_bar_style.window_maximize,
                CellAttributes::default(),
            );
            let window_maximize_hover = parse_status_text(
                &config.tab_bar_style.window_maximize_hover,
                CellAttributes::default(),
            );
            let window_close = parse_status_text(
                &config.tab_bar_style.window_close,
                CellAttributes::default(),
            );
            let window_close_hover = parse_status_text(
                &config.tab_bar_style.window_close_hover,
                CellAttributes::default(),
            );

            let hide_len = window_hide.len().max(window_hide_hover.len());
            let maximize_len = window_maximize.len().max(window_maximize_hover.len());
            let close_len = window_close.len().max(window_close_hover.len());

            let mut width_to_reserve = 0;
            for button in &config.integrated_title_buttons {
                use IntegratedTitleButton as Button;
                let button_len = match button {
                    Button::Hide => hide_len,
                    Button::Maximize => maximize_len,
                    Button::Close => close_len,
                };
                width_to_reserve += button_len;
            }

            title_width.saturating_sub(width_to_reserve)
        } else {
            title_width
        };

        if let Some(center_status) = deferred_center_status {
            let mut right_status_line = parse_status_text(right_status, black_cell.attrs().clone());
            let right_status_len = right_status_line.len();
            let center_space_available = title_width.saturating_sub(x + right_status_len);

            let mut center_status_line =
                parse_status_text(&center_status, black_cell.attrs().clone());
            if center_status_line.len() > center_space_available {
                center_status_line.resize(center_space_available, SEQ_ZERO);
            }
            while center_status_line.len() < center_space_available {
                center_status_line.insert_cell(
                    center_status_line.len(),
                    black_cell.clone(),
                    center_space_available,
                    SEQ_ZERO,
                );
            }
            if center_status_line.len() > 0 {
                items.push(TabEntry {
                    item: TabBarItem::CenterStatus,
                    title: center_status_line.clone(),
                    x,
                    width: center_status_line.len(),
                });
                x += center_status_line.len();
                line.append_line(center_status_line, SEQ_ZERO);
            }

            let status_space_available = title_width.saturating_sub(x);
            items.push(TabEntry {
                item: TabBarItem::RightStatus,
                title: right_status_line.clone(),
                x,
                width: status_space_available,
            });
            while right_status_line.len() > status_space_available {
                right_status_line.remove_cell(0, SEQ_ZERO);
            }
            line.append_line(right_status_line.clone(), SEQ_ZERO);
        } else {
            let status_space_available = title_width.saturating_sub(x);

            let mut right_status_line = parse_status_text(right_status, black_cell.attrs().clone());
            items.push(TabEntry {
                item: TabBarItem::RightStatus,
                title: right_status_line.clone(),
                x,
                width: status_space_available,
            });

            while right_status_line.len() > status_space_available {
                right_status_line.remove_cell(0, SEQ_ZERO);
            }

            line.append_line(right_status_line.clone(), SEQ_ZERO);
        }
        // Right status remains separate; pane chips are formal center items.
        while line.len() < title_width {
            line.insert_cell(x, black_cell.clone(), title_width, SEQ_ZERO);
        }

        if matches!(content_mode, TabBarContentMode::Full)
            && use_integrated_title_buttons
            && config.integrated_title_button_style != IntegratedTitleButtonStyle::MacOsNative
            && config.integrated_title_button_alignment == IntegratedTitleButtonAlignment::Right
        {
            x = title_width;
            Self::integrated_title_buttons(mouse_x, &mut x, config, &mut items, &mut line, &colors);
        }

        Self { line, items }
    }

    pub fn compute_ui_items(&self, y: usize, cell_height: usize, cell_width: usize) -> Vec<UIItem> {
        let mut items = vec![];

        for entry in self.items.iter() {
            items.push(UIItem {
                x: entry.x * cell_width,
                width: entry.width * cell_width,
                y,
                height: cell_height,
                item_type: UIItemType::TabBar(entry.item),
            });
        }

        items
    }
}

pub fn parse_status_text(text: &str, default_cell: CellAttributes) -> Line {
    let mut pen = default_cell.clone();
    let mut cells = vec![];
    let mut ignoring = false;
    let mut print_buffer = String::new();

    fn flush_print(buf: &mut String, cells: &mut Vec<Cell>, pen: &CellAttributes) {
        for g in Graphemes::new(buf.as_str()) {
            let cell = Cell::new_grapheme(g, pen.clone(), None);
            let width = cell.width();
            cells.push(cell);
            for _ in 1..width {
                // Line/Screen expect double wide graphemes to be followed by a blank in
                // the next column position, otherwise we'll render incorrectly
                cells.push(Cell::blank_with_attrs(pen.clone()));
            }
        }
        buf.clear();
    }

    let mut parser = Parser::new();
    parser.parse(text.as_bytes(), |action| {
        if ignoring {
            return;
        }
        match action {
            Action::Print(c) => print_buffer.push(c),
            Action::PrintString(s) => print_buffer.push_str(&s),
            Action::Control(c) => {
                flush_print(&mut print_buffer, &mut cells, &pen);
                match c {
                    ControlCode::CarriageReturn | ControlCode::LineFeed => {
                        ignoring = true;
                    }
                    _ => {}
                }
            }
            Action::CSI(csi) => {
                flush_print(&mut print_buffer, &mut cells, &pen);
                match csi {
                    CSI::Sgr(sgr) => match sgr {
                        Sgr::Reset => pen = default_cell.clone(),
                        Sgr::Intensity(i) => {
                            pen.set_intensity(i);
                        }
                        Sgr::Underline(u) => {
                            pen.set_underline(u);
                        }
                        Sgr::Overline(o) => {
                            pen.set_overline(o);
                        }
                        Sgr::VerticalAlign(o) => {
                            pen.set_vertical_align(o);
                        }
                        Sgr::Blink(b) => {
                            pen.set_blink(b);
                        }
                        Sgr::Italic(i) => {
                            pen.set_italic(i);
                        }
                        Sgr::Inverse(inverse) => {
                            pen.set_reverse(inverse);
                        }
                        Sgr::Invisible(invis) => {
                            pen.set_invisible(invis);
                        }
                        Sgr::StrikeThrough(strike) => {
                            pen.set_strikethrough(strike);
                        }
                        Sgr::Foreground(col) => {
                            if let ColorSpec::Default = col {
                                pen.set_foreground(default_cell.foreground());
                            } else {
                                pen.set_foreground(col);
                            }
                        }
                        Sgr::Background(col) => {
                            if let ColorSpec::Default = col {
                                pen.set_background(default_cell.background());
                            } else {
                                pen.set_background(col);
                            }
                        }
                        Sgr::UnderlineColor(col) => {
                            pen.set_underline_color(col);
                        }
                        Sgr::Font(_) => {}
                    },
                    _ => {}
                }
            }
            Action::OperatingSystemCommand(_)
            | Action::DeviceControl(_)
            | Action::Esc(_)
            | Action::KittyImage(_)
            | Action::XtGetTcap(_)
            | Action::Sixel(_) => {
                flush_print(&mut print_buffer, &mut cells, &pen);
            }
        }
    });
    flush_print(&mut print_buffer, &mut cells, &pen);
    Line::from_cells(cells, SEQ_ZERO)
}

#[cfg(test)]
mod pane_label_tests {
    use super::PaneLabelFormatter;
    use termwiz::cell::unicode_column_width;

    #[test]
    fn normalizes_empty_title_to_shell() {
        let formatter = PaneLabelFormatter::new(false);

        assert_eq!(formatter.title_for("   \t  "), "shell");
        assert_eq!(formatter.label_for(0, ""), " 1:shell ");
    }

    #[test]
    fn keeps_basename_and_collapses_whitespace() {
        let formatter = PaneLabelFormatter::new(false);

        assert_eq!(
            formatter.title_for(r"C:\Users\me\project   shell.exe"),
            "project she…"
        );
        assert_eq!(formatter.title_for("/tmp/my    app"), "my app");
    }

    #[test]
    fn supports_zero_based_indices() {
        let one_based = PaneLabelFormatter::new(false);
        let zero_based = PaneLabelFormatter::new(true);

        assert_eq!(one_based.label_for(2, "pwsh"), " 3:pwsh ");
        assert_eq!(zero_based.label_for(2, "pwsh"), " 2:pwsh ");
    }

    #[test]
    fn truncates_without_exceeding_cell_width() {
        let formatter = PaneLabelFormatter::new(false);
        let title = formatter.title_for("abcdefghijklmnopqrstuvwxyz");

        assert_eq!(title, "abcdefghijk…");
        assert!(unicode_column_width(&title, None) <= 12);
    }

    #[test]
    fn truncates_cjk_by_cell_width() {
        let formatter = PaneLabelFormatter::new(false);
        let title = formatter.title_for("界界界界界界界");

        assert_eq!(title, "界界界界界…");
        assert!(unicode_column_width(&title, None) <= 12);
    }

    #[test]
    fn truncates_emoji_without_splitting_graphemes() {
        let formatter = PaneLabelFormatter::new(false);
        let title = formatter.title_for("😀😀😀😀😀😀😀");

        assert!(title.ends_with('…'));
        assert!(unicode_column_width(&title, None) <= 12);
    }
}

#[cfg(test)]
mod tab_bar_constructor_tests {
    use super::{TabBarItem, TabBarState};
    use crate::termwindow::{PaneInformation, TabInformation};
    use config::ConfigHandle;

    #[test]
    fn primary_constructor_does_not_emit_center_status_without_center_input() {
        let config = ConfigHandle::default_config();
        let tabs: Vec<TabInformation> = vec![];
        let panes: Vec<PaneInformation> = vec![];
        let tab_bar =
            TabBarState::new_primary(80, None, &tabs, &panes, None, &config, "LEFT", "RIGHT");

        assert!(
            tab_bar
                .items()
                .iter()
                .any(|entry| entry.item == TabBarItem::LeftStatus)
        );
        assert!(
            tab_bar
                .items()
                .iter()
                .any(|entry| entry.item == TabBarItem::RightStatus)
        );
        assert!(
            !tab_bar
                .items()
                .iter()
                .any(|entry| entry.item == TabBarItem::CenterStatus)
        );
        assert!(
            !tab_bar
                .items()
                .iter()
                .any(|entry| matches!(entry.item, TabBarItem::PaneStatus { .. }))
        );
    }

    #[test]
    fn status_bar_constructor_never_emits_activation_items() {
        let config = ConfigHandle::default_config();
        let tabs: Vec<TabInformation> = vec![];
        let panes: Vec<PaneInformation> = vec![];
        let tab_bar = TabBarState::new_status_bar(
            80, &tabs, &panes, None, &config, "LEFT", "CENTER", "RIGHT",
        );

        assert!(
            tab_bar
                .items()
                .iter()
                .any(|entry| entry.item == TabBarItem::LeftStatus)
        );
        assert!(
            tab_bar
                .items()
                .iter()
                .any(|entry| entry.item == TabBarItem::CenterStatus)
        );
        assert!(
            tab_bar
                .items()
                .iter()
                .any(|entry| entry.item == TabBarItem::RightStatus)
        );
        assert!(
            !tab_bar
                .items()
                .iter()
                .any(|entry| matches!(entry.item, TabBarItem::Tab { .. }))
        );
        assert!(
            !tab_bar
                .items()
                .iter()
                .any(|entry| entry.item == TabBarItem::NewTabButton)
        );
    }
}
