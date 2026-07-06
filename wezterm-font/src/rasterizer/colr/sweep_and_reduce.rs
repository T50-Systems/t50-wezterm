pub fn paint_sweep_gradient(
    context: &Context,
    x0: f64,
    y0: f64,
    start_angle: f64,
    end_angle: f64,
    color_line: ColorLine,
) -> anyhow::Result<()> {
    let (x1, y1, x2, y2) = context.clip_extents()?;

    let max_x = ((x1 - x0) * (x1 - x0)).max((x2 - x0) * (x2 - x0));
    let max_y = ((y1 - y0) * (y1 - y0)).max((y2 - y0) * (y2 - y0));
    let radius = (max_x + max_y).sqrt();

    let mesh = Mesh::new();
    let center = Point { x: x0, y: y0 };
    apply_sweep_gradient_patches(&mesh, color_line, center, radius, start_angle, end_angle);
    context.set_source(mesh)?;
    context.paint()?;

    Ok(())
}

fn normalize_color_line(color_line: &mut ColorLine) -> (f64, f64) {
    let mut smallest = color_line.color_stops[0].offset;
    let mut largest = smallest;

    color_line
        .color_stops
        .sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());

    for stop in &color_line.color_stops[1..] {
        smallest = smallest.min(stop.offset);
        largest = largest.max(stop.offset);
    }

    if smallest != largest {
        for stop in &mut color_line.color_stops {
            stop.offset = (stop.offset - smallest) / (largest - smallest);
        }
    }

    (smallest as f64, largest as f64)
}

struct ReduceAnchorsIn {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

struct ReduceAnchorsOut {
    xx0: f64,
    yy0: f64,
    xx1: f64,
    yy1: f64,
}

fn reduce_anchors(
    ReduceAnchorsIn {
        x0,
        y0,
        x1,
        y1,
        x2,
        y2,
    }: ReduceAnchorsIn,
) -> ReduceAnchorsOut {
    let q2x = x2 - x0;
    let q2y = y2 - y0;
    let q1x = x1 - x0;
    let q1y = y1 - y0;

    let s = q2x * q2x + q2y * q2y;
    if s < 0.000001 {
        return ReduceAnchorsOut {
            xx0: x0,
            yy0: y0,
            xx1: x1,
            yy1: y1,
        };
    }

    let k = (q2x * q1x + q2y * q1y) / s;
    ReduceAnchorsOut {
        xx0: x0,
        yy0: y0,
        xx1: x1 - k * q2x,
        yy1: y1 - k * q2y,
    }
}
