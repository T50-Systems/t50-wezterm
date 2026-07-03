impl TerminfoRenderer {
    fn cursor_up<W: RenderTty + Write>(&mut self, n: u32, out: &mut W) -> Result<()> {
        if n > 0 {
            if let Some(attr) = self.get_capability::<cap::ParmUpCursor>() {
                attr.expand().count(n).to(out.by_ref())?;
            } else {
                write!(out, "{}", CSI::Cursor(Cursor::Up(n)))?;
            }
        }
        Ok(())
    }

    fn cursor_down<W: RenderTty + Write>(&mut self, n: u32, out: &mut W) -> Result<()> {
        if n > 0 {
            if let Some(attr) = self.get_capability::<cap::ParmDownCursor>() {
                attr.expand().count(n).to(out.by_ref())?;
            } else {
                write!(out, "{}", CSI::Cursor(Cursor::Down(n)))?;
            }
        }
        Ok(())
    }

    fn cursor_y_relative<W: RenderTty + Write>(&mut self, y: isize, out: &mut W) -> Result<()> {
        if y > 0 {
            self.cursor_down(y as u32, out)
        } else {
            self.cursor_up(-y as u32, out)
        }
    }

    fn cursor_left<W: RenderTty + Write>(&mut self, n: u32, out: &mut W) -> Result<()> {
        if n > 0 {
            if let Some(attr) = self.get_capability::<cap::ParmLeftCursor>() {
                attr.expand().count(n).to(out.by_ref())?;
            } else {
                write!(out, "{}", CSI::Cursor(Cursor::Left(n)))?;
            }
        }
        Ok(())
    }

    fn cursor_right<W: RenderTty + Write>(&mut self, n: u32, out: &mut W) -> Result<()> {
        if n > 0 {
            if let Some(attr) = self.get_capability::<cap::ParmRightCursor>() {
                attr.expand().count(n).to(out.by_ref())?;
            } else {
                write!(out, "{}", CSI::Cursor(Cursor::Right(n)))?;
            }
        }
        Ok(())
    }

    fn cursor_x_relative<W: RenderTty + Write>(&mut self, x: isize, out: &mut W) -> Result<()> {
        if x > 0 {
            self.cursor_right(x as u32, out)
        } else {
            self.cursor_left(-x as u32, out)
        }
    }

    fn move_cursor_absolute<W: RenderTty + Write>(
        &mut self,
        x: u32,
        y: u32,
        out: &mut W,
    ) -> Result<()> {
        if x == 0 && y == 0 {
            if let Some(attr) = self.get_capability::<cap::CursorHome>() {
                attr.expand().to(out.by_ref())?;
                return Ok(());
            }
        }

        if let Some(attr) = self.get_capability::<cap::CursorAddress>() {
            // terminfo expansion automatically converts coordinates to 1-based,
            // so we can pass in the 0-based coordinates as-is
            attr.expand().x(x).y(y).to(out.by_ref())?;
        } else {
            // We need to manually convert to 1-based as the CSI representation
            // requires it and there's no automatic conversion.
            write!(
                out,
                "{}",
                CSI::Cursor(Cursor::Position {
                    line: OneBased::from_zero_based(x),
                    col: OneBased::from_zero_based(y),
                })
            )?;
        }
        Ok(())
    }
}
