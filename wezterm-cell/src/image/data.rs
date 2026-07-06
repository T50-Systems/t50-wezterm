#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ImageCellError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    BlobLease(#[from] wezterm_blob_leases::Error),

    #[error(transparent)]
    ImageError(#[from] image::ImageError),
}

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct ImageData {
    data: Mutex<ImageDataType>,
    hash: [u8; 32],
}

struct HexSlice<'a>(&'a [u8]);
impl<'a> std::fmt::Display for HexSlice<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte in self.0 {
            write!(fmt, "{byte:x}")?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for ImageData {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("ImageData")
            .field("data", &self.data)
            .field("hash", &format_args!("{}", HexSlice(&self.hash)))
            .finish()
    }
}

impl Eq for ImageData {}
impl PartialEq for ImageData {
    fn eq(&self, rhs: &Self) -> bool {
        self.hash == rhs.hash
    }
}

impl ImageData {
    /// Create a new ImageData struct with the provided raw data.
    pub fn with_raw_data(data: Vec<u8>) -> Self {
        let hash = ImageDataType::hash_bytes(&data);
        Self::with_data_and_hash(ImageDataType::EncodedFile(data).decode(), hash)
    }

    fn with_data_and_hash(data: ImageDataType, hash: [u8; 32]) -> Self {
        Self {
            data: Mutex::new(data),
            hash,
        }
    }

    pub fn with_data(data: ImageDataType) -> Self {
        let hash = data.compute_hash();
        Self {
            data: Mutex::new(data),
            hash,
        }
    }

    /// Returns the in-memory footprint
    pub fn len(&self) -> usize {
        match &*self.data() {
            ImageDataType::EncodedFile(d) => d.len(),
            ImageDataType::EncodedLease(_) => 0,
            ImageDataType::Rgba8 { data, .. } => data.len(),
            ImageDataType::AnimRgba8 { frames, .. } => frames.len() * frames[0].len(),
        }
    }

    pub fn data(&self) -> MutexGuard<'_, ImageDataType> {
        self.data.lock().unwrap()
    }

    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
}
