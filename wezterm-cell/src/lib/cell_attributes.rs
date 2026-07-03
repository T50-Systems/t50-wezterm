impl Default for CellAttributes {
    fn default() -> Self {
        Self::blank()
    }
}

impl CellAttributes {
    bitfield!(intensity, set_intensity, Intensity, 0b11, 0);
    bitfield!(underline, set_underline, Underline, 0b111, 2);
    bitfield!(blink, set_blink, Blink, 0b11, 5);
    bitfield!(italic, set_italic, 7);
    bitfield!(reverse, set_reverse, 8);
    bitfield!(strikethrough, set_strikethrough, 9);
    bitfield!(invisible, set_invisible, 10);
    bitfield!(wrapped, set_wrapped, 11);
    bitfield!(overline, set_overline, 12);
    bitfield!(semantic_type, set_semantic_type, SemanticType, 0b11, 13);
    bitfield!(vertical_align, set_vertical_align, VerticalAlign, 0b11, 15);

    pub const fn blank() -> Self {
        Self {
            attributes: 0,
            foreground: SmallColor::Default,
            background: SmallColor::Default,
            fat: None,
        }
    }

    /// Returns true if the attribute bits in both objects are equal.
    /// This can be used to cheaply test whether the styles of the two
    /// cells are the same, and is used by some `Renderer` implementations.
    pub fn attribute_bits_equal(&self, other: &Self) -> bool {
        self.attributes == other.attributes
    }

    pub fn compute_shape_hash<H: Hasher>(&self, hasher: &mut H) {
        self.attributes.hash(hasher);
        self.foreground.hash(hasher);
        self.background.hash(hasher);
        if let Some(fat) = &self.fat {
            fat.compute_shape_hash(hasher);
        }
    }

    /// Set the foreground color for the cell to that specified
    pub fn set_foreground<C: Into<ColorAttribute>>(&mut self, foreground: C) -> &mut Self {
        let foreground: ColorAttribute = foreground.into();
        match foreground {
            ColorAttribute::Default => {
                self.foreground = SmallColor::Default;
                if let Some(fat) = self.fat.as_mut() {
                    fat.foreground = ColorAttribute::Default;
                }
                self.deallocate_fat_attributes_if_none();
            }
            ColorAttribute::PaletteIndex(idx) => {
                self.foreground = SmallColor::PaletteIndex(idx);
                if let Some(fat) = self.fat.as_mut() {
                    fat.foreground = ColorAttribute::Default;
                }
                self.deallocate_fat_attributes_if_none();
            }
            foreground => {
                self.foreground = SmallColor::Default;
                self.allocate_fat_attributes();
                self.fat.as_mut().unwrap().foreground = foreground;
            }
        }

        self
    }

    pub fn foreground(&self) -> ColorAttribute {
        if let Some(fat) = self.fat.as_ref() {
            if fat.foreground != ColorAttribute::Default {
                return fat.foreground;
            }
        }
        self.foreground.into()
    }

    pub fn set_background<C: Into<ColorAttribute>>(&mut self, background: C) -> &mut Self {
        let background: ColorAttribute = background.into();
        match background {
            ColorAttribute::Default => {
                self.background = SmallColor::Default;
                if let Some(fat) = self.fat.as_mut() {
                    fat.background = ColorAttribute::Default;
                }
                self.deallocate_fat_attributes_if_none();
            }
            ColorAttribute::PaletteIndex(idx) => {
                self.background = SmallColor::PaletteIndex(idx);
                if let Some(fat) = self.fat.as_mut() {
                    fat.background = ColorAttribute::Default;
                }
                self.deallocate_fat_attributes_if_none();
            }
            background => {
                self.background = SmallColor::Default;
                self.allocate_fat_attributes();
                self.fat.as_mut().unwrap().background = background;
            }
        }

        self
    }

    pub fn background(&self) -> ColorAttribute {
        if let Some(fat) = self.fat.as_ref() {
            if fat.background != ColorAttribute::Default {
                return fat.background;
            }
        }
        self.background.into()
    }

    /// Clear all attributes from a cell
    pub fn clear(&mut self) {
        *self = Self::blank();
    }

    fn allocate_fat_attributes(&mut self) {
        if self.fat.is_none() {
            self.fat.replace(Box::new(FatAttributes {
                hyperlink: None,
                #[cfg(feature = "use_image")]
                image: vec![],
                underline_color: ColorAttribute::Default,
                foreground: ColorAttribute::Default,
                background: ColorAttribute::Default,
            }));
        }
    }

    fn deallocate_fat_attributes_if_none(&mut self) {
        let deallocate = self
            .fat
            .as_ref()
            .map(|fat| {
                #[cfg(feature = "use_image")]
                {
                    if !fat.image.is_empty() {
                        return false;
                    }
                }
                fat.hyperlink.is_none()
                    && fat.underline_color == ColorAttribute::Default
                    && fat.foreground == ColorAttribute::Default
                    && fat.background == ColorAttribute::Default
            })
            .unwrap_or(false);
        if deallocate {
            self.fat.take();
        }
    }

    pub fn set_hyperlink(&mut self, link: Option<Arc<Hyperlink>>) -> &mut Self {
        if link.is_none() && self.fat.is_none() {
            self
        } else {
            self.allocate_fat_attributes();
            self.fat.as_mut().unwrap().hyperlink = link;
            self.deallocate_fat_attributes_if_none();
            self
        }
    }
}

#[cfg(feature = "use_image")]
impl CellAttributes {
    /// Assign a single image to a cell.
    pub fn set_image(&mut self, image: Box<ImageCell>) -> &mut Self {
        self.allocate_fat_attributes();
        self.fat.as_mut().unwrap().image = vec![image];
        self
    }

    /// Clear all images from a cell
    pub fn clear_images(&mut self) -> &mut Self {
        if let Some(fat) = self.fat.as_mut() {
            fat.image.clear();
        }
        self.deallocate_fat_attributes_if_none();
        self
    }

    pub fn detach_image_with_placement(&mut self, image_id: u32, placement_id: Option<u32>) {
        if let Some(fat) = self.fat.as_mut() {
            fat.image
                .retain(|im| !im.matches_placement(image_id, placement_id));
        }
        self.deallocate_fat_attributes_if_none();
    }

    /// Add an image attachement, preserving any existing attachments.
    /// The list of images is maintained in z-index order
    pub fn attach_image(&mut self, image: Box<ImageCell>) -> &mut Self {
        self.allocate_fat_attributes();
        let fat = self.fat.as_mut().unwrap();
        let z_index = image.z_index();
        match fat
            .image
            .binary_search_by(|probe| probe.z_index().cmp(&z_index))
        {
            Ok(idx) | Err(idx) => fat.image.insert(idx, image),
        }
        self
    }
}

impl CellAttributes {
    pub fn set_underline_color<C: Into<ColorAttribute>>(
        &mut self,
        underline_color: C,
    ) -> &mut Self {
        let underline_color = underline_color.into();
        if underline_color == ColorAttribute::Default && self.fat.is_none() {
            self
        } else {
            self.allocate_fat_attributes();
            self.fat.as_mut().unwrap().underline_color = underline_color;
            self.deallocate_fat_attributes_if_none();
            self
        }
    }

    /// Clone the attributes, but exclude fancy extras such
    /// as hyperlinks or future sprite things
    pub fn clone_sgr_only(&self) -> Self {
        let mut res = Self {
            attributes: self.attributes,
            foreground: self.foreground,
            background: self.background,
            fat: None,
        };
        if let Some(fat) = self.fat.as_ref() {
            if fat.background != ColorAttribute::Default
                || fat.foreground != ColorAttribute::Default
            {
                res.allocate_fat_attributes();
                let new_fat = res.fat.as_mut().unwrap();
                new_fat.foreground = fat.foreground;
                new_fat.background = fat.background;
            }
        }
        // Reset the semantic type; clone_sgr_only is used primarily
        // to create a "blank" cell when clearing and we want that to
        // be deterministically tagged as Output so that we have an
        // easier time in get_semantic_zones.
        res.set_semantic_type(SemanticType::default());
        res.set_underline_color(self.underline_color());

        // Turn off underline because it can have surprising results
        // if underline is on, then we get CRLF and then SGR reset:
        // If the CRLF causes a line to scroll, we'll call clone_sgr_only()
        // to get a blank cell for the new line and it would be filled
        // with underlines.
        // clone_sgr_only() is primarily for preserving the background
        // color when erasing rather than other attributes, so it should
        // be fine to clear out the actual underline attribute.
        // Let's extend this to other line attribute types as well.
        // <https://github.com/wezterm/wezterm/issues/2489>
        res.set_underline(Underline::None);
        res.set_overline(false);
        res.set_strikethrough(false);
        res
    }

    pub fn hyperlink(&self) -> Option<&Arc<Hyperlink>> {
        self.fat.as_ref().and_then(|fat| fat.hyperlink.as_ref())
    }

    /// Returns the list of attached images in z-index order.
    /// Returns None if there are no attached images; will
    /// never return Some(vec![]).
    #[cfg(feature = "use_image")]
    pub fn images(&self) -> Option<Vec<ImageCell>> {
        let fat = self.fat.as_ref()?;
        if fat.image.is_empty() {
            return None;
        }
        Some(fat.image.iter().map(|im| im.as_ref().clone()).collect())
    }

    pub fn underline_color(&self) -> ColorAttribute {
        self.fat
            .as_ref()
            .map(|fat| fat.underline_color)
            .unwrap_or(ColorAttribute::Default)
    }

    pub fn apply_change(&mut self, change: &AttributeChange) {
        use AttributeChange::*;
        match change {
            Intensity(value) => {
                self.set_intensity(*value);
            }
            Underline(value) => {
                self.set_underline(*value);
            }
            Italic(value) => {
                self.set_italic(*value);
            }
            Blink(value) => {
                self.set_blink(*value);
            }
            Reverse(value) => {
                self.set_reverse(*value);
            }
            StrikeThrough(value) => {
                self.set_strikethrough(*value);
            }
            Invisible(value) => {
                self.set_invisible(*value);
            }
            Foreground(value) => {
                self.set_foreground(*value);
            }
            Background(value) => {
                self.set_background(*value);
            }
            Hyperlink(value) => {
                self.set_hyperlink(value.clone());
            }
        }
    }
}
