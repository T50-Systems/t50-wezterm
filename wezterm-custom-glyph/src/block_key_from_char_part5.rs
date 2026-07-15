use super::*;

pub(super) fn from_char_part5(c: u32) -> Option<BlockKey> {
    Some(match c {
        // [╖] BOX DRAWINGS DOWN DOUBLE AND LEFT SINGLE
        0x2556 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::One,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::Frac(1, 2),
                    ),
                    PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::Frac(1, 2),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::One,
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╗] BOX DRAWINGS DOUBLE DOWN AND LEFT
        0x2557 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::One,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
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
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::One,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
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
        // [╘] BOX DRAWINGS UP SINGLE AND RIGHT DOUBLE
        0x2558 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(
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
            Poly {
                path: &[
                    PolyCommand::MoveTo(
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
        ]),
        // [╙] BOX DRAWINGS UP DOUBLE AND RIGHT SINGLE
        0x2559 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::Zero,
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
                        BlockCoord::Zero,
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╚] BOX DRAWINGS DOUBLE UP AND RIGHT
        0x255a => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::Zero,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
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
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::Zero,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
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
        ]),
        // [╛] BOX DRAWINGS UP SINGLE AND LEFT DOUBLE
        0x255b => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(
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
            Poly {
                path: &[
                    PolyCommand::MoveTo(
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
        ]),
        // [╜] BOX DRAWINGS UP DOUBLE AND LEFT SINGLE
        0x255c => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::Zero,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::Frac(1, 2),
                    ),
                    PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::Frac(1, 2),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::Zero,
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╝] BOX DRAWINGS DOUBLE UP AND LEFT
        0x255d => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::Zero,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
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
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::Zero,
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
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

        // [╞] BOX DRAWINGS VERTICAL SINGLE AND RIGHT DOUBLE
        0x255e => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(
                        BlockCoord::Frac(1, 2),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::One,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::Frac(1, 2),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(
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
        ]),
        // [╟] BOX DRAWINGS VERTICAL DOUBLE AND RIGHT SINGLE
        0x255f => BlockKey::Poly(&[
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
                        BlockCoord::Frac(1, 2),
                    ),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(
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

        // [╠] BOX DRAWINGS DOUBLE VERTICAL AND RIGHT
        0x2560 => BlockKey::Poly(&[
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
        // [╡] BOX DRAWINGS VERTICAL SINGLE AND LEFT DOUBLE
        0x2561 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(
                        BlockCoord::Frac(1, 2),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::Zero,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::Frac(1, 2),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(
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
        ]),
        _ => return None,
    })
}
