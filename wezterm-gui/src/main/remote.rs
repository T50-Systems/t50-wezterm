async fn async_run_ssh(opts: SshCommand) -> anyhow::Result<()> {
    let mut ssh_option = HashMap::new();
    if opts.verbose {
        ssh_option.insert("wezterm_ssh_verbose".to_string(), "true".to_string());
    }
    for (k, v) in opts.config_override {
        ssh_option.insert(k.to_lowercase().to_string(), v);
    }

    let dom = SshDomain {
        name: format!("SSH to {}", opts.user_at_host_and_port),
        remote_address: opts.user_at_host_and_port.host_and_port.clone(),
        username: opts.user_at_host_and_port.username.clone(),
        multiplexing: SshMultiplexing::None,
        ssh_option,
        ..Default::default()
    };

    let start_command = StartCommand {
        always_new_process: true,
        class: opts.class,
        cwd: None,
        no_auto_connect: true,
        position: opts.position,
        workspace: None,
        prog: opts.prog.clone(),
        ..Default::default()
    };

    let cmd = if !opts.prog.is_empty() {
        let builder = CommandBuilder::from_argv(opts.prog);
        Some(builder)
    } else {
        None
    };

    let domain: Arc<dyn Domain> = Arc::new(mux::ssh::RemoteSshDomain::with_ssh_domain(&dom)?);
    let mux = Mux::get();
    mux.add_domain(&domain);
    mux.set_default_domain(&domain);

    let should_publish = false;
    async_run_terminal_gui(cmd, start_command, should_publish).await
}

fn run_ssh(opts: SshCommand) -> anyhow::Result<()> {
    if let Some(cls) = opts.class.as_ref() {
        crate::set_window_class(cls);
    }
    if let Some(pos) = opts.position.as_ref() {
        set_window_position(pos.clone());
    }

    build_initial_mux(&config::configuration(), None, None)?;

    let gui = crate::frontend::try_new()?;

    promise::spawn::spawn(async {
        if let Err(err) = async_run_ssh(opts).await {
            terminate_with_error(err);
        }
    })
    .detach();

    maybe_show_configuration_error_window();
    gui.run_forever()
}

async fn async_run_serial(opts: SerialCommand) -> anyhow::Result<()> {
    let serial_domain = SerialDomain {
        name: format!("Serial Port {}", opts.port),
        port: Some(opts.port.clone()),
        baud: opts.baud,
    };

    let start_command = StartCommand {
        always_new_process: true,
        class: opts.class,
        cwd: None,
        no_auto_connect: true,
        position: opts.position,
        workspace: None,
        domain: Some(serial_domain.name.clone()),
        ..Default::default()
    };

    let cmd = None;

    let domain: Arc<dyn Domain> = Arc::new(LocalDomain::new_serial_domain(serial_domain)?);
    let mux = Mux::get();
    mux.add_domain(&domain);

    let should_publish = false;
    async_run_terminal_gui(cmd, start_command, should_publish).await
}

fn run_serial(config: config::ConfigHandle, opts: SerialCommand) -> anyhow::Result<()> {
    if let Some(cls) = opts.class.as_ref() {
        crate::set_window_class(cls);
    }
    if let Some(pos) = opts.position.as_ref() {
        set_window_position(pos.clone());
    }

    build_initial_mux(&config, None, None)?;

    let gui = crate::frontend::try_new()?;

    promise::spawn::spawn(async {
        if let Err(err) = async_run_serial(opts).await {
            terminate_with_error(err);
        }
    })
    .detach();

    maybe_show_configuration_error_window();
    gui.run_forever()
}

fn have_panes_in_domain_and_ws(domain: &Arc<dyn Domain>, workspace: &Option<String>) -> bool {
    let mux = Mux::get();
    let have_panes_in_domain = mux
        .iter_panes()
        .iter()
        .any(|p| p.domain_id() == domain.domain_id());

    if !have_panes_in_domain {
        return false;
    }

    if let Some(ws) = &workspace {
        for window_id in mux.iter_windows_in_workspace(ws) {
            if let Some(win) = mux.get_window(window_id) {
                for t in win.iter() {
                    for p in t.iter_panes_ignoring_zoom() {
                        if p.pane.domain_id() == domain.domain_id() {
                            return true;
                        }
                    }
                }
            }
        }
        false
    } else {
        true
    }
}
