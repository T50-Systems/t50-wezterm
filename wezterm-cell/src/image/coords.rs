use ordered_float::NotNan;
#[cfg(feature = "use_serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
#[cfg(feature = "std")]
use wezterm_blob_leases::{BlobLease, BlobManager};

#[cfg(feature = "use_serde")]
fn deserialize_notnan<'de, D>(deserializer: D) -> Result<NotNan<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = f32::deserialize(deserializer)?;
    NotNan::new(value).map_err(|e| serde::de::Error::custom(format!("{:?}", e)))
}

#[cfg(feature = "use_serde")]
#[allow(clippy::trivially_copy_pass_by_ref)]
fn serialize_notnan<S>(value: &NotNan<f32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value.into_inner().serialize(serializer)
}

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureCoordinate {
    #[cfg_attr(
        feature = "use_serde",
        serde(
            deserialize_with = "deserialize_notnan",
            serialize_with = "serialize_notnan"
        )
    )]
    pub x: NotNan<f32>,
    #[cfg_attr(
        feature = "use_serde",
        serde(
            deserialize_with = "deserialize_notnan",
            serialize_with = "serialize_notnan"
        )
    )]
    pub y: NotNan<f32>,
}

impl TextureCoordinate {
    pub fn new(x: NotNan<f32>, y: NotNan<f32>) -> Self {
        Self { x, y }
    }

    pub fn new_f32(x: f32, y: f32) -> Self {
        let x = NotNan::new(x).unwrap();
        let y = NotNan::new(y).unwrap();
        Self::new(x, y)
    }
}

/// Tracks data for displaying an image in the place of the normal cell
/// character data.  Since an Image can span multiple cells, we need to logically
/// carve up the image and track each slice of it.  Each cell needs to know
/// its "texture coordinates" within that image so that we can render the
/// right slice.
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCell {
    /// Texture coordinate for the top left of this cell.
    /// (0,0) is the top left of the ImageData. (1, 1) is
    /// the bottom right.
    top_left: TextureCoordinate,
    /// Texture coordinates for the bottom right of this cell.
    bottom_right: TextureCoordinate,
    /// References the underlying image data
    data: Arc<ImageData>,
    z_index: i32,
    /// When rendering in the cell, use this offset from the top left
    /// of the cell
    padding_left: u16,
    padding_top: u16,
    padding_right: u16,
    padding_bottom: u16,

    image_id: Option<u32>,
    placement_id: Option<u32>,
}

impl ImageCell {
    pub fn new(
        top_left: TextureCoordinate,
        bottom_right: TextureCoordinate,
        data: Arc<ImageData>,
    ) -> Self {
        Self::with_z_index(top_left, bottom_right, data, 0, 0, 0, 0, 0, None, None)
    }

    pub fn compute_shape_hash<H: Hasher>(&self, hasher: &mut H) {
        self.top_left.hash(hasher);
        self.bottom_right.hash(hasher);
        self.data.hash.hash(hasher);
        self.z_index.hash(hasher);
        self.padding_left.hash(hasher);
        self.padding_top.hash(hasher);
        self.padding_right.hash(hasher);
        self.padding_bottom.hash(hasher);
        self.image_id.hash(hasher);
        self.placement_id.hash(hasher);
    }

    pub fn with_z_index(
        top_left: TextureCoordinate,
        bottom_right: TextureCoordinate,
        data: Arc<ImageData>,
        z_index: i32,
        padding_left: u16,
        padding_top: u16,
        padding_right: u16,
        padding_bottom: u16,
        image_id: Option<u32>,
        placement_id: Option<u32>,
    ) -> Self {
        Self {
            top_left,
            bottom_right,
            data,
            z_index,
            padding_left,
            padding_top,
            padding_right,
            padding_bottom,
            image_id,
            placement_id,
        }
    }

    pub fn matches_placement(&self, image_id: u32, placement_id: Option<u32>) -> bool {
        self.image_id == Some(image_id) && self.placement_id == placement_id
    }

    pub fn has_placement_id(&self) -> bool {
        self.placement_id.is_some()
    }

    pub fn image_id(&self) -> Option<u32> {
        self.image_id
    }

    pub fn placement_id(&self) -> Option<u32> {
        self.placement_id
    }

    pub fn top_left(&self) -> TextureCoordinate {
        self.top_left
    }

    pub fn bottom_right(&self) -> TextureCoordinate {
        self.bottom_right
    }

    pub fn image_data(&self) -> &Arc<ImageData> {
        &self.data
    }

    /// negative z_index is rendered beneath the text layer.
    /// >= 0 is rendered above the text.
    /// negative z_index < INT32_MIN/2 will be drawn under cells
    /// with non-default background colors
    pub fn z_index(&self) -> i32 {
        self.z_index
    }

    /// Returns padding (left, top, right, bottom)
    pub fn padding(&self) -> (u16, u16, u16, u16) {
        (
            self.padding_left,
            self.padding_top,
            self.padding_right,
            self.padding_bottom,
        )
    }
}
