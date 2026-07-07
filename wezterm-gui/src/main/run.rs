fn run() -> anyhow::Result<()> {
    // Inform the system of our AppUserModelID.
    // Without this, our toast notifications won't be correctly
    // attributed to our application.
    #[cfg(windows)]
    {
        unsafe {
            ::windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(
                ::windows::core::PCWSTR(wide_string("org.wezfurlong.wezterm").as_ptr()),
            )
            .unwrap();
        }
    }

    let opts = Opt::parse();

    // This is a bit gross.
    // In order to not to automatically open a standard windows console when
    // we run, we use the windows_subsystem attribute at the top of this
    // source file.  That comes at the cost of causing the help output
    // to disappear if we are actually invoked from a console.
    // This AttachConsole call will attach us to the console of the parent
    // in that situation, but since we were launched as a windows subsystem
    // application we will be running asynchronously from the shell in
    // the command window, which means that it will appear to the user
    // that we hung at the end, when in reality the shell is waiting for
    // input but didn't know to re-draw the prompt.
    #[cfg(windows)]
    unsafe {
        if opts.attach_parent_console {
            winapi::um::wincon::AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS);
        }
    };

    env_bootstrap::bootstrap();
    // window_funcs is not set up by env_bootstrap as window_funcs is
    // GUI environment specific and env_bootstrap is used to setup the
    // headless mux server.
    config::lua::add_context_setup_func(window_funcs::register);
    config::lua::add_context_setup_func(crate::scripting::register);
    config::lua::add_context_setup_func(crate::stats::register);

    stats::Stats::init()?;
    let _saver = umask::UmaskSaver::new();

    config::common_init(
        opts.config_file.as_ref(),
        &opts.config_override,
        opts.skip_config,
    )?;
    let config = config::configuration();
    if let Some(value) = &config.default_ssh_auth_sock {
        std::env::set_var("SSH_AUTH_SOCK", value);
    }

    let sub = match opts.cmd.as_ref().cloned() {
        Some(SubCommand::BlockingStart(start)) => {
            // Act as if the normal start subcommand was used,
            // except that we always start a new instance.
            // This is needed for compatibility, because many tools assume
            // that "$TERMINAL -e $COMMAND" blocks until the command finished.
            SubCommand::Start(StartCommand {
                always_new_process: true,
                ..start
            })
        }
        Some(sub) => sub,
        None => {
            // Need to fake an argv0
            let mut argv = vec!["wezterm-gui".to_string()];
            for a in &config.default_gui_startup_args {
                argv.push(a.clone());
            }
            SubCommand::try_parse_from(&argv).with_context(|| {
                format!(
                    "parsing the default_gui_startup_args config: {:?}",
                    config.default_gui_startup_args
                )
            })?
        }
    };

    match sub {
        SubCommand::Start(start) => {
            log::trace!("Using configuration: {:#?}\nopts: {:#?}", config, opts);
            let res = run_terminal_gui(start, None);
            wezterm_blob_leases::clear_storage();
            res
        }
        SubCommand::BlockingStart(_) => unreachable!(),
        SubCommand::Ssh(ssh) => run_ssh(ssh),
        SubCommand::Serial(serial) => run_serial(config, serial),
        SubCommand::Connect(connect) => run_terminal_gui(
            StartCommand {
                domain: Some(connect.domain_name.clone()),
                class: connect.class,
                workspace: connect.workspace,
                position: connect.position,
                prog: connect.prog,
                new_tab: connect.new_tab,
                always_new_process: true,
                attach: true,
                _cmd: false,
                no_auto_connect: false,
                cwd: None,
            },
            Some(connect.domain_name),
        ),
        SubCommand::LsFonts(cmd) => run_ls_fonts(config, &cmd),
        SubCommand::ShowKeys(cmd) => run_show_keys(config, &cmd),
    }
}
