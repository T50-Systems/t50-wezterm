
impl ConnectionOps for XConnection {
    fn name(&self) -> String {
        match get_wm_name(
            &self.conn,
            self.root,
            self.atom_net_supporting_wm_check,
            self.atom_net_wm_name,
            self.atom_utf8_string,
        ) {
            Ok(name) => format!("X11 {name}"),
            Err(err) => {
                log::error!("error fetching window manager name: {err:#}");

                "X11".to_string()
            }
        }
    }

    fn terminate_message_loop(&self) {
        *self.should_terminate.borrow_mut() = true;
    }

    fn default_dpi(&self) -> f64 {
        *self.default_dpi.borrow()
    }

    fn get_appearance(&self) -> Appearance {
        match promise::spawn::block_on(crate::os::xdg_desktop_portal::get_appearance()) {
            Ok(Some(appearance)) => return appearance,
            Ok(None) => {}
            Err(err) => {
                log::warn!("Unable to resolve appearance using xdg-desktop-portal: {err:#}");
            }
        }
        if let Some(XSetting::String(name)) = self.xsettings.borrow().get("Net/ThemeName") {
            let lower = name.to_ascii_lowercase();
            match lower.as_str() {
                "highcontrast" => Appearance::LightHighContrast,
                "highcontrastinverse" => Appearance::DarkHighContrast,
                "adwaita" => Appearance::Light,
                "adwaita-dark" => Appearance::Dark,
                lower => {
                    if lower.contains("dark") {
                        Appearance::Dark
                    } else {
                        Appearance::Light
                    }
                }
            }
        } else {
            Appearance::Dark
        }
    }

    fn screens(&self) -> anyhow::Result<Screens> {
        if !self.has_randr {
            anyhow::bail!("XRANDR is not available, cannot query screen geometry");
        }

        let config = config::configuration();

        // NOTE: GetScreenResourcesCurrent is fast, but may sometimes return nothing. In this case,
        // fallback to slow GetScreenResources.
        //
        // references:
        // - https://github.com/qt/qtbase/blob/c234700c836777d08db6229fdc997cc7c99e45fb/src/plugins/platforms/xcb/qxcbscreen.cpp#L963
        // - https://github.com/qt/qtbase/blob/c234700c836777d08db6229fdc997cc7c99e45fb/src/plugins/platforms/xcb/qxcbconnection_screens.cpp#L390
        //
        // related issue: https://github.com/wezterm/wezterm/issues/5802
        let res = match self
            .send_and_wait_request(&xcb::randr::GetScreenResourcesCurrent { window: self.root })
            .context("get_screen_resources_current")
        {
            Ok(cur) if cur.outputs().len() > 0 => ScreenResources::Current(cur),
            _ => ScreenResources::All(
                self.send_and_wait_request(&xcb::randr::GetScreenResources { window: self.root })
                    .context("get_screen_resources")?,
            ),
        };

        let mut virtual_rect: ScreenRect = euclid::rect(0, 0, 0, 0);
        let mut by_name = HashMap::new();

        for &o in res.outputs() {
            let info = self
                .send_and_wait_request(&xcb::randr::GetOutputInfo {
                    output: o,
                    config_timestamp: res.config_timestamp(),
                })
                .context("get_output_info")?;
            let name = String::from_utf8_lossy(info.name()).to_string();
            let c = info.crtc();
            if let Ok(cinfo) = self.send_and_wait_request(&xcb::randr::GetCrtcInfo {
                crtc: c,
                config_timestamp: res.config_timestamp(),
            }) {
                let mode = cinfo.mode();
                let max_fps = res
                    .modes()
                    .iter()
                    .find(|m| m.id == mode.resource_id())
                    .and_then(|m| {
                        use xcb::randr::ModeFlag;
                        let mut vtotal = m.vtotal;
                        if m.mode_flags.contains(ModeFlag::DOUBLE_SCAN) {
                            // Doublescan doubles the number of lines
                            vtotal *= 2;
                        }
                        if m.mode_flags.contains(ModeFlag::INTERLACE) {
                            // Interlace splits the frame into two fields.
                            // The field rate is what is typically reported
                            // by monitors.
                            vtotal /= 2;
                        }
                        if m.htotal > 0 && vtotal > 0 {
                            Some(
                                (m.dot_clock as f32 / (m.htotal as f32 * vtotal as f32)).ceil()
                                    as usize,
                            )
                        } else {
                            None
                        }
                    });
                let bounds = euclid::rect(
                    cinfo.x() as isize,
                    cinfo.y() as isize,
                    cinfo.width() as isize,
                    cinfo.height() as isize,
                );
                virtual_rect = virtual_rect.union(&bounds);

                let mut effective_dpi = Some(self.default_dpi());
                if let Some(dpi) = config.dpi_by_screen.get(&name).copied() {
                    effective_dpi.replace(dpi);
                } else if let Some(dpi) = config.dpi {
                    effective_dpi.replace(dpi);
                }

                let info = ScreenInfo {
                    name: name.clone(),
                    rect: bounds,
                    scale: 1.0,
                    max_fps,
                    effective_dpi,
                };
                by_name.insert(name, info);
            }
        }

        // The main screen is the one either at the origin of
        // the virtual area, or if that doesn't exist for some weird
        // reason, the screen closest to the origin.
        let main = by_name
            .values()
            .min_by_key(|screen| {
                screen
                    .rect
                    .origin
                    .to_f32()
                    .distance_to(euclid::Point2D::origin())
                    .abs() as isize
            })
            .ok_or_else(|| anyhow::anyhow!("no screens were found"))?
            .clone();

        let active = self
            .screen_from_focused_window(&by_name)
            .unwrap_or_else(|_| main.clone());

        Ok(Screens {
            main,
            active,
            by_name,
            virtual_rect,
        })
    }

    fn run_message_loop(&self) -> anyhow::Result<()> {
        self.conn.flush()?;

        const TOK_XCB: usize = 0xffff_fffc;
        const TOK_SPAWN: usize = 0xffff_fffd;
        let tok_xcb = Token(TOK_XCB);
        let tok_spawn = Token(TOK_SPAWN);

        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(8);
        poll.registry().register(
            &mut SourceFd(&self.conn.as_raw_fd()),
            tok_xcb,
            Interest::READABLE,
        )?;
        poll.registry().register(
            &mut SourceFd(&SPAWN_QUEUE.raw_fd()),
            tok_spawn,
            Interest::READABLE,
        )?;

        while !*self.should_terminate.borrow() {
            // Process any events that might have accumulated in the local
            // buffer (eg: due to a flush) before we potentially go to sleep.
            // The locally queued events won't mark the fd as ready, so we
            // could potentially sleep when there is work to be done if we
            // relied solely on that.
            self.process_queued_xcb().context("process_queued_xcb")?;

            // Check the spawn queue before we try to sleep; there may
            // be work pending and we don't guarantee that there is a
            // 1:1 wakeup to queued function, so we need to be assertive
            // in order to avoid missing wakeups
            if SPAWN_QUEUE.run() {
                // if we processed one, we don't want to sleep because
                // there may be others to deal with
                continue;
            }

            self.dispatch_pending_events()
                .context("dispatch_pending_events")?;
            if let Err(err) = poll.poll(&mut events, None) {
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                bail!("polling for events: {:?}", err);
            }
        }

        Ok(())
    }

    fn beep(&self) {
        self.conn.send_request(&xcb::x::Bell { percent: 0 });
    }
}

fn compute_default_dpi(xrm: &HashMap<String, String>, xsettings: &XSettingsMap) -> f64 {
    if let Some(XSetting::Integer(dpi)) = xsettings.get("Xft/DPI") {
        *dpi as f64 / 1024.0
    } else {
        xrm.get("Xft.dpi")
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("96")
            .parse::<f64>()
            .unwrap_or(crate::DEFAULT_DPI)
    }
