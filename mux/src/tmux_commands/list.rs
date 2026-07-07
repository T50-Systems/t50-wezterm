#[derive(Debug)]
pub(crate) struct ListAllPanes {
    pub window_id: TmuxWindowId,
    pub prune: bool,
    pub layout_csum: String,
}

impl TmuxCommand for ListAllPanes {
    fn get_command(&self, domain_id: DomainId) -> String {
        let mux = Mux::get();
        let domain = match mux.get_domain(domain_id) {
            Some(d) => d,
            None => return "".to_string(),
        };
        let tmux_domain = match domain.downcast_ref::<TmuxDomain>() {
            Some(t) => t,
            None => return "".to_string(),
        };

        let mut gui_tabs = tmux_domain.inner.gui_tabs.lock();

        let Some(local_tab) = gui_tabs.get_mut(&self.window_id) else {
            return "".to_string();
        };

        if local_tab.layout_csum.eq(&self.layout_csum) {
            if self.prune {
                return "".to_string();
            }
        } else {
            local_tab.layout_csum = self.layout_csum.clone();
        }

        format!(
            "list-panes -F '#{{session_id}} #{{window_id}} #{{pane_id}} \
            #{{pane_index}} #{{cursor_x}} #{{cursor_y}} #{{pane_width}} #{{pane_height}} \
            #{{pane_left}} #{{pane_top}} #{{pane_active}}' -t @{}\n",
            self.window_id
        )
    }

    fn process_result(&self, domain_id: DomainId, result: &Guarded) -> anyhow::Result<()> {
        if result.error {
            let error = format!("list-pane in domain={domain_id} failed: {result:#?}");
            log::error!("{error}");
            anyhow::bail!("{error}");
        }
        let mut items = vec![];
        let mut pane_set = HashSet::new();
        for line in result.output.split('\n') {
            if line.is_empty() {
                continue;
            }
            let mut fields = line.split(' ');
            // These ids all have various sigils such as `$`, `%`, `@`,
            // so skip those prior to parsing them
            let session_id =
                parse_sigil_number(fields.next().ok_or_else(|| anyhow!("missing session_id"))?)?;
            let window_id =
                parse_sigil_number(fields.next().ok_or_else(|| anyhow!("missing window_id"))?)?;
            let pane_id =
                parse_sigil_number(fields.next().ok_or_else(|| anyhow!("missing pane_id"))?)?;
            let _pane_index = fields
                .next()
                .ok_or_else(|| anyhow!("missing pane_index"))?
                .parse()?;
            let cursor_x = fields
                .next()
                .ok_or_else(|| anyhow!("missing cursor_x"))?
                .parse()?;
            let cursor_y = fields
                .next()
                .ok_or_else(|| anyhow!("missing cursor_y"))?
                .parse()?;
            let pane_width = fields
                .next()
                .ok_or_else(|| anyhow!("missing pane_width"))?
                .parse()?;
            let pane_height = fields
                .next()
                .ok_or_else(|| anyhow!("missing pane_height"))?
                .parse()?;
            let pane_left = fields
                .next()
                .ok_or_else(|| anyhow!("missing pane_left"))?
                .parse()?;
            let pane_top = fields
                .next()
                .ok_or_else(|| anyhow!("missing pane_top"))?
                .parse()?;
            let pane_active = fields
                .next()
                .ok_or_else(|| anyhow!("missing pane_active"))?
                .parse::<usize>()?;

            let pane_active = pane_active == 1;

            pane_set.insert(pane_id);

            items.push(PaneItem {
                session_id,
                window_id,
                pane_id,
                _pane_index,
                cursor_x,
                cursor_y,
                pane_width,
                pane_height,
                pane_left,
                pane_top,
                pane_active,
            });
        }

        log::debug!("panes in domain_id {}: {:?}", domain_id, items);
        let mux = Mux::get();
        if let Some(domain) = mux.get_domain(domain_id) {
            if let Some(tmux_domain) = domain.downcast_ref::<TmuxDomain>() {
                if !self.prune {
                    return tmux_domain.inner.sync_pane_state(&items);
                } else {
                    return tmux_domain
                        .inner
                        .remove_detached_pane(self.window_id, &pane_set);
                }
            }
        }
        anyhow::bail!("Tmux domain lost");
    }
}

#[derive(Debug)]
pub(crate) struct ListAllWindows {
    pub session_id: TmuxSessionId,
    pub window_id: Option<TmuxWindowId>,
}

impl TmuxCommand for ListAllWindows {
    fn get_command(&self, _domain_id: DomainId) -> String {
        format!(
            "list-windows -F \
                '#{{session_id}} #{{window_id}} \
                #{{window_width}} #{{window_height}} \
                #{{window_active}} \
                #{{window_name}} \
                #{{window_layout}} \
                #{{history_limit}}' -t ${}\n",
            self.session_id
        )
    }

    fn process_result(&self, domain_id: DomainId, result: &Guarded) -> anyhow::Result<()> {
        if result.error {
            let error = format!("list-window in domain={domain_id} failed: {result:#?}");
            log::error!("{error}");
            anyhow::bail!("{error}");
        }
        let mut items = vec![];

        for line in result.output.split('\n') {
            if line.is_empty() {
                continue;
            }
            let mut fields = line.split(' ');
            let session_id =
                parse_sigil_number(fields.next().ok_or_else(|| anyhow!("missing session_id"))?)?;
            let window_id =
                parse_sigil_number(fields.next().ok_or_else(|| anyhow!("missing window_id"))?)?;
            let window_width = fields
                .next()
                .ok_or_else(|| anyhow!("missing window_width"))?
                .parse()?;
            let window_height = fields
                .next()
                .ok_or_else(|| anyhow!("missing window_height"))?
                .parse()?;
            let window_active = fields
                .next()
                .ok_or_else(|| anyhow!("missing window_active"))?
                .parse::<usize>()?;

            let window_name = fields
                .next()
                .ok_or_else(|| anyhow!("missing window_name"))?;

            let window_layout = fields
                .next()
                .ok_or_else(|| anyhow!("missing window_layout"))?;

            let history_limit = fields
                .next()
                .ok_or_else(|| anyhow!("missing history_limit"))?
                .parse::<isize>()?;

            let window_active = window_active == 1;

            if let Some(x) = self.window_id {
                if x != window_id {
                    continue;
                }
            }

            let layout_csum = window_layout
                .get(0..4)
                .ok_or_else(|| anyhow!("missing window_layout"))?;
            let window_layout = window_layout
                .get(5..)
                .ok_or_else(|| anyhow!("missing window_layout"))?;

            let layout = parse_layout(window_layout)?;

            items.push(WindowItem {
                session_id,
                window_id,
                window_width,
                window_height,
                window_active,
                window_name: window_name.to_string(),
                layout,
                layout_csum: layout_csum.to_string(),
                history_limit,
            });
        }

        log::debug!("layout in domain_id {}: {:#?}", domain_id, items);
        let mux = Mux::get();
        if let Some(domain) = mux.get_domain(domain_id) {
            if let Some(tmux_domain) = domain.downcast_ref::<TmuxDomain>() {
                let new_window = if let Some(_x) = self.window_id {
                    true
                } else {
                    false
                };
                return tmux_domain.inner.sync_window_state(&items, new_window);
            }
        }
        anyhow::bail!("Tmux domain lost");
    }
}
