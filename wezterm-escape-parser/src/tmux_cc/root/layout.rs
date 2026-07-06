fn parse_layout_pane(pair: Pair<Rule>) -> Result<PaneLayout> {
    let mut pairs = pair.into_inner();

    let pane_width = pairs
        .next()
        .ok_or_else(|| format_err!("wrong pane layout format"))?
        .as_str()
        .parse()?;
    let pane_height = pairs
        .next()
        .ok_or_else(|| format_err!("wrong pane layout format"))?
        .as_str()
        .parse()?;
    let pane_left = pairs
        .next()
        .ok_or_else(|| format_err!("wrong pane layout format"))?
        .as_str()
        .parse()?;
    let pane_top = pairs
        .next()
        .ok_or_else(|| format_err!("wrong pane layout format"))?
        .as_str()
        .parse()?;

    let pane_id = match pairs.next() {
        Some(x) => x.as_str().parse()?,
        None => 0,
    };

    return Ok(PaneLayout {
        pane_id,
        pane_width,
        pane_height,
        pane_left,
        pane_top,
    });
}

fn parse_layout_inner(
    mut pairs: Pairs<Rule>,
    result: &mut Vec<WindowLayout>,
) -> Result<Vec<PaneLayout>> {
    let mut stack = Vec::new();

    while let Some(pair) = pairs.next() {
        let rule = pair.as_rule();
        match rule {
            Rule::layout_split_horizontal | Rule::layout_split_vertical => {
                let mut pairs_inner = pair.into_inner();
                let pair = pairs_inner
                    .next()
                    .ok_or_else(|| format_err!("no pairs!?"))?;
                let mut pane = parse_layout_pane(pair)?;

                if result.is_empty() {
                    // Fake one, to flag it is not a TmuxLayout::SinglePane will pop
                    result.push(WindowLayout::SplitHorizontal(vec![]));
                }

                let mut layout_inner = parse_layout_inner(pairs_inner, result)?;

                let last_item = layout_inner
                    .pop()
                    .ok_or_else(|| format_err!("wrong layout format"))?;

                pane.pane_id = last_item.pane_id;

                layout_inner.insert(0, pane.clone());

                if let Rule::layout_split_horizontal = rule {
                    result.insert(0, WindowLayout::SplitHorizontal(layout_inner));
                } else {
                    result.insert(0, WindowLayout::SplitVertical(layout_inner));
                }

                stack.push(pane);
            }
            Rule::layout_pane => {
                let pane = parse_layout_pane(pair)?;

                // SinglePane
                if result.is_empty() {
                    result.insert(0, WindowLayout::SinglePane(pane));
                    return Ok(stack);
                }

                stack.push(pane);
            }
            Rule::EOI
            | Rule::any_text
            | Rule::begin
            | Rule::client_detached
            | Rule::client_name
            | Rule::client_session_changed
            | Rule::config_error
            | Rule::r#continue
            | Rule::end
            | Rule::error
            | Rule::exit
            | Rule::extended_output
            | Rule::layout_change
            | Rule::layout_split_pane
            | Rule::layout_window
            | Rule::line
            | Rule::line_entire
            | Rule::message
            | Rule::number
            | Rule::output
            | Rule::pane_id
            | Rule::pane_mode_changed
            | Rule::paste_buffer_changed
            | Rule::paste_buffer_deleted
            | Rule::pause
            | Rule::session_changed
            | Rule::session_id
            | Rule::session_renamed
            | Rule::session_window_changed
            | Rule::sessions_changed
            | Rule::subscription_changed
            | Rule::unlinked_window_add
            | Rule::unlinked_window_close
            | Rule::unlinked_window_renamed
            | Rule::window_add
            | Rule::window_close
            | Rule::window_id
            | Rule::window_layout
            | Rule::window_pane_changed
            | Rule::window_renamed
            | Rule::word => bail!("Should not reach here"),
        }
    }

    Ok(stack)
}

pub fn parse_layout(layout: &str) -> Result<Vec<WindowLayout>> {
    let mut result = Vec::new();
    let pairs = parser::TmuxParser::parse(Rule::layout_window, layout)?;

    let _ = parse_layout_inner(pairs, &mut result)?;
    if result.len() > 1 {
        let _ = result.pop();
    }

    Ok(result)
}
