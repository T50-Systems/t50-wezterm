#[derive(Debug)]
pub(crate) struct NewWindow;
impl TmuxCommand for NewWindow {
    fn get_command(&self, _domain_id: DomainId) -> String {
        "new-window\n".to_owned()
    }

    fn process_result(&self, domain_id: DomainId, result: &Guarded) -> anyhow::Result<()> {
        if result.error {
            let error = format!("new-window in domain={domain_id} failed: {result:#?}");
            log::error!("{error}");
            anyhow::bail!("{error}");
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct ListCommands;
impl TmuxCommand for ListCommands {
    fn get_command(&self, _domain_id: DomainId) -> String {
        "list-commands\n".to_owned()
    }

    fn process_result(&self, domain_id: DomainId, result: &Guarded) -> anyhow::Result<()> {
        if result.error {
            let error = format!("list-command in domain={domain_id} failed: {result:#?}");
            log::error!("{error}");
            anyhow::bail!("{error}");
        }
        let mux = Mux::get();
        let domain = match mux.get_domain(domain_id) {
            Some(d) => d,
            None => anyhow::bail!("Tmux domain lost"),
        };
        let tmux_domain = match domain.downcast_ref::<TmuxDomain>() {
            Some(t) => t,
            None => anyhow::bail!("Tmux domain lost"),
        };

        let mut support_commands = tmux_domain.inner.support_commands.lock();

        for line in result.output.split('\n') {
            if line.is_empty() {
                continue;
            }
            let v: Vec<&str> = line.split(' ').collect();
            support_commands.insert(v[0].to_string(), line.to_string());
        }

        let mut cmd_queue = tmux_domain.inner.cmd_queue.as_ref().lock();
        if let Some(session) = *tmux_domain.inner.tmux_session.lock() {
            cmd_queue.push_back(Box::new(ListAllWindows {
                session_id: session,
                window_id: None,
            }));
            TmuxDomainState::schedule_send_next_command(domain_id);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct SplitPane {
    pub pane_id: TmuxPaneId,
    pub direction: SplitDirection,
}

impl TmuxCommand for SplitPane {
    fn get_command(&self, _domain_id: DomainId) -> String {
        if self.direction == SplitDirection::Horizontal {
            format!("split-window -h -t %{}\n", self.pane_id)
        } else {
            format!("split-window -v -t %{}\n", self.pane_id)
        }
    }

    fn process_result(&self, domain_id: DomainId, result: &Guarded) -> anyhow::Result<()> {
        if result.error {
            let error = format!("split-window in domain={domain_id} failed: {result:#?}");
            log::error!("{error}");
            anyhow::bail!("{error}");
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct SelectWindow {
    pub window_id: TmuxWindowId,
}

impl TmuxCommand for SelectWindow {
    fn get_command(&self, _domain_id: DomainId) -> String {
        format!("select-window -t @{}\n", self.window_id)
    }

    fn process_result(&self, domain_id: DomainId, result: &Guarded) -> anyhow::Result<()> {
        if result.error {
            let error = format!("select-window in domain={domain_id} failed: {result:#?}");
            log::error!("{error}");
            anyhow::bail!("{error}");
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct SelectPane {
    pub pane_id: TmuxPaneId,
}

impl TmuxCommand for SelectPane {
    fn get_command(&self, _domain_id: DomainId) -> String {
        format!("select-pane -t %{}\n", self.pane_id)
    }

    fn process_result(&self, domain_id: DomainId, result: &Guarded) -> anyhow::Result<()> {
        if result.error {
            let error = format!("select-pane in domain={domain_id} failed: {result:#?}");
            log::error!("{error}");
            anyhow::bail!("{error}");
        }
        Ok(())
    }
}

// This is a dummy command which indicates the attaching is done, it prevents the tmux output
// the unexpected and unnecessary content when syncing with back end in attaching stage.
#[derive(Debug)]
pub(crate) struct AttachDone;
impl TmuxCommand for AttachDone {
    fn get_command(&self, _domain_id: DomainId) -> String {
        // The command doesn't matter, just give a legal simple command to let process_result called.
        "list-session\n".to_string()
    }

    fn process_result(&self, domain_id: DomainId, result: &Guarded) -> anyhow::Result<()> {
        if result.error {
            let error = format!("list-session in domain={domain_id} failed: {result:#?}");
            log::error!("{error}");
            anyhow::bail!("{error}");
        }
        let mux = Mux::get();
        let domain = match mux.get_domain(domain_id) {
            Some(d) => d,
            None => anyhow::bail!("Tmux domain lost"),
        };
        let tmux_domain = match domain.downcast_ref::<TmuxDomain>() {
            Some(t) => t,
            None => anyhow::bail!("Tmux domain lost"),
        };

        // Do nothing, just change the state.
        *tmux_domain.inner.attach_state.lock() = AttachState::Done;
        Ok(())
    }
}
