impl FromStr for SrgbaTuple {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Workaround <https://github.com/mazznoer/csscolorparser-rs/pull/7/files>
        if !s.is_ascii() {
            return Err(());
        }
        if s.len() > 0 && s.as_bytes()[0] == b'#' {
            // Probably `#RGB`

            let digits = (s.len() - 1) / 3;
            if 1 + (digits * 3) != s.len() {
                return Err(());
            }

            if digits == 0 || digits > 4 {
                // Max of 16 bits supported
                return Err(());
            }

            let mut chars = s.chars().skip(1);

            macro_rules! digit {
                () => {{
                    let mut component = 0u16;

                    for _ in 0..digits {
                        component = component << 4;

                        let nybble = match chars.next().unwrap().to_digit(16) {
                            Some(v) => v as u16,
                            None => return Err(()),
                        };
                        component |= nybble;
                    }

                    // From XParseColor, the `#` syntax takes the most significant
                    // bits and uses those for the color value.  That function produces
                    // 16-bit color components but we want 8-bit components so we shift
                    // or truncate the bits here depending on the number of digits
                    (match digits {
                        1 => (component << 4) as f32,
                        2 => component as f32,
                        3 => (component >> 4) as f32,
                        4 => (component >> 8) as f32,
                        _ => return Err(()),
                    }) / 255.0
                }};
            }
            Ok(Self(digit!(), digit!(), digit!(), 1.0))
        } else if let Some(value) = s.strip_prefix("rgb:") {
            let fields: Vec<&str> = value.split('/').collect();
            if fields.len() != 3 {
                return Err(());
            }

            let red = x_parse_color_component(fields[0])?;
            let green = x_parse_color_component(fields[1])?;
            let blue = x_parse_color_component(fields[2])?;
            Ok(Self(red, green, blue, 1.0))
        } else if let Some(value) = s.strip_prefix("rgba:") {
            let fields: Vec<&str> = value.split('/').collect();
            if fields.len() == 4 {
                let red = x_parse_color_component(fields[0])?;
                let green = x_parse_color_component(fields[1])?;
                let blue = x_parse_color_component(fields[2])?;
                let alpha = x_parse_color_component(fields[3])?;
                return Ok(Self(red, green, blue, alpha));
            }

            let fields: Vec<_> = s[5..].split_ascii_whitespace().collect();
            if fields.len() == 4 {
                fn field(s: &str) -> Result<f32, ()> {
                    if s.ends_with('%') {
                        let v: f32 = s[0..s.len() - 1].parse().map_err(|_| ())?;
                        Ok(v / 100.)
                    } else {
                        let v: f32 = s.parse().map_err(|_| ())?;
                        if v > 255.0 || v < 0. {
                            Err(())
                        } else {
                            Ok(v / 255.)
                        }
                    }
                }
                let r: f32 = field(fields[0])?;
                let g: f32 = field(fields[1])?;
                let b: f32 = field(fields[2])?;
                let a: f32 = field(fields[3])?;

                Ok(Self(r, g, b, a))
            } else {
                Err(())
            }
        } else if s.starts_with("hsl:") {
            let fields: Vec<_> = s[4..].split_ascii_whitespace().collect();
            if fields.len() == 3 {
                // Expected to be degrees in range 0-360, but we allow for negative and wrapping
                let h: i32 = fields[0].parse().map_err(|_| ())?;
                // Expected to be percentage in range 0-100
                let s: i32 = fields[1].parse().map_err(|_| ())?;
                // Expected to be percentage in range 0-100
                let l: i32 = fields[2].parse().map_err(|_| ())?;

                fn hsl_to_rgb(hue: i32, sat: i32, light: i32) -> (f32, f32, f32) {
                    let hue = hue % 360;
                    let hue = if hue < 0 { hue + 360 } else { hue } as f32;
                    let sat = sat as f32 / 100.;
                    let light = light as f32 / 100.;
                    let a = sat * light.min(1. - light);
                    let f = |n: f32| -> f32 {
                        let k = (n + hue / 30.) % 12.;
                        light - a * (k - 3.).min(9. - k).min(1.).max(-1.)
                    };
                    (f(0.), f(8.), f(4.))
                }

                let (r, g, b) = hsl_to_rgb(h, s, l);
                Ok(Self(r, g, b, 1.0))
            } else {
                Err(())
            }
        } else {
            #[cfg(feature = "std")]
            {
                if let Ok(c) = csscolorparser::parse(s) {
                    return Ok(Self(c.r as f32, c.g as f32, c.b as f32, c.a as f32));
                }
            }
            Self::from_named(s).ok_or(())
        }
    }
}
