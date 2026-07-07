#[async_trait(?Send)]
impl Domain for RemoteSshDomain {
    async fn spawn_pane(
        &self,
        size: TerminalSize,
        command: Option<CommandBuilder>,
        command_dir: Option<String>,
    ) -> anyhow::Result<Arc<dyn Pane>> {
        let pane_id = alloc_pane_id();

        let (command_line, env) = self
            .build_command(pane_id, command, command_dir)
            .context("build_command")?;

        // This needs to be separate from the if let block below in order
        // for the lock to be released at the appropriate time
        let mut session: Option<Session> = self.session.lock().unwrap().as_ref().cloned();

        let StartNewSessionResult { pty, child, writer } = if let Some(session) = session.take() {
            match session
                .request_pty(
                    &config::configuration().term,
                    crate::terminal_size_to_pty_size(size)
                        .context("compute pty size from terminal size")?,
                    command_line.as_ref().map(|s| s.as_str()),
                    Some(env.clone()),
                )
                .await
                .context("request ssh pty")
            {
                Ok((concrete_pty, concrete_child)) => {
                    let pty = Box::new(concrete_pty);
                    let child = Box::new(concrete_child);
                    let writer = Box::new(pty.take_writer().context("take writer from pty")?);

                    StartNewSessionResult { pty, child, writer }
                }
                Err(err) => {
                    if err
                        .root_cause()
                        .downcast_ref::<wezterm_ssh::DeadSession>()
                        .is_some()
                    {
                        // Session died (perhaps they closed the initial tab?)
                        // So we'll try making a new one
                        self.start_new_session(command_line, env, size).await?
                    } else {
                        log::error!("{err:#?}");
                        return Err(err);
                    }
                }
            }
        } else {
            self.start_new_session(command_line, env, size).await?
        };

        // Wrap up the pty etc. in a LocalPane.  That allows for
        // eg: tmux integration to be tunnelled via the remote
        // session without duplicating a lot of logic over here.

        let writer = WriterWrapper::new(writer);

        let terminal = wezterm_term::Terminal::new(
            size,
            std::sync::Arc::new(config::TermConfig::new()),
            "WezTerm",
            config::wezterm_version(),
            Box::new(writer.clone()),
        );

        let pane: Arc<dyn Pane> = Arc::new(LocalPane::new(
            pane_id,
            terminal,
            child,
            pty,
            Box::new(writer),
            self.id,
            "RemoteSshDomain".to_string(),
        ));
        let mux = Mux::get();
        mux.add_pane(&pane)?;

        Ok(pane)
    }

    fn domain_id(&self) -> DomainId {
        self.id
    }

    fn domain_name(&self) -> &str {
        &self.name
    }

    async fn attach(&self, _window_id: Option<crate::WindowId>) -> anyhow::Result<()> {
        Ok(())
    }

    fn detachable(&self) -> bool {
        false
    }

    fn detach(&self) -> anyhow::Result<()> {
        bail!("detach not implemented for RemoteSshDomain");
    }

    fn state(&self) -> DomainState {
        // Just pretend that we are always attached, as we don't
        // have a defined attach operation that is distinct from
        // a spawn.
        DomainState::Attached
    }
}
