use super::*;

pub(super) fn from_char_part4(c: u32) -> Option<BlockKey> {
    Some(match c {
        // [╄] BOX DRAWINGS RIGHT UP HEAVY and LEFT DOWN LIGHT
        0x2544 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╅] BOX DRAWINGS LEFT DOWN HEAVY and RIGHT UP LIGHT
        0x2545 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╆] BOX DRAWINGS RIGHT DOWN HEAVY and LEFT UP LIGHT
        0x2546 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╇] BOX DRAWINGS DOWN LIGHT AND UP HORIZONTAL HEAVY
        0x2547 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [╈] BOX DRAWINGS UP LIGHT AND DOWN HORIZONTAL HEAVY
        0x2548 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [╉] BOX DRAWINGS RIGHT LIGHT AND LEFT VERTICAL HEAVY
        0x2549 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [╊] BOX DRAWINGS LEFT LIGHT AND RIGHT VERTICAL HEAVY
        0x254a => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [╋] BOX DRAWINGS HEAVY VERTICAL AND HORIZONTAL
        0x254b => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::OutlineHeavy,
        }]),

        // [╌] BOX DRAWINGS LIGHT DOUBLE DASH HORIZONTAL
        // A dash segment is wider than the gap segment.
        // We use a 2:1 ratio, which gives 6 total segments
        // with a pattern of `-- -- `
        0x254c => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(2, 6), BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(3, 6), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(5, 6), BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╍] BOX DRAWINGS HEAVY DOUBLE DASH HORIZONTAL
        0x254d => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(2, 6), BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(3, 6), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(5, 6), BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [╎] BOX DRAWINGS LIGHT DOUBLE DASH VERTICAL
        0x254e => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(2, 6)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(3, 6)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(5, 6)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╏] BOX DRAWINGS HEAVY DOUBLE DASH VERTICAL
        0x254f => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(2, 6)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(3, 6)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(5, 6)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),

        // [═] BOX DRAWINGS DOUBLE HORIZONTAL
        0x2550 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::Zero,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::One,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::Zero,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::One,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [║] BOX DRAWINGS DOUBLE VERTICAL
        0x2551 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::Zero,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::One,
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::Zero,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::One,
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╒] BOX DRAWINGS DOWN SINGLE AND RIGHT DOUBLE
        0x2552 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                    PolyCommand::LineTo(
                        BlockCoord::Frac(1, 2),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::One,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::Frac(1, 2),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::One,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╓] BOX DRAWINGS DOWN DOUBLE AND RIGHT SINGLE
        0x2553 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::One,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::Frac(1, 2),
                    ),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::Frac(1, 2),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::One,
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),

        // [╔] BOX DRAWINGS DOUBLE DOWN AND RIGHT
        0x2554 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::One,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::One,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::One,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::One,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╕] BOX DRAWINGS DOWN SINGLE AND LEFT DOUBLE
        0x2555 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                    PolyCommand::LineTo(
                        BlockCoord::Frac(1, 2),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::Zero,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::Frac(1, 2),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::Zero,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        _ => return None,
    })
}
