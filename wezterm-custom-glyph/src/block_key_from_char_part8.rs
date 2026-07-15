use super::*;

pub(super) fn from_char_part8(c: u32) -> Option<BlockKey> {
    Some(match c {
        // [🬾] LOWER LEFT BLOCK DIAGONAL UPPER MIDDLE LEFT TO LOWER CENTRE
        0x1fb3e => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 3)),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🬿] LOWER LEFT BLOCK DIAGONAL UPPER MIDDLE LEFT TO LOWER RIGHT
        0x1fb3f => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 3)),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭀] LOWER LEFT BLOCK DIAGONAL UPPER LEFT TO LOWER CENTRE
        0x1fb40 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭁] LOWER RIGHT BLOCK DIAGONAL UPPER MIDDLE LEFT TO UPPER CENTRE
        0x1fb41 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭂] LOWER RIGHT BLOCK DIAGONAL UPPER MIDDLE LEFT TO UPPER RIGHT
        0x1fb42 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭃] LOWER RIGHT BLOCK DIAGONAL LOWER MIDDLE LEFT TO UPPER CENTRE
        0x1fb43 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(2, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭄] LOWER RIGHT BLOCK DIAGONAL LOWER MIDDLE LEFT TO UPPER RIGHT
        0x1fb44 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(2, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭅] LOWER RIGHT BLOCK DIAGONAL UPPER LEFT TO UPPER CENTRE
        0x1fb45 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭆] LOWER RIGHT BLOCK DIAGONAL LOWER MIDDLE LEFT TO UPPER MIDDLE RIGHT
        0x1fb46 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(2, 3)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 3)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭇] LOWER RIGHT BLOCK DIAGONAL LOWER CENTRE TO LOWER MIDDLE RIGHT
        0x1fb47 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(2, 3)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭈] LOWER RIGHT BLOCK DIAGONAL LOWER LEFT TO LOWER MIDDLE RIGHT
        0x1fb48 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(2, 3)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭉] LOWER RIGHT BLOCK DIAGONAL LOWER CENTRE TO UPPER MIDDLE RIGHT
        0x1fb49 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 3)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭊] LOWER RIGHT BLOCK DIAGONAL LOWER LEFT TO UPPER MIDDLE RIGHT
        0x1fb4a => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 3)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭋] LOWER RIGHT BLOCK DIAGONAL LOWER CENTRE TO UPPER RIGHT
        0x1fb4b => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭌] LOWER LEFT BLOCK DIAGONAL UPPER CENTRE TO UPPER MIDDLE RIGHT
        0x1fb4c => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 3)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭍] LOWER LEFT BLOCK DIAGONAL UPPER LEFT TO UPPER MIDDLE RIGHT
        0x1fb4d => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 3)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭎] LOWER LEFT BLOCK DIAGONAL UPPER CENTRE TO LOWER MIDDLE RIGHT
        0x1fb4e => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(2, 3)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭏] LOWER LEFT BLOCK DIAGONAL UPPER LEFT TO LOWER MIDDLE RIGHT
        0x1fb4f => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(2, 3)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭐] LOWER LEFT BLOCK DIAGONAL UPPER CENTRE TO LOWER RIGHT
        0x1fb50 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭑] LOWER LEFT BLOCK DIAGONAL UPPER MIDDLE LEFT TO LOWER MIDDLE RIGHT
        0x1fb51 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 3)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(2, 3)),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭒] UPPER RIGHT BLOCK DIAGONAL LOWER MIDDLE LEFT TO LOWER CENTRE
        0x1fb52 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(2, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭓] UPPER RIGHT BLOCK DIAGONAL LOWER MIDDLE LEFT TO LOWER RIGHT
        0x1fb53 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(2, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭔] UPPER RIGHT BLOCK DIAGONAL UPPER MIDDLE LEFT TO LOWER CENTRE
        0x1fb54 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭕] UPPER RIGHT BLOCK DIAGONAL UPPER MIDDLE LEFT TO LOWER RIGHT
        0x1fb55 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭖] UPPER RIGHT BLOCK DIAGONAL UPPER LEFT TO LOWER CENTRE
        0x1fb56 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭗] UPPER LEFT BLOCK DIAGONAL UPPER MIDDLE LEFT TO UPPER CENTRE
        0x1fb57 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭘] UPPER LEFT BLOCK DIAGONAL UPPER MIDDLE LEFT TO UPPER RIGHT
        0x1fb58 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭙] UPPER LEFT BLOCK DIAGONAL LOWER MIDDLE LEFT TO UPPER CENTRE
        0x1fb59 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(2, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭚] UPPER LEFT BLOCK DIAGONAL LOWER MIDDLE LEFT TO UPPER RIGHT
        0x1fb5a => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(2, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭛] UPPER LEFT BLOCK DIAGONAL LOWER LEFT TO UPPER CENTRE
        0x1fb5b => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭜] UPPER LEFT BLOCK DIAGONAL LOWER MIDDLE LEFT TO UPPER MIDDLE RIGHT
        0x1fb5c => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 3)),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(2, 3)),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭝] UPPER LEFT BLOCK DIAGONAL LOWER CENTRE TO LOWER MIDDLE RIGHT
        0x1fb5d => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(2, 3)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭞] UPPER LEFT BLOCK DIAGONAL LOWER LEFT TO LOWER MIDDLE RIGHT
        0x1fb5e => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(2, 3)),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭟] UPPER LEFT BLOCK DIAGONAL LOWER CENTRE TO UPPER MIDDLE RIGHT
        0x1fb5f => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 3)),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭠] UPPER LEFT BLOCK DIAGONAL LOWER LEFT TO UPPER MIDDLE RIGHT
        0x1fb60 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 3)),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        // [🭡] UPPER LEFT BLOCK DIAGONAL LOWER CENTRE TO UPPER RIGHT
        0x1fb61 => BlockKey::Poly(&[Poly {
            path: &[
                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                PolyCommand::Close,
            ],
            intensity: BlockAlpha::Full,
            style: PolyStyle::Fill,
        }]),
        _ => return None,
    })
}
