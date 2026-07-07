async fn spawn_tab_in_domain_if_mux_is_empty(
    cmd: Option<CommandBuilder>,
    is_connecting: bool,
    domain: Option<Arc<dyn Domain>>,
    workspace: Option<String>,
) -> anyhow::Result<()> {
    let mux = Mux::get();

    let domain = domain.unwrap_or_else(|| mux.default_domain());

    if !is_connecting {
        if have_panes_in_domain_and_ws(&domain, &workspace) {
            return Ok(());
        }
    }

    let window_id = {
        // Force the builder to notify the frontend early,
        // so that the attach await below doesn't block it.
        // This has the consequence of creating the window
        // at the initial size instead of populating it
        // from the size specified in the remote mux.
        // We use the TabAddedToWindow mux notification
        // to detect and adjust the size later on.
        let position = None;
        let builder = mux.new_empty_window(workspace.clone(), position);
        *builder
    };

    let config = config::configuration();
    config.update_ulimit()?;

    domain.attach(Some(window_id)).await?;

    if have_panes_in_domain_and_ws(&domain, &workspace) {
        trigger_and_log_gui_attached(MuxDomain(domain.domain_id())).await;
        return Ok(());
    }

    let _config_subscription = config::subscribe_to_config_reload(move || {
        promise::spawn::spawn_into_main_thread(async move {
            if let Err(err) = update_mux_domains(&config::configuration()) {
                log::error!("Error updating mux domains: {:#}", err);
            }
        })
        .detach();
        true
    });

    let dpi = config.dpi.unwrap_or_else(|| ::window::default_dpi());
    let _tab = domain
        .spawn(
            config.initial_size(dpi as u32, Some(cell_pixel_dims(&config, dpi)?)),
            cmd,
            None,
            window_id,
        )
        .await?;
    trigger_and_log_gui_attached(MuxDomain(domain.domain_id())).await;
    Ok(())
}

async fn connect_to_auto_connect_domains() -> anyhow::Result<()> {
    let mux = Mux::get();
    let domains = mux.iter_domains();
    for dom in domains {
        if let Some(dom) = dom.downcast_ref::<ClientDomain>() {
            if dom.connect_automatically() {
                dom.attach(None).await?;
            }
        }
    }
    Ok(())
}

async fn trigger_gui_startup(
    lua: Option<Rc<mlua::Lua>>,
    spawn: Option<SpawnCommand>,
) -> anyhow::Result<()> {
    if let Some(lua) = lua {
        let args = lua.pack_multi(spawn)?;
        config::lua::emit_event(&lua, ("gui-startup".to_string(), args)).await?;
    }
    Ok(())
}

async fn trigger_and_log_gui_startup(spawn_command: Option<SpawnCommand>) {
    if let Err(err) =
        config::with_lua_config_on_main_thread(move |lua| trigger_gui_startup(lua, spawn_command))
            .await
    {
        let message = format!("while processing gui-startup event: {:#}", err);
        log::error!("{}", message);
        persistent_toast_notification("Error", &message);
    }
}

async fn trigger_gui_attached(lua: Option<Rc<mlua::Lua>>, domain: MuxDomain) -> anyhow::Result<()> {
    if let Some(lua) = lua {
        let args = lua.pack_multi(domain)?;
        config::lua::emit_event(&lua, ("gui-attached".to_string(), args)).await?;
    }
    Ok(())
}

async fn trigger_and_log_gui_attached(domain: MuxDomain) {
    if let Err(err) =
        config::with_lua_config_on_main_thread(move |lua| trigger_gui_attached(lua, domain)).await
    {
        let message = format!("while processing gui-attached event: {:#}", err);
        log::error!("{}", message);
        persistent_toast_notification("Error", &message);
    }
}

fn cell_pixel_dims(config: &ConfigHandle, dpi: f64) -> anyhow::Result<(usize, usize)> {
    let fontconfig = Rc::new(FontConfiguration::new(Some(config.clone()), dpi as usize)?);
    let render_metrics = RenderMetrics::new(&fontconfig)?;
    Ok((
        render_metrics.cell_size.width as usize,
        render_metrics.cell_size.height as usize,
    ))
}

async fn async_run_terminal_gui(
    cmd: Option<CommandBuilder>,
    opts: StartCommand,
    should_publish: bool,
) -> anyhow::Result<()> {
    let unix_socket_path =
        config::RUNTIME_DIR.join(format!("gui-sock-{}", unsafe { libc::getpid() }));
    std::env::set_var("WEZTERM_UNIX_SOCKET", unix_socket_path.clone());
    wezterm_blob_leases::register_storage(Arc::new(
        wezterm_blob_leases::simple_tempdir::SimpleTempDir::new_in(&*config::CACHE_DIR)?,
    ))?;
    if let Err(err) = spawn_mux_server(unix_socket_path, should_publish) {
        log::warn!("{:#}", err);
    }

    if !opts.no_auto_connect {
        connect_to_auto_connect_domains().await?;
    }

    let spawn_command = match &cmd {
        Some(cmd) => Some(SpawnCommand::from_command_builder(cmd)?),
        None => None,
    };

    // Apply the domain to the command
    let spawn_command = match (spawn_command, &opts.domain) {
        (Some(spawn), Some(name)) => Some(SpawnCommand {
            domain: SpawnTabDomain::DomainName(name.to_string()),
            ..spawn
        }),
        (None, Some(name)) => Some(SpawnCommand {
            domain: SpawnTabDomain::DomainName(name.to_string()),
            ..SpawnCommand::default()
        }),
        (spawn, None) => spawn,
    };
    let mux = Mux::get();

    let domain = if let Some(name) = &opts.domain {
        let domain = mux
            .get_domain_by_name(name)
            .ok_or_else(|| anyhow!("invalid domain {name}"))?;
        Some(domain)
    } else {
        None
    };

    if !opts.attach {
        trigger_and_log_gui_startup(spawn_command).await;
    }

    let is_connecting = opts.attach;

    if let Some(domain) = &domain {
        if !opts.attach {
            let window_id = {
                // Force the builder to notify the frontend early,
                // so that the attach await below doesn't block it.
                let workspace = None;
                let position = None;
                let builder = mux.new_empty_window(workspace, position);
                *builder
            };

            domain.attach(Some(window_id)).await?;
            let config = config::configuration();
            let dpi = config.dpi.unwrap_or_else(|| ::window::default_dpi());
            let tab = domain
                .spawn(
                    config.initial_size(dpi as u32, Some(cell_pixel_dims(&config, dpi)?)),
                    cmd.clone(),
                    None,
                    window_id,
                )
                .await?;
            let mut window = mux
                .get_window_mut(window_id)
                .ok_or_else(|| anyhow!("failed to get mux window id {window_id}"))?;
            if let Some(tab_idx) = window.idx_by_id(tab.tab_id()) {
                window.set_active_without_saving(tab_idx);
            }
            trigger_and_log_gui_attached(MuxDomain(domain.domain_id())).await;
        }
    }
    spawn_tab_in_domain_if_mux_is_empty(cmd, is_connecting, domain, opts.workspace).await
}
