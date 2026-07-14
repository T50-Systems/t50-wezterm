use super::*;

pub(super) fn from_char_part6(c: u32) -> Option<BlockKey> {
    Some(match c {
        // [╢] BOX DRAWINGS VERTICAL DOUBLE AND LEFT SINGLE
        0x2562 => BlockKey::Poly(&[
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
                    PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(
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
        // [╣] BOX DRAWINGS DOUBLE VERTICAL AND LEFT
        0x2563 => BlockKey::Poly(&[
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
        // [╤] BOX DRAWINGS DOWN SINGLE AND HORIZONTAL DOUBLE
        0x2564 => BlockKey::Poly(&[
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
                        BlockCoord::Frac(1, 2),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
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
        ]),
        // [╥] BOX DRAWINGS DOWN DOUBLE AND HORIZONTAL SINGLE
        0x2565 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
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
        // [╦] BOX DRAWINGS DOUBLE DOWN AND HORIZONTAL
        0x2566 => BlockKey::Poly(&[
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
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
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
                        BlockCoord::One,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
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
        // [╧] BOX DRAWINGS UP SINGLE AND HORIZONTAL DOUBLE
        0x2567 => BlockKey::Poly(&[
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
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::Zero,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::Frac(1, 2),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
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
        ]),
        // [╨] BOX DRAWINGS UP DOUBLE AND HORIZONTAL SINGLE
        0x2568 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
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
        // [╩] BOX DRAWINGS DOUBLE UP AND HORIZONTAL
        0x2569 => BlockKey::Poly(&[
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
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::Zero,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                        BlockCoord::Zero,
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(
                        BlockCoord::One,
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
                    ),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(1)),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-1)),
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
        // [╪] BOX DRAWINGS VERTICAL SINGLE AND HORIZONTAL DOUBLE
        0x256a => BlockKey::Poly(&[
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
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╫] BOX DRAWINGS VERTICAL DOUBLE AND HORIZONTAL SINGLE
        0x256b => BlockKey::Poly(&[
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
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),

        _ => return None,
    })
}
