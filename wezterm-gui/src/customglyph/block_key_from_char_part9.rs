use super::*;

pub(super) fn from_char_part9(c: u32) -> Option<BlockKey> {
    Some(match c {
        // [🭢] UPPER RIGHT BLOCK DIAGONAL UPPER CENTRE TO UPPER MIDDLE RIGHT
        0x1fb62 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭣] UPPER RIGHT BLOCK DIAGONAL UPPER LEFT TO UPPER MIDDLE RIGHT
        0x1fb63 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭤] UPPER RIGHT BLOCK DIAGONAL UPPER CENTRE TO LOWER MIDDLE RIGHT
        0x1fb64 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(2, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭥] UPPER RIGHT BLOCK DIAGONAL UPPER LEFT TO LOWER MIDDLE RIGHT
        0x1fb65 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(2, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭦] UPPER RIGHT BLOCK DIAGONAL UPPER CENTRE TO LOWER RIGHT
        0x1fb66 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭧] UPPER RIGHT BLOCK DIAGONAL UPPER MIDDLE LEFT TO LOWER MIDDLE RIGHT
        0x1fb67 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(2, 3)),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭨] UPPER AND RIGHT AND LOWER TRIANGULAR THREE QUARTERS BLOCK
        0x1fb68 => BlockKey::Triangles(
            Triangle::UPPER | Triangle::RIGHT | Triangle::LOWER,
            BlockAlpha::Full,
        ),
        // [🭩] LEFT AND LOWER AND RIGHT TRIANGULAR THREE QUARTERS BLOCK
        0x1fb69 => BlockKey::Triangles(
            Triangle::LEFT | Triangle::LOWER | Triangle::RIGHT,
            BlockAlpha::Full,
        ),
        // [🭪] UPPER AND LEFT AND LOWER TRIANGULAR THREE QUARTERS BLOCK
        0x1fb6a => BlockKey::Triangles(
            Triangle::UPPER | Triangle::LEFT | Triangle::LOWER,
            BlockAlpha::Full,
        ),
        // [🭫] LEFT AND UPPER AND RIGHT TRIANGULAR THREE QUARTERS BLOCK
        0x1fb6b => BlockKey::Triangles(
            Triangle::LEFT | Triangle::UPPER | Triangle::RIGHT,
            BlockAlpha::Full,
        ),
        // [🭬] LEFT TRIANGULAR ONE QUARTER BLOCK
        0x1fb6c => BlockKey::Triangles(Triangle::LEFT, BlockAlpha::Full),
        // [🭭] UPPER TRIANGULAR ONE QUARTER BLOCK
        0x1fb6d => BlockKey::Triangles(Triangle::UPPER, BlockAlpha::Full),
        // [🭮] RIGHT TRIANGULAR ONE QUARTER BLOCK
        0x1fb6e => BlockKey::Triangles(Triangle::RIGHT, BlockAlpha::Full),
        // [🭯] LOWER TRIANGULAR ONE QUARTER BLOCK
        0x1fb6f => BlockKey::Triangles(Triangle::LOWER, BlockAlpha::Full),
        // [🭰] VERTICAL ONE EIGHTH BLOCK-2
        0x1fb70 => BlockKey::Blocks(&[Block::VerticalBlock(1, 2)]),
        // [🭱] VERTICAL ONE EIGHTH BLOCK-3
        0x1fb71 => BlockKey::Blocks(&[Block::VerticalBlock(2, 3)]),
        // [🭲] VERTICAL ONE EIGHTH BLOCK-4
        0x1fb72 => BlockKey::Blocks(&[Block::VerticalBlock(3, 4)]),
        // [🭳] VERTICAL ONE EIGHTH BLOCK-5
        0x1fb73 => BlockKey::Blocks(&[Block::VerticalBlock(4, 5)]),
        // [🭴] VERTICAL ONE EIGHTH BLOCK-6
        0x1fb74 => BlockKey::Blocks(&[Block::VerticalBlock(5, 6)]),
        // [🭵] VERTICAL ONE EIGHTH BLOCK-7
        0x1fb75 => BlockKey::Blocks(&[Block::VerticalBlock(6, 7)]),
        // [🭶] HORIZONTAL ONE EIGHTH BLOCK-2
        0x1fb76 => BlockKey::Blocks(&[Block::HorizontalBlock(1, 2)]),
        // [🭷] HORIZONTAL ONE EIGHTH BLOCK-3
        0x1fb77 => BlockKey::Blocks(&[Block::HorizontalBlock(2, 3)]),
        // [🭸] HORIZONTAL ONE EIGHTH BLOCK-4
        0x1fb78 => BlockKey::Blocks(&[Block::HorizontalBlock(3, 4)]),
        // [🭹] HORIZONTAL ONE EIGHTH BLOCK-5
        0x1fb79 => BlockKey::Blocks(&[Block::HorizontalBlock(4, 5)]),
        // [🭺] HORIZONTAL ONE EIGHTH BLOCK-6
        0x1fb7a => BlockKey::Blocks(&[Block::HorizontalBlock(5, 6)]),
        // [🭻] HORIZONTAL ONE EIGHTH BLOCK-7
        0x1fb7b => BlockKey::Blocks(&[Block::HorizontalBlock(6, 7)]),
        // [🭼] Left and lower one eighth block
        0x1fb7c => BlockKey::Blocks(&[Block::LeftBlock(1), Block::LowerBlock(1)]),
        // [🭽] Left and upper one eighth block
        0x1fb7d => BlockKey::Blocks(&[Block::LeftBlock(1), Block::UpperBlock(1)]),
        // [🭾] Right and upper one eighth block
        0x1fb7e => BlockKey::Blocks(&[Block::RightBlock(1), Block::UpperBlock(1)]),
        // [🭿] Right and lower one eighth block
        0x1fb7f => BlockKey::Blocks(&[Block::RightBlock(1), Block::LowerBlock(1)]),
        // [🮀] UPPER AND LOWER ONE EIGHTH BLOCK
        0x1fb80 => BlockKey::Blocks(&[Block::UpperBlock(1), Block::LowerBlock(1)]),
        // [🮁] HORIZONTAL ONE EIGHTH BLOCK-1358
        0x1fb81 => BlockKey::Blocks(&[
            Block::UpperBlock(1),
            Block::HorizontalBlock(2, 3),
            Block::HorizontalBlock(4, 5),
            Block::LowerBlock(1),
        ]),
        // [🮂] Upper One Quarter Block
        0x1fb82 => BlockKey::Blocks(&[Block::UpperBlock(2)]),
        // [🮃] Upper three eighths block
        0x1fb83 => BlockKey::Blocks(&[Block::UpperBlock(3)]),
        // [🮄] Upper five eighths block
        0x1fb84 => BlockKey::Blocks(&[Block::UpperBlock(5)]),
        // [🮅] Upper three quarters block
        0x1fb85 => BlockKey::Blocks(&[Block::UpperBlock(6)]),
        // [🮆] Upper seven eighths block
        0x1fb86 => BlockKey::Blocks(&[Block::UpperBlock(7)]),
        // [🮇] Right One Quarter Block
        0x1fb87 => BlockKey::Blocks(&[Block::RightBlock(2)]),
        // [🮈] Right three eighths block
        0x1fb88 => BlockKey::Blocks(&[Block::RightBlock(3)]),
        // [🮉] Right five eighths block
        0x1fb89 => BlockKey::Blocks(&[Block::RightBlock(5)]),
        // [🮊] Right three quarters block
        0x1fb8a => BlockKey::Blocks(&[Block::RightBlock(6)]),
        // [🮋] Right seven eighths block
        0x1fb8b => BlockKey::Blocks(&[Block::RightBlock(7)]),
        // [🮌] LEFT HALF MEDIUM SHADE
        0x1fb8c => BlockKey::Blocks(&[Block::Custom(0, 4, 0, 8, BlockAlpha::Medium)]),
        // [🮍] RIGHT HALF MEDIUM SHADE
        0x1fb8d => BlockKey::Blocks(&[Block::Custom(4, 8, 0, 8, BlockAlpha::Medium)]),
        // [🮎] UPPER HALF MEDIUM SHADE
        0x1fb8e => BlockKey::Blocks(&[Block::Custom(0, 8, 0, 4, BlockAlpha::Medium)]),
        // [🮏] LOWER HALF MEDIUM SHADE
        0x1fb8f => BlockKey::Blocks(&[Block::Custom(0, 8, 4, 8, BlockAlpha::Medium)]),
        // [🮐] INVERSE MEDIUM SHADE
        0x1fb90 => BlockKey::Blocks(&[Block::Custom(0, 8, 0, 8, BlockAlpha::Medium)]),
        // [🮑] UPPER HALF BLOCK AND LOWER HALF INVERSE MEDIUM SHADE
        0x1fb91 => BlockKey::Blocks(&[
            Block::UpperBlock(4),
            Block::Custom(0, 8, 4, 8, BlockAlpha::Medium),
        ]),
        // [🮒] UPPER HALF INVERSE MEDIUM SHADE AND LOWER HALF BLOCK
        0x1fb92 => BlockKey::Blocks(&[
            Block::Custom(0, 8, 0, 4, BlockAlpha::Medium),
            Block::LowerBlock(4),
        ]),
        // [🮓] LEFT HALF BLOCK AND RIGHT HALF INVERSE MEDIUM SHADE
        // NOTE: not official!
        0x1fb93 => BlockKey::Blocks(&[
            Block::LeftBlock(4),
            Block::Custom(4, 8, 0, 8, BlockAlpha::Medium),
        ]),
        // [🮔] LEFT HALF INVERSE MEDIUM SHADE AND RIGHT HALF BLOCK
        0x1fb94 => BlockKey::Blocks(&[
            Block::Custom(0, 4, 0, 8, BlockAlpha::Medium),
            Block::RightBlock(4),
        ]),
        // [🮕] CHECKER BOARD FILL
        0x1fb95 => BlockKey::Blocks(&[
            Block::Custom(0, 2, 0, 2, BlockAlpha::Full),
            Block::Custom(0, 2, 4, 6, BlockAlpha::Full),
            Block::Custom(2, 4, 2, 4, BlockAlpha::Full),
            Block::Custom(2, 4, 6, 8, BlockAlpha::Full),
            Block::Custom(4, 6, 0, 2, BlockAlpha::Full),
            Block::Custom(4, 6, 4, 6, BlockAlpha::Full),
            Block::Custom(6, 8, 2, 4, BlockAlpha::Full),
            Block::Custom(6, 8, 6, 8, BlockAlpha::Full),
        ]),
        // [🮖] INVERSE CHECKER BOARD FILL
        0x1fb96 => BlockKey::Blocks(&[
            Block::Custom(0, 2, 2, 4, BlockAlpha::Full),
            Block::Custom(0, 2, 6, 8, BlockAlpha::Full),
            Block::Custom(2, 4, 0, 2, BlockAlpha::Full),
            Block::Custom(2, 4, 4, 6, BlockAlpha::Full),
            Block::Custom(4, 6, 2, 4, BlockAlpha::Full),
            Block::Custom(4, 6, 6, 8, BlockAlpha::Full),
            Block::Custom(6, 8, 0, 2, BlockAlpha::Full),
            Block::Custom(6, 8, 4, 6, BlockAlpha::Full),
        ]),
        // [🮗] HEAVY HORIZONTAL FILL
        0x1fb97 => BlockKey::Blocks(&[Block::HorizontalBlock(2, 4), Block::HorizontalBlock(6, 8)]),
        // [🮘] UPPER LEFT TO LOWER RIGHT FILL
        // NOTE: This is a quick placeholder which doesn't scale correctly
        0x1fb98 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 10)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 6), BlockCoord::Zero),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(3, 10)),
                    PolyCommand::LineTo(BlockCoord::Frac(3, 6), BlockCoord::Zero),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(5, 10)),
                    PolyCommand::LineTo(BlockCoord::Frac(5, 6), BlockCoord::Zero),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(7, 10)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 10)),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(9, 10)),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(3, 10)),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 6), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(5, 10)),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(3, 6), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(7, 10)),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(5, 6), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(9, 10)),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
        ]),
        // [🮙] UPPER RIGHT TO LOWER LEFT FILL
        // NOTE: This is a quick placeholder which doesn't scale correctly
        0x1fb99 => BlockKey::Poly(&[
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(1, 10)),
                    PolyCommand::LineTo(BlockCoord::Frac(5, 6), BlockCoord::Zero),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(3, 10)),
                    PolyCommand::LineTo(BlockCoord::Frac(3, 6), BlockCoord::Zero),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(5, 10)),
                    PolyCommand::LineTo(BlockCoord::Frac(1, 6), BlockCoord::Zero),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(7, 10)),
                    PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 10)),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(9, 10)),
                    PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(3, 10)),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(5, 6), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(5, 10)),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(3, 6), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(7, 10)),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
            Poly {
                path: &[
                    PolyCommand::MoveTo(BlockCoord::Frac(1, 6), BlockCoord::One),
                    PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(9, 10)),
                    PolyCommand::Close,
                ],
                intensity: BlockAlpha::Full,
                style: PolyStyle::OutlineThin,
            },
        ]),
        // [🮚] UPPER AND LOWER TRIANGULAR HALF BLOCK
        0x1fb9a => BlockKey::Triangles(Triangle::UPPER | Triangle::LOWER, BlockAlpha::Full),
        // [🮛] LEFT AND RIGHT TRIANGULAR HALF BLOCK
        0x1fb9b => BlockKey::Triangles(Triangle::LEFT | Triangle::RIGHT, BlockAlpha::Full),
        // [🮜] UPPER UPPER LEFT TRIANGULAR MEDIUM SHADE
        0x1fb9c => BlockKey::Triangles(Triangle::LEFT | Triangle::UPPER, BlockAlpha::Medium),
        // [🮝] UPPER RIGHT TRIANGULAR MEDIUM SHADE
        0x1fb9d => BlockKey::Triangles(Triangle::RIGHT | Triangle::UPPER, BlockAlpha::Medium),
        // [🮞] LOWER RIGHT TRIANGULAR MEDIUM SHADE
        0x1fb9e => BlockKey::Triangles(Triangle::RIGHT | Triangle::LOWER, BlockAlpha::Medium),
        // [🮟] LOWER LEFT TRIANGULAR MEDIUM SHADE
        0x1fb9f => BlockKey::Triangles(Triangle::LEFT | Triangle::LOWER, BlockAlpha::Medium),
        // [🮠] BOX DRAWINGS LIGHT DIAGONAL UPPER CENTRE TO MIDDLE LEFT
        0x1fba0 => BlockKey::CellDiagonals(CellDiagonal::UPPER_LEFT),
        // [🮡] BOX DRAWINGS LIGHT DIAGONAL UPPER CENTRE TO MIDDLE RIGHT
        0x1fba1 => BlockKey::CellDiagonals(CellDiagonal::UPPER_RIGHT),
        // [🮢] BOX DRAWINGS LIGHT DIAGONAL MIDDLE LEFT TO LOWER CENTRE
        0x1fba2 => BlockKey::CellDiagonals(CellDiagonal::LOWER_LEFT),
        // [🮣] BOX DRAWINGS LIGHT DIAGONAL MIDDLE RIGHT TO LOWER CENTRE
        0x1fba3 => BlockKey::CellDiagonals(CellDiagonal::LOWER_RIGHT),
        // [🮤] BOX DRAWINGS LIGHT DIAGONAL UPPER CENTRE TO MIDDLE LEFT TO LOWER CENTRE
        0x1fba4 => BlockKey::CellDiagonals(CellDiagonal::UPPER_LEFT | CellDiagonal::LOWER_LEFT),
        // [🮥] BOX DRAWINGS LIGHT DIAGONAL UPPER CENTRE TO MIDDLE RIGHT TO LOWER CENTRE
        0x1fba5 => BlockKey::CellDiagonals(CellDiagonal::UPPER_RIGHT | CellDiagonal::LOWER_RIGHT),
        // [🮦] BOX DRAWINGS LIGHT DIAGONAL MIDDLE LEFT TO LOWER CENTRE TO MIDDLE RIGHT
        0x1fba6 => BlockKey::CellDiagonals(CellDiagonal::LOWER_LEFT | CellDiagonal::LOWER_RIGHT),
        // [🮧] BOX DRAWINGS LIGHT DIAGONAL MIDDLE LEFT TO UPPER CENTRE TO MIDDLE RIGHT
        0x1fba7 => BlockKey::CellDiagonals(CellDiagonal::UPPER_LEFT | CellDiagonal::UPPER_RIGHT),
        // [🮨] BOX DRAWINGS LIGHT DIAGONAL UPPER CENTRE TO MIDDLE LEFT AND MIDDLE RIGHT TO LOWER CENTRE
        0x1fba8 => BlockKey::CellDiagonals(CellDiagonal::UPPER_LEFT | CellDiagonal::LOWER_RIGHT),
        // [🮩] BOX DRAWINGS LIGHT DIAGONAL UPPER CENTRE TO MIDDLE RIGHT AND MIDDLE LEFT TO LOWER CENTRE
        0x1fba9 => BlockKey::CellDiagonals(CellDiagonal::UPPER_RIGHT | CellDiagonal::LOWER_LEFT),
        // [🮪] BOX DRAWINGS LIGHT DIAGONAL UPPER CENTRE TO MIDDLE RIGHT TO LOWER CENTRE TO MIDDLE LEFT
        0x1fbaa => BlockKey::CellDiagonals(
            CellDiagonal::UPPER_RIGHT | CellDiagonal::LOWER_LEFT | CellDiagonal::LOWER_RIGHT,
        ),
        // [🮫] BOX DRAWINGS LIGHT DIAGONAL UPPER CENTRE TO MIDDLE LEFT TO LOWER CENTRE TO MIDDLE RIGHT
        0x1fbab => BlockKey::CellDiagonals(
            CellDiagonal::UPPER_LEFT | CellDiagonal::LOWER_LEFT | CellDiagonal::LOWER_RIGHT,
        ),
        // [🮬] BOX DRAWINGS LIGHT DIAGONAL MIDDLE LEFT TO UPPER CENTRE TO MIDDLE RIGHT TO LOWER CENTRE
        0x1fbac => BlockKey::CellDiagonals(
            CellDiagonal::UPPER_LEFT | CellDiagonal::UPPER_RIGHT | CellDiagonal::LOWER_RIGHT,
        ),
        // [🮭] BOX DRAWINGS LIGHT DIAGONAL MIDDLE RIGHT TO UPPER CENTRE TO MIDDLE LEFT TO LOWER CENTRE
        0x1fbad => BlockKey::CellDiagonals(
            CellDiagonal::UPPER_LEFT | CellDiagonal::UPPER_RIGHT | CellDiagonal::LOWER_LEFT,
        ),
        // [🮮] BOX DRAWINGS LIGHT DIAGONAL DIAMOND
        0x1fbae => BlockKey::CellDiagonals(
            CellDiagonal::UPPER_LEFT
                | CellDiagonal::UPPER_RIGHT
                | CellDiagonal::LOWER_LEFT
                | CellDiagonal::LOWER_RIGHT,
        ),
        _ => return None,
    })
}
