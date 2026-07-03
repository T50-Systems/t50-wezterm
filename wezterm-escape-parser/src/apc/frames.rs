#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KittyFrameCompositionMode {
    AlphaBlending,
    Overwrite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KittyImageFrameCompose {
    /// i=...
    pub image_id: Option<u32>,
    /// I=...
    pub image_number: Option<u32>,

    /// 1-based number of the frame which should be the base
    /// data for the new frame being created.
    /// If omitted, use background_pixel to specify color.
    /// c=...
    pub target_frame: Option<u32>,

    /// 1-based number of the frame which should be edited.
    /// If omitted, a new frame is created.
    /// r=...
    pub source_frame: Option<u32>,

    /// Left edge in pixels to update
    /// x=...
    pub x: Option<u32>,
    /// Top edge in pixels to update
    /// y=...
    pub y: Option<u32>,

    /// Width (in pixels) of the source and destination rectangles.
    /// By default the full width is used.
    /// w=...
    pub w: Option<u32>,

    /// Height (in pixels) of the source and destination rectangles.
    /// By default the full height is used.
    /// h=...
    pub h: Option<u32>,

    /// Left edge in pixels of the source rectangle
    /// X=...
    pub src_x: Option<u32>,
    /// Top edge in pixels of the source rectangle
    /// Y=...
    pub src_y: Option<u32>,

    /// Composition mode.
    /// Default is AlphaBlending
    /// C=...
    pub composition_mode: KittyFrameCompositionMode,
}

impl KittyImageFrameCompose {
    fn from_keys(keys: &BTreeMap<&str, &str>) -> Option<Self> {
        Some(Self {
            image_id: geti(keys, "i"),
            image_number: geti(keys, "I"),
            x: geti(keys, "x"),
            y: geti(keys, "y"),
            src_x: geti(keys, "X"),
            src_y: geti(keys, "Y"),
            w: geti(keys, "w"),
            h: geti(keys, "h"),
            target_frame: match geti(keys, "c") {
                None | Some(0) => None,
                n => n,
            },
            source_frame: match geti(keys, "r") {
                None | Some(0) => None,
                n => n,
            },
            composition_mode: match geti(keys, "C") {
                None | Some(0) => KittyFrameCompositionMode::AlphaBlending,
                Some(1) => KittyFrameCompositionMode::Overwrite,
                _ => return None,
            },
        })
    }

    fn to_keys(&self, keys: &mut BTreeMap<&'static str, String>) {
        set(keys, "i", &self.image_id);
        set(keys, "I", &self.image_number);
        set(keys, "w", &self.w);
        set(keys, "h", &self.h);
        set(keys, "x", &self.x);
        set(keys, "y", &self.y);
        set(keys, "X", &self.src_x);
        set(keys, "Y", &self.src_y);
        set(keys, "c", &self.target_frame);
        set(keys, "r", &self.source_frame);
        match &self.composition_mode {
            KittyFrameCompositionMode::AlphaBlending => {}
            KittyFrameCompositionMode::Overwrite => {
                keys.insert("C", "1".to_string());
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KittyImageFrame {
    /// Left edge in pixels to update
    pub x: Option<u32>,
    /// Top edge in pixels to update
    pub y: Option<u32>,

    /// 1-based number of the frame which should be the base
    /// data for the new frame being created.
    /// If omitted, use background_pixel to specify color.
    /// c=...
    pub base_frame: Option<u32>,

    /// 1-based number of the frame which should be edited.
    /// If omitted, a new frame is created.
    /// r=...
    pub frame_number: Option<u32>,

    /// Gap in milliseconds of this frame from the next one.
    /// Zero or omitted values are interpreted as 40ms.
    /// z=...
    pub duration_ms: Option<u32>,

    /// Composition mode.
    /// Default is AlphaBlending
    /// X=...
    pub composition_mode: KittyFrameCompositionMode,

    /// Background color for pixels not specified in the frame data.
    /// If omitted, use a black, fully-transparent pixel (0)
    /// Y=...
    pub background_pixel: Option<u32>,
}

impl KittyImageFrame {
    fn from_keys(keys: &BTreeMap<&str, &str>) -> Option<Self> {
        Some(Self {
            x: geti(keys, "x"),
            y: geti(keys, "y"),
            base_frame: match geti(keys, "c") {
                None | Some(0) => None,
                n => n,
            },
            frame_number: match geti(keys, "r") {
                None | Some(0) => None,
                n => n,
            },
            duration_ms: match geti(keys, "Z") {
                None | Some(0) => None,
                n => n,
            },
            composition_mode: match geti(keys, "X") {
                None | Some(0) => KittyFrameCompositionMode::AlphaBlending,
                Some(1) => KittyFrameCompositionMode::Overwrite,
                _ => return None,
            },
            background_pixel: geti(keys, "Y"),
        })
    }

    fn to_keys(&self, keys: &mut BTreeMap<&'static str, String>) {
        set(keys, "x", &self.x);
        set(keys, "y", &self.y);
        set(keys, "c", &self.base_frame);
        set(keys, "r", &self.frame_number);
        set(keys, "Z", &self.duration_ms);
        match &self.composition_mode {
            KittyFrameCompositionMode::AlphaBlending => {}
            KittyFrameCompositionMode::Overwrite => {
                keys.insert("X", "1".to_string());
            }
        }
        set(keys, "Y", &self.background_pixel);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KittyImage {
    /// a='t'
    TransmitData {
        transmit: KittyImageTransmit,
        verbosity: KittyImageVerbosity,
    },
    /// a='T'
    TransmitDataAndDisplay {
        transmit: KittyImageTransmit,
        placement: KittyImagePlacement,
        verbosity: KittyImageVerbosity,
    },
    /// a='p'
    Display {
        image_id: Option<u32>,
        image_number: Option<u32>,
        placement: KittyImagePlacement,
        verbosity: KittyImageVerbosity,
    },
    /// a='d'
    Delete {
        what: KittyImageDelete,
        verbosity: KittyImageVerbosity,
    },
    /// a='q'
    Query { transmit: KittyImageTransmit },
    /// a='f'
    TransmitFrame {
        transmit: KittyImageTransmit,
        frame: KittyImageFrame,
        verbosity: KittyImageVerbosity,
    },
    /// a='c'
    ComposeFrame {
        frame: KittyImageFrameCompose,
        verbosity: KittyImageVerbosity,
    },
}

impl KittyImage {
    pub fn verbosity(&self) -> KittyImageVerbosity {
        match self {
            Self::TransmitData { verbosity, .. } => *verbosity,
            Self::Query { .. } => KittyImageVerbosity::Verbose,
            Self::TransmitDataAndDisplay { verbosity, .. } => *verbosity,
            Self::Display { verbosity, .. } => *verbosity,
            Self::Delete { verbosity, .. } => *verbosity,
            Self::TransmitFrame { verbosity, .. } => *verbosity,
            Self::ComposeFrame { verbosity, .. } => *verbosity,
        }
    }

    pub fn parse_apc(data: &[u8]) -> Option<Self> {
        if data.is_empty() || data[0] != b'G' {
            return None;
        }
        let mut keys_payload_iter = data[1..].splitn(2, |&d| d == b';');
        let keys = keys_payload_iter.next()?;
        let key_string = core::str::from_utf8(keys).ok()?;
        let mut keys: BTreeMap<&str, &str> = BTreeMap::new();
        for k_v in key_string.split(',') {
            let mut k_v = k_v.splitn(2, '=');
            let k = k_v.next()?;
            let v = k_v.next()?;
            keys.insert(k, v);
        }

        let payload = keys_payload_iter.next().unwrap_or(b"");
        let action = get(&keys, "a").unwrap_or("t");
        let verbosity = KittyImageVerbosity::from_keys(&keys)?;
        match action {
            "t" => Some(Self::TransmitData {
                transmit: KittyImageTransmit::from_keys(&keys, payload)?,
                verbosity,
            }),
            "q" => Some(Self::Query {
                transmit: KittyImageTransmit::from_keys(&keys, payload)?,
            }),
            "T" => Some(Self::TransmitDataAndDisplay {
                transmit: KittyImageTransmit::from_keys(&keys, payload)?,
                placement: KittyImagePlacement::from_keys(&keys)?,
                verbosity,
            }),
            "p" => Some(Self::Display {
                placement: KittyImagePlacement::from_keys(&keys)?,
                image_id: geti(&keys, "i"),
                image_number: geti(&keys, "I"),
                verbosity,
            }),
            "d" => Some(Self::Delete {
                what: KittyImageDelete::from_keys(&keys)?,
                verbosity,
            }),
            "f" => Some(Self::TransmitFrame {
                transmit: KittyImageTransmit::from_keys(&keys, payload)?,
                frame: KittyImageFrame::from_keys(&keys)?,
                verbosity,
            }),
            "c" => Some(Self::ComposeFrame {
                frame: KittyImageFrameCompose::from_keys(&keys)?,
                verbosity,
            }),
            _ => None,
        }
    }

    fn to_keys(&self, keys: &mut BTreeMap<&'static str, String>) {
        match self {
            Self::TransmitData {
                transmit,
                verbosity,
            } => {
                // Implied: keys.insert("a", "t".to_string());
                verbosity.to_keys(keys);
                transmit.to_keys(keys);
            }
            Self::Query { transmit } => {
                keys.insert("a", "q".to_string());
                transmit.to_keys(keys);
            }
            Self::TransmitDataAndDisplay {
                transmit,
                verbosity,
                placement,
            } => {
                keys.insert("a", "Q".to_string());
                verbosity.to_keys(keys);
                placement.to_keys(keys);
                transmit.to_keys(keys);
            }
            Self::Display {
                image_id,
                image_number,
                placement,
                verbosity,
            } => {
                keys.insert("a", "p".to_string());
                verbosity.to_keys(keys);
                placement.to_keys(keys);
                if let Some(image_id) = image_id {
                    keys.insert("i", image_id.to_string());
                }
                if let Some(image_number) = image_number {
                    keys.insert("I", image_number.to_string());
                }
            }
            Self::Delete { what, verbosity } => {
                keys.insert("a", "d".to_string());
                verbosity.to_keys(keys);
                what.to_keys(keys);
            }
            Self::TransmitFrame {
                transmit,
                verbosity,
                frame,
            } => {
                keys.insert("a", "f".to_string());
                transmit.to_keys(keys);
                frame.to_keys(keys);
                verbosity.to_keys(keys);
            }
            Self::ComposeFrame { frame, verbosity } => {
                keys.insert("a", "c".to_string());
                frame.to_keys(keys);
                verbosity.to_keys(keys);
            }
        }
    }
}

impl Display for KittyImage {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "\x1b_G")?;
        let mut keys = BTreeMap::new();
        self.to_keys(&mut keys);
        let mut payload = None;
        let mut first = true;
        for (k, v) in keys {
            if k == "payload" {
                payload = Some(v);
            } else {
                if first {
                    first = false;
                } else {
                    write!(f, ",")?;
                }

                write!(f, "{}={}", k, v)?;
            }
        }

        if let Some(p) = payload {
            write!(f, ";{}", p)?;
        }

        Ok(())
    }
}
