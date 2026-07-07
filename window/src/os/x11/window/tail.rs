fn parse_texturi_list(url_list: &[u8]) -> Vec<PathBuf> {
    String::from_utf8_lossy(url_list)
        .lines()
        .filter_map(|line| {
            if line.starts_with('#') || line.trim().is_empty() {
                // text/uri-list: Any lines beginning with the '#' character
                // are comment lines and are ignored during processing
                return None;
            }
            let url = Url::parse(line)
                .map_err(|err| {
                    log::error!("Error parsing dropped file line {line} as url: {err:#}");
                })
                .ok()?;
            url.to_file_path()
                .map_err(|_| {
                    log::error!("Error converting url {url:?} from line {line} to pathbuf");
                })
                .ok()
        })
        .collect()
}

fn parse_xmozurl_list(url_list: &str) -> Vec<Url> {
    url_list
        .lines()
        .step_by(2)
        .filter_map(|line| {
            // the lines alternate between the urls and their titles
            Url::parse(line)
                .map_err(|err| {
                    log::error!("Error parsing dropped file line {line} as url: {err:#}");
                })
                .ok()
        })
        .collect()
}

/// Data may be UTF16 in either byte order, or UTF8
fn decode_dropped_url_string(raw: &[u8]) -> String {
    if raw.len() >= 2 && ((raw[0], raw[1]) == (0xfe, 0xff) || (raw[0] != 0x00 && raw[1] == 0x00)) {
        String::from_utf16_lossy(
            raw.chunks_exact(2)
                .map(|x: &[u8]| u16::from(x[1]) << 8 | u16::from(x[0]))
                .collect::<Vec<u16>>()
                .as_slice(),
        )
    } else if raw.len() >= 2
        && ((raw[0], raw[1]) == (0xff, 0xfe) || (raw[0] == 0x00 && raw[1] != 0x00))
    {
        String::from_utf16_lossy(
            raw.chunks_exact(2)
                .map(|x: &[u8]| u16::from(x[0]) << 8 | u16::from(x[1]))
                .collect::<Vec<u16>>()
                .as_slice(),
        )
    } else {
        String::from_utf8_lossy(raw).to_string()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
enum NetWmStateAction {
    Remove = 0,
    Add = 1,
    #[allow(dead_code)]
    Toggle = 2,
}

impl NetWmStateAction {
    fn with_bool(enable: bool) -> Self {
        if enable {
            Self::Add
        } else {
            Self::Remove
        }
    }
}
