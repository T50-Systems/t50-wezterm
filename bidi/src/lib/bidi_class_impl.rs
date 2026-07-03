impl BidiClass {
    pub fn is_iso_init(self) -> bool {
        match self {
            BidiClass::RightToLeftIsolate
            | BidiClass::LeftToRightIsolate
            | BidiClass::FirstStrongIsolate => true,
            _ => false,
        }
    }

    pub fn is_iso_control(self) -> bool {
        match self {
            BidiClass::RightToLeftIsolate
            | BidiClass::LeftToRightIsolate
            | BidiClass::PopDirectionalIsolate
            | BidiClass::FirstStrongIsolate => true,
            _ => false,
        }
    }

    pub fn is_neutral(self) -> bool {
        match self {
            BidiClass::OtherNeutral
            | BidiClass::WhiteSpace
            | BidiClass::SegmentSeparator
            | BidiClass::ParagraphSeparator => true,
            _ => self.is_iso_control(),
        }
    }
}
