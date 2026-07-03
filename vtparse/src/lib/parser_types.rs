struct OscState {
    #[cfg(any(feature = "std", feature = "alloc"))]
    buffer: Vec<u8>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    buffer: heapless::Vec<u8, { MAX_OSC * 16 }>,
    param_indices: [usize; MAX_OSC],
    num_params: usize,
    full: bool,
}

impl OscState {
    fn put(&mut self, param: char) {
        if param == ';' {
            match self.num_params {
                MAX_OSC => {
                    self.full = true;
                }
                num => {
                    self.param_indices[num.saturating_sub(1)] = self.buffer.len();
                    self.num_params += 1;
                }
            }
        } else if !self.full {
            let mut buf = [0u8; 8];
            let extend_result = self
                .buffer
                .extend_from_slice(param.encode_utf8(&mut buf).as_bytes());

            #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
            {
                if extend_result.is_err() {
                    self.full = true;
                    return;
                }
            }

            let _ = extend_result;

            if self.num_params == 0 {
                self.num_params = 1;
            }
        }
    }
}

/// The virtual terminal parser.  It works together with an implementation of `VTActor`.
pub struct VTParser {
    state: State,

    intermediates: [u8; MAX_INTERMEDIATES],
    num_intermediates: usize,
    ignored_excess_intermediates: bool,

    osc: OscState,

    params: [CsiParam; MAX_PARAMS],
    num_params: usize,
    current_param: Option<CsiParam>,
    params_full: bool,
    #[cfg(any(feature = "std", feature = "alloc"))]
    apc_data: Vec<u8>,

    utf8_parser: Utf8Parser,
    utf8_return_state: State,
}

/// Represents a parameter to a CSI-based escaped sequence.
///
/// CSI escapes typically have the form: `CSI 3 m`, but can also
/// bundle multiple values together: `CSI 3 ; 4 m`.  In both
/// of those examples the parameters are simple integer values
/// and latter of which would be expressed as a slice containing
/// `[CsiParam::Integer(3), CsiParam::Integer(4)]`.
///
/// There are some escape sequences that use colons to subdivide and
/// extend the meaning.  For example: `CSI 4:3 m` is a sequence used
/// to denote a curly underline.  That would be represented as:
/// `[CsiParam::ColonList(vec![Some(4), Some(3)])]`.
///
/// Later: reading ECMA 48, CSI is defined as:
/// CSI P ... P  I ... I  F
/// Where P are parameter bytes in the range 0x30-0x3F [0-9:;<=>?]
/// and I are intermediate bytes in the range 0x20-0x2F
/// and F is the final byte in the range 0x40-0x7E
///
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum CsiParam {
    Integer(i64),
    P(u8),
}

impl core::fmt::Debug for CsiParam {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Integer(i) => write!(fmt, "Integer({i})"),
            Self::P(n) => write!(fmt, "P({})", char::from(*n)),
        }
    }
}

impl Default for CsiParam {
    fn default() -> Self {
        Self::Integer(0)
    }
}

impl CsiParam {
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }
}

impl core::fmt::Display for CsiParam {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            CsiParam::Integer(v) => {
                write!(f, "{}", v)?;
            }
            CsiParam::P(p) => {
                write!(f, "{}", *p as char)?;
            }
        }
        Ok(())
    }
}
