impl XConnection {

    pub(crate) fn send_and_wait_request<R>(
        &self,
        req: &R,
    ) -> anyhow::Result<<<R as xcb::Request>::Cookie as xcb::CookieWithReplyChecked>::Reply>
    where
        R: xcb::Request + std::fmt::Debug,
        R::Cookie: xcb::CookieWithReplyChecked,
    {
        let cookie = self.conn.send_request(req);
        self.conn
            .wait_for_reply(cookie)
            .with_context(|| format!("{req:#?}"))
    }

    pub(crate) fn send_request_no_reply<R>(&self, req: &R) -> anyhow::Result<()>
    where
        R: xcb::RequestWithoutReply + std::fmt::Debug,
    {
        self.conn
            .send_and_check_request(req)
            .with_context(|| format!("{req:#?}"))
    }

    pub(crate) fn send_request_no_reply_log<R>(&self, req: &R)
    where
        R: xcb::RequestWithoutReply + std::fmt::Debug,
    {
        if let Err(err) = self.send_request_no_reply(req) {
            log::error!("{err:#}");
        }
    }

    pub fn atom_name(&self, atom: Atom) -> String {
        if let Some(name) = self.atom_names.borrow().get(&atom) {
            return name.to_string();
        }
        let cookie = self.conn.send_request(&xcb::x::GetAtomName { atom });
        let name = if let Ok(reply) = self.conn.wait_for_reply(cookie) {
            reply.name().to_string()
        } else {
            format!("{:?}", atom)
        };

        self.atom_names.borrow_mut().insert(atom, name.to_string());
        name
    }

    pub fn conn(&self) -> &xcb::Connection {
        &self.conn
    }

    pub fn screen_num(&self) -> i32 {
        self.screen_num
    }

    pub fn atom_delete(&self) -> Atom {
        self.atom_delete
    }

    pub(crate) fn with_window_inner<
        R,
        F: FnOnce(&mut XWindowInner) -> anyhow::Result<R> + Send + 'static,
    >(
        window: xcb::x::Window,
        f: F,
    ) -> promise::Future<R>
    where
        R: Send + 'static,
    {
        let mut prom = promise::Promise::new();
        let future = prom.get_future().unwrap();

        promise::spawn::spawn_into_main_thread(async move {
            if let Some(handle) = Connection::get().unwrap().x11().window_by_id(window) {
                let mut inner = handle.lock().unwrap();
                if inner.window_id != window {
                    prom.result(Err(anyhow!("window {window:?} has been destroyed")));
                } else {
                    prom.result(f(&mut inner));
                }
            }
        })
        .detach();

        future
    }

    fn screen_from_focused_window(
        &self,
        by_name: &HashMap<String, ScreenInfo>,
    ) -> anyhow::Result<ScreenInfo> {
        let focused = self
            .send_and_wait_request(&xcb::x::GetInputFocus {})
            .context("querying focused window")?;
        let geom = self
            .send_and_wait_request(&xcb::x::GetGeometry {
                drawable: xcb::x::Drawable::Window(focused.focus()),
            })
            .context("querying geometry")?;
        let trans_geom = self
            .send_and_wait_request(&xcb::x::TranslateCoordinates {
                src_window: focused.focus(),
                dst_window: self.root,
                src_x: 0,
                src_y: 0,
            })
            .context("querying root coordinates")?;
        let window_rect: ScreenRect = euclid::rect(
            trans_geom.dst_x().into(),
            trans_geom.dst_y().into(),
            geom.width() as isize,
            geom.height() as isize,
        );
        Ok(by_name
            .values()
            .filter_map(|screen| {
                screen
                    .rect
                    .intersection(&window_rect)
                    .map(|r| (screen, r.area()))
            })
            .max_by_key(|s| s.1)
            .ok_or_else(|| anyhow::anyhow!("active window is not in any screen"))?
            .0
            .clone())
}
