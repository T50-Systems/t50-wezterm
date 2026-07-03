impl TerminfoRenderer {
    pub fn new(caps: Capabilities) -> Self {
        Self {
            caps,
            current_attr: CellAttributes::default(),
            pending_attr: None,
        }
    }

    fn get_capability<'a, T: TermInfoCapability<'a>>(&'a self) -> Option<T> {
        self.caps.terminfo_db().and_then(|db| db.get::<T>())
    }

    fn attr_apply<F: FnOnce(&mut CellAttributes)>(&mut self, func: F) {
        self.pending_attr = Some(match self.pending_attr.take() {
            Some(mut attr) => {
                func(&mut attr);
                attr
            }
            None => {
                let mut attr = self.current_attr.clone();
                func(&mut attr);
                attr
            }
        });
    }

    #[allow(clippy::cognitive_complexity)]
    fn flush_pending_attr<W: RenderTty + Write>(&mut self, out: &mut W) -> Result<()> {
        macro_rules! attr_on {
            ($cap:ident, $sgr:expr) => {{
                let cap = if self.caps.force_terminfo_render_to_use_ansi_sgr() {
                    None
                } else {
                    self.get_capability::<cap::$cap>()
                };
                if let Some(attr) = cap {
                    attr.expand().to(out.by_ref())?;
                } else {
                    write!(out, "{}", CSI::Sgr($sgr))?;
                }
            }};
            ($sgr:expr) => {
                write!(out, "{}", CSI::Sgr($sgr))?;
            };
        }

        if let Some(attr) = self.pending_attr.take() {
            let mut current_foreground = self.current_attr.foreground();
            let mut current_background = self.current_attr.background();

            if !attr.attribute_bits_equal(&self.current_attr) {
                // Updating the attribute bits also resets the colors.
                current_foreground = ColorAttribute::Default;
                current_background = ColorAttribute::Default;

                let sgr = if self.caps.force_terminfo_render_to_use_ansi_sgr() {
                    None
                } else {
                    self.get_capability::<cap::SetAttributes>()
                };
                // The SetAttributes capability can only handle single underline and slow blink.
                if let Some(sgr) = sgr {
                    sgr.expand()
                        .bold(attr.intensity() == Intensity::Bold)
                        .dim(attr.intensity() == Intensity::Half)
                        .underline(attr.underline() == Underline::Single)
                        .blink(attr.blink() == Blink::Slow)
                        .reverse(attr.reverse())
                        .invisible(attr.invisible())
                        .to(out.by_ref())?;
                } else {
                    attr_on!(ExitAttributeMode, Sgr::Reset);

                    match attr.intensity() {
                        Intensity::Bold => attr_on!(EnterBoldMode, Sgr::Intensity(Intensity::Bold)),
                        Intensity::Half => attr_on!(EnterDimMode, Sgr::Intensity(Intensity::Half)),
                        _ => {}
                    }

                    if attr.underline() == Underline::Single {
                        attr_on!(Sgr::Underline(Underline::Single));
                    }

                    if attr.blink() == Blink::Slow {
                        attr_on!(Sgr::Blink(Blink::Slow));
                    }

                    if attr.reverse() {
                        attr_on!(EnterReverseMode, Sgr::Inverse(true));
                    }

                    if attr.invisible() {
                        attr_on!(Sgr::Invisible(true));
                    }
                }

                if attr.underline() == Underline::Double {
                    attr_on!(Sgr::Underline(Underline::Double));
                }

                if attr.blink() == Blink::Rapid {
                    attr_on!(Sgr::Blink(Blink::Rapid));
                }

                if attr.italic() {
                    attr_on!(EnterItalicsMode, Sgr::Italic(true));
                }

                if attr.strikethrough() {
                    attr_on!(Sgr::StrikeThrough(true));
                }
            }

            let has_true_color = self.caps.color_level() == ColorLevel::TrueColor;
            // Whether to use terminfo to render 256 colors. If this is too large (ex. 16777216 from xterm-direct),
            // then setaf expects the index to be true color, in which case we cannot use it to render 256 (or even 16) colors.
            let terminfo_256_color: i32 = match self.get_capability::<cap::MaxColors>() {
                Some(cap::MaxColors(n)) => {
                    if n > 256 {
                        0
                    } else {
                        n
                    }
                }
                None => 0,
            };

            if attr.foreground() != current_foreground
                && self.caps.color_level() != ColorLevel::MonoChrome
            {
                match (has_true_color, attr.foreground()) {
                    (true, ColorAttribute::TrueColorWithPaletteFallback(tc, _))
                    | (true, ColorAttribute::TrueColorWithDefaultFallback(tc)) => {
                        write!(
                            out,
                            "{}",
                            CSI::Sgr(Sgr::Foreground(ColorSpec::TrueColor(tc)))
                        )?;
                    }
                    (false, ColorAttribute::TrueColorWithDefaultFallback(_))
                    | (_, ColorAttribute::Default) => {
                        // Terminfo doesn't define a reset color to default, so
                        // we use the ANSI code.
                        write!(out, "{}", CSI::Sgr(Sgr::Foreground(ColorSpec::Default)))?;
                    }
                    (false, ColorAttribute::TrueColorWithPaletteFallback(_, idx))
                    | (_, ColorAttribute::PaletteIndex(idx)) => {
                        match self.get_capability::<cap::SetAForeground>() {
                            Some(set) if (idx as i32) < terminfo_256_color => {
                                set.expand().color(idx).to(out.by_ref())?;
                            }
                            _ => {
                                write!(
                                    out,
                                    "{}",
                                    CSI::Sgr(Sgr::Foreground(ColorSpec::PaletteIndex(idx)))
                                )?;
                            }
                        }
                    }
                }
            }

            if attr.background() != current_background
                && self.caps.color_level() != ColorLevel::MonoChrome
            {
                match (has_true_color, attr.background()) {
                    (true, ColorAttribute::TrueColorWithPaletteFallback(tc, _))
                    | (true, ColorAttribute::TrueColorWithDefaultFallback(tc)) => {
                        write!(
                            out,
                            "{}",
                            CSI::Sgr(Sgr::Background(ColorSpec::TrueColor(tc)))
                        )?;
                    }
                    (false, ColorAttribute::TrueColorWithDefaultFallback(_))
                    | (_, ColorAttribute::Default) => {
                        // Terminfo doesn't define a reset color to default, so
                        // we use the ANSI code.
                        write!(out, "{}", CSI::Sgr(Sgr::Background(ColorSpec::Default)))?;
                    }
                    (false, ColorAttribute::TrueColorWithPaletteFallback(_, idx))
                    | (_, ColorAttribute::PaletteIndex(idx)) => {
                        match self.get_capability::<cap::SetABackground>() {
                            Some(set) if (idx as i32) < terminfo_256_color => {
                                set.expand().color(idx).to(out.by_ref())?;
                            }
                            _ => {
                                write!(
                                    out,
                                    "{}",
                                    CSI::Sgr(Sgr::Background(ColorSpec::PaletteIndex(idx)))
                                )?;
                            }
                        }
                    }
                }
            }

            if self.caps.hyperlinks() {
                if let Some(link) = attr.hyperlink() {
                    let osc = OperatingSystemCommand::SetHyperlink(Some((**link).clone()));
                    write!(out, "{}", osc)?;
                } else if self.current_attr.hyperlink().is_some() {
                    // Close out the old hyperlink
                    let osc = OperatingSystemCommand::SetHyperlink(None);
                    write!(out, "{}", osc)?;
                }
            }

            self.current_attr = attr;
        }

        Ok(())
    }
}
