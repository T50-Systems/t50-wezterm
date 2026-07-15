use super::*;

pub(crate) fn block_sprite_part2(
        block: BlockKey,
        metrics: &RasterizeGlyphParams,
        mut buffer: &mut Pixmap,
    ) -> Option<()> {
        Some(match block {
            BlockKey::Progress(chunks) => {
                let mut draw = |cmd: &'static [PolyCommand], style: PolyStyle| {
                    draw_polys(
                        &metrics,
                        &[Poly {
                            path: cmd,
                            intensity: BlockAlpha::Full,
                            style: style,
                        }],
                        &mut buffer,
                        poly_aa(metrics.anti_alias),
                        BlendMode::default(),
                    );
                };

                if chunks.contains(ProgressChunk::LEFT) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(1, 6)),
                            PolyCommand::LineTo(BlockCoord::Frac(1, 6), BlockCoord::Frac(1, 6)),
                            PolyCommand::LineTo(BlockCoord::Frac(1, 6), BlockCoord::Frac(6 - 1, 6)),
                            PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(6 - 1, 6)),
                        ],
                        PolyStyle::OutlineHeavy,
                    );

                    if chunks.contains(ProgressChunk::FULL) {
                        draw(
                            &[
                                PolyCommand::MoveTo(
                                    BlockCoord::One,
                                    BlockCoord::FracWithOffset(1, 6, LineScale::Mul(6)),
                                ),
                                PolyCommand::LineTo(
                                    BlockCoord::FracWithOffset(1, 6, LineScale::Mul(6)),
                                    BlockCoord::FracWithOffset(1, 6, LineScale::Mul(6)),
                                ),
                                PolyCommand::LineTo(
                                    BlockCoord::FracWithOffset(1, 6, LineScale::Mul(6)),
                                    BlockCoord::FracWithOffset(6 - 1, 6, LineScale::Mul(-6)),
                                ),
                                PolyCommand::LineTo(
                                    BlockCoord::One,
                                    BlockCoord::FracWithOffset(6 - 1, 6, LineScale::Mul(-6)),
                                ),
                                PolyCommand::Close,
                            ],
                            PolyStyle::Fill,
                        );
                    }
                }
                if chunks.contains(ProgressChunk::RIGHT) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 6)),
                            PolyCommand::LineTo(BlockCoord::Frac(6 - 1, 6), BlockCoord::Frac(1, 6)),
                            PolyCommand::LineTo(
                                BlockCoord::Frac(6 - 1, 6),
                                BlockCoord::Frac(6 - 1, 6),
                            ),
                            PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(6 - 1, 6)),
                        ],
                        PolyStyle::OutlineHeavy,
                    );

                    if chunks.contains(ProgressChunk::FULL) {
                        draw(
                            &[
                                PolyCommand::MoveTo(
                                    BlockCoord::Zero,
                                    BlockCoord::FracWithOffset(1, 6, LineScale::Mul(6)),
                                ),
                                PolyCommand::LineTo(
                                    BlockCoord::FracWithOffset(6 - 1, 6, LineScale::Mul(-6)),
                                    BlockCoord::FracWithOffset(1, 6, LineScale::Mul(6)),
                                ),
                                PolyCommand::LineTo(
                                    BlockCoord::FracWithOffset(6 - 1, 6, LineScale::Mul(-6)),
                                    BlockCoord::FracWithOffset(6 - 1, 6, LineScale::Mul(-6)),
                                ),
                                PolyCommand::LineTo(
                                    BlockCoord::Zero,
                                    BlockCoord::FracWithOffset(6 - 1, 6, LineScale::Mul(-6)),
                                ),
                                PolyCommand::Close,
                            ],
                            PolyStyle::Fill,
                        );
                    }
                }
                if chunks.contains(ProgressChunk::MIDDLE) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 6)),
                            PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 6)),
                            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(6 - 1, 6)),
                            PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(6 - 1, 6)),
                        ],
                        PolyStyle::OutlineHeavy,
                    );

                    if chunks.contains(ProgressChunk::FULL) {
                        draw(
                            &[
                                PolyCommand::MoveTo(
                                    BlockCoord::Zero,
                                    BlockCoord::FracWithOffset(1, 6, LineScale::Mul(6)),
                                ),
                                PolyCommand::LineTo(
                                    BlockCoord::One,
                                    BlockCoord::FracWithOffset(1, 6, LineScale::Mul(6)),
                                ),
                                PolyCommand::LineTo(
                                    BlockCoord::One,
                                    BlockCoord::FracWithOffset(6 - 1, 6, LineScale::Mul(-6)),
                                ),
                                PolyCommand::LineTo(
                                    BlockCoord::Zero,
                                    BlockCoord::FracWithOffset(6 - 1, 6, LineScale::Mul(-6)),
                                ),
                                PolyCommand::Close,
                            ],
                            PolyStyle::Fill,
                        );
                    }
                }
            }
            BlockKey::Branches(pattern) => {
                let mut draw =
                    |cmd: &'static [PolyCommand], style: PolyStyle, blend_mode: BlendMode| {
                        draw_polys(
                            &metrics,
                            &[Poly {
                                path: cmd,
                                intensity: BlockAlpha::Full,
                                style: style,
                            }],
                            &mut buffer,
                            poly_aa(metrics.anti_alias),
                            blend_mode,
                        );
                    };

                if pattern.contains(Branch::VERTICAL) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                        ],
                        PolyStyle::OutlineHeavy,
                        BlendMode::default(),
                    );
                }
                if pattern.contains(Branch::HORIZONTAL) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                            PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                        ],
                        PolyStyle::OutlineHeavy,
                        BlendMode::default(),
                    );
                }
                if pattern.contains(Branch::RIGHT_TO_DOWN) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(7, 8)),
                            PolyCommand::QuadTo {
                                control: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                to: (BlockCoord::Frac(7, 8), BlockCoord::Frac(1, 2)),
                            },
                            PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                        ],
                        PolyStyle::OutlineHeavy,
                        BlendMode::default(),
                    );
                }
                if pattern.contains(Branch::LEFT_TO_DOWN) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(7, 8)),
                            PolyCommand::QuadTo {
                                control: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                to: (BlockCoord::Frac(1, 8), BlockCoord::Frac(1, 2)),
                            },
                            PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                        ],
                        PolyStyle::OutlineHeavy,
                        BlendMode::default(),
                    );
                }
                if pattern.contains(Branch::RIGHT_TO_UP) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 8)),
                            PolyCommand::QuadTo {
                                control: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                to: (BlockCoord::Frac(7, 8), BlockCoord::Frac(1, 2)),
                            },
                            PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                        ],
                        PolyStyle::OutlineHeavy,
                        BlendMode::default(),
                    );
                }
                if pattern.contains(Branch::LEFT_TO_UP) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 8)),
                            PolyCommand::QuadTo {
                                control: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                to: (BlockCoord::Frac(1, 8), BlockCoord::Frac(1, 2)),
                            },
                            PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                        ],
                        PolyStyle::OutlineHeavy,
                        BlendMode::default(),
                    );
                }
                if pattern.contains(Branch::LEFT) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
                            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                        ],
                        PolyStyle::OutlineHeavy,
                        BlendMode::default(),
                    );
                }
                if pattern.contains(Branch::RIGHT) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
                            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                        ],
                        PolyStyle::OutlineHeavy,
                        BlendMode::default(),
                    );
                }
                if pattern.contains(Branch::UP) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
                            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                        ],
                        PolyStyle::OutlineHeavy,
                        BlendMode::default(),
                    );
                }
                if pattern.contains(Branch::DOWN) {
                    draw(
                        &[
                            PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One),
                            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                        ],
                        PolyStyle::OutlineHeavy,
                        BlendMode::default(),
                    );
                }
                if pattern.contains(Branch::CIRCLE_FILLED) {
                    draw(
                        &[PolyCommand::Circle {
                            center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                            radius: BlockCoord::Frac(2, 5),
                        }],
                        PolyStyle::Fill,
                        BlendMode::default(),
                    );
                }
                if pattern.contains(Branch::CIRCLE_OUTLINE) {
                    draw(
                        &[PolyCommand::Circle {
                            center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                            radius: BlockCoord::Frac(2, 5),
                        }],
                        PolyStyle::Fill,
                        BlendMode::default(),
                    );
                    draw(
                        &[PolyCommand::Circle {
                            center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                            radius: BlockCoord::Frac(3, 10),
                        }],
                        PolyStyle::Fill,
                        BlendMode::Clear,
                    );
                }
            }
            _ => return None,
        })
}
