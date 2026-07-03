impl TerminfoRenderer {
    #[allow(clippy::cyclomatic_complexity, clippy::cognitive_complexity)]
    pub fn render_to<W: RenderTty + Write>(
        &mut self,
        changes: &[Change],
        out: &mut W,
    ) -> Result<()> {
        macro_rules! record {
            ($accesor:ident, $value:expr) => {
                self.attr_apply(|attr| {
                    attr.$accesor(*$value);
                });
            };
        }

        for change in changes {
            match change {
                Change::ClearScreen(color) => {
                    // ClearScreen implicitly resets all to default
                    let defaults = CellAttributes::default().set_background(*color).clone();
                    if self.current_attr != defaults {
                        self.pending_attr = Some(defaults);
                        self.flush_pending_attr(out)?;
                    }
                    self.pending_attr = None;

                    if self.current_attr.background() == ColorAttribute::Default || self.caps.bce()
                    {
                        // The erase operation respects "background color erase",
                        // or we're clearing to the default background color, so we can
                        // simply emit a clear screen op.
                        if let Some(clr) = self.get_capability::<cap::ClearScreen>() {
                            clr.expand().to(out.by_ref())?;
                        } else {
                            self.move_cursor_absolute(0, 0, out)?;
                            write!(
                                out,
                                "{}",
                                CSI::Edit(Edit::EraseInDisplay(EraseInDisplay::EraseDisplay))
                            )?;
                        }
                    } else {
                        // We're setting the background to a specific color, so we get to
                        // paint the whole thing.
                        self.move_cursor_absolute(0, 0, out)?;

                        let (cols, rows) = out.get_size_in_cells()?;
                        let num_spaces = cols * rows;
                        let mut buf = Vec::with_capacity(num_spaces);
                        buf.resize(num_spaces, b' ');
                        out.write_all(buf.as_slice())?;
                    }
                }
                Change::ClearToEndOfLine(color) => {
                    // ClearScreen implicitly resets all to default
                    let defaults = CellAttributes::default().set_background(*color).clone();
                    if self.current_attr != defaults {
                        self.pending_attr = Some(defaults);
                        self.flush_pending_attr(out)?;
                    }
                    self.pending_attr = None;

                    // FIXME: this doesn't behave correctly for terminals without bce.
                    // If we knew the current cursor position, we would be able to
                    // emit the correctly colored background for that case.
                    if let Some(clr) = self.get_capability::<cap::ClrEol>() {
                        clr.expand().to(out.by_ref())?;
                    } else {
                        write!(
                            out,
                            "{}",
                            CSI::Edit(Edit::EraseInLine(EraseInLine::EraseToEndOfLine))
                        )?;
                    }
                }
                Change::ClearToEndOfScreen(color) => {
                    // ClearScreen implicitly resets all to default
                    let defaults = CellAttributes::default().set_background(*color).clone();
                    if self.current_attr != defaults {
                        self.pending_attr = Some(defaults);
                        self.flush_pending_attr(out)?;
                    }
                    self.pending_attr = None;

                    // FIXME: this doesn't behave correctly for terminals without bce.
                    // If we knew the current cursor position, we would be able to
                    // emit the correctly colored background for that case.
                    if let Some(clr) = self.get_capability::<cap::ClrEos>() {
                        clr.expand().to(out.by_ref())?;
                    } else {
                        write!(
                            out,
                            "{}",
                            CSI::Edit(Edit::EraseInDisplay(EraseInDisplay::EraseToEndOfDisplay))
                        )?;
                    }
                }
                Change::Attribute(AttributeChange::Intensity(value)) => {
                    record!(set_intensity, value);
                }
                Change::Attribute(AttributeChange::Italic(value)) => {
                    record!(set_italic, value);
                }
                Change::Attribute(AttributeChange::Reverse(value)) => {
                    record!(set_reverse, value);
                }
                Change::Attribute(AttributeChange::StrikeThrough(value)) => {
                    record!(set_strikethrough, value);
                }
                Change::Attribute(AttributeChange::Blink(value)) => {
                    record!(set_blink, value);
                }
                Change::Attribute(AttributeChange::Invisible(value)) => {
                    record!(set_invisible, value);
                }
                Change::Attribute(AttributeChange::Underline(value)) => {
                    record!(set_underline, value);
                }
                Change::Attribute(AttributeChange::Foreground(col)) => {
                    self.attr_apply(|attr| {
                        attr.set_foreground(*col);
                    });
                }
                Change::Attribute(AttributeChange::Background(col)) => {
                    self.attr_apply(|attr| {
                        attr.set_background(*col);
                    });
                }
                Change::Attribute(AttributeChange::Hyperlink(link)) => {
                    self.attr_apply(|attr| {
                        attr.set_hyperlink(link.clone());
                    });
                }
                Change::AllAttributes(all) => {
                    self.pending_attr = Some(all.clone());
                }
                Change::Text(text) => {
                    self.flush_pending_attr(out)?;
                    out.by_ref().write_all(text.as_bytes())?;
                }

                Change::CursorPosition { x, y } => {
                    // Note: we use `cursor_up(screen_height)` to move the cursor all the way to
                    // the top of the screen when we need to absolutely position only y.

                    match (x, y) {
                        (Position::Relative(0), Position::Relative(0)) => {}
                        (Position::Absolute(0), Position::Relative(1)) => {
                            out.by_ref().write_all(b"\r\n")?;
                        }
                        (Position::Absolute(x), Position::Relative(y)) => {
                            self.cursor_y_relative(*y, out)?;
                            out.by_ref().write_all(b"\r")?;
                            self.cursor_right(*x as u32, out)?;
                        }

                        (Position::Relative(x), Position::EndRelative(y)) => {
                            let (_cols, rows) = out.get_size_in_cells()?;
                            self.cursor_up(rows as u32, out)?;
                            self.cursor_down(rows.saturating_sub(y + 1) as u32, out)?;
                            self.cursor_x_relative(*x, out)?;
                        }
                        (Position::Relative(x), Position::Relative(y)) => {
                            self.cursor_y_relative(*y, out)?;
                            self.cursor_x_relative(*x, out)?;
                        }
                        (Position::EndRelative(x), Position::Relative(y)) => {
                            self.cursor_y_relative(*y, out)?;
                            let (cols, _rows) = out.get_size_in_cells()?;
                            out.by_ref().write_all(b"\r")?;
                            self.cursor_right(cols.saturating_sub(x + 1) as u32, out)?;
                        }

                        (Position::Absolute(x), Position::Absolute(y)) => {
                            self.move_cursor_absolute(*x as u32, *y as u32, out)?;
                        }
                        (Position::Absolute(x), Position::EndRelative(y)) => {
                            let (_cols, rows) = out.get_size_in_cells()?;
                            self.move_cursor_absolute(
                                *x as u32,
                                rows.saturating_sub(y + 1) as u32,
                                out,
                            )?;
                        }
                        (Position::EndRelative(x), Position::EndRelative(y)) => {
                            let (cols, rows) = out.get_size_in_cells()?;
                            self.move_cursor_absolute(
                                cols.saturating_sub(x + 1) as u32,
                                rows.saturating_sub(y + 1) as u32,
                                out,
                            )?;
                        }

                        (Position::Relative(x), Position::Absolute(y)) => {
                            let (_cols, rows) = out.get_size_in_cells()?;
                            self.cursor_up(rows as u32, out)?;
                            self.cursor_down(*y as u32, out)?;
                            self.cursor_x_relative(*x, out)?;
                        }

                        (Position::EndRelative(x), Position::Absolute(y)) => {
                            let (cols, _rows) = out.get_size_in_cells()?;
                            self.move_cursor_absolute(
                                cols.saturating_sub(x + 1) as u32,
                                *y as u32,
                                out,
                            )?;
                        }
                    }
                }

                Change::CursorColor(_color) => {
                    // TODO: this isn't spec'd by terminfo, but some terminals
                    // support it.  Add this to capabilities?
                }
                Change::CursorShape(shape) => match shape {
                    CursorShape::Default => {
                        if let Some(normal) = self.get_capability::<cap::CursorNormal>() {
                            normal.expand().to(out.by_ref())?;
                        }
                        if let Some(reset) = self.get_capability::<cap::ResetCursorStyle>() {
                            reset.expand().to(out.by_ref())?;
                        }
                    }
                    _ => {
                        let param = match shape {
                            CursorShape::Default => unreachable!(),
                            CursorShape::BlinkingBlock => 1,
                            CursorShape::SteadyBlock => 2,
                            CursorShape::BlinkingUnderline => 3,
                            CursorShape::SteadyUnderline => 4,
                            CursorShape::BlinkingBar => 5,
                            CursorShape::SteadyBar => 6,
                        };
                        if let Some(set) = self.get_capability::<cap::SetCursorStyle>() {
                            set.expand().kind(param).to(out.by_ref())?;
                        }
                    }
                },
                Change::CursorVisibility(visibility) => match visibility {
                    CursorVisibility::Visible => {
                        if let Some(show) = self.get_capability::<cap::CursorNormal>() {
                            show.expand().to(out.by_ref())?;
                        }
                    }
                    CursorVisibility::Hidden => {
                        if let Some(hide) = self.get_capability::<cap::CursorInvisible>() {
                            hide.expand().to(out.by_ref())?;
                        }
                    }
                },
                #[cfg(feature = "use_image")]
                Change::Image(image) => {
                    if self.caps.iterm2_image() {
                        let data = if image.top_left == TextureCoordinate::new_f32(0.0, 0.0)
                            && image.bottom_right == TextureCoordinate::new_f32(1.0, 1.0)
                        {
                            // The whole image is requested, so we can send the
                            // original image bytes over
                            match &*image.image.data() {
                                ImageDataType::EncodedFile(data) => data.to_vec(),
                                ImageDataType::EncodedLease(lease) => lease.get_data()?,
                                ImageDataType::AnimRgba8 { .. } | ImageDataType::Rgba8 { .. } => {
                                    unimplemented!()
                                }
                            }
                        } else {
                            // TODO: slice out the requested region of the image,
                            // and encode as a PNG.
                            unimplemented!();
                        };

                        let file = ITermFileData {
                            name: None,
                            size: Some(data.len()),
                            width: ITermDimension::Cells(image.width as i64),
                            height: ITermDimension::Cells(image.height as i64),
                            preserve_aspect_ratio: true,
                            inline: true,
                            do_not_move_cursor: false,
                            data,
                        };

                        let osc = OperatingSystemCommand::ITermProprietary(ITermProprietary::File(
                            Box::new(file),
                        ));

                        write!(out, "{}", osc)?;

                    // TODO: } else if self.caps.sixel() {
                    } else {
                        // Blank out the cells and move the cursor to the right spot
                        for y in 0..image.height {
                            for _ in 0..image.width {
                                write!(out, " ")?;
                            }

                            if y != image.height - 1 {
                                writeln!(out)?;
                                self.cursor_left(image.width as u32, out)?;
                            }
                        }
                        self.cursor_up(image.height as u32, out)?;
                    }
                }
                Change::ScrollRegionUp {
                    first_row,
                    region_size,
                    scroll_count,
                } => {
                    if *region_size > 0 {
                        if let Some(csr) = self.get_capability::<cap::ChangeScrollRegion>() {
                            let top = *first_row as u32;
                            let bottom = (*first_row + *region_size - 1) as u32;
                            let scroll_count = *scroll_count as u32;
                            csr.expand().top(top).bottom(bottom).to(out.by_ref())?;
                            if scroll_count > 0 {
                                if let Some(scroll) = self.get_capability::<cap::ParmIndex>() {
                                    scroll.expand().count(scroll_count).to(out.by_ref())?
                                } else {
                                    let scroll = self.get_capability::<cap::ScrollForward>();
                                    let set_position = self.get_capability::<cap::CursorAddress>();
                                    if let (Some(scroll), Some(set_position)) =
                                        (scroll, set_position)
                                    {
                                        set_position.expand().x(0).y(bottom).to(out.by_ref())?;
                                        for _ in 0..scroll_count {
                                            scroll.expand().to(out.by_ref())?
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Change::ScrollRegionDown {
                    first_row,
                    region_size,
                    scroll_count,
                } => {
                    if *region_size > 0 {
                        if let Some(csr) = self.get_capability::<cap::ChangeScrollRegion>() {
                            let top = *first_row as u32;
                            let bottom = (*first_row + *region_size - 1) as u32;
                            let scroll_count = *scroll_count as u32;
                            csr.expand().top(top).bottom(bottom).to(out.by_ref())?;
                            if scroll_count > 0 {
                                if let Some(scroll) = self.get_capability::<cap::ParmRindex>() {
                                    scroll.expand().count(scroll_count).to(out.by_ref())?
                                } else {
                                    let scroll = self.get_capability::<cap::ScrollReverse>();
                                    let set_position = self.get_capability::<cap::CursorAddress>();
                                    if let (Some(scroll), Some(set_position)) =
                                        (scroll, set_position)
                                    {
                                        set_position.expand().x(0).y(top).to(out.by_ref())?;
                                        for _ in 0..scroll_count {
                                            scroll.expand().to(out.by_ref())?
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Change::Title(text) => {
                    let osc = OperatingSystemCommand::SetWindowTitle(text.to_string());
                    write!(out, "{}", osc)?;
                }

                Change::LineAttribute(attr) => {
                    let esc = Esc::Code(match attr {
                        LineAttribute::DoubleHeightTopHalfLine => {
                            EscCode::DecDoubleHeightTopHalfLine
                        }
                        LineAttribute::DoubleHeightBottomHalfLine => {
                            EscCode::DecDoubleHeightBottomHalfLine
                        }
                        LineAttribute::DoubleWidthLine => EscCode::DecDoubleWidthLine,
                        LineAttribute::SingleWidthLine => EscCode::DecSingleWidthLine,
                    });
                    write!(out, "{esc}")?;
                }
            }
        }

        self.flush_pending_attr(out)?;
        out.flush()?;
        Ok(())
    }
}
