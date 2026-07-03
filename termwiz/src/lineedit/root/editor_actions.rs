impl<'term> LineEditor<'term> {
    fn resolve_action(
        &mut self,
        event: &InputEvent,
        host: &mut dyn LineEditorHost,
    ) -> Option<Action> {
        if let Some(action) = host.resolve_action(event, self) {
            return Some(action);
        }

        match event {
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('C'),
                modifiers: Modifiers::CTRL,
            }) => Some(Action::Cancel),

            InputEvent::Key(KeyEvent {
                key: KeyCode::Tab,
                modifiers: Modifiers::NONE,
            }) => Some(Action::Complete),

            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('D'),
                modifiers: Modifiers::CTRL,
            }) => Some(Action::EndOfFile),

            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('J'),
                modifiers: Modifiers::CTRL,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::Char('M'),
                modifiers: Modifiers::CTRL,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::Enter,
                modifiers: Modifiers::NONE,
            }) => Some(Action::AcceptLine),
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('H'),
                modifiers: Modifiers::CTRL,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::Backspace,
                modifiers: Modifiers::NONE,
            }) => Some(Action::Kill(Movement::BackwardChar(1))),
            InputEvent::Key(KeyEvent {
                key: KeyCode::Delete,
                modifiers: Modifiers::NONE,
            }) => Some(Action::KillAndMove(
                Movement::ForwardChar(1),
                Movement::None,
            )),

            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('P'),
                modifiers: Modifiers::CTRL,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::UpArrow,
                modifiers: Modifiers::NONE,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::ApplicationUpArrow,
                modifiers: Modifiers::NONE,
            }) => Some(Action::HistoryPrevious),

            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('N'),
                modifiers: Modifiers::CTRL,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::DownArrow,
                modifiers: Modifiers::NONE,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::ApplicationDownArrow,
                modifiers: Modifiers::NONE,
            }) => Some(Action::HistoryNext),

            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('B'),
                modifiers: Modifiers::CTRL,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::ApplicationLeftArrow,
                modifiers: Modifiers::NONE,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::LeftArrow,
                modifiers: Modifiers::NONE,
            }) => Some(Action::Move(Movement::BackwardChar(1))),

            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('W'),
                modifiers: Modifiers::CTRL,
            }) => Some(Action::Kill(Movement::BackwardWord(1))),

            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('b'),
                modifiers: Modifiers::ALT,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::LeftArrow,
                modifiers: Modifiers::ALT,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::ApplicationLeftArrow,
                modifiers: Modifiers::ALT,
            }) => Some(Action::Move(Movement::BackwardWord(1))),

            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('f'),
                modifiers: Modifiers::ALT,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::RightArrow,
                modifiers: Modifiers::ALT,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::ApplicationRightArrow,
                modifiers: Modifiers::ALT,
            }) => Some(Action::Move(Movement::ForwardWord(1))),

            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('A'),
                modifiers: Modifiers::CTRL,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::Home,
                modifiers: Modifiers::NONE,
            }) => Some(Action::Move(Movement::StartOfLine)),
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('E'),
                modifiers: Modifiers::CTRL,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::End,
                modifiers: Modifiers::NONE,
            }) => Some(Action::Move(Movement::EndOfLine)),
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('F'),
                modifiers: Modifiers::CTRL,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::RightArrow,
                modifiers: Modifiers::NONE,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::ApplicationRightArrow,
                modifiers: Modifiers::NONE,
            }) => Some(Action::Move(Movement::ForwardChar(1))),
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char(c),
                modifiers: Modifiers::SHIFT,
            })
            | InputEvent::Key(KeyEvent {
                key: KeyCode::Char(c),
                modifiers: Modifiers::NONE,
            }) => Some(Action::InsertChar(1, *c)),
            InputEvent::Paste(text) => Some(Action::InsertText(1, text.clone())),
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('L'),
                modifiers: Modifiers::CTRL,
            }) => Some(Action::Repaint),
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('K'),
                modifiers: Modifiers::CTRL,
            }) => Some(Action::Kill(Movement::EndOfLine)),

            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('R'),
                modifiers: Modifiers::CTRL,
            }) => Some(Action::HistoryIncSearchBackwards),

            // This is the common binding for forwards, but it is usually
            // masked by the stty stop setting
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('S'),
                modifiers: Modifiers::CTRL,
            }) => Some(Action::HistoryIncSearchForwards),

            _ => None,
        }
    }

    fn kill_text(&mut self, kill_movement: Movement, move_movement: Movement) {
        self.clear_completion();
        self.line.kill_text(kill_movement, move_movement);
    }

    fn clear_completion(&mut self) {
        self.completion = None;
    }

    fn cancel_search_state(&mut self) {
        if let EditorState::Searching {
            matching_line,
            cursor,
            ..
        } = &self.state
        {
            self.line.set_line_and_cursor(matching_line, *cursor);
            self.state = EditorState::Editing;
        }
    }

    /// Returns the current line and cursor position.
    /// You don't normally need to call this unless you are defining
    /// a custom editor operation on the line buffer contents.
    /// The cursor position is the byte index into the line UTF-8 bytes.
    pub fn get_line_and_cursor(&mut self) -> (&str, usize) {
        (self.line.get_line(), self.line.get_cursor())
    }

    /// Sets the current line and cursor position.
    /// You don't normally need to call this unless you are defining
    /// a custom editor operation on the line buffer contents.
    /// The cursor position is the byte index into the line UTF-8 bytes.
    /// Panics: the cursor must be the first byte in a UTF-8 code point
    /// sequence or the end of the provided line.
    pub fn set_line_and_cursor(&mut self, line: &str, cursor: usize) {
        self.line.set_line_and_cursor(line, cursor);
    }

    /// Call this after changing modifying the line buffer.
    /// If the editor is in search mode this will update the search
    /// results, otherwise it will be a NOP.
    fn reapply_search_pattern(&mut self, host: &mut dyn LineEditorHost) {
        if let EditorState::Searching {
            style,
            direction,
            matching_line,
            cursor,
        } = &self.state
        {
            // We always start again from the bottom
            self.history_pos.take();

            let history_pos = match host.history().last() {
                Some(p) => p,
                None => {
                    // TODO: there's no way we can match anything.
                    // Generate a failed match result?
                    return;
                }
            };

            let last_matching_line;
            let last_cursor;

            if let Some(result) =
                host.history()
                    .search(history_pos, *style, *direction, self.line.get_line())
            {
                self.history_pos.replace(result.idx);
                last_matching_line = result.line.to_string();
                last_cursor = result.cursor;
            } else {
                last_matching_line = matching_line.clone();
                last_cursor = *cursor;
            }

            self.state = EditorState::Searching {
                style: *style,
                direction: *direction,
                matching_line: last_matching_line,
                cursor: last_cursor,
            };
        }
    }

    fn trigger_search(
        &mut self,
        style: SearchStyle,
        direction: SearchDirection,
        host: &mut dyn LineEditorHost,
    ) {
        self.clear_completion();

        if let EditorState::Searching { .. } = &self.state {
            // Already searching
        } else {
            // Not yet searching, so we start a new search
            // with an empty pattern
            self.line.clear();
            self.history_pos.take();
        }

        let history_pos = match self.history_pos {
            Some(p) => match direction.next(p) {
                Some(p) => p,
                None => return,
            },
            None => match host.history().last() {
                Some(p) => p,
                None => {
                    // TODO: there's no way we can match anything.
                    // Generate a failed match result?
                    return;
                }
            },
        };

        let search_result =
            host.history()
                .search(history_pos, style, direction, self.line.get_line());

        let last_matching_line;
        let last_cursor;

        if let Some(result) = search_result {
            self.history_pos.replace(result.idx);
            last_matching_line = result.line.to_string();
            last_cursor = result.cursor;
        } else if let EditorState::Searching {
            matching_line,
            cursor,
            ..
        } = &self.state
        {
            last_matching_line = matching_line.clone();
            last_cursor = *cursor;
        } else {
            last_matching_line = String::new();
            last_cursor = 0;
        }

        self.state = EditorState::Searching {
            style,
            direction,
            matching_line: last_matching_line,
            cursor: last_cursor,
        };
    }

    /// Applies the effect of the specified action to the line editor.
    /// You don't normally need to call this unless you are defining
    /// custom key mapping or custom actions in your embedding application.
    pub fn apply_action(&mut self, host: &mut dyn LineEditorHost, action: Action) -> Result<()> {
        // When searching, reinterpret history next/prev as repeated
        // search actions in the appropriate direction
        let action = match (action, &self.state) {
            (
                Action::HistoryPrevious,
                EditorState::Searching {
                    style: SearchStyle::Substring,
                    ..
                },
            ) => Action::HistoryIncSearchBackwards,
            (
                Action::HistoryNext,
                EditorState::Searching {
                    style: SearchStyle::Substring,
                    ..
                },
            ) => Action::HistoryIncSearchForwards,
            (action, _) => action,
        };

        match action {
            Action::Cancel => self.state = EditorState::Cancelled,
            Action::NoAction => {}
            Action::AcceptLine => {
                // Make sure that hitting Enter for a line that
                // shows in the incremental search causes that
                // line to be accepted, rather than the search pattern!
                self.cancel_search_state();

                self.state = EditorState::Accepted;
            }
            Action::EndOfFile => {
                return Err(
                    std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "End Of File").into(),
                )
            }
            Action::Kill(movement) => {
                self.kill_text(movement, movement);
                self.reapply_search_pattern(host);
            }
            Action::KillAndMove(kill_movement, move_movement) => {
                self.kill_text(kill_movement, move_movement);
                self.reapply_search_pattern(host);
            }

            Action::Move(movement) => {
                self.clear_completion();
                self.cancel_search_state();
                self.line.exec_movement(movement);
            }

            Action::InsertChar(rep, c) => {
                self.clear_completion();
                for _ in 0..rep {
                    self.line.insert_char(c);
                }
                self.reapply_search_pattern(host);
            }
            Action::InsertText(rep, text) => {
                self.clear_completion();
                for _ in 0..rep {
                    self.line.insert_text(&text);
                }
                self.reapply_search_pattern(host);
            }
            Action::Repaint => {
                self.terminal
                    .render(&[Change::ClearScreen(Default::default())])?;
            }
            Action::HistoryPrevious => {
                self.clear_completion();
                self.cancel_search_state();

                if let Some(cur_pos) = self.history_pos.as_ref() {
                    let prior_idx = cur_pos.saturating_sub(1);
                    if let Some(prior) = host.history().get(prior_idx) {
                        self.history_pos = Some(prior_idx);
                        self.line.set_line_and_cursor(&prior, prior.len());
                    }
                } else if let Some(last) = host.history().last() {
                    self.bottom_line = Some(self.line.get_line().to_string());
                    self.history_pos = Some(last);
                    let line = host
                        .history()
                        .get(last)
                        .expect("History::last and History::get to be consistent");
                    self.line.set_line_and_cursor(&line, line.len())
                }
            }
            Action::HistoryNext => {
                self.clear_completion();
                self.cancel_search_state();

                if let Some(cur_pos) = self.history_pos.as_ref() {
                    let next_idx = cur_pos.saturating_add(1);
                    if let Some(next) = host.history().get(next_idx) {
                        self.history_pos = Some(next_idx);
                        self.line.set_line_and_cursor(&next, next.len());
                    } else if let Some(bottom) = self.bottom_line.take() {
                        self.line.set_line_and_cursor(&bottom, bottom.len());
                    } else {
                        self.line.clear();
                    }
                }
            }

            Action::HistoryIncSearchBackwards => {
                self.trigger_search(SearchStyle::Substring, SearchDirection::Backwards, host);
            }
            Action::HistoryIncSearchForwards => {
                self.trigger_search(SearchStyle::Substring, SearchDirection::Forwards, host);
            }

            Action::Complete => {
                self.cancel_search_state();

                if self.completion.is_none() {
                    let candidates = host.complete(self.line.get_line(), self.line.get_cursor());
                    if !candidates.is_empty() {
                        let state = CompletionState {
                            candidates,
                            index: 0,
                            original_line: self.line.get_line().to_string(),
                            original_cursor: self.line.get_cursor(),
                        };

                        let (cursor, line) = state.current();
                        self.line.set_line_and_cursor(&line, cursor);

                        // If there is only a single completion then don't
                        // leave us in a state where we just cycle on the
                        // same completion over and over.
                        if state.candidates.len() > 1 {
                            self.completion = Some(state);
                        }
                    }
                } else if let Some(state) = self.completion.as_mut() {
                    state.next();
                    let (cursor, line) = state.current();
                    self.line.set_line_and_cursor(&line, cursor);
                }
            }
        }

        Ok(())
    }
}
