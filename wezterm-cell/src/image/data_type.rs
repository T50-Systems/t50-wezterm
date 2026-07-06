#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq)]
pub enum ImageDataType {
    /// Data is in the native image file format
    /// (best for file formats that have animated content)
    EncodedFile(Vec<u8>),
    /// Data is in the native image file format,
    /// (best for file formats that have animated content)
    /// and is stored as a blob via the blob manager.
    #[cfg(feature = "std")]
    EncodedLease(
        #[cfg_attr(
            feature = "use_serde",
            serde(with = "wezterm_blob_leases::lease_bytes")
        )]
        BlobLease,
    ),
    /// Data is RGBA u8 data
    Rgba8 {
        data: Vec<u8>,
        width: u32,
        height: u32,
        hash: [u8; 32],
    },
    /// Data is an animated sequence
    AnimRgba8 {
        width: u32,
        height: u32,
        durations: Vec<Duration>,
        frames: Vec<Vec<u8>>,
        hashes: Vec<[u8; 32]>,
    },
}

impl std::fmt::Debug for ImageDataType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::EncodedFile(data) => fmt
                .debug_struct("EncodedFile")
                .field("data_of_len", &data.len())
                .finish(),
            Self::EncodedLease(lease) => lease.fmt(fmt),
            Self::Rgba8 {
                data,
                width,
                height,
                hash,
            } => fmt
                .debug_struct("Rgba8")
                .field("data_of_len", &data.len())
                .field("width", &width)
                .field("height", &height)
                .field("hash", &hash)
                .finish(),
            Self::AnimRgba8 {
                frames,
                width,
                height,
                durations,
                hashes,
            } => fmt
                .debug_struct("AnimRgba8")
                .field("frames_of_len", &frames.len())
                .field("width", &width)
                .field("height", &height)
                .field("durations", durations)
                .field("hashes", hashes)
                .finish(),
        }
    }
}

impl ImageDataType {
    pub fn new_single_frame(width: u32, height: u32, data: Vec<u8>) -> Self {
        let hash = Self::hash_bytes(&data);
        assert_eq!(
            width * height * 4,
            data.len() as u32,
            "invalid dimensions {}x{} for pixel data of length {}",
            width,
            height,
            data.len()
        );
        Self::Rgba8 {
            width,
            height,
            data,
            hash,
        }
    }

    /// Black pixels
    pub fn placeholder() -> Self {
        let mut data = vec![];
        let size = 8;
        for _ in 0..size * size {
            data.extend_from_slice(&[0, 0, 0, 0xff]);
        }
        ImageDataType::new_single_frame(size, size, data)
    }

    pub fn hash_bytes(bytes: &[u8]) -> [u8; 32] {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(bytes);
        hasher.finalize().into()
    }

    pub fn compute_hash(&self) -> [u8; 32] {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        match self {
            ImageDataType::EncodedFile(data) => hasher.update(data),
            ImageDataType::EncodedLease(lease) => return lease.content_id().as_hash_bytes(),
            ImageDataType::Rgba8 { data, .. } => hasher.update(data),
            ImageDataType::AnimRgba8 {
                frames, durations, ..
            } => {
                for data in frames {
                    hasher.update(data);
                }
                for d in durations {
                    let d = d.as_secs_f32();
                    let b = d.to_ne_bytes();
                    hasher.update(b);
                }
            }
        };
        hasher.finalize().into()
    }

    /// Divides the animation frame durations by the provided
    /// speed_factor, so a factor of 2 will halve the duration.
    /// # Panics
    /// if the speed_factor is negative, non-finite or the result
    /// overflows the allow Duration range.
    pub fn adjust_speed(&mut self, speed_factor: f32) {
        match self {
            Self::AnimRgba8 { durations, .. } => {
                for d in durations {
                    *d = d.mul_f32(1. / speed_factor);
                }
            }
            _ => {}
        }
    }

    #[cfg(feature = "use_image")]
    pub fn dimensions(&self) -> Result<(u32, u32), ImageCellError> {
        fn dimensions_for_data(data: &[u8]) -> image::ImageResult<(u32, u32)> {
            let reader =
                image::ImageReader::new(std::io::Cursor::new(data)).with_guessed_format()?;
            let (width, height) = reader.into_dimensions()?;

            Ok((width, height))
        }

        match self {
            ImageDataType::EncodedFile(data) => Ok(dimensions_for_data(data)?),
            ImageDataType::EncodedLease(lease) => Ok(dimensions_for_data(&lease.get_data()?)?),
            ImageDataType::AnimRgba8 { width, height, .. }
            | ImageDataType::Rgba8 { width, height, .. } => Ok((*width, *height)),
        }
    }

    /// Migrate an in-memory encoded image blob to on-disk to reduce
    /// the memory footprint
    pub fn swap_out(self) -> Result<Self, ImageCellError> {
        match self {
            Self::EncodedFile(data) => match BlobManager::store(&data) {
                Ok(lease) => Ok(Self::EncodedLease(lease)),
                Err(wezterm_blob_leases::Error::StorageNotInit) => Ok(Self::EncodedFile(data)),
                Err(err) => Err(err.into()),
            },
            other => Ok(other),
        }
    }

    /// Decode an encoded file into either an Rgba8 or AnimRgba8 variant
    /// if we recognize the file format, otherwise the EncodedFile data
    /// is preserved as is.
    #[cfg(feature = "use_image")]
    pub fn decode(self) -> Self {
        use image::{AnimationDecoder, ImageFormat};

        match self {
            Self::EncodedFile(data) => {
                let format = match image::guess_format(&data) {
                    Ok(format) => format,
                    Err(err) => {
                        log::warn!("Unable to decode raw image data: {:#}", err);
                        return Self::EncodedFile(data);
                    }
                };
                let cursor = std::io::Cursor::new(&*data);
                match format {
                    ImageFormat::Gif => image::codecs::gif::GifDecoder::new(cursor)
                        .and_then(|decoder| decoder.into_frames().collect_frames())
                        .and_then(|frames| {
                            if frames.is_empty() {
                                log::error!("decoded image has 0 frames, using placeholder");
                                Ok(Self::placeholder())
                            } else {
                                Ok(Self::decode_frames(frames))
                            }
                        })
                        .unwrap_or_else(|err| {
                            log::error!(
                                "Unable to parse animated gif: {:#}, trying as single frame",
                                err
                            );
                            Self::decode_single(data)
                        }),
                    ImageFormat::Png => {
                        let decoder = match image::codecs::png::PngDecoder::new(cursor) {
                            Ok(d) => d,
                            _ => return Self::EncodedFile(data),
                        };
                        if decoder.is_apng().unwrap_or(false) {
                            match decoder
                                .apng()
                                .and_then(|d| d.into_frames().collect_frames())
                            {
                                Ok(frames) if frames.is_empty() => {
                                    log::error!("decoded image has 0 frames, using placeholder");
                                    Self::placeholder()
                                }
                                Ok(frames) => Self::decode_frames(frames),
                                _ => Self::EncodedFile(data),
                            }
                        } else {
                            Self::decode_single(data)
                        }
                    }
                    ImageFormat::WebP => {
                        let decoder = match image::codecs::webp::WebPDecoder::new(cursor) {
                            Ok(d) => d,
                            _ => return Self::EncodedFile(data),
                        };
                        match decoder.into_frames().collect_frames() {
                            Ok(frames) if frames.is_empty() => {
                                log::error!("decoded image has 0 frames, using placeholder");
                                Self::placeholder()
                            }
                            Ok(frames) => Self::decode_frames(frames),
                            _ => Self::EncodedFile(data),
                        }
                    }
                    _ => Self::decode_single(data),
                }
            }
            data => data,
        }
    }

    #[cfg(not(feature = "use_image"))]
    pub fn decode(self) -> Self {
        self
    }

    #[cfg(feature = "use_image")]
    fn decode_frames(img_frames: Vec<image::Frame>) -> Self {
        let mut width = 0;
        let mut height = 0;
        let mut frames = vec![];
        let mut durations = vec![];
        let mut hashes = vec![];
        for frame in img_frames.into_iter() {
            let duration: Duration = frame.delay().into();
            durations.push(duration);
            let image = image::DynamicImage::ImageRgba8(frame.into_buffer()).to_rgba8();
            let (w, h) = image.dimensions();
            width = w;
            height = h;
            let data = image.into_vec();
            hashes.push(Self::hash_bytes(&data));
            frames.push(data);
        }
        Self::AnimRgba8 {
            width,
            height,
            frames,
            durations,
            hashes,
        }
    }

    #[cfg(feature = "use_image")]
    fn decode_single(data: Vec<u8>) -> Self {
        match image::load_from_memory(&data) {
            Ok(image) => {
                let image = image.to_rgba8();
                let (width, height) = image.dimensions();
                let data = image.into_vec();
                let hash = Self::hash_bytes(&data);
                Self::Rgba8 {
                    width,
                    height,
                    data,
                    hash,
                }
            }
            _ => Self::EncodedFile(data),
        }
    }
}
