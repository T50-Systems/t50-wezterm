impl BidiContext {
    /// This function runs Rules I1 and I2 together.
    fn resolve_implicit_levels(&mut self) {
        for (idx, level) in self.levels.iter_mut().enumerate() {
            if level.removed_by_x9() {
                continue;
            }

            match level.direction() {
                Direction::LeftToRight => {
                    // I1
                    match self.char_types[idx] {
                        BidiClass::RightToLeft => {
                            level.0 += 1;
                        }
                        BidiClass::ArabicNumber | BidiClass::EuropeanNumber => {
                            level.0 += 2;
                        }
                        _ => {}
                    }
                }
                Direction::RightToLeft => {
                    // I2
                    match self.char_types[idx] {
                        BidiClass::LeftToRight
                        | BidiClass::ArabicNumber
                        | BidiClass::EuropeanNumber => {
                            level.0 += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// This function runs Rule L1.
    ///
    /// The strategy here for Rule L1 is to scan forward through
    /// the text searching for segment separators or paragraph
    /// separators. If a segment separator or paragraph
    /// separator is found, it is reset to the paragraph embedding
    /// level. Then scan backwards from the separator to
    /// find any contiguous stretch of whitespace characters
    /// and reset any which are found to the paragraph embedding
    /// level, as well. When we reach the *last* character in the
    /// text (which will also constitute, by definition, the last
    /// character in the line being processed here), check if it
    /// is whitespace. If so, reset it to the paragraph embedding
    /// level. Then scan backwards to find any contiguous stretch
    /// of whitespace characters and reset those as well.
    ///
    /// These checks for whitespace are done with the *original*
    /// Bidi_Class values for characters, not the resolved values.
    ///
    /// As for many rules, this rule simply ignores any character
    /// whose level has been set to NO_LEVEL, which is the way
    /// this reference algorithm "deletes" boundary neutrals and
    /// embedding and override controls from the text.
    fn reset_whitespace_levels(&self, line_range: Range<usize>) -> Vec<Level> {
        fn reset_contiguous_whitespace_before(
            line_range: Range<usize>,
            base_level: Level,
            orig_char_types: &[BidiClass],
            levels: &mut Vec<Level>,
        ) {
            for i in line_range.rev() {
                if orig_char_types[i] == BidiClass::WhiteSpace
                    || orig_char_types[i].is_iso_control()
                {
                    levels[i] = base_level;
                } else if levels[i].removed_by_x9() {
                    // Skip over deleted entries
                } else {
                    // end of contiguous section
                    break;
                }
            }
        }

        let mut levels = self.levels.clone();

        for (idx, orig_bc) in self
            .orig_char_types
            .iter()
            .enumerate()
            .skip(line_range.start)
            .take(line_range.end - line_range.start)
        {
            match orig_bc {
                // Explicit boundary
                BidiClass::SegmentSeparator | BidiClass::ParagraphSeparator => {
                    levels[idx] = self.base_level;
                    reset_contiguous_whitespace_before(
                        line_range.start..idx,
                        self.base_level,
                        &self.orig_char_types,
                        &mut levels,
                    );
                }
                _ => {}
            }
        }

        reset_contiguous_whitespace_before(
            line_range.clone(),
            self.base_level,
            &self.orig_char_types,
            &mut levels,
        );

        levels[line_range].to_vec()
    }

    /// Rules X1 through X8
    fn explicit_embedding_levels(&mut self) {
        // X1: initialize stack and other variables
        let mut stack = LevelStack::new();
        stack.push(self.base_level, Override::Neutral, false);

        let len = self.char_types.len();
        self.levels.resize(len, Level::default());

        let mut overflow_isolate = 0;
        let mut overflow_embedding = 0;
        let mut valid_isolate = 0;

        // X2..X8: process each character, setting embedding levels
        // and override status
        for idx in 0..len {
            let bc = self.char_types[idx];
            trace!("Considering idx={} {:?}", idx, bc);
            match bc {
                // X2
                BidiClass::RightToLeftEmbedding => {
                    if let Some(level) = stack.embedding_level().least_greater_odd() {
                        if overflow_isolate == 0 && overflow_embedding == 0 {
                            stack.push(level, Override::Neutral, false);
                            continue;
                        }
                    }
                    if overflow_isolate == 0 {
                        overflow_embedding += 1;
                    }
                }
                // X3
                BidiClass::LeftToRightEmbedding => {
                    if let Some(level) = stack.embedding_level().least_greater_even() {
                        if overflow_isolate == 0 && overflow_embedding == 0 {
                            stack.push(level, Override::Neutral, false);
                            continue;
                        }
                    }
                    if overflow_isolate == 0 {
                        overflow_embedding += 1;
                    }
                }
                // X4
                BidiClass::RightToLeftOverride => {
                    if let Some(level) = stack.embedding_level().least_greater_odd() {
                        if overflow_isolate == 0 && overflow_embedding == 0 {
                            stack.push(level, Override::RTL, false);
                            continue;
                        }
                    }
                    if overflow_isolate == 0 {
                        overflow_embedding += 1;
                    }
                }
                // X5
                BidiClass::LeftToRightOverride => {
                    if let Some(level) = stack.embedding_level().least_greater_even() {
                        if overflow_isolate == 0 && overflow_embedding == 0 {
                            stack.push(level, Override::LTR, false);
                            continue;
                        }
                    }
                    if overflow_isolate == 0 {
                        overflow_embedding += 1;
                    }
                }
                // X5a
                BidiClass::RightToLeftIsolate => {
                    self.levels[idx] = stack.embedding_level();
                    stack.apply_override(&mut self.char_types[idx]);
                    if let Some(level) = stack.embedding_level().least_greater_odd() {
                        if overflow_isolate == 0 && overflow_embedding == 0 {
                            valid_isolate += 1;
                            stack.push(level, Override::Neutral, true);
                            continue;
                        }
                    }
                    overflow_isolate += 1;
                }
                // X5b
                BidiClass::LeftToRightIsolate => {
                    self.levels[idx] = stack.embedding_level();
                    stack.apply_override(&mut self.char_types[idx]);
                    if let Some(level) = stack.embedding_level().least_greater_even() {
                        if overflow_isolate == 0 && overflow_embedding == 0 {
                            valid_isolate += 1;
                            stack.push(level, Override::Neutral, true);
                            continue;
                        }
                    }
                    overflow_isolate += 1;
                }
                // X5c
                BidiClass::FirstStrongIsolate => {
                    let level =
                        paragraph_level(&self.char_types[idx + 1..], true, Direction::LeftToRight);
                    self.levels[idx] = stack.embedding_level();
                    stack.apply_override(&mut self.char_types[idx]);
                    let level = if level.0 == 1 {
                        stack.embedding_level().least_greater_odd()
                    } else {
                        stack.embedding_level().least_greater_even()
                    };
                    trace!(
                        "picked {:?} based on current stack level {:?}",
                        level,
                        stack.embedding_level()
                    );

                    if let Some(level) = level {
                        if overflow_isolate == 0 && overflow_embedding == 0 {
                            valid_isolate += 1;
                            stack.push(level, Override::Neutral, true);
                            continue;
                        }
                    }
                    overflow_isolate += 1;
                }
                // X6a
                BidiClass::PopDirectionalIsolate => {
                    if overflow_isolate > 0 {
                        overflow_isolate -= 1;
                    } else if valid_isolate == 0 {
                        // Do nothing
                    } else {
                        overflow_embedding = 0;
                        loop {
                            if stack.isolate_status() {
                                break;
                            }
                            stack.pop();
                        }
                        stack.pop();
                        valid_isolate -= 1;
                    }

                    self.levels[idx] = stack.embedding_level();
                    stack.apply_override(&mut self.char_types[idx]);
                }
                // X7
                BidiClass::PopDirectionalFormat => {
                    if overflow_isolate > 0 {
                        // Do nothing
                    } else if overflow_embedding > 0 {
                        overflow_embedding -= 1;
                    } else {
                        if !stack.isolate_status() {
                            if stack.depth() >= 2 {
                                stack.pop();
                            }
                        }
                    }
                }
                BidiClass::BoundaryNeutral => {}
                // X8
                BidiClass::ParagraphSeparator => {
                    // Terminates all embedding contexts.
                    // Should only ever be the last character in
                    // a paragraph if present at all.
                    self.levels[idx] = self.base_level;
                }
                // X6
                _ => {
                    self.levels[idx] = stack.embedding_level();
                    stack.apply_override(&mut self.char_types[idx]);
                }
            }
        }
    }
}
