#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn named_rgb() {
        let dark_green = SrgbaTuple::from_named("DarkGreen").unwrap();
        assert_eq!(dark_green.to_rgb_string(), "#006400");
    }

    #[test]
    fn from_hsl() {
        let foo = SrgbaTuple::from_str("hsl:235 100  50").unwrap();
        assert_eq!(foo.to_rgb_string(), "#0015ff");
    }

    #[test]
    fn from_rgba() {
        assert_eq!(
            SrgbaTuple::from_str("clear").unwrap().to_rgba_string(),
            "rgba(0% 0% 0% 0%)"
        );
        assert_eq!(
            SrgbaTuple::from_str("rgba:100% 0 0 50%")
                .unwrap()
                .to_rgba_string(),
            "rgba(100% 0% 0% 50%)"
        );
    }

    #[test]
    fn from_css() {
        assert_eq!(
            SrgbaTuple::from_str("rgb(255,0,0)")
                .unwrap()
                .to_rgb_string(),
            "#ff0000"
        );

        let rgba = SrgbaTuple::from_str("rgba(255,0,0,1)").unwrap();
        let round_trip = SrgbaTuple::from_str(&rgba.to_rgba_string()).unwrap();
        assert_eq!(rgba, round_trip);
        assert_eq!(rgba.to_rgba_string(), "rgba(100% 0% 0% 100%)");
    }

    #[test]
    fn from_rgb() {
        assert!(SrgbaTuple::from_str("").is_err());
        assert!(SrgbaTuple::from_str("#xyxyxy").is_err());

        let foo = SrgbaTuple::from_str("#f00f00f00").unwrap();
        assert_eq!(foo.to_rgb_string(), "#f0f0f0");

        let black = SrgbaTuple::from_str("#000").unwrap();
        assert_eq!(black.to_rgb_string(), "#000000");

        let black = SrgbaTuple::from_str("#FFF").unwrap();
        assert_eq!(black.to_rgb_string(), "#f0f0f0");

        let black = SrgbaTuple::from_str("#000000").unwrap();
        assert_eq!(black.to_rgb_string(), "#000000");

        let grey = SrgbaTuple::from_str("rgb:D6/D6/D6").unwrap();
        assert_eq!(grey.to_rgb_string(), "#d6d6d6");

        let grey = SrgbaTuple::from_str("rgb:f0f0/f0f0/f0f0").unwrap();
        assert_eq!(grey.to_rgb_string(), "#f0f0f0");
    }

    #[test]
    fn linear_rgb_contrast_ratio() {
        let a = LinearRgba::with_srgba(255, 0, 0, 1);
        let b = LinearRgba::with_srgba(0, 255, 0, 1);
        let contrast_ratio = a.contrast_ratio(&b);
        assert!(
            (2.91 - contrast_ratio).abs() < 0.01,
            "contrast({}) == 2.91",
            contrast_ratio
        );
    }

    #[test]
    fn srgba_contrast_ratio() {
        let a = SrgbaTuple::from_str("hsl:0   100  50").unwrap();
        let b = SrgbaTuple::from_str("hsl:120 100  50").unwrap();
        let contrast_ratio = a.contrast_ratio(&b);
        assert!(
            (2.91 - contrast_ratio).abs() < 0.01,
            "contrast({}) == 2.91",
            contrast_ratio
        );
    }
}
