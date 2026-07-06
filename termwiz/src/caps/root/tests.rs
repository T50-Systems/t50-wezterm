#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn version_cmp() {
        assert!(version_ge("1", "0"));
        assert!(version_ge("1.0", "0"));
        assert!(!version_ge("0", "1"));
        assert!(version_ge("3.2", "2.9"));
        assert!(version_ge("3.2.0beta5", "2.9"));
        assert!(version_ge("3.2.0beta5", "3.2.0"));
        assert!(version_ge("3.2.0beta5", "3.2.0beta1"));
    }

    fn load_terminfo() -> terminfo::Database {
        // Load our own compiled data so that the tests have an
        // environment that doesn't vary machine by machine.
        let data = include_bytes!("../../../data/xterm-256color");
        terminfo::Database::from_buffer(data.as_ref()).unwrap()
    }

    #[test]
    fn empty_hint() {
        let caps = Capabilities::new_with_hints(ProbeHints::default()).unwrap();

        assert_eq!(caps.color_level(), ColorLevel::Sixteen);
        assert_eq!(caps.sixel(), false);
        assert_eq!(caps.hyperlinks(), true);
        assert_eq!(caps.iterm2_image(), false);
        assert_eq!(caps.bce(), false);
    }

    #[test]
    fn bce() {
        let caps =
            Capabilities::new_with_hints(ProbeHints::default().colorterm_bce(Some("1".into())))
                .unwrap();

        assert_eq!(caps.bce(), true);
    }

    #[test]
    fn bce_terminfo() {
        let caps =
            Capabilities::new_with_hints(ProbeHints::default().terminfo_db(Some(load_terminfo())))
                .unwrap();

        assert_eq!(caps.bce(), true);
    }

    #[test]
    fn terminfo_color() {
        let caps =
            Capabilities::new_with_hints(ProbeHints::default().terminfo_db(Some(load_terminfo())))
                .unwrap();

        assert_eq!(caps.color_level(), ColorLevel::TrueColor);
    }

    #[test]
    fn term_but_not_colorterm() {
        let caps =
            Capabilities::new_with_hints(ProbeHints::default().term(Some("xterm-256color".into())))
                .unwrap();

        assert_eq!(caps.color_level(), ColorLevel::TwoFiftySix);
    }

    #[test]
    fn colorterm_but_no_term() {
        let caps =
            Capabilities::new_with_hints(ProbeHints::default().colorterm(Some("24bit".into())))
                .unwrap();

        assert_eq!(caps.color_level(), ColorLevel::TrueColor);
    }

    #[test]
    fn term_and_colorterm() {
        let caps = Capabilities::new_with_hints(
            ProbeHints::default()
                .term(Some("xterm-256color".into()))
                // bogus value
                .colorterm(Some("24bot".into())),
        )
        .unwrap();

        assert_eq!(caps.color_level(), ColorLevel::TwoFiftySix);

        let caps = Capabilities::new_with_hints(
            ProbeHints::default()
                .term(Some("xterm-256color".into()))
                .colorterm(Some("24bit".into())),
        )
        .unwrap();

        assert_eq!(caps.color_level(), ColorLevel::TrueColor);

        let caps = Capabilities::new_with_hints(
            ProbeHints::default()
                .term(Some("xterm-256color".into()))
                .colorterm(Some("truecolor".into())),
        )
        .unwrap();

        assert_eq!(caps.color_level(), ColorLevel::TrueColor);
    }

    #[test]
    fn iterm2_image() {
        let caps = Capabilities::new_with_hints(
            ProbeHints::default()
                .term_program(Some("iTerm.app".into()))
                .term_program_version(Some("1.0.0".into())),
        )
        .unwrap();
        assert_eq!(caps.iterm2_image(), false);

        let caps = Capabilities::new_with_hints(
            ProbeHints::default()
                .term_program(Some("iTerm.app".into()))
                .term_program_version(Some("2.9.0".into())),
        )
        .unwrap();
        assert_eq!(caps.iterm2_image(), false);

        let caps = Capabilities::new_with_hints(
            ProbeHints::default()
                .term_program(Some("iTerm.app".into()))
                .term_program_version(Some("2.9.20150512".into())),
        )
        .unwrap();
        assert_eq!(caps.iterm2_image(), true);

        let caps = Capabilities::new_with_hints(
            ProbeHints::default()
                .term_program(Some("iTerm.app".into()))
                .term_program_version(Some("3.2.0beta5".into())),
        )
        .unwrap();
        assert_eq!(caps.iterm2_image(), true);
    }
}
