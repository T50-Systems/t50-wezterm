use super::*;

pub(crate) fn block_sprite_part1(
        block: BlockKey,
        metrics: &RasterizeGlyphParams,
        mut buffer: &mut Pixmap,
    ) -> Option<()> {
        Some(match block {
            BlockKey::Blocks(blocks) => {
                let width = metrics.cell_size.width as f32;
                let height = metrics.cell_size.height as f32;
                let (x_half, y_half) = (width / 2., height / 2.);
                let (x_eighth, y_eighth) = (width / 8., height / 8.);

                for block in blocks.iter() {
                    match block {
                        Block::Custom(x0, x1, y0, y1, alpha) => {
                            let left = (*x0 as f32) * x_eighth;
                            let right = (*x1 as f32) * x_eighth;
                            let top = (*y0 as f32) * y_eighth;
                            let bottom = (*y1 as f32) * y_eighth;
                            fill_rect(&mut buffer, left..right, top..bottom, *alpha);
                        }
                        Block::UpperBlock(num) => {
                            let lower = (*num as f32) * y_eighth;
                            fill_rect(&mut buffer, 0.0..width, 0.0..lower, BlockAlpha::Full);
                        }
                        Block::LowerBlock(num) => {
                            let upper = ((8 - num) as f32) * y_eighth;
                            fill_rect(&mut buffer, 0.0..width, upper..height, BlockAlpha::Full);
                        }
                        Block::LeftBlock(num) => {
                            let right = (*num as f32) * x_eighth;
                            fill_rect(&mut buffer, 0.0..right, 0.0..height, BlockAlpha::Full);
                        }
                        Block::RightBlock(num) => {
                            let left = ((8 - num) as f32) * x_eighth;
                            fill_rect(&mut buffer, left..width, 0.0..height, BlockAlpha::Full);
                        }
                        Block::VerticalBlock(x0, x1) => {
                            let left = (*x0 as f32) * x_eighth;
                            let right = (*x1 as f32) * x_eighth;
                            fill_rect(&mut buffer, left..right, 0.0..height, BlockAlpha::Full);
                        }
                        Block::HorizontalBlock(y0, y1) => {
                            let top = (*y0 as f32) * y_eighth;
                            let bottom = (*y1 as f32) * y_eighth;
                            fill_rect(&mut buffer, 0.0..width, top..bottom, BlockAlpha::Full);
                        }
                        Block::QuadrantUL => {
                            fill_rect(&mut buffer, 0.0..x_half, 0.0..y_half, BlockAlpha::Full)
                        }
                        Block::QuadrantUR => {
                            fill_rect(&mut buffer, x_half..width, 0.0..y_half, BlockAlpha::Full)
                        }
                        Block::QuadrantLL => {
                            fill_rect(&mut buffer, 0.0..x_half, y_half..height, BlockAlpha::Full)
                        }
                        Block::QuadrantLR => {
                            fill_rect(&mut buffer, x_half..width, y_half..height, BlockAlpha::Full)
                        }
                    }
                }
            }
            BlockKey::Triangles(triangles, alpha) => {
                let mut draw = |cmd: &'static [PolyCommand], style: PolyStyle| {
                    draw_polys(
                        &metrics,
                        &[Poly {
                            path: cmd,
                            intensity: alpha,
                            style: style,
                        }],
                        &mut buffer,
                        poly_aa(metrics.anti_alias),
                        BlendMode::default(),
                    );
                };

                macro_rules! start {
                    () => {
                        PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2))
                    };
                }
                macro_rules! close {
                    () => {
                        PolyCommand::Close
                    };
                }
                macro_rules! p0 {
                    () => {
                        PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Zero)
                    };
                }
                macro_rules! p1 {
                    () => {
                        PolyCommand::LineTo(BlockCoord::One, BlockCoord::Zero)
                    };
                }
                macro_rules! p2 {
                    () => {
                        PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One)
                    };
                }
                macro_rules! p3 {
                    () => {
                        PolyCommand::LineTo(BlockCoord::One, BlockCoord::One)
                    };
                }

                // Draw triangles
                if triangles.contains(Triangle::UPPER) {
                    draw(&[start!(), p0!(), p1!(), close!()], PolyStyle::Fill);
                }
                if triangles.contains(Triangle::LOWER) {
                    draw(&[start!(), p2!(), p3!(), close!()], PolyStyle::Fill);
                }
                if triangles.contains(Triangle::LEFT) {
                    draw(&[start!(), p0!(), p2!(), close!()], PolyStyle::Fill);
                }
                if triangles.contains(Triangle::RIGHT) {
                    draw(&[start!(), p1!(), p3!(), close!()], PolyStyle::Fill);
                }

                // Fill antialiased lines between triangles
                let style = if alpha == BlockAlpha::Full {
                    PolyStyle::Outline
                } else {
                    PolyStyle::OutlineAlpha
                };
                if triangles.contains(Triangle::UPPER | Triangle::LEFT) {
                    draw(&[start!(), p0!()], style);
                }
                if triangles.contains(Triangle::UPPER | Triangle::RIGHT) {
                    draw(&[start!(), p1!()], style);
                }
                if triangles.contains(Triangle::LOWER | Triangle::LEFT) {
                    draw(&[start!(), p2!()], style);
                }
                if triangles.contains(Triangle::LOWER | Triangle::RIGHT) {
                    draw(&[start!(), p3!()], style);
                }
            }
            BlockKey::CellDiagonals(diagonals) => {
                let mut draw = |cmd: &'static [PolyCommand]| {
                    draw_polys(
                        &metrics,
                        &[Poly {
                            path: cmd,
                            intensity: BlockAlpha::Full,
                            style: PolyStyle::Outline,
                        }],
                        &mut buffer,
                        poly_aa(metrics.anti_alias),
                        BlendMode::default(),
                    );
                };

                macro_rules! U {
                    () => {
                        PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero)
                    };
                }
                macro_rules! D {
                    () => {
                        PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::One)
                    };
                }
                macro_rules! L {
                    () => {
                        PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::Frac(1, 2))
                    };
                }
                macro_rules! R {
                    () => {
                        PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2))
                    };
                }

                if diagonals.contains(CellDiagonal::UPPER_LEFT) {
                    draw(&[U!(), L!()]);
                }
                if diagonals.contains(CellDiagonal::UPPER_RIGHT) {
                    draw(&[U!(), R!()]);
                }
                if diagonals.contains(CellDiagonal::LOWER_LEFT) {
                    draw(&[D!(), L!()]);
                }
                if diagonals.contains(CellDiagonal::LOWER_RIGHT) {
                    draw(&[D!(), R!()]);
                }
            }
            BlockKey::Sextant(pattern) => {
                let width = metrics.cell_size.width as f32;
                let height = metrics.cell_size.height as f32;
                let (x_half, y_third) = (width / 2., height / 3.);
                for row in 0..3 {
                    for col in 0..2 {
                        let bit = 2 * row + col;
                        if pattern & (1u8 << bit) != 0 {
                            fill_rect(
                                &mut buffer,
                                col as f32 * x_half..(col + 1) as f32 * x_half,
                                row as f32 * y_third..(row + 1) as f32 * y_third,
                                BlockAlpha::Full,
                            );
                        }
                    }
                }
            }
            BlockKey::Octant(pattern) => {
                let width = metrics.cell_size.width as f32;
                let height = metrics.cell_size.height as f32;
                let (x_half, y_fourth) = (width / 2., height / 4.);
                for row in 0..4 {
                    for col in 0..2 {
                        let bit = 2 * row + col;
                        if pattern & (1u8 << bit) != 0 {
                            fill_rect(
                                &mut buffer,
                                col as f32 * x_half..(col + 1) as f32 * x_half,
                                row as f32 * y_fourth..(row + 1) as f32 * y_fourth,
                                BlockAlpha::Full,
                            );
                        }
                    }
                }
            }
            BlockKey::Braille(dots_pattern) => {
                // `dots_pattern` is a byte whose bits corresponds to dots
                // on a 2 by 4 dots-grid.
                // The position of a dot for a bit position (1-indexed) is as follow:
                // 1 4  |
                // 2 5  |<- These 3 lines are filled first (for the first 64 symbols)
                // 3 6  |
                // 7 8  <- This last line is filled last (for the remaining 192 symbols)
                //
                // NOTE: for simplicity & performance reasons, a dot is a square not a circle.

                let dot_area_width = metrics.cell_size.width as f32 / 2.;
                let dot_area_height = metrics.cell_size.height as f32 / 4.;
                let square_length = dot_area_width / 2.;
                let topleft_offset_x = dot_area_width / 2. - square_length / 2.;
                let topleft_offset_y = dot_area_height / 2. - square_length / 2.;

                let mut pixmap = buffer.as_mut();
                let mut paint = Paint::default();
                paint.set_color(tiny_skia::Color::WHITE);
                paint.force_hq_pipeline = true;
                paint.anti_alias = true;
                let identity = Transform::identity();

                const BIT_MASK_AND_DOT_POSITION: [(u8, f32, f32); 8] = [
                    (1 << 0, 0., 0.),
                    (1 << 1, 0., 1.),
                    (1 << 2, 0., 2.),
                    (1 << 3, 1., 0.),
                    (1 << 4, 1., 1.),
                    (1 << 5, 1., 2.),
                    (1 << 6, 0., 3.),
                    (1 << 7, 1., 3.),
                ];
                for (bit_mask, dot_pos_x, dot_pos_y) in &BIT_MASK_AND_DOT_POSITION {
                    if dots_pattern & bit_mask == 0 {
                        // Bit for this dot position is not set
                        continue;
                    }
                    let topleft_x = (*dot_pos_x) * dot_area_width + topleft_offset_x;
                    let topleft_y = (*dot_pos_y) * dot_area_height + topleft_offset_y;

                    let path = PathBuilder::from_rect(
                        tiny_skia::Rect::from_xywh(
                            topleft_x,
                            topleft_y,
                            square_length,
                            square_length,
                        )
                        .expect("valid rect"),
                    );
                    pixmap.fill_path(&path, &paint, FillRule::Winding, identity, None);
                }
            }
            _ => return None,
        })
}
