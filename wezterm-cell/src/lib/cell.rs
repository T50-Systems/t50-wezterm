/// Models the contents of a cell on the terminal display
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq)]
pub struct Cell {
    #[cfg_attr(
        feature = "use_serde",
        serde(
            deserialize_with = "deserialize_teenystring",
            serialize_with = "serialize_teenystring"
        )
    )]
    text: TeenyString,
    attrs: CellAttributes,
}

impl core::fmt::Debug for Cell {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        fmt.debug_struct("Cell")
            .field("text", &self.str())
            .field("width", &self.width())
            .field("attrs", &self.attrs)
            .finish()
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::blank()
    }
}

impl Cell {
    /// Create a new cell holding the specified character and with the
    /// specified cell attributes.
    /// All control and movement characters are rewritten as a space.
    pub fn new(text: char, attrs: CellAttributes) -> Self {
        let storage = TeenyString::from_char(text);
        Self {
            text: storage,
            attrs,
        }
    }

    pub const fn blank() -> Self {
        Self {
            text: TeenyString::space(),
            attrs: CellAttributes::blank(),
        }
    }

    pub const fn blank_with_attrs(attrs: CellAttributes) -> Self {
        Self {
            text: TeenyString::space(),
            attrs,
        }
    }

    /// Indicates whether this cell has text or emoji presentation.
    /// The width already reflects that choice; this information
    /// is also useful when selecting an appropriate font.
    pub fn presentation(&self) -> Presentation {
        match Presentation::for_grapheme(self.str()) {
            (_, Some(variation)) => variation,
            (presentation, None) => presentation,
        }
    }

    /// Create a new cell holding the specified grapheme.
    /// The grapheme is passed as a string slice and is intended to hold
    /// double-width characters, or combining unicode sequences, that need
    /// to be treated as a single logical "character" that can be cursored
    /// over.  This function technically allows for an arbitrary string to
    /// be passed but it should not be used to hold strings other than
    /// graphemes.
    pub fn new_grapheme(
        text: &str,
        attrs: CellAttributes,
        unicode_version: Option<&UnicodeVersion>,
    ) -> Self {
        let storage = TeenyString::from_str(text, None, unicode_version);

        Self {
            text: storage,
            attrs,
        }
    }

    pub fn new_grapheme_with_width(text: &str, width: usize, attrs: CellAttributes) -> Self {
        let storage = TeenyString::from_str(text, Some(width), None);
        Self {
            text: storage,
            attrs,
        }
    }

    /// Returns the textual content of the cell
    pub fn str(&self) -> &str {
        self.text.str()
    }

    /// Returns the number of cells visually occupied by this grapheme
    pub fn width(&self) -> usize {
        self.text.width()
    }

    /// Returns the attributes of the cell
    pub fn attrs(&self) -> &CellAttributes {
        &self.attrs
    }

    pub fn attrs_mut(&mut self) -> &mut CellAttributes {
        &mut self.attrs
    }
}
