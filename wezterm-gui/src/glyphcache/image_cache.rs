impl GlyphCache {
    fn cached_image_impl(
        frame_cache: &mut HashMap<[u8; 32], Sprite>,
        atlas: &mut Atlas,
        decoded: &DecodedImage,
        padding: Option<usize>,
        min_frame_duration: Duration,
        allow_image: AllowImage,
    ) -> anyhow::Result<(Sprite, Option<Instant>, LoadState)> {
        let mut handle = DecodedImageHandle {
            h: decoded.image.data(),
            current_frame: *decoded.current_frame.borrow(),
        };

        let scale_down = match allow_image {
            AllowImage::Scale(n) => Some(n),
            _ => None,
        };

        match &*handle.h {
            ImageDataType::Rgba8 { hash, .. } => {
                if let Some(sprite) = frame_cache.get(hash) {
                    return Ok((sprite.clone(), None, LoadState::Loaded));
                }
                let sprite = atlas
                    .allocate_with_padding(&handle, padding, scale_down)
                    .context("atlas.allocate_with_padding")?;
                frame_cache.insert(*hash, sprite.clone());

                return Ok((sprite, None, LoadState::Loaded));
            }
            ImageDataType::AnimRgba8 {
                hashes,
                frames,
                durations,
                ..
            } => {
                let mut next = None;
                let mut decoded_frame_start = decoded.frame_start.borrow_mut();
                let mut decoded_current_frame = decoded.current_frame.borrow_mut();
                if frames.len() > 1 {
                    let now = Instant::now();

                    // We round up the frame duration to at least the minimum
                    // frame duration that wezterm can use when rendering.
                    // There's no point trying to deal with smaller intervals
                    // because we simply cannot render them without dropping
                    // frames.
                    // In addition, with a 1ms frame delay, there's a good chance
                    // that any given cell may switch to a different frame from
                    // its neighbor while we are rendering the entire terminal
                    // frame, so we want to avoid that.
                    // <https://github.com/wezterm/wezterm/issues/3260>
                    let mut next_due = *decoded_frame_start
                        + durations[*decoded_current_frame].max(min_frame_duration);
                    if now >= next_due {
                        // Advance to next frame
                        *decoded_current_frame = *decoded_current_frame + 1;
                        if *decoded_current_frame >= frames.len() {
                            *decoded_current_frame = 0;
                            // Skip potential 0-duration root frame
                            if durations[0].as_millis() == 0 && frames.len() > 1 {
                                *decoded_current_frame = *decoded_current_frame + 1;
                            }
                        }
                        *decoded_frame_start = now;
                        next_due = *decoded_frame_start
                            + durations[*decoded_current_frame].max(min_frame_duration);
                        handle.current_frame = *decoded_current_frame;
                    }

                    next.replace(next_due);
                }

                let hash = hashes[*decoded_current_frame];

                if let Some(sprite) = frame_cache.get(&hash) {
                    return Ok((sprite.clone(), next, LoadState::Loaded));
                }

                let sprite = atlas
                    .allocate_with_padding(&handle, padding, scale_down)
                    .context("atlas.allocate_with_padding")?;

                frame_cache.insert(hash, sprite.clone());

                return Ok((
                    sprite,
                    Some(
                        *decoded_frame_start
                            + durations[*decoded_current_frame].max(min_frame_duration),
                    ),
                    LoadState::Loaded,
                ));
            }
            ImageDataType::EncodedLease(_) | ImageDataType::EncodedFile(_) => {
                let mut frames = decoded.frames.borrow_mut();
                let frames = frames.as_mut().expect("to have frames");

                let mut next = None;
                let mut decoded_frame_start = decoded.frame_start.borrow_mut();
                let mut decoded_current_frame = decoded.current_frame.borrow_mut();

                // Wait up to the approx limit of human tolerable delay for
                // the first frame to be decoded, so that we can avoid showing
                // a flash of the black frame in the common case
                let max_duration = Duration::from_millis(125).max(min_frame_duration);
                if let Some(remain) = max_duration.checked_sub(decoded_frame_start.elapsed()) {
                    frames.wait_for_first_frame(remain);
                }

                let now = Instant::now();
                // We round up the frame duration to at least the minimum
                // frame duration that wezterm can use when rendering.
                // There's no point trying to deal with smaller intervals
                // because we simply cannot render them without dropping
                // frames.
                // In addition, with a 1ms frame delay, there's a good chance
                // that any given cell may switch to a different frame from
                // its neighbor while we are rendering the entire terminal
                // frame, so we want to avoid that.
                // <https://github.com/wezterm/wezterm/issues/3260>
                let mut next_due =
                    *decoded_frame_start + frames.frame_duration().max(min_frame_duration);
                if now >= next_due {
                    // Advance to next frame
                    if frames.load_next_frame() {
                        *decoded_current_frame = *decoded_current_frame + 1;
                        *decoded_frame_start = now;
                        next_due =
                            *decoded_frame_start + frames.frame_duration().max(min_frame_duration);
                        handle.current_frame = *decoded_current_frame;
                    }
                }

                next.replace(next_due);

                let hash = frames.frame_hash();

                if let Some(sprite) = frame_cache.get(&hash) {
                    return Ok((sprite.clone(), next, frames.load_state));
                }

                let expected_byte_size =
                    frames.current_frame.width * frames.current_frame.height * 4;

                let frame_data = match frames.current_frame.lease.get_data() {
                    Ok(data) => {
                        // If the size isn't right, ignore this frame and replace
                        // it with a blank one instead. This might happen if
                        // some process is truncating the files, or perhaps if
                        // the disk is full.
                        // We need to check for this because the consequence of
                        // a mismatched size is a panic in a layer where we
                        // cannot handle the error case.
                        if data.len() != expected_byte_size {
                            report_frame_error(format!("frame data is corrupted: expected size {expected_byte_size} but have {}", data.len()));
                            vec![0u8; expected_byte_size]
                        } else {
                            data
                        }
                    }
                    Err(err) => {
                        report_frame_error(format!("frame data error: {err:#}"));
                        vec![0u8; expected_byte_size]
                    }
                };

                let frame = Image::from_raw(
                    frames.current_frame.width,
                    frames.current_frame.height,
                    frame_data,
                );
                let sprite = atlas.allocate_with_padding(&frame, padding, scale_down)?;

                frame_cache.insert(hash, sprite.clone());

                Ok((
                    sprite,
                    Some(*decoded_frame_start + frames.frame_duration().max(min_frame_duration)),
                    frames.load_state,
                ))
            }
        }
    }

    pub fn cached_image(
        &mut self,
        image_data: &Arc<ImageData>,
        padding: Option<usize>,
        allow_image: AllowImage,
    ) -> anyhow::Result<(Sprite, Option<Instant>, LoadState)> {
        let hash = image_data.hash();

        if let Some(decoded) = self.image_cache.get(&hash) {
            Self::cached_image_impl(
                &mut self.frame_cache,
                &mut self.atlas,
                decoded,
                padding,
                self.min_frame_duration,
                allow_image,
            )
        } else {
            let decoded = DecodedImage::load(image_data);
            let res = Self::cached_image_impl(
                &mut self.frame_cache,
                &mut self.atlas,
                &decoded,
                padding,
                self.min_frame_duration,
                allow_image,
            )?;
            self.image_cache.put(hash, decoded);
            Ok(res)
        }
    }
}
