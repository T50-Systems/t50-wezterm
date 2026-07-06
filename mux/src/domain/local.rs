impl LocalDomain {
    pub fn new(name: &str) -> Result<Self, Error> {
        Ok(Self::with_pty_system(name, native_pty_system()))
    }

    fn resolve_exec_domain(&self) -> Option<ExecDomain> {
        config::configuration()
            .exec_domains
            .iter()
            .find(|ed| ed.name == self.name)
            .cloned()
    }

    fn resolve_wsl_domain(&self) -> Option<WslDomain> {
        config::configuration()
            .wsl_domains()
            .iter()
            .find(|d| d.name == self.name)
            .cloned()
    }

    pub fn with_pty_system(name: &str, pty_system: Box<dyn PtySystem + Send>) -> Self {
        let id = alloc_domain_id();
        Self {
            pty_system: Mutex::new(pty_system),
            id,
            name: name.to_string(),
        }
    }

    pub fn new_wsl(wsl: WslDomain) -> Result<Self, Error> {
        Self::new(&wsl.name)
    }

    pub fn new_exec_domain(exec_domain: ExecDomain) -> anyhow::Result<Self> {
        Self::new(&exec_domain.name)
    }

    pub fn new_serial_domain(serial_domain: SerialDomain) -> anyhow::Result<Self> {
        let port = serial_domain.port.as_ref().unwrap_or(&serial_domain.name);
        let mut serial = portable_pty::serial::SerialTty::new(&port);
        if let Some(baud) = serial_domain.baud {
            serial.set_baud_rate(baud as u32);
        }
        let pty_system = Box::new(serial);
        Ok(Self::with_pty_system(&serial_domain.name, pty_system))
    }

    #[cfg(unix)]
    fn is_conpty(&self) -> bool {
        false
    }

    #[cfg(windows)]
    fn is_conpty(&self) -> bool {
        let pty_system = self.pty_system.lock();
        let pty_system: &dyn PtySystem = &**pty_system;
        pty_system
            .downcast_ref::<portable_pty::win::conpty::ConPtySystem>()
            .is_some()
    }

    async fn fixup_command(&self, cmd: &mut CommandBuilder) -> anyhow::Result<()> {
        if let Some(wsl) = self.resolve_wsl_domain() {
            let mut args: Vec<OsString> = cmd.get_argv().clone();

            if args.is_empty() {
                if let Some(def_prog) = &wsl.default_prog {
                    for arg in def_prog {
                        args.push(arg.into());
                    }
                }
            }

            let mut argv: Vec<OsString> = vec![
                "wsl.exe".into(),
                "--distribution".into(),
                wsl.distribution
                    .as_deref()
                    .unwrap_or(wsl.name.as_str())
                    .into(),
            ];

            if let Some(cwd) = cmd.get_cwd() {
                argv.push("--cd".into());
                argv.push(cwd.into());
            }

            if let Some(user) = &wsl.username {
                argv.push("--user".into());
                argv.push(user.into());
            }

            if !args.is_empty() {
                argv.push("--exec".into());
                for arg in args {
                    argv.push(arg);
                }
            }

            // TODO: process env list and update WLSENV so that they
            // get passed through

            cmd.clear_cwd();
            *cmd.get_argv_mut() = argv;
        } else if let Some(ed) = self.resolve_exec_domain() {
            let mut args = vec![];
            let mut set_environment_variables = HashMap::new();
            for arg in cmd.get_argv() {
                args.push(
                    arg.to_str()
                        .ok_or_else(|| anyhow::anyhow!("command argument is not utf8"))?
                        .to_string(),
                );
            }
            for (k, v) in cmd.iter_full_env_as_str() {
                set_environment_variables.insert(k.to_string(), v.to_string());
            }
            let cwd = match cmd.get_cwd() {
                Some(cwd) => Some(PathBuf::from(cwd)),
                None => None,
            };
            let spawn_command = SpawnCommand {
                label: None,
                domain: SpawnTabDomain::DomainName(ed.name.clone()),
                args: if args.is_empty() { None } else { Some(args) },
                set_environment_variables,
                cwd,
                position: None,
            };

            let spawn_command = config::with_lua_config_on_main_thread(|lua| async {
                let lua = lua.ok_or_else(|| anyhow::anyhow!("missing lua context"))?;
                let value = config::lua::emit_async_callback(
                    &*lua,
                    (ed.fixup_command.clone(), (spawn_command.clone())),
                )
                .await?;
                let cmd: SpawnCommand =
                    luahelper::from_lua_value_dynamic(value).with_context(|| {
                        format!(
                            "interpreting SpawnCommand result from ExecDomain {}",
                            ed.name
                        )
                    })?;
                Ok(cmd)
            })
            .await
            .with_context(|| format!("calling ExecDomain {} function", ed.name))?;

            // Reinterpret the SpawnCommand into the builder

            cmd.get_argv_mut().clear();
            if let Some(args) = &spawn_command.args {
                for arg in args {
                    cmd.get_argv_mut().push(arg.into());
                }
            }
            cmd.env_clear();
            for (k, v) in &spawn_command.set_environment_variables {
                cmd.env(k, v);
            }
            cmd.clear_cwd();
            if let Some(cwd) = &spawn_command.cwd {
                cmd.cwd(cwd);
            }
        } else if Path::new("/.flatpak-info").exists() {
            // We're running inside a flatpak sandbox.
            // Run the command outside the sandbox via flatpak-spawn
            let mut args = vec![
                "flatpak-spawn".to_string(),
                "--host".to_string(),
                "--watch-bus".to_string(),
            ];
            if let Some(cwd) = cmd.get_cwd() {
                args.push(format!("--directory={}", Path::new(cwd).display()));
            }

            let is_default_prog = cmd.is_default_prog();

            // Note: WEZTERM_UNIX_SOCKET, WEZTERM_CONFIG_(FILE|DIR) and other env
            // vars are not included in this.
            // We can't include them: their paths are only meaningful in the sandbox
            // and cannot be reasonably accessed from outside it in the shell.
            for (k, v) in cmd.iter_extra_env_as_str() {
                args.push(format!("--env={k}={v}"));
            }

            for arg in cmd.get_argv() {
                args.push(
                    arg.to_str()
                        .ok_or_else(|| anyhow::anyhow!("command argument is not utf8"))?
                        .to_string(),
                );
            }

            if is_default_prog {
                // We can't read $SHELL from inside the sandbox, so ask the host.
                let output = std::process::Command::new("flatpak-spawn")
                    .args(["--host", "sh", "-c", "echo $SHELL"])
                    .output()?;
                let shell = String::from_utf8_lossy(&output.stdout);

                args.push(shell.trim().to_string());
                // Assume we can pass `-l` for a login shell
                args.push("-l".to_string());
            }

            // Avoid setting up the controlling tty as that is not compatible
            // with flatpak:
            // <https://github.com/flatpak/flatpak/issues/3697>
            // <https://github.com/flatpak/flatpak/issues/3285>
            cmd.set_controlling_tty(false);

            // Re-apply to the builder
            cmd.get_argv_mut().clear();
            for arg in args {
                cmd.get_argv_mut().push(arg.into());
            }
            cmd.clear_cwd();
            log::trace!("made: {cmd:#?}");
        } else if let Some(dir) = cmd.get_cwd() {
            // I'm not normally a fan of existence checking, but not checking here
            // can be painful; in the case where a tab is local but has connected
            // to a remote system and that remote has used OSC 7 to set a path
            // that doesn't exist on the local system, process spawning can fail.
            // Another situation is `sudo -i` has the pane with set to a cwd
            // that is not accessible to the user.
            if let Err(err) = Path::new(&dir).read_dir() {
                log::warn!(
                    "Directory {:?} is not readable and will not be \
                     used for the command we are spawning: {:#}",
                    dir,
                    err
                );
                cmd.clear_cwd();
            }
        }
        Ok(())
    }

    async fn build_command(
        &self,
        command: Option<CommandBuilder>,
        command_dir: Option<String>,
        pane_id: PaneId,
    ) -> anyhow::Result<CommandBuilder> {
        let config = configuration();

        let wsl = self.resolve_wsl_domain();
        let default_prog = wsl
            .as_ref()
            .map(|wsl| wsl.default_prog.as_ref())
            .unwrap_or(config.default_prog.as_ref());

        let mut cmd = match command {
            Some(mut cmd) => {
                config.apply_cmd_defaults(&mut cmd, default_prog, config.default_cwd.as_ref());
                cmd
            }
            None => config.build_prog(
                None,
                default_prog,
                wsl.as_ref()
                    .map(|wsl| wsl.default_cwd.as_ref())
                    .unwrap_or(config.default_cwd.as_ref()),
            )?,
        };
        if let Some(dir) = command_dir {
            cmd.cwd(dir);
        }
        if let Ok(sock) = std::env::var("WEZTERM_UNIX_SOCKET") {
            cmd.env("WEZTERM_UNIX_SOCKET", sock);
        }
        cmd.env("WEZTERM_PANE", pane_id.to_string());
        if let Some(agent) = Mux::get().agent.as_ref() {
            cmd.env("SSH_AUTH_SOCK", agent.path());
        }
        self.fixup_command(&mut cmd).await?;
        Ok(cmd)
    }
}
