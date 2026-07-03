#[cfg(feature = "use_serde")]
fn deserialize_teenystring<'de, D>(deserializer: D) -> Result<TeenyString, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    Ok(TeenyString::from_str(&text, None, None))
}

#[cfg(feature = "use_serde")]
fn serialize_teenystring<S>(value: &TeenyString, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // unsafety: this is safe because the Cell constructor guarantees
    // that the storage is valid utf8
    let s = unsafe { core::str::from_utf8_unchecked(value.as_bytes()) };
    s.serialize(serializer)
}

/// TeenyString encodes string storage in a single u64.
/// The scheme is simple but effective: strings that encode into a
/// byte slice that is 1 less byte than the machine word size can
/// be encoded directly into the usize bits stored in the struct.
/// A marker bit (LSB for big endian, MSB for little endian) is
/// set to indicate that the string is stored inline.
/// If the string is longer than this then a `Vec<u8>` is allocated
/// from the heap and the usize holds its raw pointer address.
///
/// When the string is inlined, the next-MSB is used to short-cut
/// calling grapheme_column_width; if it is set, then the TeenyString
/// has length 2, otherwise, it has length 1 (we don't allow zero-length
/// strings).
struct TeenyString(u64);
struct TeenyStringHeap {
    bytes: Vec<u8>,
    width: usize,
}

impl TeenyString {
    const fn marker_mask() -> u64 {
        if cfg!(target_endian = "little") {
            0x80000000_00000000
        } else {
            0x1
        }
    }

    const fn double_wide_mask() -> u64 {
        if cfg!(target_endian = "little") {
            0xc0000000_00000000
        } else {
            0x3
        }
    }

    const fn is_marker_bit_set(word: u64) -> bool {
        let mask = Self::marker_mask();
        word & mask == mask
    }

    const fn is_double_width(word: u64) -> bool {
        let mask = Self::double_wide_mask();
        word & mask == mask
    }

    const fn set_marker_bit(word: u64, width: usize) -> u64 {
        if width > 1 {
            word | Self::double_wide_mask()
        } else {
            word | Self::marker_mask()
        }
    }

    pub fn from_str(
        s: &str,
        width: Option<usize>,
        unicode_version: Option<&UnicodeVersion>,
    ) -> Self {
        // De-fang the input text such that it has no special meaning
        // to a terminal.  All control and movement characters are rewritten
        // as a space.
        let s = if s.is_empty() || s == "\r\n" {
            " "
        } else if s.len() == 1 {
            let b = s.as_bytes()[0];
            if b < 0x20 || b == 0x7f {
                " "
            } else {
                s
            }
        } else {
            s
        };

        let bytes = s.as_bytes();
        let len = bytes.len();
        let width = width.unwrap_or_else(|| grapheme_column_width(s, unicode_version));

        if len < core::mem::size_of::<u64>() && width < 3 {
            let mut word = 0u64;
            unsafe {
                core::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    &mut word as *mut u64 as *mut u8,
                    len,
                );
            }
            let word = Self::set_marker_bit(word as u64, width);
            Self(word)
        } else {
            let vec = Box::new(TeenyStringHeap {
                bytes: bytes.to_vec(),
                width,
            });
            let ptr = Box::into_raw(vec);
            Self(ptr as u64)
        }
    }

    pub const fn space() -> Self {
        Self(if cfg!(target_endian = "little") {
            0x80000000_00000020
        } else {
            0x20000000_00000001
        })
    }

    pub fn from_char(c: char) -> Self {
        let mut bytes = [0u8; 8];
        Self::from_str(c.encode_utf8(&mut bytes), None, None)
    }

    pub fn width(&self) -> usize {
        if Self::is_marker_bit_set(self.0) {
            if Self::is_double_width(self.0) {
                2
            } else {
                1
            }
        } else {
            let heap = self.0 as *const u64 as *const TeenyStringHeap;
            unsafe { (*heap).width }
        }
    }

    pub fn str(&self) -> &str {
        // unsafety: this is safe because the constructor guarantees
        // that the storage is valid utf8
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        if Self::is_marker_bit_set(self.0) {
            let bytes = &self.0 as *const u64 as *const u8;
            let bytes =
                unsafe { core::slice::from_raw_parts(bytes, core::mem::size_of::<u64>() - 1) };
            let len = bytes
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(core::mem::size_of::<u64>() - 1);

            &bytes[0..len]
        } else {
            let heap = self.0 as *const u64 as *const TeenyStringHeap;
            unsafe { (*heap).bytes.as_slice() }
        }
    }
}

impl Drop for TeenyString {
    fn drop(&mut self) {
        if !Self::is_marker_bit_set(self.0) {
            let vec = unsafe { Box::from_raw(self.0 as *mut usize as *mut TeenyStringHeap) };
            drop(vec);
        }
    }
}

impl core::clone::Clone for TeenyString {
    fn clone(&self) -> Self {
        if Self::is_marker_bit_set(self.0) {
            Self(self.0)
        } else {
            Self::from_str(self.str(), None, None)
        }
    }
}

impl core::cmp::PartialEq for TeenyString {
    fn eq(&self, rhs: &Self) -> bool {
        self.as_bytes().eq(rhs.as_bytes())
    }
}
impl core::cmp::Eq for TeenyString {}
