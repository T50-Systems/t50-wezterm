use super::block_key::{OCTANT_PATTERNS, SEXTANT_PATTERNS};
use super::*;

pub(super) fn from_char_part7(c: u32) -> Option<BlockKey> {
    Some(match c {
        // [╬] BOX DRAWINGS DOUBLE VERTICAL AND HORIZONTAL
        0x256c => BlockKey::Poly(&[
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

        // [╭] BOX DRAWINGS LIGHT ARC DOWN AND RIGHT
        0x256d => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(3, 4)),
                PolyCommand::QuadTo {
                    control: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    to: (BlockCoord::Frac(3, 4), BlockCoord::Frac(1, 2)),
                },
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [╮] BOX DRAWINGS LIGHT ARC DOWN AND LEFT
        0x256e => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(3, 4)),
                PolyCommand::QuadTo {
                    control: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    to: (BlockCoord::Frac(1, 4), BlockCoord::Frac(1, 2)),
                },
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [╯] BOX DRAWINGS LIGHT ARC UP AND LEFT
        0x256f => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 4)),
                PolyCommand::QuadTo {
                    control: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    to: (BlockCoord::Frac(1, 4), BlockCoord::Frac(1, 2)),
                },
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [╰] BOX DRAWINGS LIGHT ARC UP AND RIGHT
        0x2570 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 4)),
                PolyCommand::QuadTo {
                    control: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                    to: (BlockCoord::Frac(3, 4), BlockCoord::Frac(1, 2)),
                },
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),

        // [╱] BOX DRAWINGS LIGHT DIAGONAL UPPER RIGHT TO LOWER LEFT
        0x2571 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [╲] BOX DRAWINGS LIGHT DIAGONAL UPPER LEFT TO LOWER RIGHT
        0x2572 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [╳] BOX DRAWINGS LIGHT DIAGONAL CROSS
        0x2573 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),
        // [╴] BOX DRAWINGS LIGHT LEFT
        0x2574 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [╵] BOX DRAWINGS LIGHT UP
        0x2575 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [╶] BOX DRAWINGS LIGHT RIGHT
        0x2576 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [╷] BOX DRAWINGS LIGHT DOWN
        0x2577 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Outline,
        }]),
        // [╸] BOX DRAWINGS HEAVY LEFT
        0x2578 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::OutlineHeavy,
        }]),
        // [╹] BOX DRAWINGS HEAVY UP
        0x2579 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::OutlineHeavy,
        }]),
        // [╺] BOX DRAWINGS HEAVY RIGHT
        0x257a => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::OutlineHeavy,
        }]),
        // [╻] BOX DRAWINGS HEAVY DOWN
        0x257b => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::OutlineHeavy,
        }]),
        // [╼] BOX DRAWINGS LIGHT LEFT AND HEAVY RIGHT
        0x257c => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
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
        // [╽] BOX DRAWINGS LIGHT UP AND HEAVY DOWN
        0x257d => BlockKey::Poly(&[
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
                    PolyCommand::MoveTo(
                        BlockCoord::Frac(1, 2),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Div(-1)),
                    ),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineHeavy,
            },
        ]),
        // [╾] BOX DRAWINGS HEAVY LEFT AND LIGHT RIGHT
        0x257e => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
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
        // [╿] BOX DRAWINGS HEAVY UP AND LIGHT DOWN
        0x257f => BlockKey::Poly(&[
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
                    PolyCommand::MoveTo(
                        BlockCoord::Frac(1, 2),
                        BlockCoord::FracWithOffset(1, 2, LineScale::Div(-1)),
                    ),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::Outline,
            },
        ]),

        // [▀] UPPER HALF BLOCK
        0x2580 => BlockKey::Blocks(&[Block::UpperBlock(4)]),
        // [▁] LOWER 1 EIGHTH BLOCK
        0x2581 => BlockKey::Blocks(&[Block::LowerBlock(1)]),
        // [▂] LOWER 2 EIGHTHS BLOCK
        0x2582 => BlockKey::Blocks(&[Block::LowerBlock(2)]),
        // [▃] LOWER 3 EIGHTHS BLOCK
        0x2583 => BlockKey::Blocks(&[Block::LowerBlock(3)]),
        // [▄] LOWER 4 EIGHTHS BLOCK
        0x2584 => BlockKey::Blocks(&[Block::LowerBlock(4)]),
        // [▅] LOWER 5 EIGHTHS BLOCK
        0x2585 => BlockKey::Blocks(&[Block::LowerBlock(5)]),
        // [▆] LOWER 6 EIGHTHS BLOCK
        0x2586 => BlockKey::Blocks(&[Block::LowerBlock(6)]),
        // [▇] LOWER 7 EIGHTHS BLOCK
        0x2587 => BlockKey::Blocks(&[Block::LowerBlock(7)]),
        // [█] FULL BLOCK
        0x2588 => BlockKey::Blocks(&[Block::Custom(0, 8, 0, 8, BlockAlpha::Full)]),
        // [▉] LEFT 7 EIGHTHS BLOCK
        0x2589 => BlockKey::Blocks(&[Block::LeftBlock(7)]),
        // [▊] LEFT 6 EIGHTHS BLOCK
        0x258a => BlockKey::Blocks(&[Block::LeftBlock(6)]),
        // [▋] LEFT 5 EIGHTHS BLOCK
        0x258b => BlockKey::Blocks(&[Block::LeftBlock(5)]),
        // [▌] LEFT 4 EIGHTHS BLOCK
        0x258c => BlockKey::Blocks(&[Block::LeftBlock(4)]),
        // [▍] LEFT 3 EIGHTHS BLOCK
        0x258d => BlockKey::Blocks(&[Block::LeftBlock(3)]),
        // [▎] LEFT 2 EIGHTHS BLOCK
        0x258e => BlockKey::Blocks(&[Block::LeftBlock(2)]),
        // [▏] LEFT 1 EIGHTHS BLOCK
        0x258f => BlockKey::Blocks(&[Block::LeftBlock(1)]),
        // [▐] RIGHT HALF BLOCK
        0x2590 => BlockKey::Blocks(&[Block::RightBlock(4)]),
        // [░] LIGHT SHADE
        0x2591 => BlockKey::Blocks(&[Block::Custom(0, 8, 0, 8, BlockAlpha::Light)]),
        // [▒] MEDIUM SHADE
        0x2592 => BlockKey::Blocks(&[Block::Custom(0, 8, 0, 8, BlockAlpha::Medium)]),
        // [▓] DARK SHADE
        0x2593 => BlockKey::Blocks(&[Block::Custom(0, 8, 0, 8, BlockAlpha::Dark)]),
        // [▔] UPPER ONE EIGHTH BLOCK
        0x2594 => BlockKey::Blocks(&[Block::UpperBlock(1)]),
        // [▕] RIGHT ONE EIGHTH BLOCK
        0x2595 => BlockKey::Blocks(&[Block::RightBlock(1)]),
        // [▖] QUADRANT LOWER LEFT
        0x2596 => BlockKey::Blocks(&[Block::QuadrantLL]),
        // [▗] QUADRANT LOWER RIGHT
        0x2597 => BlockKey::Blocks(&[Block::QuadrantLR]),
        // [▘] QUADRANT UPPER LEFT
        0x2598 => BlockKey::Blocks(&[Block::QuadrantUL]),
        // [▙] QUADRANT UPPER LEFT AND LOWER LEFT AND LOWER RIGHT
        0x2599 => BlockKey::Blocks(&[Block::QuadrantUL, Block::QuadrantLL, Block::QuadrantLR]),
        // [▚] QUADRANT UPPER LEFT AND LOWER RIGHT
        0x259a => BlockKey::Blocks(&[Block::QuadrantUL, Block::QuadrantLR]),
        // [▛] QUADRANT UPPER LEFT AND UPPER RIGHT AND LOWER LEFT
        0x259b => BlockKey::Blocks(&[Block::QuadrantUL, Block::QuadrantUR, Block::QuadrantLL]),
        // [▜] QUADRANT UPPER LEFT AND UPPER RIGHT AND LOWER RIGHT
        0x259c => BlockKey::Blocks(&[Block::QuadrantUL, Block::QuadrantUR, Block::QuadrantLR]),
        // [▝] QUADRANT UPPER RIGHT
        0x259d => BlockKey::Blocks(&[Block::QuadrantUR]),
        // [▞] QUADRANT UPPER RIGHT AND LOWER LEFT
        0x259e => BlockKey::Blocks(&[Block::QuadrantUR, Block::QuadrantLL]),
        // [▟] QUADRANT UPPER RIGHT AND LOWER LEFT AND LOWER RIGHT
        0x259f => BlockKey::Blocks(&[Block::QuadrantUR, Block::QuadrantLL, Block::QuadrantLR]),
        // Sextant blocks
        n @ 0x1fb00..=0x1fb3b => BlockKey::Sextant(SEXTANT_PATTERNS[(n & 0x3f) as usize]),
        // Octant blocks
        n @ 0x1cd00..=0x1cde5 => BlockKey::Octant(OCTANT_PATTERNS[(n & 0xff) as usize]),
        // [𜺠] RIGHT HALF LOWER ONE QUARTER BLOCK (corresponds to OCTANT-8)
        0x1cea0 => BlockKey::Octant(0b10000000),
        // [𜺣; EFT HALF LOWER ONE QUARTER BLOCK (corresponds to OCTANT-7)
        0x1cea3 => BlockKey::Octant(0b01000000),
        // [𜺨] LEFT HALF UPPER ONE QUARTER BLOCK (corresponds to OCTANT-1)
        0x1cea8 => BlockKey::Octant(0b00000001),
        // [𜺫] RIGHT HALF UPPER ONE QUARTER BLOCK (corresponds to OCTANT-2)
        0x1ceab => BlockKey::Octant(0b00000010),
        // [🯦] MIDDLE LEFT ONE QUARTER BLOCK (corresponds to OCTANT-35)
        0x1fbe6 => BlockKey::Octant(0b00010100),
        // [🯧] MIDDLE RIGHT ONE QUARTER BLOCK (corresponds to OCTANT-46)
        0x1fbe7 => BlockKey::Octant(0b00101000),
        // [🬼] LOWER LEFT BLOCK DIAGONAL LOWER MIDDLE LEFT TO LOWER CENTRE
        0x1fb3c => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(2, 3)),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🬽] LOWER LEFT BLOCK DIAGONAL LOWER MIDDLE LEFT TO LOWER RIGHT
        0x1fb3d => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(2, 3)),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        _ => return None,
    })
}
