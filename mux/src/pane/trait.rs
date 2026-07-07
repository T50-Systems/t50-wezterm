#[async_trait(?Send)]
pub trait Pane: Downcast + Send + Sync {
    fn pane_id(&self) -> PaneId;

    /// Returns the 0-based cursor position relative to the top left of
    /// the visible screen
    fn get_cursor_position(&self) -> StableCursorPosition;

    fn get_current_seqno(&self) -> SequenceNo;

    /// Returns misc metadata that is pane-specific
    fn get_metadata(&self) -> Value {
        Value::Null
    }

    /// Given a range of lines, return the subset of those lines that
    /// have changed since the supplied sequence no.
    fn get_changed_since(
        &self,
        lines: Range<StableRowIndex>,
        seqno: SequenceNo,
    ) -> RangeSet<StableRowIndex>;

    /// Returns a set of lines from the scrollback or visible portion of
    /// the display.  The lines are indexed using StableRowIndex, which
    /// can be invalidated if the scrollback is busy, or when switching
    /// to the alternate screen.
    /// To deal with this, this function will adjust the input so that
    /// a range that has been scrolled off the top will return the top
    /// n rows of the scrollback (where n is the size of the input range),
    /// or the bottom n rows of the scrollback when switching to the alt
    /// screen and the index would go off the bottom.
    /// Because of this, we also return the adjusted StableRowIndex for
    /// the first row in the range.
    fn get_lines(&self, lines: Range<StableRowIndex>) -> (StableRowIndex, Vec<Line>);

    fn with_lines_mut(&self, lines: Range<StableRowIndex>, with_lines: &mut dyn WithPaneLines);

    fn for_each_logical_line_in_stable_range_mut(
        &self,
        lines: Range<StableRowIndex>,
        for_line: &mut dyn ForEachPaneLogicalLine,
    );

    fn get_logical_lines(&self, lines: Range<StableRowIndex>) -> Vec<LogicalLine>;

    fn apply_hyperlinks(&self, lines: Range<StableRowIndex>, rules: &[Rule]) {
        struct ApplyHyperLinks<'a> {
            rules: &'a [Rule],
        }
        impl<'a> ForEachPaneLogicalLine for ApplyHyperLinks<'a> {
            fn with_logical_line_mut(
                &mut self,
                _: Range<StableRowIndex>,
                lines: &mut [&mut Line],
            ) -> bool {
                Line::apply_hyperlink_rules(self.rules, lines);

                true
            }
        }

        self.for_each_logical_line_in_stable_range_mut(lines, &mut ApplyHyperLinks { rules });
    }

    /// Returns render related dimensions
    fn get_dimensions(&self) -> RenderableDimensions;

    fn get_title(&self) -> String;
    fn get_progress(&self) -> Progress {
        Progress::None
    }
    fn send_paste(&self, text: &str) -> anyhow::Result<()>;
    fn reader(&self) -> anyhow::Result<Option<Box<dyn std::io::Read + Send>>>;
    fn writer(&self) -> MappedMutexGuard<'_, dyn std::io::Write>;
    fn resize(&self, size: TerminalSize) -> anyhow::Result<()>;
    /// Called as a hint that the pane is being resized as part of
    /// a zoom-to-fill-all-the-tab-space operation.
    fn set_zoomed(&self, _zoomed: bool) {}
    fn key_down(&self, key: KeyCode, mods: KeyModifiers) -> anyhow::Result<()>;
    fn key_up(&self, key: KeyCode, mods: KeyModifiers) -> anyhow::Result<()>;
    fn perform_assignment(&self, _assignment: &KeyAssignment) -> PerformAssignmentResult {
        PerformAssignmentResult::Unhandled
    }
    fn mouse_event(&self, event: MouseEvent) -> anyhow::Result<()>;
    fn perform_actions(&self, _actions: Vec<termwiz::escape::Action>) {}
    fn is_dead(&self) -> bool;
    fn kill(&self) {}
    fn palette(&self) -> ColorPalette;
    fn domain_id(&self) -> DomainId;

    fn get_keyboard_encoding(&self) -> KeyboardEncoding {
        KeyboardEncoding::Xterm
    }

    fn copy_user_vars(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn erase_scrollback(&self, _erase_mode: ScrollbackEraseMode) {}

    /// Called to advise on whether this tab has focus
    fn focus_changed(&self, _focused: bool) {}

    /// Called to advise remote mux that this is the active tab
    /// for the current identity
    fn advise_focus(&self) {}

    fn has_unseen_output(&self) -> bool {
        false
    }

    /// Certain panes are OK to be closed with impunity (no prompts)
    fn can_close_without_prompting(&self, _reason: CloseReason) -> bool {
        false
    }

    /// Performs a search bounded to the specified range.
    /// If the result is empty then there are no matches.
    /// Otherwise, if limit.is_none(), the result shall contain all possible
    /// matches.
    /// If limit.is_some(), then the maximum number of results that will be
    /// returned is limited to the specified number, and the
    /// SearchResult::start_y of the last item
    /// in the result can be used as the start of the next region to search.
    /// You can tell that you have reached the end of the results if the number
    /// of results is smaller than the limit you set.
    async fn search(
        &self,
        _pattern: Pattern,
        _range: Range<StableRowIndex>,
        _limit: Option<u32>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        Ok(vec![])
    }

    /// Retrieve the set of semantic zones
    fn get_semantic_zones(&self) -> anyhow::Result<Vec<SemanticZone>> {
        Ok(vec![])
    }

    /// Returns true if the terminal has grabbed the mouse and wants to
    /// give the embedded application a chance to process events.
    /// In practice this controls whether the gui will perform local
    /// handling of clicks.
    fn is_mouse_grabbed(&self) -> bool;
    fn is_alt_screen_active(&self) -> bool;

    fn set_clipboard(&self, _clipboard: &Arc<dyn Clipboard>) {}
    fn set_download_handler(&self, _handler: &Arc<dyn DownloadHandler>) {}
    fn set_config(&self, _config: Arc<dyn TerminalConfiguration>) {}
    fn get_config(&self) -> Option<Arc<dyn TerminalConfiguration>> {
        None
    }

    fn get_current_working_dir(&self, policy: CachePolicy) -> Option<Url>;
    fn get_foreground_process_name(&self, _policy: CachePolicy) -> Option<String> {
        None
    }
    fn get_foreground_process_info(
        &self,
        _policy: CachePolicy,
    ) -> Option<procinfo::LocalProcessInfo> {
        None
    }

    fn tty_name(&self) -> Option<String> {
        None
    }

    fn exit_behavior(&self) -> Option<ExitBehavior> {
        None
    }
}
impl_downcast!(Pane);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePolicy {
    FetchImmediate,
    AllowStale,
}

/// This trait is used to implement/provide a callback that is used together
/// with the Pane::with_lines_mut method.
/// Ideally we'd simply pass an FnMut with the same signature as the trait
/// method defined here, but doing so results in Pane not being object-safe.
pub trait WithPaneLines {
    /// The `first_row` parameter is set to the StableRowIndex of the resolved
    /// first row from the Pane::with_lines_mut method. It will usually be
    /// the start of the lines range, but in case that row is no longer in
    /// a valid range (scrolled out of scrollback), it may be revised.
    ///
    /// `lines` is a mutable slice of the mutable lines in the requested
    /// stable range.
    fn with_lines_mut(&mut self, first_row: StableRowIndex, lines: &mut [&mut Line]);
}

/// This trait is used to implement/provide a callback that is used together
/// with the Pane::for_each_logical_line_in_stable_range_mut method.
/// Ideally we'd simply pass an FnMut with the same signature as the trait
/// method defined here, but doing so results in Pane not being object-safe.
pub trait ForEachPaneLogicalLine {
    /// The `stable_range` parameter is set to the range of physical lines
    /// that comprise the current logical line.
    ///
    /// `lines` is a mutable slice of the mutable physical lines that comprise
    /// the current logical line.
    ///
    /// Return `true` to continue with the next logical line in the requested
    /// range, or `false` to cease iteration.
    fn with_logical_line_mut(
        &mut self,
        stable_range: Range<StableRowIndex>,
        lines: &mut [&mut Line],
    ) -> bool;
}
