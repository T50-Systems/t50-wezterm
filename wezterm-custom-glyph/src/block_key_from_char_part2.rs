use super::*;

pub(super) fn from_char_part2(c: u32) -> Option<BlockKey> {
    Some(match c {
        // [└] BOX DRAWINGS LIGHT UP AND RIGHT
        0x2514 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [┕] BOX DRAWINGS UP LIGHT AND RIGHT HEAVY
        0x2515 => BlockKey::Poly(&[
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
                    PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Div(-2)),
                        BlockCoord::Frac(1, 2),
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [┖] BOX DRAWINGS UP HEAVY AND RIGHT LIGHT
        0x2516 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Div(-1)),
                        BlockCoord::Frac(1, 2),
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [┗] BOX DRAWINGS HEAVY UP AND RIGHT
        0x2517 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::OutlineHeavy,
        }]),

        // [┘] BOX DRAWINGS LIGHT UP AND LEFT
        0x2518 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [┙] BOX DRAWINGS UP LIGHT AND LEFT HEAVY
        0x2519 => BlockKey::Poly(&[
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
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Div(2)),
                        BlockCoord::Frac(1, 2),
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [┚] BOX DRAWINGS UP HEAVY AND LEFT LIGHT
        0x251a => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(
                        BlockCoord::FracWithOffset(1, 2, LineScale::Div(1)),
                        BlockCoord::Frac(1, 2),
                    ),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [┛] BOX DRAWINGS HEAVY UP AND LEFT
        0x251b => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::OutlineHeavy,
        }]),

        // [├] BOX DRAWINGS LIGHT VERTICAL AND RIGHT
        0x251c => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [┝] BOX DRAWINGS LIGHT VERTICAL LIGHT AND RIGHT HEAVY
        0x251d => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [┞] BOX DRAWINGS UP HEAVY and RIGHT DOWN LIGHT
        0x251e => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [┟] BOX DRAWINGS DOWN HEAVY and RIGHT UP LIGHT
        0x251f => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),

        // [┠] BOX DRAWINGS HEAVY VERTICAL and RIGHT LIGHT
        0x2520 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [┡] BOX DRAWINGS DOWN LIGHT AND RIGHT UP HEAVY
        0x2521 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [┢] BOX DRAWINGS UP LIGHT AND RIGHT DOWN HEAVY
        0x2522 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [┣] BOX DRAWINGS HEAVY VERTICAL and RIGHT
        0x2523 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::OutlineHeavy,
        }]),
        // [┤] BOX DRAWINGS LIGHT VERTICAL and LEFT
        0x2524 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [┥] BOX DRAWINGS VERTICAL LIGHT and LEFT HEAVY
        0x2525 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [┦] BOX DRAWINGS UP HEAVY and LEFT DOWN LIGHT
        0x2526 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
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
        // [┧] BOX DRAWINGS DOWN HEAVY and LEFT UP LIGHT
        0x2527 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
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
        // [┨] BOX DRAWINGS VERTICAL HEAVY and LEFT LIGHT
        0x2528 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [┩] BOX DRAWINGS DOWN LIGHT and LEFT UP HEAVY
        0x2529 => BlockKey::Poly(&[
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
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [┪] BOX DRAWINGS UP LIGHT and LEFT DOWN HEAVY
        0x252a => BlockKey::Poly(&[
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
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [┫] BOX DRAWINGS HEAVY VERTICAL and LEFT
        0x252b => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::OutlineHeavy,
        }]),
        // [┬] BOX DRAWINGS LIGHT DOWN AND HORIZONTAL
        0x252c => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        _ => return None,
    })
}
