use super::*;

const FULL_RECT: &[Poly] = &[Poly {
    path: &[
        PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
        PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
        PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
        PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
        PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Zero),
    ],
    intensity: BlockAlpha::Full,
    style: PolyStyle::Fill,
}];

#[test]
fn maps_representative_chars_to_block_keys() {
    assert_eq!(
        BlockKey::from_char('\u{2580}'),
        Some(BlockKey::Blocks(&[Block::UpperBlock(4)]))
    );
    assert_eq!(BlockKey::from_char('\u{28ff}'), Some(BlockKey::Braille(0xff)));
    assert_eq!(
        BlockKey::from_char('\u{ee03}'),
        Some(BlockKey::Progress(ProgressChunk::LEFT | ProgressChunk::FULL))
    );
    assert_eq!(
        BlockKey::from_char('\u{f5d6}'),
        Some(BlockKey::Branches(Branch::RIGHT_TO_DOWN))
    );
}

#[test]
fn rasterizes_upper_half_block_deterministically() {
    let glyph = rasterize_block(
        BlockKey::Blocks(&[Block::UpperBlock(4)]),
        &RasterizeGlyphParams {
            underline_height: 1,
            cell_size: CellSize::new(8, 8),
            anti_alias: false,
        },
    );

    let mut expected = vec![0u8; 8 * 8 * 4];
    for y in 0..4 {
        for x in 0..8 {
            let idx = (y * 8 + x) * 4;
            expected[idx..idx + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }

    assert_eq!(glyph.width, 8);
    assert_eq!(glyph.height, 8);
    assert_eq!(glyph.data, expected);
}

#[test]
fn poly_with_custom_metrics_overrides_dimensions() {
    let params = RasterizeGlyphParams {
        underline_height: 1,
        cell_size: CellSize::new(3, 3),
        anti_alias: false,
    };
    let custom = BlockKeyMetrics {
        underline_height: 2,
        cell_size: CellSize::new(6, 4),
    };

    let via_variant = rasterize_block(
        BlockKey::PolyWithCustomMetrics {
            polys: FULL_RECT,
            metrics: custom,
        },
        &params,
    );
    let direct = rasterize_polys(
        FULL_RECT,
        &RasterizeGlyphParams {
            underline_height: custom.underline_height,
            cell_size: custom.cell_size,
            anti_alias: params.anti_alias,
        },
    );

    assert_eq!((via_variant.width, via_variant.height), (6, 4));
    assert_eq!(via_variant, direct);
    assert!(via_variant.data.iter().any(|&byte| byte != 0));
}
