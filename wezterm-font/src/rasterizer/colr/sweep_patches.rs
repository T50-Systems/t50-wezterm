#[derive(Debug)]
struct Patch {
    p0: Point,
    c0: Point,
    c1: Point,
    p1: Point,
    color0: SrgbaTuple,
    color1: SrgbaTuple,
}

impl Patch {
    fn add_to_mesh(&self, center: Point, mesh: &Mesh) {
        mesh.begin_patch();
        mesh.move_to(center.x, center.y);
        mesh.line_to(self.p0.x, self.p0.y);
        mesh.curve_to(
            self.c0.x, self.c0.y, self.c1.x, self.c1.y, self.p1.x, self.p1.y,
        );
        mesh.line_to(center.x, center.y);

        fn set_corner_color(mesh: &Mesh, corner: MeshCorner, color: SrgbaTuple) {
            let SrgbaTuple(r, g, b, a) = color;

            mesh.set_corner_color_rgba(corner, r.into(), g.into(), b.into(), a.into());
        }

        set_corner_color(mesh, MeshCorner::MeshCorner0, self.color0);
        set_corner_color(mesh, MeshCorner::MeshCorner1, self.color0);
        set_corner_color(mesh, MeshCorner::MeshCorner2, self.color1);
        set_corner_color(mesh, MeshCorner::MeshCorner3, self.color1);

        mesh.end_patch();
    }
}

fn add_sweep_gradient_patches(
    mesh: &Mesh,
    center: Point,
    radius: f64,
    a0: f64,
    c0: SrgbaTuple,
    a1: f64,
    c1: SrgbaTuple,
) {
    const MAX_ANGLE: f64 = std::f64::consts::PI / 8.;
    let num_splits = ((a1 - a0).abs() / MAX_ANGLE).ceil() as usize;

    let mut p0 = Point::from_angle(a0);
    let mut color0 = c0;

    for idx in 0..num_splits {
        let k = (idx as f64 + 1.) / num_splits as f64;

        let angle1 = interpolate(a0, a1, k);
        let color1 = c0.interpolate(c1, k);

        let p1 = Point::from_angle(angle1);

        let a = p0.sum(p1).normalize();
        let u = Point { x: -a.y, y: a.x };

        fn compute_control(a: Point, u: Point, p: Point, center: Point, radius: f64) -> Point {
            let c = a.sum(u.scale(p.difference(a).dot(p) / u.dot(p)));
            c.difference(p)
                .scale(0.33333)
                .sum(c)
                .scale(radius)
                .sum(center)
        }

        let patch = Patch {
            color0,
            color1,
            p0: center.sum(p0.scale(radius)),
            p1: center.sum(p1.scale(radius)),
            c0: compute_control(a, u, p0, center, radius),
            c1: compute_control(a, u, p1, center, radius),
        };

        patch.add_to_mesh(center, mesh);

        p0 = p1;
        color0 = color1;
    }
}

const PI_TIMES_2: f64 = std::f64::consts::PI * 2.;

fn apply_sweep_gradient_patches(
    mesh: &Mesh,
    mut color_line: ColorLine,
    center: Point,
    radius: f64,
    mut start_angle: f64,
    mut end_angle: f64,
) {
    if start_angle == end_angle {
        if color_line.extend == Extend::Pad {
            if start_angle > 0. {
                let c = color_line.color_stops[0].color.into();
                add_sweep_gradient_patches(mesh, center, radius, 0., c, start_angle, c);
            }
            if end_angle < PI_TIMES_2 {
                let c = color_line.color_stops.last().unwrap().color.into();
                add_sweep_gradient_patches(mesh, center, radius, end_angle, c, PI_TIMES_2, c);
            }
        }
        return;
    }

    if end_angle < start_angle {
        std::mem::swap(&mut start_angle, &mut end_angle);
        color_line.color_stops.reverse();
        for stop in &mut color_line.color_stops {
            stop.offset = 1.0 - stop.offset;
        }
    }

    let angles: Vec<f64> = color_line
        .color_stops
        .iter()
        .map(|stop| start_angle + stop.offset * (end_angle - start_angle))
        .collect();
    let colors: Vec<SrgbaTuple> = color_line
        .color_stops
        .iter()
        .map(|stop| stop.color.into())
        .collect();

    let n_stops = angles.len();

    if color_line.extend == Extend::Pad {
        let mut color0 = colors[0];
        let mut pos = 0;
        while pos < n_stops {
            if angles[pos] >= 0. {
                if pos > 0 {
                    let k = (0. - angles[pos - 1]) / (angles[pos] - angles[pos - 1]);

                    color0 = colors[pos - 1].interpolate(colors[pos], k);
                }
                break;
            }
            pos += 1;
        }
        if pos == n_stops {
            /* everything is below 0 */
            color0 = colors[n_stops - 1];
            add_sweep_gradient_patches(mesh, center, radius, 0., color0, PI_TIMES_2, color0);
            return;
        }

        add_sweep_gradient_patches(mesh, center, radius, 0., color0, angles[pos], colors[pos]);

        pos += 1;
        while pos < n_stops {
            if angles[pos] <= PI_TIMES_2 {
                add_sweep_gradient_patches(
                    mesh,
                    center,
                    radius,
                    angles[pos - 1],
                    colors[pos - 1],
                    angles[pos],
                    colors[pos],
                );
            } else {
                let k = (PI_TIMES_2 - angles[pos - 1]) / (angles[pos] - angles[pos - 1]);
                let color1 = colors[pos - 1].interpolate(colors[pos], k);
                add_sweep_gradient_patches(
                    mesh,
                    center,
                    radius,
                    angles[pos - 1],
                    colors[pos - 1],
                    PI_TIMES_2,
                    color1,
                );
                break;
            }
            pos += 1;
        }

        if pos == n_stops {
            /* everything is below 2*M_PI */
            color0 = colors[n_stops - 1];
            add_sweep_gradient_patches(
                mesh,
                center,
                radius,
                angles[n_stops - 1],
                color0,
                PI_TIMES_2,
                color0,
            );
            return;
        }
    } else {
        let span = angles[n_stops - 1] - angles[0];
        let mut k = 0isize;
        if angles[0] >= 0. {
            let mut ss = angles[0];
            while ss > 0. {
                if span > 0. {
                    ss -= span;
                    k -= 1;
                } else {
                    ss += span;
                    k += 1;
                }
            }
        } else if angles[0] < 0. {
            let mut ee = angles[n_stops - 1];
            while ee < 0. {
                if span > 0. {
                    ee += span;
                    k += 1;
                } else {
                    ee -= span;
                    k -= 1;
                }
            }
        }

        debug_assert!(
            angles[0] + (k as f64) * span <= 0. && 0. < angles[n_stops - 1] + (k as f64) * span
        );
        let span = span.abs();

        for l in k..k.min(1000) {
            for i in 1..n_stops {
                let (a0, a1, c0, c1);

                if l % 2 != 0 && color_line.extend == Extend::Reflect {
                    a0 = angles[0] + angles[n_stops - 1] - angles[n_stops - 1 - (i - 1)]
                        + (l as f64) * span;
                    a1 = angles[0] + angles[n_stops - 1] - angles[n_stops - 1 - i]
                        + (l as f64) * span;
                    c0 = colors[n_stops - 1 - (i - 1)];
                    c1 = colors[n_stops - 1 - i];
                } else {
                    a0 = angles[i - 1] + (l as f64) * span;
                    a1 = angles[i] + (l as f64) * span;
                    c0 = colors[i - 1];
                    c1 = colors[i];
                }

                if a1 < 0. {
                    continue;
                }

                if a0 < 0. {
                    let f = (0. - a0) / (a1 - a0);
                    let color = c0.interpolate(c1, f);
                    add_sweep_gradient_patches(mesh, center, radius, 0., color, a1, c1);
                } else if a1 >= PI_TIMES_2 {
                    let f = (PI_TIMES_2 - a0) / (a1 - a0);
                    let color = c0.interpolate(c1, f);
                    add_sweep_gradient_patches(mesh, center, radius, a0, c0, PI_TIMES_2, color);
                    return;
                } else {
                    add_sweep_gradient_patches(mesh, center, radius, a0, c0, a1, c1);
                }
            }
        }
    }
}
