#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive)]
pub enum SgrCode {
    Reset = 0,
    IntensityBold = 1,
    IntensityDim = 2,
    ItalicOn = 3,
    UnderlineOn = 4,
    /// Blinks < 150 times per minute
    BlinkOn = 5,
    /// Blinks > 150 times per minute
    RapidBlinkOn = 6,
    InverseOn = 7,
    InvisibleOn = 8,
    StrikeThroughOn = 9,
    DefaultFont = 10,
    AltFont1 = 11,
    AltFont2 = 12,
    AltFont3 = 13,
    AltFont4 = 14,
    AltFont5 = 15,
    AltFont6 = 16,
    AltFont7 = 17,
    AltFont8 = 18,
    AltFont9 = 19,
    // Fraktur = 20,
    UnderlineDouble = 21,
    NormalIntensity = 22,
    ItalicOff = 23,
    UnderlineOff = 24,
    BlinkOff = 25,
    InverseOff = 27,
    InvisibleOff = 28,
    StrikeThroughOff = 29,
    ForegroundBlack = 30,
    ForegroundRed = 31,
    ForegroundGreen = 32,
    ForegroundYellow = 33,
    ForegroundBlue = 34,
    ForegroundMagenta = 35,
    ForegroundCyan = 36,
    ForegroundWhite = 37,
    ForegroundDefault = 39,
    BackgroundBlack = 40,
    BackgroundRed = 41,
    BackgroundGreen = 42,
    BackgroundYellow = 43,
    BackgroundBlue = 44,
    BackgroundMagenta = 45,
    BackgroundCyan = 46,
    BackgroundWhite = 47,
    BackgroundDefault = 49,
    OverlineOn = 53,
    OverlineOff = 55,

    UnderlineColor = 58,
    ResetUnderlineColor = 59,

    VerticalAlignSuperScript = 73,
    VerticalAlignSubScript = 74,
    VerticalAlignBaseLine = 75,

    ForegroundBrightBlack = 90,
    ForegroundBrightRed = 91,
    ForegroundBrightGreen = 92,
    ForegroundBrightYellow = 93,
    ForegroundBrightBlue = 94,
    ForegroundBrightMagenta = 95,
    ForegroundBrightCyan = 96,
    ForegroundBrightWhite = 97,

    BackgroundBrightBlack = 100,
    BackgroundBrightRed = 101,
    BackgroundBrightGreen = 102,
    BackgroundBrightYellow = 103,
    BackgroundBrightBlue = 104,
    BackgroundBrightMagenta = 105,
    BackgroundBrightCyan = 106,
    BackgroundBrightWhite = 107,

    /// Maybe followed either either a 256 color palette index or
    /// a sequence describing a true color rgb value
    ForegroundColor = 38,
    BackgroundColor = 48,
}

impl<'a> Iterator for CSIParser<'a> {
    type Item = CSI;

    fn next(&mut self) -> Option<CSI> {
        let params = match self.params.take() {
            None => return None,
            Some(params) => params,
        };

        match self.parse_next(&params) {
            Ok(csi) => Some(csi),
            Err(()) => Some(CSI::Unspecified(Box::new(Unspecified {
                params: params.to_vec(),
                parameters_truncated: self.parameters_truncated,
                control: self.control,
            }))),
        }
    }
}
