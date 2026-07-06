pub fn paint_linear_gradient(
    context: &Context,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    mut color_line: ColorLine,
) -> anyhow::Result<()> {
    let (min_stop, max_stop) = normalize_color_line(&mut color_line);
    let anchors = reduce_anchors(ReduceAnchorsIn {
        x0,
        y0,
        x1,
        y1,
        x2,
        y2,
    });

    let xxx0 = anchors.xx0 + min_stop * (anchors.xx1 - anchors.xx0);
    let yyy0 = anchors.yy0 + min_stop * (anchors.yy1 - anchors.yy0);
    let xxx1 = anchors.xx0 + max_stop * (anchors.xx1 - anchors.xx0);
    let yyy1 = anchors.yy0 + max_stop * (anchors.yy1 - anchors.yy0);

    let pattern = LinearGradient::new(xxx0, yyy0, xxx1, yyy1);
    pattern.set_extend(color_line.extend);

    for stop in &color_line.color_stops {
        let (r, g, b, a) = stop.color.as_srgba_tuple();
        pattern.add_color_stop_rgba(stop.offset.into(), r.into(), g.into(), b.into(), a.into());
    }

    context.set_source(pattern)?;
    context.paint()?;

    Ok(())
}

pub fn paint_radial_gradient(
    context: &Context,
    x0: f64,
    y0: f64,
    r0: f64,
    x1: f64,
    y1: f64,
    r1: f64,
    mut color_line: ColorLine,
) -> anyhow::Result<()> {
    let (min_stop, max_stop) = normalize_color_line(&mut color_line);

    let xx0 = x0 + min_stop * (x1 - x0);
    let yy0 = y0 + min_stop * (y1 - y0);
    let xx1 = x0 + max_stop * (x1 - x0);
    let yy1 = y0 + max_stop * (y1 - y0);
    let rr0 = r0 + min_stop * (r1 - r0);
    let rr1 = r0 + max_stop * (r1 - r0);

    let pattern = RadialGradient::new(xx0, yy0, rr0, xx1, yy1, rr1);
    pattern.set_extend(color_line.extend);

    for stop in &color_line.color_stops {
        let (r, g, b, a) = stop.color.as_srgba_tuple();
        pattern.add_color_stop_rgba(stop.offset.into(), r.into(), g.into(), b.into(), a.into());
    }

    context.set_source(pattern)?;
    context.paint()?;

    Ok(())
}

#[derive(Copy, Clone, Debug)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn dot(&self, other: Self) -> f64 {
        (self.x * other.x) + (self.y * other.y)
    }

    fn normalize(self) -> Self {
        let len = self.dot(self).sqrt();
        Self {
            x: self.x / len,
            y: self.y / len,
        }
    }

    pub fn sum(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn difference(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn scale(self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }

    /// Compute a vector from the supplied angle
    pub fn from_angle(angle: f64) -> Self {
        let (y, x) = angle.sin_cos();
        Self { x, y }
    }
}

fn interpolate(f0: f64, f1: f64, f: f64) -> f64 {
    f0 + f * (f1 - f0)
}
