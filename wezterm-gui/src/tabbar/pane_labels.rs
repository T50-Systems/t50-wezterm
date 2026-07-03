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
