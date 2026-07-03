#[test]
fn diff_screens() {
    let mut s = Surface::new(4, 3);
    s.add_change("w00t");
    s.add_change("foo");
    s.add_change("baar");
    s.add_change("baz");
    assert_eq!(
        s.screen_chars_to_string(),
        "foob\n\
             aarb\n\
             az  \n"
    );

    let s2 = Surface::new(2, 2);

    {
        // We want to sample the top left corner
        let changes = s2.diff_region(0, 0, 2, 2, &s, 0, 0);
        assert_eq!(
            vec![
                Change::CursorPosition {
                    x: Position::Absolute(0),
                    y: Position::Absolute(0),
                },
                Change::AllAttributes(CellAttributes::default()),
                Change::Text("fo".into()),
                Change::CursorPosition {
                    x: Position::Absolute(0),
                    y: Position::Absolute(1),
                },
                Change::Text("aa".into()),
            ],
            changes
        );
    }

    // Throw in some attribute changes too
    s.add_change(Change::CursorPosition {
        x: Position::Absolute(1),
        y: Position::Absolute(1),
    });
    s.add_change(Change::Attribute(AttributeChange::Intensity(
        Intensity::Bold,
    )));
    s.add_change("XO");

    {
        let changes = s2.diff_region(0, 0, 2, 2, &s, 1, 1);
        assert_eq!(
            vec![
                Change::CursorPosition {
                    x: Position::Absolute(0),
                    y: Position::Absolute(0),
                },
                Change::AllAttributes(
                    CellAttributes::default()
                        .set_intensity(Intensity::Bold)
                        .clone(),
                ),
                Change::Text("XO".into()),
                Change::CursorPosition {
                    x: Position::Absolute(0),
                    y: Position::Absolute(1),
                },
                Change::AllAttributes(CellAttributes::default()),
                Change::Text("z".into()),
                /* There's no change for the final character
                 * position because it is a space in both regions. */
            ],
            changes
        );
    }
}

#[test]
fn draw_screens() {
    let mut s = Surface::new(4, 4);

    let mut s1 = Surface::new(2, 2);
    s1.add_change("1234");

    let mut s2 = Surface::new(2, 2);
    s2.add_change("XYZA");

    s.draw_from_screen(&s1, 0, 0);
    s.draw_from_screen(&s2, 2, 2);

    assert_eq!(
        s.screen_chars_to_string(),
        "12  \n\
             34  \n\
             \x20\x20XY\n\
             \x20\x20ZA\n"
    );
}

#[test]
fn draw_colored_region() {
    let mut dest = Surface::new(4, 4);
    dest.add_change("A");
    let mut src = Surface::new(2, 2);
    src.add_change(Change::ClearScreen(AnsiColor::Blue.into()));
    dest.draw_from_screen(&src, 2, 2);

    assert_eq!(
        dest.screen_chars_to_string(),
        "A   \n\
             \x20   \n\
             \x20   \n\
             \x20   \n"
    );

    let blue_space = Cell::new(
        ' ',
        CellAttributes::default()
            .set_background(AnsiColor::Blue)
            .clone(),
    );

    assert_eq!(
        dest.screen_cells(),
        [
            [
                Cell::new('A', CellAttributes::default()),
                Cell::default(),
                Cell::default(),
                Cell::default(),
            ],
            [
                Cell::default(),
                Cell::default(),
                Cell::default(),
                Cell::default(),
            ],
            [
                Cell::default(),
                Cell::default(),
                blue_space.clone(),
                blue_space.clone(),
            ],
            [
                Cell::default(),
                Cell::default(),
                blue_space.clone(),
                blue_space.clone(),
            ]
        ]
    );

    assert_eq!(dest.xpos, 1);
    assert_eq!(dest.ypos, 0);
    assert_eq!(dest.attributes, Default::default());
    dest.add_change("B");

    assert_eq!(
        dest.screen_chars_to_string(),
        "AB  \n\
             \x20   \n\
             \x20   \n\
             \x20   \n"
    );
}

#[test]
fn copy_region() {
    let mut s = Surface::new(4, 3);
    s.add_change("w00t");
    s.add_change("foo");
    s.add_change("baar");
    s.add_change("baz");
    assert_eq!(
        s.screen_chars_to_string(),
        "foob\n\
             aarb\n\
             az  \n"
    );

    // Copy top left to bottom left
    s.copy_region(0, 0, 2, 2, 2, 1);
    assert_eq!(
        s.screen_chars_to_string(),
        "foob\n\
             aafo\n\
             azaa\n"
    );
}

#[test]
fn double_width() {
    let mut s = Surface::new(4, 1);
    s.add_change("🤷12");
    assert_eq!(s.screen_chars_to_string(), "🤷12\n");
    s.add_change(Change::CursorPosition {
        x: Position::Absolute(1),
        y: Position::Absolute(0),
    });
    s.add_change("a🤷");
    assert_eq!(s.screen_chars_to_string(), " a🤷\n");
    s.add_change(Change::CursorPosition {
        x: Position::Absolute(2),
        y: Position::Absolute(0),
    });
    s.add_change("x");
    assert_eq!(s.screen_chars_to_string(), " ax \n");
}

#[test]
fn draw_double_width() {
    let mut s = Surface::new(4, 1);
    s.add_change("か a");
    assert_eq!(s.screen_chars_to_string(), "か a\n");

    let mut s2 = Surface::new(4, 1);
    s2.draw_from_screen(&s, 0, 0);
    // Verify no issue when the second visible cells on both sides
    // are identical (' 's) but they are at different cell indices.
    assert_eq!(s2.screen_chars_to_string(), "か a\n");

    let s3 = Surface::new(4, 1);
    s2.draw_from_screen(&s3, 0, 0);
    // Verify same but in other direction
    assert_eq!(s2.screen_chars_to_string(), "    \n");

    let mut s4 = Surface::new(4, 1);
    s4.add_change("abcd");
    s.draw_from_screen(&s4, 0, 0);
    // Verify that all overlapping cells are updated when cell widths
    // differ on each side.
    assert_eq!(s.screen_chars_to_string(), "abcd\n");
}

#[test]
fn diff_cursor_double_width() {
    let mut s = Surface::new(3, 1);
    s.add_change("かa");

    let s2 = Surface::new(3, 1);
    let changes = s2.diff_region(0, 0, 3, 1, &s, 0, 0);

    assert_eq!(
        changes
            .iter()
            .filter(|change| matches!(change, Change::CursorPosition { .. }))
            .count(),
        1
    );
}

#[test]
fn zero_width() {
    let mut s = Surface::new(4, 1);
    // https://en.wikipedia.org/wiki/Zero-width_space
    s.add_change("A\u{200b}B");
    assert_eq!(s.screen_chars_to_string(), "A\u{200b}B \n");
}

#[test]
fn images() {
    // a dummy image blob with nonsense content
    let data = Arc::new(ImageData::with_raw_data(vec![]));
    let mut s = Surface::new(2, 2);
    s.add_change(Change::Image(Image {
        top_left: TextureCoordinate::new_f32(0.0, 0.0),
        bottom_right: TextureCoordinate::new_f32(1.0, 1.0),
        image: data.clone(),
        width: 4,
        height: 2,
    }));

    // We're checking that we slice the image up and assign the correct
    // texture coordinates for each cell.  The width and height are
    // different from each other to help ensure that the right terms
    // are used by add_image() function.
    assert_eq!(
        s.screen_cells(),
        [
            [
                Cell::new(
                    ' ',
                    CellAttributes::default()
                        .set_image(Box::new(ImageCell::new(
                            TextureCoordinate::new_f32(0.0, 0.0),
                            TextureCoordinate::new_f32(0.25, 0.5),
                            data.clone()
                        )))
                        .clone()
                ),
                Cell::new(
                    ' ',
                    CellAttributes::default()
                        .set_image(Box::new(ImageCell::new(
                            TextureCoordinate::new_f32(0.25, 0.0),
                            TextureCoordinate::new_f32(0.5, 0.5),
                            data.clone()
                        )))
                        .clone()
                ),
                Cell::new(
                    ' ',
                    CellAttributes::default()
                        .set_image(Box::new(ImageCell::new(
                            TextureCoordinate::new_f32(0.5, 0.0),
                            TextureCoordinate::new_f32(0.75, 0.5),
                            data.clone()
                        )))
                        .clone()
                ),
                Cell::new(
                    ' ',
                    CellAttributes::default()
                        .set_image(Box::new(ImageCell::new(
                            TextureCoordinate::new_f32(0.75, 0.0),
                            TextureCoordinate::new_f32(1.0, 0.5),
                            data.clone()
                        )))
                        .clone()
                ),
            ],
            [
                Cell::new(
                    ' ',
                    CellAttributes::default()
                        .set_image(Box::new(ImageCell::new(
                            TextureCoordinate::new_f32(0.0, 0.5),
                            TextureCoordinate::new_f32(0.25, 1.0),
                            data.clone()
                        )))
                        .clone()
                ),
                Cell::new(
                    ' ',
                    CellAttributes::default()
                        .set_image(Box::new(ImageCell::new(
                            TextureCoordinate::new_f32(0.25, 0.5),
                            TextureCoordinate::new_f32(0.5, 1.0),
                            data.clone()
                        )))
                        .clone()
                ),
                Cell::new(
                    ' ',
                    CellAttributes::default()
                        .set_image(Box::new(ImageCell::new(
                            TextureCoordinate::new_f32(0.5, 0.5),
                            TextureCoordinate::new_f32(0.75, 1.0),
                            data.clone()
                        )))
                        .clone()
                ),
                Cell::new(
                    ' ',
                    CellAttributes::default()
                        .set_image(Box::new(ImageCell::new(
                            TextureCoordinate::new_f32(0.75, 0.5),
                            TextureCoordinate::new_f32(1.0, 1.0),
                            data.clone()
                        )))
                        .clone()
                ),
            ],
        ]
    );

    // Check that starting at not the texture origin coordinates
    // gives reasonable values in the resultant cell
    let mut other = Surface::new(1, 1);
    other.add_change(Change::Image(Image {
        top_left: TextureCoordinate::new_f32(0.25, 0.3),
        bottom_right: TextureCoordinate::new_f32(0.75, 0.8),
        image: data.clone(),
        width: 1,
        height: 1,
    }));
    assert_eq!(
        other.screen_cells(),
        [[Cell::new(
            ' ',
            CellAttributes::default()
                .set_image(Box::new(ImageCell::new(
                    TextureCoordinate::new_f32(0.25, 0.3),
                    TextureCoordinate::new_f32(0.75, 0.8),
                    data.clone()
                )))
                .clone()
        ),]]
    );
}
