impl Face {
    pub fn get_color_glyph_paint(
        &mut self,
        glyph_index: FT_UInt,
        root_transform: FT_Color_Root_Transform,
    ) -> anyhow::Result<FT_Opaque_Paint_> {
        unsafe {
            let mut result = MaybeUninit::<FT_Opaque_Paint_>::zeroed();
            let status = FT_Get_Color_Glyph_Paint(
                self.face,
                glyph_index,
                root_transform,
                result.as_mut_ptr(),
            );
            if status == 1 {
                Ok(result.assume_init())
            } else {
                anyhow::bail!("FT_Get_Color_Glyph_Paint for glyph {glyph_index} failed");
            }
        }
    }

    pub fn get_color_glyph_clip_box(
        &mut self,
        glyph_index: FT_UInt,
    ) -> anyhow::Result<FT_ClipBox_> {
        unsafe {
            let mut result = MaybeUninit::<FT_ClipBox_>::zeroed();
            let status = FT_Get_Color_Glyph_ClipBox(self.face, glyph_index, result.as_mut_ptr());
            if status == 1 {
                Ok(result.assume_init())
            } else {
                anyhow::bail!("FT_Get_Color_Glyph_ClipBox for glyph {glyph_index} failed");
            }
        }
    }

    pub fn get_paint(&mut self, paint: FT_Opaque_Paint_) -> anyhow::Result<FT_COLR_Paint_> {
        unsafe {
            let mut result = MaybeUninit::<FT_COLR_Paint_>::zeroed();
            let status = FT_Get_Paint(self.face, paint, result.as_mut_ptr());
            if status == 1 {
                Ok(result.assume_init())
            } else {
                anyhow::bail!("FT_Get_Paint failed");
            }
        }
    }

    /// Replace any palette entry overrides and select the specified palette
    pub fn select_palette(&mut self, index: FT_UShort) -> anyhow::Result<()> {
        unsafe {
            self.palette.take();

            let mut pdata = MaybeUninit::<FT_Palette_Data>::zeroed();
            ft_result(FT_Palette_Data_Get(self.face, pdata.as_mut_ptr()), ())
                .context("FT_Palette_Data_Get")?;
            let pdata = pdata.assume_init();

            let mut palette_ptr = std::ptr::null_mut();

            ft_result(FT_Palette_Select(self.face, index, &mut palette_ptr), ())
                .with_context(|| format!("FT_Palette_Select for index={index}. Note: {pdata:?}"))?;

            let palette =
                std::slice::from_raw_parts_mut(palette_ptr, pdata.num_palette_entries as usize);

            self.palette.replace(palette);

            Ok(())
        }
    }

    pub fn get_palette_entry(&self, index: u32) -> anyhow::Result<FT_Color> {
        self.palette
            .as_ref()
            .and_then(|slice| slice.get(index as usize))
            .copied()
            .ok_or_else(|| anyhow::anyhow!("invalid palette entry {index}"))
    }

    pub fn get_palette_data(&self) -> anyhow::Result<PaletteInfo> {
        unsafe {
            let mut result = MaybeUninit::<FT_Palette_Data>::zeroed();
            ft_result(FT_Palette_Data_Get(self.face, result.as_mut_ptr()), ())
                .context("FT_Palette_Data_Get")?;

            let data = result.assume_init();
            let mut palettes = vec![];

            let name_ids = from_raw_parts(data.palette_name_ids, data.num_palettes as usize);
            let flagses = from_raw_parts(data.palette_flags, data.num_palettes as usize);
            let entry_name_ids = from_raw_parts(
                data.palette_entry_name_ids,
                data.num_palette_entries as usize,
            );

            let entry_names: Vec<String> = entry_name_ids
                .iter()
                .map(|&id| {
                    self.get_sfnt_name(id as _)
                        .map(|rec| rec.name)
                        .unwrap_or_else(|_| String::new())
                })
                .collect();

            for (palette_index, (&name_id, &flags)) in
                name_ids.iter().zip(flagses.iter()).enumerate()
            {
                palettes.push(Palette {
                    palette_index,
                    flags,
                    name: self
                        .get_sfnt_name(name_id as _)
                        .map(|rec| rec.name)
                        .unwrap_or_else(|_| String::new()),
                    entry_names: entry_names.clone(),
                });
            }
            Ok(PaletteInfo {
                num_palettes: data.num_palettes as usize,
                palettes,
            })
        }
    }

    pub fn get_sfnt_name(&self, i: FT_UInt) -> anyhow::Result<NameRecord> {
        unsafe {
            let mut sfnt_name = MaybeUninit::<FT_SfntName>::zeroed();
            ft_result(FT_Get_Sfnt_Name(self.face, i, sfnt_name.as_mut_ptr()), ())
                .context("FT_Get_Sfnt_Name")?;
            let sfnt_name = sfnt_name.assume_init();
            let bytes =
                from_raw_parts(sfnt_name.string as *const u8, sfnt_name.string_len as usize);

            let encoding = match (sfnt_name.platform_id as u32, sfnt_name.encoding_id as u32) {
                (TT_PLATFORM_MACINTOSH, TT_MAC_ID_JAPANESE)
                | (TT_PLATFORM_MICROSOFT, TT_MS_ID_SJIS) => encoding_rs::SHIFT_JIS,
                (TT_PLATFORM_MACINTOSH, TT_MAC_ID_SIMPLIFIED_CHINESE)
                | (TT_PLATFORM_MICROSOFT, TT_MS_ID_PRC) => encoding_rs::GBK,
                (TT_PLATFORM_MACINTOSH, TT_MAC_ID_TRADITIONAL_CHINESE)
                | (TT_PLATFORM_MICROSOFT, TT_MS_ID_BIG_5) => encoding_rs::BIG5,
                (
                    TT_PLATFORM_MICROSOFT,
                    TT_MS_ID_UCS_4 | TT_MS_ID_UNICODE_CS | TT_MS_ID_SYMBOL_CS,
                ) => encoding_rs::UTF_16BE,
                (TT_PLATFORM_MICROSOFT, TT_MS_ID_WANSUNG) => encoding_rs::EUC_KR,
                (TT_PLATFORM_APPLE_UNICODE | TT_PLATFORM_ISO, _) => encoding_rs::UTF_16BE,
                (TT_PLATFORM_MACINTOSH, TT_MAC_ID_ROMAN) => encoding_rs::MACINTOSH,
                _ => {
                    anyhow::bail!(
                        "Skipping name_id={} because platform_id={} encoding_id={}",
                        sfnt_name.name_id,
                        sfnt_name.platform_id,
                        sfnt_name.encoding_id
                    );
                }
            };

            let (name, _) = encoding.decode_with_bom_removal(bytes);

            Ok(NameRecord {
                platform_id: sfnt_name.platform_id,
                encoding_id: sfnt_name.encoding_id,
                name_id: sfnt_name.name_id,
                language_id: sfnt_name.language_id,
                name: name.to_string(),
            })
        }
    }

    pub fn get_paint_layers(
        &mut self,
        iter: &mut FT_LayerIterator_,
    ) -> anyhow::Result<FT_Opaque_Paint_> {
        unsafe {
            let mut result = MaybeUninit::<FT_Opaque_Paint_>::zeroed();
            let status = FT_Get_Paint_Layers(self.face, iter, result.as_mut_ptr());
            if status == 1 {
                Ok(result.assume_init())
            } else {
                anyhow::bail!("FT_Get_Paint_Layers failed");
            }
        }
    }
}
