/// A helper type to avoid accidentally tripping over problems with
/// 1-based values in escape sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OneBased {
    value: u32,
}

impl OneBased {
    pub fn new(value: u32) -> Self {
        debug_assert!(
            value != 0,
            "programmer error: deliberately assigning zero to a OneBased"
        );
        Self { value }
    }

    pub fn from_zero_based(value: u32) -> Self {
        Self { value: value + 1 }
    }

    /// Map a value from an escape sequence parameter.
    /// 0 is equivalent to 1
    pub fn from_esc_param(v: &CsiParam) -> core::result::Result<Self, ()> {
        match v {
            CsiParam::Integer(v) if *v == 0 => Ok(Self {
                value: num_traits::one(),
            }),
            CsiParam::Integer(v) if *v > 0 && *v <= i64::from(u32::max_value()) => {
                Ok(Self { value: *v as u32 })
            }
            _ => Err(()),
        }
    }

    /// Map a value from an escape sequence parameter.
    /// 0 is equivalent to max_value.
    pub fn from_esc_param_with_big_default(v: &CsiParam) -> core::result::Result<Self, ()> {
        match v {
            CsiParam::Integer(v) if *v == 0 => Ok(Self {
                value: u32::max_value(),
            }),
            CsiParam::Integer(v) if *v > 0 && *v <= i64::from(u32::max_value()) => {
                Ok(Self { value: *v as u32 })
            }
            _ => Err(()),
        }
    }

    /// Map a value from an optional escape sequence parameter
    pub fn from_optional_esc_param(o: Option<&CsiParam>) -> core::result::Result<Self, ()> {
        Self::from_esc_param(o.unwrap_or(&CsiParam::Integer(1)))
    }

    /// Return the underlying value as a 0-based value
    pub fn as_zero_based(self) -> u32 {
        self.value.saturating_sub(1)
    }

    pub fn as_one_based(self) -> u32 {
        self.value
    }
}

impl Display for OneBased {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.value.fmt(f)
    }
}
