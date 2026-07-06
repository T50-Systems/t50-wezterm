pub struct LoadedBackgroundLayer {
    pub source: Arc<ImageData>,
    pub def: BackgroundLayer,
}

fn load_background_layer(
    layer: &BackgroundLayer,
    dimensions: &Dimensions,
    render_metrics: &RenderMetrics,
) -> anyhow::Result<LoadedBackgroundLayer> {
    let h_context = DimensionContext {
        dpi: dimensions.dpi as f32,
        pixel_max: dimensions.pixel_width as f32,
        pixel_cell: render_metrics.cell_size.width as f32,
    };
    let v_context = DimensionContext {
        dpi: dimensions.dpi as f32,
        pixel_max: dimensions.pixel_height as f32,
        pixel_cell: render_metrics.cell_size.height as f32,
    };

    let data = match &layer.source {
        BackgroundSource::Gradient(g) => {
            let mut width = match layer.width {
                BackgroundSize::Dimension(d) => d.evaluate_as_pixels(h_context),
                unsup => anyhow::bail!(
                    "{unsup:?} is not implemented for background gradients. \
                     Use e.g. `width = '100%'` instead"
                ),
            } as u32;
            let mut height = match layer.height {
                BackgroundSize::Dimension(d) => d.evaluate_as_pixels(v_context),
                unsup => anyhow::bail!(
                    "{unsup:?} is not implemented for background gradients. \
                     Use e.g. `height = '100%'` instead"
                ),
            } as u32;

            if matches!(g.orientation, GradientOrientation::Radial { .. }) {
                // To simplify the math, we compute a perfect circle
                // for the radial gradient, and let the texture sampler
                // perturb it to fill the window
                width = width.min(height);
                height = height.min(width);
            }

            CachedGradient::load(g, width, height)?
        }
        BackgroundSource::Color(color) => {
            // In theory we could just make a 1x1 texture and allow
            // the shader to stretch it, but if we do that, it'll blend
            // around the edges and look weird.
            // So we make a square texture in the ballpark of the window
            // surface.
            // It's not ideal.
            let width = match layer.width {
                BackgroundSize::Dimension(d) => d.evaluate_as_pixels(h_context),
                unsup => anyhow::bail!(
                    "{unsup:?} is not implemented for background color. \
                     Use e.g. `width = '100%'` instead"
                ),
            } as u32;
            let height = match layer.height {
                BackgroundSize::Dimension(d) => d.evaluate_as_pixels(v_context),
                unsup => anyhow::bail!(
                    "{unsup:?} is not implemented for background color. \
                     Use e.g. `height = '100%'` instead"
                ),
            } as u32;

            let size = width.min(height);

            let mut imgbuf = image::RgbaImage::new(size, size);
            let src_pixel = {
                let (r, g, b, a) = color.to_srgb_u8();
                image::Rgba([r, g, b, a])
            };
            for (_x, _y, pixel) in imgbuf.enumerate_pixels_mut() {
                *pixel = src_pixel;
            }
            let data = imgbuf.into_vec();
            Arc::new(ImageData::with_data(ImageDataType::new_single_frame(
                size, size, data,
            )))
        }
        BackgroundSource::File(source) => CachedImage::load(&source.path, source.speed)?,
    };

    Ok(LoadedBackgroundLayer {
        source: data,
        def: layer.clone(),
    })
}

pub fn load_background_image(
    config: &ConfigHandle,
    dimensions: &Dimensions,
    render_metrics: &RenderMetrics,
) -> Vec<LoadedBackgroundLayer> {
    let mut layers = vec![];
    for layer in &config.background {
        let load_start = std::time::Instant::now();
        match load_background_layer(layer, dimensions, render_metrics) {
            Ok(layer) => {
                log::trace!("loaded layer in {:?}", load_start.elapsed());
                layers.push(layer);
            }
            Err(err) => {
                log::error!("Failed to load background: {:#}", err);
            }
        }
    }
    layers
}

pub fn reload_background_image(
    config: &ConfigHandle,
    existing: &[LoadedBackgroundLayer],
    dimensions: &Dimensions,
    render_metrics: &RenderMetrics,
) -> Vec<LoadedBackgroundLayer> {
    // We want to reuse the existing version of the image where possible
    // so that the textures we may have cached can be re-used and so that
    // animation state can be preserved across the reload.
    let map: HashMap<_, _> = existing
        .iter()
        .map(|layer| (layer.source.hash(), &layer.source))
        .collect();

    CachedImage::mark();
    CachedGradient::mark();

    let result = load_background_image(config, dimensions, render_metrics)
        .into_iter()
        .map(|mut layer| {
            let hash = layer.source.hash();

            if let Some(existing) = map.get(&hash) {
                layer.source = Arc::clone(existing);
            }

            layer
        })
        .collect();

    CachedImage::sweep();
    CachedGradient::sweep();

    result
}
