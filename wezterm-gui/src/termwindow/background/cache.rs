struct CachedGradient {
    g: Gradient,
    width: u32,
    height: u32,
    image: Arc<ImageData>,
    marked: bool,
}

impl CachedGradient {
    fn compute(g: &Gradient, width: u32, height: u32) -> anyhow::Result<Arc<ImageData>> {
        let grad = g
            .build()
            .with_context(|| format!("building gradient {:?}", g))?;

        let mut imgbuf = image::RgbaImage::new(width, height);
        let fw = width as f64;
        let fh = height as f64;

        fn to_pixel(c: colorgrad::Color) -> image::Rgba<u8> {
            image::Rgba(c.to_rgba8())
        }

        // Map t which is in range [a, b] to range [c, d]
        fn remap(t: f64, a: f64, b: f64, c: f64, d: f64) -> f64 {
            (t - a) * ((d - c) / (b - a)) + c
        }

        let (dmin, dmax) = grad.domain();

        let mut rng = fastrand::Rng::new();

        // We add some randomness to the position that we use to
        // index into the color gradient, so that we can avoid
        // visible color banding.  The default 64 was selected
        // because it it was the smallest value on my mac where
        // the banding wasn't obvious.
        let noise_amount = g.noise.unwrap_or_else(|| {
            if matches!(g.orientation, GradientOrientation::Radial { .. }) {
                16
            } else {
                64
            }
        });

        fn noise(rng: &mut fastrand::Rng, noise_amount: usize) -> f64 {
            if noise_amount == 0 {
                0.
            } else {
                rng.usize(0..noise_amount) as f64 * -1.
            }
        }

        match g.orientation {
            GradientOrientation::Horizontal => {
                for (x, _, pixel) in imgbuf.enumerate_pixels_mut() {
                    *pixel = to_pixel(grad.at(remap(
                        x as f64 + noise(&mut rng, noise_amount),
                        0.0,
                        fw,
                        dmin,
                        dmax,
                    )));
                }
            }
            GradientOrientation::Vertical => {
                for (_, y, pixel) in imgbuf.enumerate_pixels_mut() {
                    *pixel = to_pixel(grad.at(remap(
                        y as f64 + noise(&mut rng, noise_amount),
                        0.0,
                        fh,
                        dmin,
                        dmax,
                    )));
                }
            }
            GradientOrientation::Linear { angle } => {
                let angle = angle.unwrap_or(0.0).to_radians();
                for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
                    let (x, y) = (x as f64, y as f64);
                    let (x, y) = (x - fw / 2., y - fh / 2.);
                    let t = x * f64::cos(angle) - y * f64::sin(angle);
                    *pixel = to_pixel(grad.at(remap(
                        t + noise(&mut rng, noise_amount),
                        -fw / 2.,
                        fw / 2.,
                        dmin,
                        dmax,
                    )));
                }
            }
            GradientOrientation::Radial { radius, cx, cy } => {
                let radius = fw * radius.unwrap_or(0.5);
                let cx = fw * cx.unwrap_or(0.5);
                let cy = fh * cy.unwrap_or(0.5);

                for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
                    let x = x as f64;
                    let y = y as f64;

                    // If we are close to the center, stop applying noise,
                    // as the noise can wrap around and start using the
                    // color from the other end of the gradient and look weird
                    let nx = if ((cx - x).abs() as usize) < noise_amount {
                        0.
                    } else {
                        noise(&mut rng, noise_amount)
                    };
                    let ny = if ((cy - y).abs() as usize) < noise_amount {
                        0.
                    } else {
                        noise(&mut rng, noise_amount)
                    };

                    let t = (nx + (x - cx).powi(2) + (ny + y - cy).powi(2)).sqrt() / radius;
                    *pixel = to_pixel(grad.at(t));
                }
            }
        }

        let data = imgbuf.into_vec();
        let image = Arc::new(ImageData::with_data(ImageDataType::new_single_frame(
            width, height, data,
        )));

        Ok(image)
    }

    fn load(g: &Gradient, width: u32, height: u32) -> anyhow::Result<Arc<ImageData>> {
        let mut cache = GRADIENT_CACHE.lock().unwrap();

        if let Some(entry) = cache
            .iter_mut()
            .find(|entry| entry.g == *g && entry.width == width && entry.height == height)
        {
            entry.marked = false;
            return Ok(Arc::clone(&entry.image));
        }

        let image = Self::compute(g, width, height)?;

        cache.push(Self {
            g: g.clone(),
            width,
            height,
            image: Arc::clone(&image),
            marked: false,
        });
        Ok(image)
    }

    fn mark() {
        let mut cache = GRADIENT_CACHE.lock().unwrap();
        for entry in cache.iter_mut() {
            entry.marked = true;
        }
    }

    fn sweep() {
        let mut cache = GRADIENT_CACHE.lock().unwrap();
        cache.retain(|entry| !entry.marked);
    }
}

struct CachedImage {
    modified: SystemTime,
    image: Arc<ImageData>,
    marked: bool,
    speed: f32,
}

impl CachedImage {
    fn load(path: &str, speed: f32) -> anyhow::Result<Arc<ImageData>> {
        let modified = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .with_context(|| format!("getting metadata for {}", path))?;
        let mut cache = IMAGE_CACHE.lock().unwrap();
        if let Some(cached) = cache.get_mut(path) {
            if cached.modified == modified && cached.speed == speed {
                cached.marked = false;
                return Ok(Arc::clone(&cached.image));
            }
        }

        let data = std::fs::read(path)
            .with_context(|| format!("Failed to load window_background_image {}", path))?;
        log::trace!("loaded {}", path);
        let mut data = ImageDataType::EncodedFile(data);
        data.adjust_speed(speed);
        let image = Arc::new(ImageData::with_data(data));

        cache.insert(
            path.to_string(),
            Self {
                modified,
                image: Arc::clone(&image),
                marked: false,
                speed,
            },
        );

        Ok(image)
    }

    fn mark() {
        let mut cache = IMAGE_CACHE.lock().unwrap();
        for entry in cache.values_mut() {
            entry.marked = true;
        }
    }

    fn sweep() {
        let mut cache = IMAGE_CACHE.lock().unwrap();
        cache.retain(|k, entry| {
            if entry.marked {
                log::trace!("Unloading {} from cache", k);
            }
            !entry.marked
        });
    }
}
