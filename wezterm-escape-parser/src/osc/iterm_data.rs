#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    None,
    SetPercentage(u8),
    SetError(u8),
    SetIndeterminate,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ITermProprietary {
    /// The "Set Mark" command allows you to record a location and then jump back to it later
    SetMark,
    /// To bring iTerm2 to the foreground
    StealFocus,
    /// To erase the scrollback history
    ClearScrollback,
    /// To inform iTerm2 of the current directory to help semantic history
    CurrentDir(String),
    /// To change the session's profile on the fly
    SetProfile(String),
    /// Currently defined values for the string parameter are "rule", "find", "font"
    /// or an empty string.  iTerm2 will go into paste mode until EndCopy is received.
    CopyToClipboard(String),
    /// Ends CopyToClipboard mode in iTerm2.
    EndCopy,
    /// The boolean should be yes or no. This shows or hides the cursor guide
    HighlightCursorLine(bool),
    /// Request that the terminal send a ReportCellSize response
    RequestCellSize,
    /// The response to RequestCellSize.  The height and width are the dimensions
    /// of a cell measured in points according to the docs, but in practice, they
    /// are actually pixels.
    /// If scale is_some(), the width and height will be multiplied by scale to
    /// get the true device dimensions
    ReportCellSize {
        height_pixels: NotNan<f32>,
        width_pixels: NotNan<f32>,
        scale: Option<NotNan<f32>>,
    },
    /// Place a string in the systems pasteboard
    Copy(String),
    /// Each iTerm2 session has internal variables (as described in
    /// <https://www.iterm2.com/documentation-badges.html>). This escape sequence reports
    /// a variable's value.  The response is another ReportVariable.
    ReportVariable(String),
    /// User-defined variables may be set with the following escape sequence
    SetUserVar {
        name: String,
        value: String,
    },
    SetBadgeFormat(String),
    /// Download file data from the application.
    File(Box<ITermFileData>),

    /// Configure unicode version
    UnicodeVersion(ITermUnicodeVersionOp),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ITermUnicodeVersionOp {
    Set(u8),
    Push(Option<String>),
    Pop(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ITermFileData {
    /// file name
    pub name: Option<String>,
    /// size of the data in bytes; this is used by iterm to show progress
    /// while waiting for the rest of the payload
    pub size: Option<usize>,
    /// width to render
    pub width: ITermDimension,
    /// height to render
    pub height: ITermDimension,
    /// if true, preserve aspect ratio when fitting to width/height
    pub preserve_aspect_ratio: bool,
    /// if true, attempt to display in the terminal rather than downloading to
    /// the users download directory
    pub inline: bool,
    /// if true, do not move the cursor
    pub do_not_move_cursor: bool,
    /// The data to transfer
    pub data: Vec<u8>,
}

impl ITermFileData {
    fn parse(osc: &[&[u8]]) -> Result<Self> {
        let mut params = HashMap::new();

        // Unfortunately, the encoding for the file download data is
        // awkward to fit in the conventional OSC data that our parser
        // expects at a higher level.
        // We have a mix of '=', ';' and ':' separated keys and values,
        // and a number of them are optional.
        // ESC ] 1337 ; File = [optional arguments] : base-64 encoded file contents ^G

        let mut data = None;

        let last = osc.len() - 1;
        for (idx, s) in osc.iter().enumerate().skip(1) {
            let param = if idx == 1 {
                if s.len() >= 5 {
                    // skip over File=
                    &s[5..]
                } else {
                    bail!("failed to parse file data; File= not found");
                }
            } else {
                s
            };

            let param = if idx == last {
                // The final argument contains `:base64`, so look for that
                if let Some(colon) = param.iter().position(|c| *c == b':') {
                    data = Some(base64_decode(&param[colon + 1..])?);
                    &param[..colon]
                } else {
                    // If we don't find the colon in the last piece, we've
                    // got nothing useful
                    bail!("failed to parse file data; no colon found");
                }
            } else {
                param
            };

            // eg: `File=;size=1234` case. <https://github.com/wezterm/wezterm/issues/1291>
            if param.is_empty() {
                continue;
            }

            // look for k=v in param
            if let Some(equal) = param.iter().position(|c| *c == b'=') {
                let key = &param[..equal];
                let value = &param[equal + 1..];
                params.insert(str::from_utf8(key)?, str::from_utf8(value)?);
            } else if idx != last {
                bail!("failed to parse file data; no equals found");
            }
        }

        let name = params
            .get("name")
            .and_then(|s| base64_decode(s).ok())
            .and_then(|b| String::from_utf8(b).ok());
        let size = params.get("size").and_then(|s| s.parse().ok());
        let width = params
            .get("width")
            .and_then(|s| ITermDimension::parse(s).ok())
            .unwrap_or(ITermDimension::Automatic);
        let height = params
            .get("height")
            .and_then(|s| ITermDimension::parse(s).ok())
            .unwrap_or(ITermDimension::Automatic);
        let preserve_aspect_ratio = params
            .get("preserveAspectRatio")
            .map(|s| *s != "0")
            .unwrap_or(true);
        let inline = params.get("inline").map(|s| *s != "0").unwrap_or(false);
        let do_not_move_cursor = params
            .get("doNotMoveCursor")
            .map(|s| *s != "0")
            .unwrap_or(false);
        let data = data.ok_or_else(|| format!("didn't set data"))?;
        Ok(Self {
            name,
            size,
            width,
            height,
            preserve_aspect_ratio,
            inline,
            do_not_move_cursor,
            data,
        })
    }
}

impl Display for ITermFileData {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "File")?;
        let mut sep = "=";
        let emit_sep = |sep, f: &mut Formatter| -> core::result::Result<&str, FmtError> {
            write!(f, "{}", sep)?;
            Ok(";")
        };
        if let Some(size) = self.size {
            sep = emit_sep(sep, f)?;
            write!(f, "size={}", size)?;
        }
        if let Some(ref name) = self.name {
            sep = emit_sep(sep, f)?;
            write!(f, "name={}", base64_encode(name))?;
        }
        if self.width != ITermDimension::Automatic {
            sep = emit_sep(sep, f)?;
            write!(f, "width={}", self.width)?;
        }
        if self.height != ITermDimension::Automatic {
            sep = emit_sep(sep, f)?;
            write!(f, "height={}", self.height)?;
        }
        if !self.preserve_aspect_ratio {
            sep = emit_sep(sep, f)?;
            write!(f, "preserveAspectRatio=0")?;
        }
        if self.inline {
            sep = emit_sep(sep, f)?;
            write!(f, "inline=1")?;
        }
        if self.do_not_move_cursor {
            sep = emit_sep(sep, f)?;
            write!(f, "doNotMoveCursor=1")?;
        }
        // Ensure that we emit a sep if we didn't already.
        // It will still be set to '=' in that case.
        if sep == "=" {
            write!(f, "=")?;
        }
        write!(f, ":{}", base64_encode(&self.data))?;
        Ok(())
    }
}
