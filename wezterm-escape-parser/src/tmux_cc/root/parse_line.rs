fn parse_line(line: &[u8]) -> Result<Event> {
    let binding = String::from_utf8_lossy(line);
    let parsed_line = binding.as_ref();
    let mut pairs = parser::TmuxParser::parse(Rule::line_entire, parsed_line)?;
    let pair = pairs.next().ok_or_else(|| format_err!("no pairs!?"))?;
    match pair.as_rule() {
        // Tmux generic rules
        Rule::begin => {
            let (timestamp, number, flags) = parse_guard(pair.into_inner())?;
            Ok(Event::Begin {
                timestamp,
                number,
                flags,
            })
        }
        Rule::end => {
            let (timestamp, number, flags) = parse_guard(pair.into_inner())?;
            Ok(Event::End {
                timestamp,
                number,
                flags,
            })
        }
        Rule::error => {
            let (timestamp, number, flags) = parse_guard(pair.into_inner())?;
            Ok(Event::Error {
                timestamp,
                number,
                flags,
            })
        }

        // Tmux specific rules
        Rule::client_detached => {
            let mut pairs = pair.into_inner();
            let client_name = unvis(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing name"))?
                    .as_str(),
            )?;
            Ok(Event::ClientDetached { client_name })
        }
        Rule::client_session_changed => {
            let mut pairs = pair.into_inner();
            let client_name = unvis(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing name"))?
                    .as_str(),
            )?;
            let session = parse_session_id(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing session id"))?,
            )?;
            let session_name = unvis(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing session name"))?
                    .as_str(),
            )?;
            Ok(Event::ClientSessionChanged {
                client_name,
                session,
                session_name,
            })
        }
        Rule::config_error => {
            let mut pairs = pair.into_inner();
            let error = unvis(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing name"))?
                    .as_str(),
            )?;
            Ok(Event::ConfigError { error })
        }
        Rule::r#continue => {
            let mut pairs = pair.into_inner();
            let pane = parse_pane_id(pairs.next().ok_or_else(|| format_err!("missing pane id"))?)?;
            Ok(Event::Continue { pane })
        }
        Rule::extended_output => {
            let mut pairs = pair.into_inner();
            let pane = parse_pane_id(pairs.next().ok_or_else(|| format_err!("missing pane id"))?)?;
            let pair = pairs.next().ok_or_else(|| format_err!("missing text"))?;

            let (_, pos) = pair.line_col();
            let text = unvis_bytes(&line[pos - 1..])?;
            Ok(Event::ExtendedOutput { pane, text })
        }
        Rule::exit => {
            let mut pairs = pair.into_inner();
            let reason = pairs.next().map(|pair| pair.as_str().to_owned());
            Ok(Event::Exit { reason })
        }
        Rule::layout_change => {
            let mut pairs = pair.into_inner();
            let window = parse_window_id(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing window id"))?,
            )?;
            let layout = unvis(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing layout"))?
                    .as_str(),
            )?;
            let visible_layout = pairs.next().map(|pair| pair.as_str().to_owned());
            let raw_flags = pairs.next().map(|r| r.as_str().to_owned());
            Ok(Event::LayoutChange {
                window,
                layout,
                visible_layout,
                raw_flags,
            })
        }
        Rule::message => {
            let mut pairs = pair.into_inner();
            let message = unvis(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing text"))?
                    .as_str(),
            )?;
            Ok(Event::Message { message })
        }
        Rule::output => {
            let mut pairs = pair.into_inner();
            let pane = parse_pane_id(pairs.next().ok_or_else(|| format_err!("missing pane id"))?)?;
            let pair = pairs.next().ok_or_else(|| format_err!("missing text"))?;

            let (_, pos) = pair.line_col();
            let text = unvis_bytes(&line[pos - 1..])?;
            Ok(Event::Output { pane, text })
        }
        Rule::pane_mode_changed => {
            let mut pairs = pair.into_inner();
            let pane = parse_pane_id(pairs.next().ok_or_else(|| format_err!("missing pane id"))?)?;
            Ok(Event::PaneModeChanged { pane })
        }
        Rule::paste_buffer_changed => {
            let mut pairs = pair.into_inner();
            let buffer = unvis(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing text"))?
                    .as_str(),
            )?;
            Ok(Event::PasteBufferChanged { buffer })
        }
        Rule::paste_buffer_deleted => {
            let mut pairs = pair.into_inner();
            let buffer = unvis(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing text"))?
                    .as_str(),
            )?;
            Ok(Event::PasteBufferDeleted { buffer })
        }
        Rule::pause => {
            let mut pairs = pair.into_inner();
            let pane = parse_pane_id(pairs.next().ok_or_else(|| format_err!("missing pane id"))?)?;
            Ok(Event::Pause { pane })
        }
        Rule::session_changed => {
            let mut pairs = pair.into_inner();
            let session = parse_session_id(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing session id"))?,
            )?;
            let name = unvis(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing name"))?
                    .as_str(),
            )?;
            Ok(Event::SessionChanged { session, name })
        }
        Rule::session_renamed => {
            let mut pairs = pair.into_inner();
            let name = unvis(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing name"))?
                    .as_str(),
            )?;
            Ok(Event::SessionRenamed { name })
        }
        Rule::session_window_changed => {
            let mut pairs = pair.into_inner();
            let session = parse_session_id(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing session id"))?,
            )?;
            let window = parse_window_id(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing window id"))?,
            )?;
            Ok(Event::SessionWindowChanged { session, window })
        }
        Rule::sessions_changed => Ok(Event::SessionsChanged),
        Rule::subscription_changed => Ok(Event::SubscriptionChanged),
        Rule::unlinked_window_add => {
            let mut pairs = pair.into_inner();
            let window = parse_window_id(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing window id"))?,
            )?;
            Ok(Event::UnlinkedWindowAdd { window })
        }
        Rule::unlinked_window_close => {
            let mut pairs = pair.into_inner();
            let window = parse_window_id(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing window id"))?,
            )?;
            Ok(Event::UnlinkedWindowClose { window })
        }
        Rule::unlinked_window_renamed => {
            let mut pairs = pair.into_inner();
            let window = parse_window_id(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing window id"))?,
            )?;
            Ok(Event::UnlinkedWindowRenamed { window })
        }
        Rule::window_add => {
            let mut pairs = pair.into_inner();
            let window = parse_window_id(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing window id"))?,
            )?;
            Ok(Event::WindowAdd { window })
        }
        Rule::window_close => {
            let mut pairs = pair.into_inner();
            let window = parse_window_id(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing window id"))?,
            )?;
            Ok(Event::WindowClose { window })
        }
        Rule::window_pane_changed => {
            let mut pairs = pair.into_inner();
            let window = parse_window_id(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing window id"))?,
            )?;
            let pane = parse_pane_id(pairs.next().ok_or_else(|| format_err!("missing pane id"))?)?;
            Ok(Event::WindowPaneChanged { window, pane })
        }
        Rule::window_renamed => {
            let mut pairs = pair.into_inner();
            let window = parse_window_id(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing window id"))?,
            )?;
            let name = unvis(
                pairs
                    .next()
                    .ok_or_else(|| format_err!("missing name"))?
                    .as_str(),
            )?;
            Ok(Event::WindowRenamed { window, name })
        }
        Rule::EOI
        | Rule::any_text
        | Rule::client_name
        | Rule::layout_pane
        | Rule::layout_split_horizontal
        | Rule::layout_split_pane
        | Rule::layout_split_vertical
        | Rule::layout_window
        | Rule::line
        | Rule::line_entire
        | Rule::number
        | Rule::pane_id
        | Rule::session_id
        | Rule::window_id
        | Rule::window_layout
        | Rule::word => bail!("Should not reach here"),
    }
}
