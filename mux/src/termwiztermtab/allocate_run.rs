pub fn allocate(
    size: TerminalSize,
    config: Arc<dyn TerminalConfiguration + Send + Sync>,
) -> (TermWizTerminal, Arc<dyn Pane>) {
    let render_pipe = Pipe::new().expect("Pipe creation not to fail");

    let (input_tx, input_rx) = channel();

    let renderer = termwiz_funcs::new_wezterm_terminfo_renderer();

    let tw_term = TermWizTerminal {
        render_tx: TermWizTerminalRenderTty {
            render_tx: BufWriter::new(render_pipe.write),
            screen_size: ScreenSize {
                cols: size.cols as usize,
                rows: size.rows as usize,
                xpixel: (size.pixel_width / size.cols) as usize,
                ypixel: (size.pixel_height / size.rows) as usize,
            },
        },
        input_rx,
        renderer,
        grab_mouse: true,
    };

    let domain_id = 0;
    let pane = TermWizTerminalPane::new(domain_id, size, input_tx, render_pipe.read, Some(config));

    // Add the tab to the mux so that the output is processed
    let pane: Arc<dyn Pane> = Arc::new(pane);

    let mux = Mux::get();
    mux.add_pane(&pane).expect("to be able to add pane to mux");

    (tw_term, pane)
}

/// This function spawns a thread and constructs a GUI window with an
/// associated termwiz Terminal object to execute the provided function.
/// The function is expected to run in a loop to manage input and output
/// from the terminal window.
/// When it completes its loop it will fulfil a promise and yield
/// the return value from the function.
pub async fn run<
    T: Send + 'static,
    F: Send + 'static + FnOnce(TermWizTerminal) -> anyhow::Result<T>,
>(
    size: TerminalSize,
    window_id: Option<WindowId>,
    f: F,
    term_config: Option<Arc<dyn TerminalConfiguration + Send + Sync>>,
) -> anyhow::Result<T> {
    let render_pipe = Pipe::new().expect("Pipe creation not to fail");
    let render_rx = render_pipe.read;
    let (input_tx, input_rx) = channel();
    let should_close_window = window_id.is_none();

    let renderer = termwiz_funcs::new_wezterm_terminfo_renderer();

    let tw_term = TermWizTerminal {
        render_tx: TermWizTerminalRenderTty {
            render_tx: BufWriter::new(render_pipe.write),
            screen_size: ScreenSize {
                cols: size.cols as usize,
                rows: size.rows as usize,
                xpixel: (size.pixel_width / size.cols) as usize,
                ypixel: (size.pixel_height / size.rows) as usize,
            },
        },
        input_rx,
        renderer,
        grab_mouse: true,
    };

    async fn register_tab(
        input_tx: Sender<InputEvent>,
        render_rx: FileDescriptor,
        size: TerminalSize,
        window_id: Option<WindowId>,
        term_config: Option<Arc<dyn TerminalConfiguration + Send + Sync>>,
    ) -> anyhow::Result<(PaneId, WindowId)> {
        let mux = Mux::get();

        // TODO: make a singleton
        let domain: Arc<dyn Domain> = Arc::new(TermWizTerminalDomain::new());
        mux.add_domain(&domain);

        let window_builder;
        let window_id = match window_id {
            Some(id) => id,
            None => {
                window_builder = mux.new_empty_window(None, None);
                *window_builder
            }
        };

        let pane =
            TermWizTerminalPane::new(domain.domain_id(), size, input_tx, render_rx, term_config);
        let pane: Arc<dyn Pane> = Arc::new(pane);

        let tab = Arc::new(Tab::new(&size));
        tab.assign_pane(&pane);

        mux.add_tab_and_active_pane(&tab)?;
        mux.add_tab_to_window(&tab, window_id)?;

        let mut window = mux
            .get_window_mut(window_id)
            .ok_or_else(|| anyhow::anyhow!("invalid window id {}", window_id))?;
        let tab_idx = window.len().saturating_sub(1);
        window.save_and_then_set_active(tab_idx);

        Ok((pane.pane_id(), window_id))
    }

    let (pane_id, window_id) = promise::spawn::spawn_into_main_thread(async move {
        register_tab(input_tx, render_rx, size, window_id, term_config).await
    })
    .await?;

    let result = promise::spawn::spawn_into_new_thread(move || f(tw_term)).await;

    // Since we're typically called with an outstanding Activity token active,
    // the dead status of the tab will be ignored until after the activity
    // resolves.  In the case of SSH where (currently!) several prompts may
    // be shown in succession, we don't want to leave lingering dead windows
    // on the screen so let's ask the mux to kill off our window now.
    promise::spawn::spawn_into_main_thread(async move {
        let mux = Mux::get();
        if should_close_window {
            mux.kill_window(window_id);
        } else if let Some(pane) = mux.get_pane(pane_id) {
            pane.kill();
            mux.remove_pane(pane.pane_id());
        }
    })
    .detach();

    result
}
