impl Face {
    /// This is a wrapper around set_char_size and select_size
    /// that accounts for some weirdness with eg: color emoji
    pub fn set_font_size(&mut self, point_size: f64, dpi: u32) -> anyhow::Result<SelectedFontSize> {
        if let Some(face_size) = self.size.as_ref() {
            if face_size.size == point_size && face_size.dpi == dpi {
                return Ok(SelectedFontSize {
                    width: face_size.cell_width,
                    height: face_size.cell_height,
                    is_scaled: face_size.is_scaled,
                    cap_height: face_size.cap_height,
                    cap_height_to_height_ratio: face_size.cap_height_to_height_ratio,
                });
            }
        }

        let pixel_height = point_size * dpi as f64 / 72.0;
        log::debug!(
            "set_char_size computing {} dpi={} (pixel height={})",
            point_size,
            dpi,
            pixel_height
        );

        // Scaling before truncating to integer minimizes the chances of hitting
        // the fallback code for set_pixel_sizes below.
        let size = FT_F26Dot6::from_num(point_size);

        let selected_size = match self.set_char_size(size, size, dpi, dpi) {
            Ok(_) => {
                // Compute metrics for the nominal monospace cell
                let ComputedCellMetrics { width, height } = self.cell_metrics();
                SelectedFontSize {
                    width,
                    height,
                    cap_height: None,
                    cap_height_to_height_ratio: None,
                    is_scaled: true,
                }
            }
            Err(err) => {
                log::debug!("set_char_size: {:?}, will inspect strikes", err);

                let sizes = unsafe {
                    let rec = &(*self.face);
                    from_raw_parts(rec.available_sizes, rec.num_fixed_sizes as usize)
                };
                if sizes.is_empty() {
                    return Err(err);
                }
                // Find the best matching size; we look for the strike whose height
                // is closest to the desired size.
                struct Best {
                    idx: usize,
                    distance: usize,
                    height: i16,
                    width: i16,
                }
                let mut best: Option<Best> = None;

                for (idx, info) in sizes.iter().enumerate() {
                    log::debug!("idx={} info={:?}", idx, info);
                    let distance = (info.height - (pixel_height as i16)).abs() as usize;
                    let candidate = Best {
                        idx,
                        distance,
                        height: info.height,
                        width: info.width,
                    };

                    match best.take() {
                        Some(existing) => {
                            best.replace(if candidate.distance < existing.distance {
                                candidate
                            } else {
                                existing
                            });
                        }
                        None => {
                            best.replace(candidate);
                        }
                    }
                }
                let best = best.unwrap();
                self.select_size(best.idx)?;
                // Compute the cell metrics at this size.
                // This stuff is a bit weird; for GohuFont.otb, cell_metrics()
                // returns (8.0, 0.0) when the selected bitmap strike is (4, 14).
                // 4 pixels is too thin for this font, so we take the max of the
                // known dimensions to produce the size.
                // <https://github.com/wezterm/wezterm/issues/1165>
                let m = self.cell_metrics();
                let height = f64::from(best.height).max(m.height);
                SelectedFontSize {
                    width: f64::from(best.width).max(m.width),
                    height,
                    is_scaled: false,
                    cap_height: None,
                    cap_height_to_height_ratio: None,
                }
            }
        };

        self.size.replace(FaceSize {
            size: point_size,
            dpi,
            cell_width: selected_size.width,
            cap_height: None,
            cap_height_to_height_ratio: None,
            cell_height: selected_size.height,
            is_scaled: selected_size.is_scaled,
        });

        // Can't compute cap height until after we've assigned self.size
        if let Ok(cap_height) = self.compute_cap_height() {
            let cap_height_to_height_ratio = cap_height / selected_size.height;

            self.size.replace(FaceSize {
                size: point_size,
                dpi,
                cell_width: selected_size.width,
                cap_height: Some(cap_height),
                cap_height_to_height_ratio: Some(cap_height_to_height_ratio),
                cell_height: selected_size.height,
                is_scaled: selected_size.is_scaled,
            });

            Ok(SelectedFontSize {
                cap_height: Some(cap_height),
                cap_height_to_height_ratio: Some(cap_height_to_height_ratio),
                ..selected_size
            })
        } else {
            Ok(selected_size)
        }
    }

    fn set_char_size(
        &mut self,
        char_width: FT_F26Dot6,
        char_height: FT_F26Dot6,
        horz_resolution: FT_UInt,
        vert_resolution: FT_UInt,
    ) -> anyhow::Result<()> {
        ft_result(
            unsafe {
                FT_Set_Char_Size(
                    self.face,
                    char_width,
                    char_height,
                    horz_resolution,
                    vert_resolution,
                )
            },
            (),
        )
        .context("FT_Set_Char_Size")?;

        unsafe {
            if (*self.face).height == 0 {
                anyhow::bail!("font has 0 height, fallback to bitmaps");
            }
        }

        Ok(())
    }

    fn select_size(&mut self, idx: usize) -> anyhow::Result<()> {
        ft_result(unsafe { FT_Select_Size(self.face, idx as i32) }, ()).context("FT_Select_Size")
    }

    pub fn set_transform(&mut self, matrix: Option<FT_Matrix>) {
        let mut matrix = matrix;
        unsafe {
            FT_Set_Transform(
                self.face,
                match &mut matrix {
                    Some(m) => m as *mut _,
                    None => std::ptr::null_mut(),
                },
                std::ptr::null_mut(),
            )
        }
    }
}
