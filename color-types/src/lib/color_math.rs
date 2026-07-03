/// Convert an RGB color space hue angle to an RYB colorspace hue angle
/// <https://github.com/TNMEM/Material-Design-Color-Picker/blob/1afe330c67d9db4deef7031d601324b538b43b09/rybcolor.js#L33>
#[cfg(feature = "std")]
fn rgb_hue_to_ryb_hue(hue: f64) -> f64 {
    if hue < 35. {
        map_range(hue, 0., 35., 0., 60.)
    } else if hue < 60. {
        map_range(hue, 35., 60., 60., 122.)
    } else if hue < 120. {
        map_range(hue, 60., 120., 122., 165.)
    } else if hue < 180. {
        map_range(hue, 120., 180., 165., 218.)
    } else if hue < 240. {
        map_range(hue, 180., 240., 218., 275.)
    } else if hue < 300. {
        map_range(hue, 240., 300., 275., 330.)
    } else {
        map_range(hue, 300., 360., 330., 360.)
    }
}

/// Convert an RYB color space hue angle to an RGB colorspace hue angle
#[cfg(feature = "std")]
fn ryb_huge_to_rgb_hue(hue: f64) -> f64 {
    if hue < 60. {
        map_range(hue, 0., 60., 0., 35.)
    } else if hue < 122. {
        map_range(hue, 60., 122., 35., 60.)
    } else if hue < 165. {
        map_range(hue, 122., 165., 60., 120.)
    } else if hue < 218. {
        map_range(hue, 165., 218., 120., 180.)
    } else if hue < 275. {
        map_range(hue, 218., 275., 180., 240.)
    } else if hue < 330. {
        map_range(hue, 275., 330., 240., 300.)
    } else {
        map_range(hue, 330., 360., 300., 360.)
    }
}

#[cfg(feature = "std")]
fn map_range(x: f64, x1: f64, x2: f64, y1: f64, y2: f64) -> f64 {
    let a_slope = (y2 - y1) / (x2 - x1);
    let a_slope_intercept = y1 - (a_slope * x1);
    x * a_slope + a_slope_intercept
}

#[cfg(feature = "std")]
fn normalize_angle(t: f64) -> f64 {
    let mut t = t % 360.0;
    if t < 0.0 {
        t += 360.0;
    }
    t
}

#[cfg(feature = "std")]
fn apply_scale(current: f64, factor: f64) -> f64 {
    let difference = if factor >= 0. { 1.0 - current } else { current };
    let delta = difference.max(0.) * factor;
    (current + delta).max(0.)
}

#[cfg(feature = "std")]
fn apply_fixed(current: f64, amount: f64) -> f64 {
    (current + amount).max(0.)
}

impl Hash for SrgbaTuple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_ne_bytes().hash(state);
        self.1.to_ne_bytes().hash(state);
        self.2.to_ne_bytes().hash(state);
        self.3.to_ne_bytes().hash(state);
    }
}

impl Eq for SrgbaTuple {}

fn x_parse_color_component(value: &str) -> Result<f32, ()> {
    let mut component = 0u16;
    let mut num_digits = 0;

    for c in value.chars() {
        num_digits += 1;
        component = component << 4;

        let nybble = match c.to_digit(16) {
            Some(v) => v as u16,
            None => return Err(()),
        };
        component |= nybble;
    }

    // From XParseColor, the `rgb:` prefixed syntax scales the
    // value into 16 bits from the number of bits specified
    Ok((match num_digits {
        1 => (component | component << 4) as f32,
        2 => component as f32,
        3 => (component >> 4) as f32,
        4 => (component >> 8) as f32,
        _ => return Err(()),
    }) / 255.0)
}
