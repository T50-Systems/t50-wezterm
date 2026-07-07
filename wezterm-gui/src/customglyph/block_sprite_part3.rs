use super::*;

impl GlyphCache {
    pub(super) fn block_sprite_part3(
        &mut self,
        block: BlockKey,
        metrics: &RenderMetrics,
        mut buffer: &mut Image,
    ) -> Option<()> {
        Some(match block {
            BlockKey::Spinner(segment) => {
                let mut draw =
                    |cmd: &'static [PolyCommand], style: PolyStyle, blend_mode: BlendMode| {
                        self.draw_polys(
                            &metrics,
                            &[Poly {
                                path: cmd,
                                intensity: BlockAlpha::Full,
                                style: style,
                            }],
                            &mut buffer,
                            if config::configuration().anti_alias_custom_block_glyphs {
                                PolyAA::AntiAlias
                            } else {
                                PolyAA::MoarPixels
                            },
                            blend_mode,
                        );
                    };

                match segment {
                    0 => {
                        draw(
                            &[PolyCommand::Circle {
                                center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                radius: BlockCoord::Frac(1, 2),
                            }],
                            PolyStyle::Fill,
                            BlendMode::default(),
                        );
                        draw(
                            &[PolyCommand::Circle {
                                center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                radius: BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-3)),
                            }],
                            PolyStyle::Fill,
                            BlendMode::Clear,
                        );
                        draw(
                            &[
                                PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                PolyCommand::LineTo(BlockCoord::SquareOne, BlockCoord::SquareZero),
                                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                                PolyCommand::LineTo(BlockCoord::SquareZero, BlockCoord::SquareZero),
                                PolyCommand::Close,
                            ],
                            PolyStyle::Fill,
                            BlendMode::Clear,
                        );
                    }
                    1 => {
                        draw(
                            &[PolyCommand::Circle {
                                center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                radius: BlockCoord::Frac(1, 2),
                            }],
                            PolyStyle::Fill,
                            BlendMode::default(),
                        );
                        draw(
                            &[PolyCommand::Circle {
                                center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                radius: BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-3)),
                            }],
                            PolyStyle::Fill,
                            BlendMode::Clear,
                        );
                        draw(
                            &[
                                PolyCommand::MoveTo(
                                    BlockCoord::SquareFrac(1, 2),
                                    BlockCoord::SquareFrac(1, 2),
                                ),
                                PolyCommand::LineTo(BlockCoord::SquareFrac(1, 2), BlockCoord::Zero),
                                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Zero),
                                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                                PolyCommand::LineTo(BlockCoord::SquareOne, BlockCoord::One),
                                PolyCommand::LineTo(BlockCoord::SquareOne, BlockCoord::SquareOne),
                                PolyCommand::Close,
                            ],
                            PolyStyle::Fill,
                            BlendMode::Clear,
                        );
                    }
                    2 => {
                        draw(
                            &[PolyCommand::Circle {
                                center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                radius: BlockCoord::Frac(1, 2),
                            }],
                            PolyStyle::Fill,
                            BlendMode::default(),
                        );
                        draw(
                            &[PolyCommand::Circle {
                                center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                radius: BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-3)),
                            }],
                            PolyStyle::Fill,
                            BlendMode::Clear,
                        );
                        draw(
                            &[
                                PolyCommand::MoveTo(
                                    BlockCoord::SquareFrac(1, 2),
                                    BlockCoord::SquareFrac(1, 2),
                                ),
                                PolyCommand::LineTo(BlockCoord::SquareOne, BlockCoord::SquareZero),
                                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Zero),
                                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                                PolyCommand::LineTo(
                                    BlockCoord::SquareFrac(1, 3),
                                    BlockCoord::SquareOne,
                                ),
                            ],
                            PolyStyle::Fill,
                            BlendMode::Clear,
                        );
                    }
                    3 => {
                        draw(
                            &[PolyCommand::Circle {
                                center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                radius: BlockCoord::Frac(1, 2),
                            }],
                            PolyStyle::Fill,
                            BlendMode::default(),
                        );
                        draw(
                            &[PolyCommand::Circle {
                                center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                radius: BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-3)),
                            }],
                            PolyStyle::Fill,
                            BlendMode::Clear,
                        );
                        draw(
                            &[
                                PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::SquareFrac(1, 2)),
                                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Zero),
                                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                                PolyCommand::LineTo(BlockCoord::One, BlockCoord::SquareFrac(1, 2)),
                            ],
                            PolyStyle::Fill,
                            BlendMode::Clear,
                        );
                    }
                    4 => {
                        draw(
                            &[PolyCommand::Circle {
                                center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                radius: BlockCoord::Frac(1, 2),
                            }],
                            PolyStyle::Fill,
                            BlendMode::default(),
                        );
                        draw(
                            &[PolyCommand::Circle {
                                center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                radius: BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-3)),
                            }],
                            PolyStyle::Fill,
                            BlendMode::Clear,
                        );
                        draw(
                            &[
                                PolyCommand::MoveTo(
                                    BlockCoord::SquareFrac(1, 2),
                                    BlockCoord::SquareFrac(1, 2),
                                ),
                                PolyCommand::LineTo(BlockCoord::SquareZero, BlockCoord::SquareZero),
                                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Zero),
                                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                                PolyCommand::LineTo(
                                    BlockCoord::SquareFrac(2, 3),
                                    BlockCoord::SquareOne,
                                ),
                            ],
                            PolyStyle::Fill,
                            BlendMode::Clear,
                        );
                    }
                    5 => {
                        draw(
                            &[PolyCommand::Circle {
                                center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                radius: BlockCoord::Frac(1, 2),
                            }],
                            PolyStyle::Fill,
                            BlendMode::default(),
                        );
                        draw(
                            &[PolyCommand::Circle {
                                center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
                                radius: BlockCoord::FracWithOffset(1, 2, LineScale::Mul(-3)),
                            }],
                            PolyStyle::Fill,
                            BlendMode::Clear,
                        );
                        draw(
                            &[
                                PolyCommand::MoveTo(
                                    BlockCoord::SquareFrac(1, 2),
                                    BlockCoord::SquareFrac(1, 2),
                                ),
                                PolyCommand::LineTo(BlockCoord::SquareFrac(1, 2), BlockCoord::Zero),
                                PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero),
                                PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
                                PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
                                PolyCommand::LineTo(BlockCoord::SquareZero, BlockCoord::SquareOne),
                                PolyCommand::Close,
                            ],
                            PolyStyle::Fill,
                            BlendMode::Clear,
                        );
                    }
                    _ => {}
                }
            }
            BlockKey::Poly(polys) | BlockKey::PolyWithCustomMetrics { polys, .. } => {
                self.draw_polys(
                    &metrics,
                    polys,
                    &mut buffer,
                    if config::configuration().anti_alias_custom_block_glyphs {
                        PolyAA::AntiAlias
                    } else {
                        PolyAA::MoarPixels
                    },
                    BlendMode::default(),
                );
            }
            _ => return None,
        })
    }
}
